use futures::{SinkExt, StreamExt};
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde_derive::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::time::SystemTime;
use tokio::task::JoinHandle;
use tracing::info;
use tungstenite::Message;
use url::Url;

use crate::exchanges::formatter::OrderbookFormatter;

#[derive(thiserror::Error, Debug)]
pub enum BybitSocketError {
    ConnectionError(tungstenite::Error),
    SerdeJsonError(serde_json::Error),
}

impl std::fmt::Display for BybitSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BybitSocketError::ConnectionError(e) => write!(f, "Bybit ConnectionError: {e:?}"),
            BybitSocketError::SerdeJsonError(e) => write!(f, "Bybit SerdeJsonError: {e:?}"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BybitDepth {
    pub topic: String,
    #[serde(rename = "type")]
    pub data_type: String,
    pub ts: u64,
    pub data: BybitDepthData,
}

#[derive(Debug, Deserialize)]
pub struct BybitDepthData {
    pub s: String,
    pub b: Vec<[String; 2]>,
    pub a: Vec<[String; 2]>,
    pub u: u64,
    pub seq: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Subscription {
    pub op: String,
    pub args: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Pong {
    pub op: String,
}

pub struct BybitOrderbook {
    pub bid: HashMap<String, HashMap<Decimal, Decimal>>,
    pub ask: HashMap<String, HashMap<Decimal, Decimal>>,
}

impl BybitOrderbook {
    pub fn from(symbols: impl Iterator<Item = String>) -> Self {
        let mut obj = Self {
            bid: HashMap::new(),
            ask: HashMap::new(),
        };
        for symbol in symbols {
            obj.bid.insert(symbol.clone(), HashMap::new());
            obj.ask.insert(symbol.clone(), HashMap::new());
        }
        obj
    }

    pub fn init_orderbook(&mut self, data: &BybitDepthData) {
        let bid_table = self.bid.get_mut(&data.s).unwrap();
        let ask_table = self.ask.get_mut(&data.s).unwrap();

        for bid in data.b.iter() {
            let price = Decimal::from_str(&bid[0]).unwrap();
            let quantity = Decimal::from_str(&bid[1]).unwrap();
            bid_table.insert(price, quantity);
        }

        for ask in data.a.iter() {
            let price = Decimal::from_str(&ask[0]).unwrap();
            let quantity = Decimal::from_str(&ask[1]).unwrap();
            ask_table.insert(price, quantity);
        }
    }

    pub fn update_orderbook(&mut self, data: &BybitDepthData) {
        let bid_table = self.bid.get_mut(&data.s).unwrap();
        let ask_table = self.ask.get_mut(&data.s).unwrap();

        for bid in data.b.iter() {
            let price = Decimal::from_str(&bid[0]).unwrap();
            let quantity = Decimal::from_str(&bid[1]).unwrap();

            if bid_table.contains_key(&price) {
                if quantity == dec!(0) {
                    bid_table.remove(&price);
                } else {
                    bid_table.insert(price, quantity);
                }
            } else {
                bid_table.insert(price, quantity);
            }
        }

        for ask in data.a.iter() {
            let price = Decimal::from_str(&ask[0]).unwrap();
            let quantity = Decimal::from_str(&ask[1]).unwrap();

            if ask_table.contains_key(&price) {
                if quantity == dec!(0) {
                    ask_table.remove(&price);
                } else {
                    ask_table.insert(price, quantity);
                }
            } else {
                ask_table.insert(price, quantity);
            }
        }
    }
}

pub fn bybit_orderbook_stream(
    symbols: Vec<&'static str>,
) -> JoinHandle<Result<(), BybitSocketError>> {
    let symbols = symbols.into_iter().map(|x| x.to_string());
    let mut orderbook = BybitOrderbook::from(symbols.clone());

    let stream_handle = tokio::spawn(async move {
        loop {
            let url =
                Url::parse("wss://stream.bybit.com/v5/public/linear").expect("URL parse failed");

            let channels: Vec<String> = symbols
                .clone()
                .into_iter()
                .map(|x| format!("orderbook.50.{}", x.to_uppercase()))
                .collect();

            let (mut stream, _) = tokio_tungstenite::connect_async(url)
                .await
                .map_err(BybitSocketError::ConnectionError)?;

            let subscription = Subscription {
                op: String::from("subscribe"),
                args: channels,
            };
            let subscribe_msg = serde_json::to_string(&subscription).unwrap();

            stream
                .send(Message::Text(subscribe_msg.into()))
                .await
                .unwrap();

            let mut ponged = SystemTime::now();

            while let Some(Ok(message)) = stream.next().await {
                match message {
                    tungstenite::Message::Text(raw_msg) => {
                        if raw_msg.contains("orderbook.50") {
                            let data: BybitDepth = serde_json::from_str(&raw_msg)
                                .map_err(BybitSocketError::SerdeJsonError)?;

                            if data.data_type == "snapshot".to_string() {
                                orderbook.init_orderbook(&data.data);
                            } else if data.data_type == "delta".to_string() {
                                orderbook.update_orderbook(&data.data);
                            }

                            let mut sorted_bids: Vec<_> =
                                orderbook.bid[&data.data.s].iter().collect();
                            sorted_bids.sort_by_key(|a| a.0);
                            sorted_bids.reverse();

                            let mut sorted_asks: Vec<_> =
                                orderbook.ask[&data.data.s].iter().collect();
                            sorted_asks.sort_by_key(|a| a.0);

                            let formatted =
                                OrderbookFormatter::from_bybit(data, sorted_bids, sorted_asks);
                            info!("{:?}", formatted);
                        }

                        // send pong frames every 15 seconds
                        if ponged.elapsed().unwrap().as_secs() >= 15 {
                            let pong = Pong {
                                op: String::from("pong"),
                            };
                            let pong_msg = serde_json::to_string(&pong).unwrap();

                            stream.send(Message::Text(pong_msg.into())).await.unwrap();

                            ponged = SystemTime::now();
                        }
                    }
                    tungstenite::Message::Ping(_) => {
                        stream.send(Message::Pong(vec![])).await.ok();
                    }
                    tungstenite::Message::Close(_) => {
                        info!("closed. retrying...");
                        break;
                    }
                    other => {
                        info!("{other:?}")
                    }
                }
            }
        }
    });

    stream_handle
}
