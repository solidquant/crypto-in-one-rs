// use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

use crate::exchanges::binance::ws::BinanceDepth;
use crate::exchanges::bybit::ws::BybitDepth;

#[derive(Debug)]
pub enum Exchange {
    Binance,
    Bybit,
}

#[derive(Debug)]
pub enum Instrument {
    USDM,
}

#[derive(Debug)]
pub struct SymbolInfo {
    pub exchange: Exchange,
    pub instrument: Instrument,
    pub symbol: String,
    pub price_tick_size: Decimal,
    pub limit_max_qty: Decimal,
    pub limit_min_qty: Decimal,
    pub limit_step_size: Decimal,
    pub market_max_qty: Decimal,
    pub market_min_qty: Decimal,
    pub market_step_size: Decimal,
    pub min_notional: Decimal,
}

impl SymbolInfo {
    pub fn new(exchange: Exchange, instrument: Instrument, symbol: String) -> Self {
        Self {
            exchange,
            instrument,
            symbol,
            price_tick_size: dec!(0),
            limit_max_qty: dec!(0),
            limit_min_qty: dec!(0),
            limit_step_size: dec!(0),
            market_max_qty: dec!(0),
            market_min_qty: dec!(0),
            market_step_size: dec!(0),
            min_notional: dec!(0),
        }
    }
}

#[derive(Debug)]
pub struct Level {
    pub exchange: Exchange,
    pub price: Decimal,
    pub quantity: Decimal,
}

#[derive(Debug)]
pub struct Orderbook {
    pub exchange: Exchange,
    pub symbol: String,
    pub updated: u64,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
}

pub struct OrderbookFormatter {}

impl OrderbookFormatter {
    pub fn from_binance(data: BinanceDepth) -> Orderbook {
        let _vec_of_levels = |x: Vec<[String; 2]>| -> Vec<Level> {
            return x
                .into_iter()
                .map(|el| Level {
                    exchange: Exchange::Binance,
                    price: Decimal::from_str(&el[0]).unwrap(),
                    quantity: Decimal::from_str(&el[1]).unwrap(),
                })
                .collect();
        };
        Orderbook {
            exchange: Exchange::Binance,
            symbol: data.s,
            updated: data.E,
            bids: _vec_of_levels(data.b),
            asks: _vec_of_levels(data.a),
        }
    }

    pub fn from_bybit(
        data: BybitDepth,
        bids: Vec<(&Decimal, &Decimal)>,
        asks: Vec<(&Decimal, &Decimal)>,
    ) -> Orderbook {
        let _vec_of_levels = |x: Vec<(&Decimal, &Decimal)>| -> Vec<Level> {
            return x
                .into_iter()
                .map(|el| Level {
                    exchange: Exchange::Bybit,
                    price: el.0.clone(),
                    quantity: el.1.clone(),
                })
                .collect();
        };
        Orderbook {
            exchange: Exchange::Bybit,
            symbol: data.data.s,
            updated: data.ts,
            bids: _vec_of_levels(bids),
            asks: _vec_of_levels(asks),
        }
    }
}
