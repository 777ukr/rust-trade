// Simple Moving Average indicator implementation
use crate::indicators::{TechnicalIndicator, IndicatorValue};

pub struct SMA {
    period: usize,
}

impl SMA {
    pub fn new(period: usize) -> Self {
        SMA { period }
    }
}

impl TechnicalIndicator for SMA {
    fn calculate(&self, prices: &[f64]) -> Result<IndicatorValue, String> {
        if prices.len() < self.period {
            return Err("Not enough data for SMA".to_string());
        }
        let sum: f64 = prices.iter().take(self.period).sum();
        Ok(IndicatorValue::Scalar(sum / self.period as f64))
    }
    
    fn name(&self) -> &str {
        "SMA"
    }
}
