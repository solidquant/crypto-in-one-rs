use futures::{SinkExt, StreamExt};
use serde_derive::{Deserialize, Serialize};
use serde_json;
use tokio::task::JoinHandle;
use tracing::info;
use tungstenite::Message;
use url::Url;

use crate::exchanges::formatter::OrderbookFormatter;

pub fn okx_orderbook_stream(symbols: Vec<&'static str>) -> JoinHandle<Result<(), OkxSocketError>> {
    let symbols = symbols.into_iter().map(|x| x.to_string());

    let stream_handle = tokio::spawn(async move {
        loop {
            let url = Url::parse("wss://ws.okx.com:8443/ws/v5/public").expect("URL parse failed");
        }
    });
}
