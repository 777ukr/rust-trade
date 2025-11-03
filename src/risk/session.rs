use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionAction {
    None,
    BlockTrading,
}

#[derive(Debug, Clone)]
pub struct SessionState {
    pub pnl: f64,
    pub trades_count: usize,
    pub start_time: DateTime<Utc>,
    pub last_reset: DateTime<Utc>,
    pub auto_reset_interval: Option<Duration>,
    pub max_loss_per_trades: Option<(f64, usize)>,
    pub max_loss_per_time: Option<(f64, Duration, usize)>,
    pub order_size_multiplier: f64,
    pub penalty_until: Option<DateTime<Utc>>,
}

impl Default for SessionState {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            pnl: 0.0,
            trades_count: 0,
            start_time: now,
            last_reset: now,
            auto_reset_interval: None,
            max_loss_per_trades: None,
            max_loss_per_time: None,
            order_size_multiplier: 1.0,
            penalty_until: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct SessionManager {
    sessions: HashMap<String, SessionState>,
}

impl SessionManager {
    pub fn new() -> Self { Self { sessions: HashMap::new() } }

    pub fn update_session(&mut self, key: &str, pnl_delta: f64) {
        let entry = self.sessions.entry(key.to_string()).or_default();
        entry.pnl += pnl_delta;
        entry.trades_count += 1;
    }

    pub fn check_stop_conditions(&self, key: &str) -> SessionAction {
        if let Some(state) = self.sessions.get(key) {
            if let Some((max_loss, min_trades)) = state.max_loss_per_trades {
                if state.trades_count >= min_trades && state.pnl <= -max_loss { return SessionAction::BlockTrading; }
            }
            if let Some((max_loss, window, min_trades)) = state.max_loss_per_time {
                if state.trades_count >= min_trades {
                    if Utc::now() - state.last_reset <= window && state.pnl <= -max_loss { return SessionAction::BlockTrading; }
                }
            }
        }
        SessionAction::None
    }

    pub fn should_reset(&self, key: &str) -> bool {
        if let Some(state) = self.sessions.get(key) {
            if let Some(interval) = state.auto_reset_interval {
                return Utc::now() - state.last_reset >= interval;
            }
        }
        false
    }

    pub fn get_order_size_multiplier(&self, key: &str) -> f64 {
        self.sessions.get(key).map(|s| s.order_size_multiplier).unwrap_or(1.0)
    }

    pub fn reset(&mut self, key: &str) {
        if let Some(state) = self.sessions.get_mut(key) {
            state.pnl = 0.0;
            state.trades_count = 0;
            state.last_reset = Utc::now();
        }
    }
}
