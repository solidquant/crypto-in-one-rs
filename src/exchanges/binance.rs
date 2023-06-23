use futures::{SinkExt, StreamExt};
use serde_derive::{Deserialize, Serialize};
use serde_json;
use tokio::task::JoinHandle;
use tracing::info;
use tungstenite::Message;
use url::Url;

use crate::exchanges::formatter::OrderbookFormatter;

#[derive(thiserror::Error, Debug)]
pub enum BinanceSocketError {
    ConnectionError(tungstenite::Error),
    SerdeJsonError(serde_json::Error),
}

impl std::fmt::Display for BinanceSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinanceSocketError::ConnectionError(e) => write!(f, "Binance ConnectionError: {e:?}"),
            BinanceSocketError::SerdeJsonError(e) => write!(f, "Binance SerdeJsonError: {e:?}"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BinanceDepth {
    pub e: String,
    pub E: u64,
    pub T: u64,
    pub s: String,
    pub U: u64,
    pub u: u64,
    pub pu: u64,
    pub b: Vec<[String; 2]>,
    pub a: Vec<[String; 2]>,
}

#[derive(Serialize, Deserialize)]
pub struct Subscription {
    pub method: String,
    pub params: Vec<String>,
    pub id: u32,
}

pub fn binance_orderbook_stream(
    symbols: Vec<&'static str>,
) -> JoinHandle<Result<(), BinanceSocketError>> {
    let symbols = symbols.into_iter().map(|x| x.to_string());

    let stream_handle = tokio::spawn(async move {
        loop {
            let url = Url::parse("wss://fstream.binance.com/ws/").expect("URL parse failed");

            let channels: Vec<String> = symbols
                .clone()
                .into_iter()
                .map(|x| format!("{}@depth20@100ms", x.to_lowercase()))
                .collect();

            let (mut stream, _) = tokio_tungstenite::connect_async(url)
                .await
                .map_err(BinanceSocketError::ConnectionError)?;

            let subscription = Subscription {
                method: String::from("SUBSCRIBE"),
                params: channels,
                id: 1,
            };
            let subscribe_msg = serde_json::to_string(&subscription).unwrap();

            stream
                .send(Message::Text(subscribe_msg.into()))
                .await
                .unwrap();

            while let Some(Ok(message)) = stream.next().await {
                match message {
                    tungstenite::Message::Text(raw_msg) => {
                        if raw_msg.contains("depthUpdate") {
                            let data: BinanceDepth = serde_json::from_str(&raw_msg)
                                .map_err(BinanceSocketError::SerdeJsonError)?;
                            let formatted = OrderbookFormatter::from_binance(data);
                            info!("{:?}", formatted);
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
