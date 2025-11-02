//! Database repository for PostgreSQL operations
//! Implements repository pattern for data persistence

use crate::database::types::*;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde_json::Value;
use sqlx::{PgPool, FromRow};
use std::sync::Arc;

// Intermediate structs for querying results
#[derive(FromRow)]
struct BacktestResultRow {
    strategy_name: String,
    symbol: String,
    initial_balance: Decimal,
    leverage: i32,
    final_balance: Decimal,
    total_pnl: Decimal,
    total_fees: Decimal,
    total_trades: i32,
    winning_trades: i32,
    losing_trades: i32,
    win_rate: Decimal,
    roi: Decimal,
    profit_factor: Option<Decimal>,
    max_drawdown: Option<Decimal>,
    sharpe_ratio: Option<Decimal>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    config: Option<Value>,
    notes: Option<String>,
}

#[derive(FromRow)]
struct StrategyLogRow {
    backtest_id: Option<i64>,
    timestamp: DateTime<Utc>,
    strategy_name: String,
    symbol: String,
    signal_type: String,
    signal_data: Option<Value>,
    current_price: Decimal,
    position_size: Option<Decimal>,
    entry_price: Option<Decimal>,
    unrealized_pnl: Option<Decimal>,
    portfolio_value: Decimal,
    total_pnl: Decimal,
    win_rate: Option<Decimal>,
    profit_factor: Option<Decimal>,
    metadata: Option<Value>,
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

/// Main database repository
pub struct DatabaseRepository {
    pool: Arc<PgPool>,
}

impl DatabaseRepository {
    /// Create new repository instance
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }

    /// Create database pool from connection string
    pub async fn create_pool(database_url: &str) -> Result<PgPool> {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect(database_url)
            .await
            .context("Failed to connect to PostgreSQL database")
    }

    /// Test database connection
    pub async fn test_connection(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(self.pool.as_ref())
            .await
            .context("Database connection test failed")?;
        Ok(())
    }

    // =================================================================
    // Tick Data Operations
    // =================================================================

    /// Insert single tick data
    pub async fn insert_tick(&self, tick: &TickData) -> Result<i64> {
        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO tick_data (timestamp, symbol, price, quantity, side, trade_id, is_buyer_maker, exchange)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (symbol, trade_id, timestamp, exchange) DO NOTHING
            RETURNING id
            "#,
        )
        .bind(tick.timestamp)
        .bind(&tick.symbol)
        .bind(tick.price)
        .bind(tick.quantity)
        .bind(&tick.side)
        .bind(&tick.trade_id)
        .bind(tick.is_buyer_maker)
        .bind(&tick.exchange)
        .fetch_optional(self.pool.as_ref())
        .await
        .context("Failed to insert tick data")?
        .unwrap_or(0);

        Ok(id)
    }

    /// Batch insert tick data (more efficient)
    pub async fn insert_ticks_batch(&self, ticks: &[TickData]) -> Result<usize> {
        if ticks.is_empty() {
            return Ok(0);
        }

        let mut inserted = 0;
        for tick in ticks {
            match self.insert_tick(tick).await {
                Ok(id) if id > 0 => inserted += 1,
                Ok(_) => {} // Duplicate, skipped
                Err(e) => {
                    eprintln!("Warning: Failed to insert tick {:?}: {}", tick.trade_id, e);
                }
            }
        }

        Ok(inserted)
    }

    /// Query tick data
    pub async fn query_ticks(&self, query: &TickQuery) -> Result<Vec<TickData>> {
        let mut sql = String::from(
            "SELECT timestamp, symbol, price, quantity, side, trade_id, is_buyer_maker, exchange 
             FROM tick_data WHERE symbol = $1",
        );

        let mut bind_idx = 2;

        if let Some(start) = query.start_time {
            sql.push_str(&format!(" AND timestamp >= ${}", bind_idx));
            bind_idx += 1;
        }
        if let Some(end) = query.end_time {
            sql.push_str(&format!(" AND timestamp <= ${}", bind_idx));
            bind_idx += 1;
        }
        if let Some(exch) = &query.exchange {
            sql.push_str(&format!(" AND exchange = ${}", bind_idx));
            bind_idx += 1;
        }

        sql.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let mut query_builder = sqlx::query_as::<_, (DateTime<Utc>, String, Decimal, Decimal, String, String, bool, String)>(&sql)
            .bind(&query.symbol);

        if let Some(start) = query.start_time {
            query_builder = query_builder.bind(start);
        }
        if let Some(end) = query.end_time {
            query_builder = query_builder.bind(end);
        }
        if let Some(exch) = &query.exchange {
            query_builder = query_builder.bind(exch);
        }

        let rows = query_builder
            .fetch_all(self.pool.as_ref())
            .await
            .context("Failed to query tick data")?;

        let ticks = rows
            .into_iter()
            .map(|(ts, sym, price, qty, side, tid, maker, exch)| TickData {
                timestamp: ts,
                symbol: sym,
                price,
                quantity: qty,
                side,
                trade_id: tid,
                is_buyer_maker: maker,
                exchange: exch,
            })
            .collect();

        Ok(ticks)
    }

    // =================================================================
    // OHLCV Data Operations
    // =================================================================

    /// Insert OHLCV candlestick
    pub async fn insert_ohlcv(&self, ohlcv: &OHLCVData) -> Result<i64> {
        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO ohlcv_data (timestamp, symbol, interval, open, high, low, close, volume, exchange)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (symbol, interval, timestamp, exchange) DO UPDATE SET
                open = EXCLUDED.open,
                high = EXCLUDED.high,
                low = EXCLUDED.low,
                close = EXCLUDED.close,
                volume = EXCLUDED.volume
            RETURNING id
            "#,
        )
        .bind(ohlcv.timestamp)
        .bind(&ohlcv.symbol)
        .bind(&ohlcv.interval)
        .bind(ohlcv.open)
        .bind(ohlcv.high)
        .bind(ohlcv.low)
        .bind(ohlcv.close)
        .bind(ohlcv.volume)
        .bind(&ohlcv.exchange)
        .fetch_one(self.pool.as_ref())
        .await
        .context("Failed to insert OHLCV data")?;

        Ok(id)
    }

    /// Query OHLCV data
    pub async fn query_ohlcv(&self, query: &OHLCVQuery) -> Result<Vec<OHLCVData>> {
        let mut sql = String::from(
            "SELECT timestamp, symbol, interval, open, high, low, close, volume, exchange 
             FROM ohlcv_data WHERE symbol = $1 AND interval = $2",
        );

        let mut bind_idx = 3;

        if let Some(start) = query.start_time {
            sql.push_str(&format!(" AND timestamp >= ${}", bind_idx));
            bind_idx += 1;
        }
        if let Some(end) = query.end_time {
            sql.push_str(&format!(" AND timestamp <= ${}", bind_idx));
            bind_idx += 1;
        }
        if let Some(exch) = &query.exchange {
            sql.push_str(&format!(" AND exchange = ${}", bind_idx));
            bind_idx += 1;
        }

        sql.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let mut query_builder = sqlx::query_as::<_, (DateTime<Utc>, String, String, Decimal, Decimal, Decimal, Decimal, Decimal, String)>(&sql)
            .bind(&query.symbol)
            .bind(&query.interval);

        if let Some(start) = query.start_time {
            query_builder = query_builder.bind(start);
        }
        if let Some(end) = query.end_time {
            query_builder = query_builder.bind(end);
        }
        if let Some(exch) = &query.exchange {
            query_builder = query_builder.bind(exch);
        }

        let rows = query_builder
            .fetch_all(self.pool.as_ref())
            .await
            .context("Failed to query OHLCV data")?;

        let ohlcvs = rows
            .into_iter()
            .map(|(ts, sym, iv, o, h, l, c, v, exch)| OHLCVData {
                timestamp: ts,
                symbol: sym,
                interval: iv,
                open: o,
                high: h,
                low: l,
                close: c,
                volume: v,
                exchange: exch,
            })
            .collect();

        Ok(ohlcvs)
    }

    // =================================================================
    // Backtest Results Operations
    // =================================================================

    /// Insert backtest result
    pub async fn insert_backtest_result(&self, result: &BacktestResult) -> Result<i64> {
        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO backtest_results (
                strategy_name, symbol, initial_balance, leverage,
                final_balance, total_pnl, total_fees,
                total_trades, winning_trades, losing_trades, win_rate,
                roi, profit_factor, max_drawdown, sharpe_ratio,
                start_time, end_time, config, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            RETURNING id
            "#,
        )
        .bind(&result.strategy_name)
        .bind(&result.symbol)
        .bind(result.initial_balance)
        .bind(result.leverage)
        .bind(result.final_balance)
        .bind(result.total_pnl)
        .bind(result.total_fees)
        .bind(result.total_trades)
        .bind(result.winning_trades)
        .bind(result.losing_trades)
        .bind(result.win_rate)
        .bind(result.roi)
        .bind(result.profit_factor)
        .bind(result.max_drawdown)
        .bind(result.sharpe_ratio)
        .bind(result.start_time)
        .bind(result.end_time)
        .bind(result.config.as_ref())
        .bind(result.notes.as_ref())
        .fetch_one(self.pool.as_ref())
        .await
        .context("Failed to insert backtest result")?;

        Ok(id)
    }

    /// Query backtest results
    pub async fn query_backtest_results(&self, query: &BacktestQuery) -> Result<Vec<BacktestResult>> {
        let mut sql_params: Vec<String> = Vec::new();
        let mut bind_idx = 1;

        if let Some(strategy) = &query.strategy_name {
            sql_params.push(format!("strategy_name = ${}", bind_idx));
            bind_idx += 1;
        }
        if let Some(symbol) = &query.symbol {
            sql_params.push(format!("symbol = ${}", bind_idx));
            bind_idx += 1;
        }
        if let Some(start) = query.start_date {
            sql_params.push(format!("created_at >= ${}", bind_idx));
            bind_idx += 1;
        }
        if let Some(end) = query.end_date {
            sql_params.push(format!("created_at <= ${}", bind_idx));
            bind_idx += 1;
        }
        if let Some(min_roi) = query.min_roi {
            sql_params.push(format!("roi >= ${}", bind_idx));
            bind_idx += 1;
        }

        let where_clause = if sql_params.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", sql_params.join(" AND "))
        };

        let limit_clause = if let Some(limit) = query.limit {
            format!("LIMIT {}", limit)
        } else {
            String::new()
        };

        let sql = format!(
            "SELECT strategy_name, symbol, initial_balance, leverage,
                    final_balance, total_pnl, total_fees,
                    total_trades, winning_trades, losing_trades, win_rate,
                    roi, profit_factor, max_drawdown, sharpe_ratio,
                    start_time, end_time, config, notes
             FROM backtest_results
             {}
             ORDER BY created_at DESC
             {}",
            where_clause, limit_clause
        );

        let mut query_builder = sqlx::query_as::<_, BacktestResultRow>(&sql);

        if let Some(strategy) = &query.strategy_name {
            query_builder = query_builder.bind(strategy);
        }
        if let Some(symbol) = &query.symbol {
            query_builder = query_builder.bind(symbol);
        }
        if let Some(start) = query.start_date {
            query_builder = query_builder.bind(start);
        }
        if let Some(end) = query.end_date {
            query_builder = query_builder.bind(end);
        }
        if let Some(min_roi) = query.min_roi {
            query_builder = query_builder.bind(min_roi);
        }

        let rows = query_builder
            .fetch_all(self.pool.as_ref())
            .await
            .context("Failed to query backtest results")?;

        let results = rows
            .into_iter()
            .map(|row| BacktestResult {
                strategy_name: row.strategy_name,
                symbol: row.symbol,
                initial_balance: row.initial_balance,
                leverage: row.leverage,
                final_balance: row.final_balance,
                total_pnl: row.total_pnl,
                total_fees: row.total_fees,
                total_trades: row.total_trades,
                winning_trades: row.winning_trades,
                losing_trades: row.losing_trades,
                win_rate: row.win_rate,
                roi: row.roi,
                profit_factor: row.profit_factor,
                max_drawdown: row.max_drawdown,
                sharpe_ratio: row.sharpe_ratio,
                start_time: row.start_time,
                end_time: row.end_time,
                config: row.config,
                notes: row.notes,
            })
            .collect();

        Ok(results)
    }

    // =================================================================
    // Strategy Logs Operations
    // =================================================================

    /// Insert strategy log
    pub async fn insert_strategy_log(&self, log: &StrategyLog) -> Result<i64> {
        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO strategy_logs (
                backtest_id, timestamp, strategy_name, symbol,
                signal_type, signal_data, current_price,
                position_size, entry_price, unrealized_pnl,
                portfolio_value, total_pnl, win_rate, profit_factor, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id
            "#,
        )
        .bind(log.backtest_id)
        .bind(log.timestamp)
        .bind(&log.strategy_name)
        .bind(&log.symbol)
        .bind(&log.signal_type)
        .bind(log.signal_data.as_ref())
        .bind(log.current_price)
        .bind(log.position_size)
        .bind(log.entry_price)
        .bind(log.unrealized_pnl)
        .bind(log.portfolio_value)
        .bind(log.total_pnl)
        .bind(log.win_rate)
        .bind(log.profit_factor)
        .bind(log.metadata.as_ref())
        .fetch_one(self.pool.as_ref())
        .await
        .context("Failed to insert strategy log")?;

        Ok(id)
    }

    /// Batch insert strategy logs
    pub async fn insert_strategy_logs_batch(&self, logs: &[StrategyLog]) -> Result<usize> {
        if logs.is_empty() {
            return Ok(0);
        }

        let mut inserted = 0;
        for log in logs {
            match self.insert_strategy_log(log).await {
                Ok(_) => inserted += 1,
                Err(e) => {
                    eprintln!("Warning: Failed to insert strategy log: {}", e);
                }
            }
        }

        Ok(inserted)
    }

    /// Query strategy logs by backtest ID
    pub async fn query_strategy_logs(&self, backtest_id: i64) -> Result<Vec<StrategyLog>> {
        let rows = sqlx::query_as::<_, StrategyLogRow>(
            r#"
            SELECT backtest_id, timestamp, strategy_name, symbol,
                   signal_type, signal_data, current_price,
                   position_size, entry_price, unrealized_pnl,
                   portfolio_value, total_pnl, win_rate, profit_factor, metadata
            FROM strategy_logs
            WHERE backtest_id = $1
            ORDER BY timestamp ASC
            "#,
        )
        .bind(backtest_id)
        .fetch_all(self.pool.as_ref())
        .await
        .context("Failed to query strategy logs")?;

        let logs = rows
            .into_iter()
            .map(|row| StrategyLog {
                backtest_id: row.backtest_id,
                timestamp: row.timestamp,
                strategy_name: row.strategy_name,
                symbol: row.symbol,
                signal_type: row.signal_type,
                signal_data: row.signal_data,
                current_price: row.current_price,
                position_size: row.position_size,
                entry_price: row.entry_price,
                unrealized_pnl: row.unrealized_pnl,
                portfolio_value: row.portfolio_value,
                total_pnl: row.total_pnl,
                win_rate: row.win_rate,
                profit_factor: row.profit_factor,
                metadata: row.metadata,
            })
            .collect();

        Ok(logs)
    }
}

