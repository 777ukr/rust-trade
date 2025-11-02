pub mod market_data;
pub mod price_parser;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedData {
    pub timestamp: u64,
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
}

pub fn parse_market_data(data: &str) -> Result<ParsedData, String> {
    // Implement parsing logic
    todo!()
}
