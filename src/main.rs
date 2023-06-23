use tracing::{info, Level};
use tracing_subscriber::{filter, prelude::*};

use crypto_in_one_rs::exchanges::{binance, bybit};

#[tokio::main]
async fn main() {
    let filter = filter::Targets::new().with_target("crypto_in_one_rs", Level::INFO);
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    let symbols = vec!["BTCUSDT"];
    let handle = binance::binance_orderbook_stream(symbols.clone());
    match handle.await {
        Ok(_) => info!("done"),
        Err(e) => info!("{e:?}"),
    }
}
