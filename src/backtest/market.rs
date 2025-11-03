//! Состояние рынка и потоки данных

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeTick {
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub side: TradeSide, // true = buy (taker buy), false = sell (taker sell)
    pub trade_id: String,
    pub best_bid: Option<f64>, // Лучшая цена покупки из стакана
    pub best_ask: Option<f64>, // Лучшая цена продажи из стакана
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeSide {
    Buy,  // Taker buy (hit the ask)
    Sell, // Taker sell (hit the bid)
}

impl From<bool> for TradeSide {
    fn from(b: bool) -> Self {
        if b { TradeSide::Buy } else { TradeSide::Sell }
    }
}

#[derive(Debug, Clone)]
pub struct TradeStream {
    pub symbol: String,
    pub trades: Vec<TradeTick>,
    pub current_index: Option<usize>,
}

impl TradeStream {
    pub fn new(symbol: String, trades: Vec<TradeTick>) -> Self {
        Self {
            symbol,
            trades,
            current_index: Some(0),
        }
    }
    
    pub fn has_more(&self) -> bool {
        if let Some(idx) = self.current_index {
            idx < self.trades.len()
        } else {
            false
        }
    }
    
    pub fn reset(&mut self) {
        self.current_index = Some(0);
    }
    
    pub fn get_current_tick(&self) -> Option<&TradeTick> {
        if let Some(idx) = self.current_index {
            self.trades.get(idx)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct MarketState {
    pub symbol: String,
    pub current_price: f64,
    pub best_bid: Option<f64>,
    pub best_ask: Option<f64>,
    pub volume_24h: f64,
    pub last_update: DateTime<Utc>,
}

impl MarketState {
    pub fn new() -> Self {
        Self {
            symbol: String::new(),
            current_price: 0.0,
            best_bid: None,
            best_ask: None,
            volume_24h: 0.0,
            last_update: Utc::now(),
        }
    }
    
    pub fn update_from_tick(&mut self, tick: &TradeTick) {
        self.symbol = tick.symbol.clone();
        self.current_price = tick.price;
        self.last_update = tick.timestamp;
        
        // В реальности нужно обновлять best_bid/best_ask из orderbook
        // Здесь упрощенно
        match tick.side {
            TradeSide::Buy => {
                // Taker buy - купили по ASK, значит ASK был <= tick.price
                self.best_ask = Some(tick.price);
            }
            TradeSide::Sell => {
                // Taker sell - продали по BID, значит BID был >= tick.price
                self.best_bid = Some(tick.price);
            }
        }
    }
}

