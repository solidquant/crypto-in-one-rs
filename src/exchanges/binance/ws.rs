use futures::{SinkExt, StreamExt};
use serde_derive::{Deserialize, Serialize};
use serde_json;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tracing::info;
use tungstenite::Message;
use url::Url;

use crate::error::SocketError;
use crate::exchanges::formatter::OrderbookFormatter;

use crate::exchanges::formatter::Orderbook;

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
    tx: Sender<Orderbook>,
) -> JoinHandle<Result<(), SocketError>> {
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
                .map_err(SocketError::BinanceConnectionError)?;

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
                                .map_err(SocketError::BinanceSerdeJsonError)?;
                            let formatted = OrderbookFormatter::from_binance(data);
                            tx.send(formatted)
                                .await
                                .expect("Failed to send Orderbook data");
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
