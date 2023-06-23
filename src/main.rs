use tracing::{info, Level};
use tracing_subscriber::{filter, prelude::*};

use bid_ask_service_clone::exchanges::{binance, bybit};

#[tokio::main]
async fn main() {
    let filter = filter::Targets::new().with_target("bid_ask_service_clone", Level::INFO);
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    let symbols = vec!["BTCUSDT"];
    let handle = bybit::bybit_orderbook_stream(symbols.clone());
    match handle.await {
        Ok(_) => info!("done"),
        Err(e) => info!("{e:?}"),
    }
}
