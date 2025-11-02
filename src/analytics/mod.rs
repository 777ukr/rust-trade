//! Аналитика торговли - оценка успешности стратегий
//! Анализ исторических данных и оценка производительности

pub mod performance;
pub mod trade_analyzer;
pub mod strategy_comparator;
pub mod log_analyzer;
pub mod channel_analyzer;

pub use performance::*;
pub use trade_analyzer::*;
pub use strategy_comparator::*;
pub use log_analyzer::*;
pub use channel_analyzer::*;

