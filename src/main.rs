use futures::future;
use tokio::sync::mpsc;
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

    let (tx, mut rx) = mpsc::channel(100);
    let mut handles = Vec::new();

    let symbols = vec!["BTCUSDT"];

    // TODO:: add bybit symbols api function
    let binance_symbols = binance::api::get_binance_symbols().await.unwrap();

    handles.push(binance::binance_orderbook_stream(
        symbols.clone(),
        tx.clone(),
    ));

    handles.push(bybit::bybit_orderbook_stream(symbols.clone(), tx.clone()));

    tokio::spawn(async move {
        while let Some(ob) = rx.recv().await {
            println!("{:?}", ob.exchange);
        }
    });

    match future::try_join_all(handles).await {
        Ok(_) => info!("done"),
        Err(e) => info!("{e:?}"),
    }
}
