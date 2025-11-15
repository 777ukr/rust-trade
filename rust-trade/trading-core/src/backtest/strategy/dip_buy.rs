use super::base::{Signal, Strategy};
use crate::data::types::TickData;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

/// Low-frequency dip buying strategy
/// 
/// Strategy logic:
/// - Buy when price drops by buy_dip_percent from recent high
/// - Sell when price rises by take_profit_percent from buy price
/// - Stop loss when price drops by stop_loss_percent from buy price
pub struct DipBuyStrategy {
    // Strategy parameters
    buy_dip_percent: Decimal,      // 0.2% = 0.002
    take_profit_percent: Decimal,   // 0.6% = 0.006
    stop_loss_percent: Decimal,     // 0.22% = 0.0022
    
    // State tracking
    recent_high: Option<Decimal>,   // Highest price since last buy
    buy_price: Option<Decimal>,      // Price at which we bought
    position: bool,                 // Whether we have an open position
    symbol: Option<String>,        // Trading symbol
}

impl DipBuyStrategy {
    pub fn new() -> Self {
        Self {
            buy_dip_percent: dec!(0.002),   // 0.2% default
            take_profit_percent: dec!(0.006), // 0.6% default
            stop_loss_percent: dec!(0.0022),  // 0.22% default
            recent_high: None,
            buy_price: None,
            position: false,
            symbol: None,
        }
    }
}

impl Strategy for DipBuyStrategy {
    fn name(&self) -> &str {
        "Dip Buy Strategy (Low Frequency)"
    }

    fn initialize(&mut self, params: HashMap<String, String>) -> Result<(), String> {
        if let Some(buy_dip) = params.get("buy_dip_percent") {
            let value: f64 = buy_dip.parse().map_err(|_| "Invalid buy_dip_percent")?;
            self.buy_dip_percent = Decimal::try_from(value / 100.0)
                .map_err(|_| "buy_dip_percent must be a valid decimal")?;
        }
        
        if let Some(take_profit) = params.get("take_profit_percent") {
            let value: f64 = take_profit.parse().map_err(|_| "Invalid take_profit_percent")?;
            self.take_profit_percent = Decimal::try_from(value / 100.0)
                .map_err(|_| "take_profit_percent must be a valid decimal")?;
        }
        
        if let Some(stop_loss) = params.get("stop_loss_percent") {
            let value: f64 = stop_loss.parse().map_err(|_| "Invalid stop_loss_percent")?;
            self.stop_loss_percent = Decimal::try_from(value / 100.0)
                .map_err(|_| "stop_loss_percent must be a valid decimal")?;
        }

        println!(
            "Dip Buy Strategy initialized: buy_dip={:.2}%, take_profit={:.2}%, stop_loss={:.2}%",
            self.buy_dip_percent * dec!(100),
            self.take_profit_percent * dec!(100),
            self.stop_loss_percent * dec!(100)
        );
        Ok(())
    }

    fn reset(&mut self) {
        self.recent_high = None;
        self.buy_price = None;
        self.position = false;
        self.symbol = None;
    }

    fn on_tick(&mut self, tick: &TickData) -> Signal {
        // Store symbol on first tick
        if self.symbol.is_none() {
            self.symbol = Some(tick.symbol.clone());
        }

        // If we have a position, check for take profit or stop loss
        if self.position {
            if let Some(buy_price) = self.buy_price {
                let price_change = (tick.price - buy_price) / buy_price;
                
                // Take profit: sell when price rises by take_profit_percent
                if price_change >= self.take_profit_percent {
                    self.position = false;
                    self.buy_price = None;
                    // Reset recent_high to current price to start tracking new high
                    self.recent_high = Some(tick.price);
                    return Signal::Sell {
                        symbol: tick.symbol.clone(),
                        quantity: Decimal::from(100), // Use all position
                    };
                }
                
                // Stop loss: sell when price drops by stop_loss_percent
                if price_change <= -self.stop_loss_percent {
                    self.position = false;
                    self.buy_price = None;
                    // Reset recent_high to current price to start tracking new high
                    self.recent_high = Some(tick.price);
                    return Signal::Sell {
                        symbol: tick.symbol.clone(),
                        quantity: Decimal::from(100), // Use all position
                    };
                }
            }
        } else {
            // No position - look for buy opportunity
            
            // Update recent high
            match self.recent_high {
                Some(high) => {
                    if tick.price > high {
                        self.recent_high = Some(tick.price);
                    }
                }
                None => {
                    self.recent_high = Some(tick.price);
                }
            }
            
            // Check if price has dropped by buy_dip_percent from recent high
            if let Some(high) = self.recent_high {
                let drop = (high - tick.price) / high;
                
                if drop >= self.buy_dip_percent {
                    // Buy signal: price dropped by buy_dip_percent
                    self.position = true;
                    self.buy_price = Some(tick.price);
                    // Keep recent_high to track new highs after buy
                    return Signal::Buy {
                        symbol: tick.symbol.clone(),
                        quantity: Decimal::from(100),
                    };
                }
            }
        }

        Signal::Hold
    }
}

