pub mod gateway;
pub mod client;

use crate::models::MarketData;

pub trait ExchangeAPI {
    fn get_price(&self, symbol: &str) -> Result<f64, String>;
    fn place_order(&self, order: &OrderRequest) -> Result<String, String>;
}

pub struct OrderRequest {
    pub symbol: String,
    pub side: String, // "buy" or "sell"
    pub amount: f64,
    pub price: Option<f64>, // None for market orders
}
