use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoPair {
    pub symbol: String,
    pub base: String,
    pub quote: String,
    pub price: f64,
    pub volume_24h: f64,
    pub change_24h: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub symbol: String,
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub symbol: String,
    pub side: String, // "buy" or "sell"
    pub amount: f64,
    pub price: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub side: String,
    pub entry_price: f64,
    pub amount: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub entry_time: u64,
}
