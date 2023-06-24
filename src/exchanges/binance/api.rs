use reqwest;
use serde_derive::{Deserialize, Serialize};
use serde_json;
use rust_decimal::prelude::*;
use std::collections::HashMap;

use crate::exchanges::formatter::{Exchange, Instrument, SymbolInfo};
use crate::error::HttpError;

#[derive(Debug, Deserialize)]
pub struct BinanceExchangeInfo {
    pub symbols: Vec<BinanceSymbol>,
}

#[derive(Debug, Deserialize)]
pub struct BinanceSymbol {
    symbol: String,
    #[serde(rename = "contractType")]
    contract_type: String,
    status: String,
    filters: Vec<BinanceFilter>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "filterType")]
pub enum BinanceFilter {
    PRICE_FILTER {
        #[serde(rename = "tickSize")]
        tick_size: String,
    },
    LOT_SIZE {
        #[serde(rename = "maxQty")]
        max_qty: String,
        #[serde(rename = "minQty")]
        min_qty: String,
        #[serde(rename = "stepSize")]
        step_size: String,
    },
    MARKET_LOT_SIZE {
        #[serde(rename = "maxQty")]
        max_qty: String,
        #[serde(rename = "minQty")]
        min_qty: String,
        #[serde(rename = "stepSize")]
        step_size: String,
    },
    MIN_NOTIONAL {
        notional: String,
    },
    #[serde(other)]
    Unknown,
}

pub async fn get_binance_symbols() -> Result<HashMap<String, SymbolInfo>, HttpError> {
    let client = reqwest::Client::new();

    let response = client.get("https://fapi.binance.com/fapi/v1/exchangeInfo")   
        .send()
        .await
        .map_err(HttpError::BinanceApiError)?;

    if !response.status().is_success() {
        return Err(HttpError::BinanceSymbolError);
    }

    let body = response.text().await.unwrap();
    let data: BinanceExchangeInfo = serde_json::from_str(&body).expect("Failed to deserialize JSON");

    let symbols: Vec<(String, SymbolInfo)> = data.symbols
        .into_iter()
        .filter(|x| x.contract_type == "PERPETUAL" && x.status == "TRADING")
        .map(|x| {
            let mut info = SymbolInfo::new(
                Exchange::Binance,
                Instrument::USDM,
                x.symbol.clone()
            );

            let _to_decimal = |x: &str| Decimal::from_str(x).unwrap();

            for filter in &x.filters {
                match filter {
                    BinanceFilter::PRICE_FILTER { tick_size } => {
                        info.price_tick_size = _to_decimal(&tick_size);
                    },
                    BinanceFilter::LOT_SIZE { max_qty, min_qty, step_size } => {
                        info.limit_max_qty = _to_decimal(&max_qty);
                        info.limit_min_qty = _to_decimal(&min_qty);
                        info.limit_step_size = _to_decimal(&step_size);
                    },
                    BinanceFilter::MARKET_LOT_SIZE { max_qty, min_qty, step_size } => {
                        info.market_max_qty = _to_decimal(&max_qty);
                        info.market_min_qty = _to_decimal(&min_qty);
                        info.market_step_size = _to_decimal(&step_size);
                    },
                    BinanceFilter::MIN_NOTIONAL { notional } => {
                        info.min_notional = _to_decimal(&notional);
                    }
                    _ => {},
                }
            }

            (x.symbol, info)
        })
        .collect();

    let symbols: HashMap<_, _> = HashMap::from_iter(symbols);
    Ok(symbols)
}