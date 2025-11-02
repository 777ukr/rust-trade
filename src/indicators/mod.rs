pub mod rsi;
pub mod macd;
pub mod bollinger;
pub mod sma;

use serde::{Deserialize, Serialize};

pub trait TechnicalIndicator {
    fn calculate(&self, prices: &[f64]) -> Result<IndicatorValue, String>;
    fn name(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndicatorValue {
    Scalar(f64),
    Vector(Vec<f64>),
    Crossover { signal: String, value: f64 },
}

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
            return Err("Not enough data".to_string());
        }
        // RSI calculation
        todo!()
    }
    
    fn name(&self) -> &str {
        "RSI"
    }
}
