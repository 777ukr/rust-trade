// RSI indicator implementation
use crate::indicators::{TechnicalIndicator, IndicatorValue};

pub struct RSI {
    period: usize,
}

impl RSI {
    pub fn new(period: usize) -> Self {
        RSI { period }
    }
}

impl TechnicalIndicator for RSI {
    fn calculate(&self, prices: &[f64]) -> Result<IndicatorValue, String> {
        if prices.len() < self.period + 1 {
            return Err("Not enough data for RSI".to_string());
        }
        // TODO: Implement RSI calculation
        Ok(IndicatorValue::Scalar(50.0))
    }
    
    fn name(&self) -> &str {
        "RSI"
    }
}
