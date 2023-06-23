// use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::prelude::*;

use crate::exchanges::binance::BinanceDepth;
use crate::exchanges::bybit::BybitDepth;

#[derive(Debug)]
pub enum Exchange {
    Binance,
    Bybit,
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
