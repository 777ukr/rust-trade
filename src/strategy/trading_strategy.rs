// Trading strategy implementation
use crate::strategy::TradingStrategy;
use crate::models::CryptoPair;

impl TradingStrategy {
    pub fn with_stop_loss(mut self, percent: f64) -> Self {
        self.stop_loss_percent = percent;
        self
    }
    
    pub fn with_take_profit(mut self, percent: f64) -> Self {
        self.take_profit_percent = percent;
        self
    }
}
