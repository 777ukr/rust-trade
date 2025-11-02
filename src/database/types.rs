//! Database types and data structures

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tick data for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickData {
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub side: String, // "BUY" or "SELL"
    pub trade_id: String,
    pub is_buyer_maker: bool,
    pub exchange: String,
}

/// OHLCV candlestick data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OHLCVData {
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub interval: String, // "1m", "5m", "15m", "1h", "1d"
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub exchange: String,
}

/// Backtest result for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub strategy_name: String,
    pub symbol: String,
    pub initial_balance: Decimal,
    pub leverage: i32,
    pub final_balance: Decimal,
    pub total_pnl: Decimal,
    pub total_fees: Decimal,
    pub total_trades: i32,
    pub winning_trades: i32,
    pub losing_trades: i32,
    pub win_rate: Decimal,
    pub roi: Decimal,
    pub profit_factor: Option<Decimal>,
    pub max_drawdown: Option<Decimal>,
    pub sharpe_ratio: Option<Decimal>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub config: Option<serde_json::Value>,
    pub notes: Option<String>,
}

/// Strategy log entry (detailed execution log)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyLog {
    pub backtest_id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub strategy_name: String,
    pub symbol: String,
    pub signal_type: String,
    pub signal_data: Option<serde_json::Value>,
    pub current_price: Decimal,
    pub position_size: Option<Decimal>,
    pub entry_price: Option<Decimal>,
    pub unrealized_pnl: Option<Decimal>,
    pub portfolio_value: Decimal,
    pub total_pnl: Decimal,
    pub win_rate: Option<Decimal>,
    pub profit_factor: Option<Decimal>,
    pub metadata: Option<serde_json::Value>,
}

/// Account history snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSnapshot {
    pub exchange: String,
    pub settle: String,
    pub total: Decimal,
    pub available: Decimal,
    pub locked: Decimal,
    pub account_data: Option<serde_json::Value>,
}

/// Query parameters for tick data
#[derive(Debug, Clone)]
pub struct TickQuery {
    pub symbol: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub exchange: Option<String>,
}

/// Query parameters for OHLCV data
#[derive(Debug, Clone)]
pub struct OHLCVQuery {
    pub symbol: String,
    pub interval: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub exchange: Option<String>,
}

/// Query parameters for backtest results
#[derive(Debug, Clone)]
pub struct BacktestQuery {
    pub strategy_name: Option<String>,
    pub symbol: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub min_roi: Option<Decimal>,
    pub limit: Option<i64>,
}

