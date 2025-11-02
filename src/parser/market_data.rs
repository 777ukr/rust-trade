// Market data parsing implementation
use crate::models::MarketData;

pub fn parse_json_market_data(json: &str) -> Result<MarketData, serde_json::Error> {
    serde_json::from_str(json)
}
