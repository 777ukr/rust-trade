use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskAction {
    None,
    StopTrading,
    PanicSell,
}

#[derive(Debug, Clone)]
pub struct GlobalRiskManager {
    pub max_loss_per_trades: Option<(f64, usize)>,
    pub max_loss_per_hours: Option<(f64, u32, usize)>,
    pub auto_reset_interval_hours: Option<u32>,
    pub panic_sell_on_btc_delta: Option<(f64, f64)>, // (drop, raise)
    pub panic_sell_on_market_delta: Option<f64>,

    pub session_start_time: DateTime<Utc>,
    pub session_trades: usize,
    pub current_session_loss: f64,
}

impl GlobalRiskManager {
    pub fn new() -> Self {
        Self {
            max_loss_per_trades: None,
            max_loss_per_hours: None,
            auto_reset_interval_hours: None,
            panic_sell_on_btc_delta: None,
            panic_sell_on_market_delta: None,
            session_start_time: Utc::now(),
            session_trades: 0,
            current_session_loss: 0.0,
        }
    }

    pub fn record_trade_pnl(&mut self, pnl: f64) {
        self.current_session_loss += pnl;
        self.session_trades += 1;
    }

    pub fn maybe_reset_session(&mut self, now: DateTime<Utc>) {
        if let Some(h) = self.auto_reset_interval_hours {
            let elapsed = now - self.session_start_time;
            if elapsed >= Duration::hours(h as i64) {
                self.session_start_time = now;
                self.session_trades = 0;
                self.current_session_loss = 0.0;
            }
        }
    }

    pub fn check_stop_conditions(&self) -> RiskAction {
        if let Some((max_loss, min_trades)) = self.max_loss_per_trades {
            if self.session_trades >= min_trades && self.current_session_loss <= -max_loss {
                return RiskAction::StopTrading;
            }
        }
        if let Some((max_loss, _hours, min_trades)) = self.max_loss_per_hours {
            if self.session_trades >= min_trades && self.current_session_loss <= -max_loss {
                return RiskAction::StopTrading;
            }
        }
        RiskAction::None
    }

    pub fn check_btc_delta_panic(&self, btc_delta_1h: f64) -> bool {
        if let Some((drop, rise)) = self.panic_sell_on_btc_delta {
            return btc_delta_1h <= -drop || btc_delta_1h >= rise;
        }
        false
    }

    pub fn check_market_delta_panic(&self, market_delta_1h: f64) -> bool {
        if let Some(drop) = self.panic_sell_on_market_delta {
            return market_delta_1h <= -drop;
        }
        false
    }
}
