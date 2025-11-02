-- =================================================================
-- PostgreSQL Schema for CryptoTrader Backtesting System
-- Design Principles: High-performance queries, data integrity, scalability
-- =================================================================

-- =================================================================
-- Table 1: Tick Data (Real-time trades)
-- =================================================================
CREATE TABLE IF NOT EXISTS tick_data (
    -- Primary identifier
    id BIGSERIAL PRIMARY KEY,
    
    -- Timestamp with timezone (UTC)
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    
    -- Trading pair symbol (e.g., 'BTCUSDT', 'ETHUSDT')
    symbol VARCHAR(20) NOT NULL,
    
    -- Trade price (DECIMAL for precision)
    price DECIMAL(20, 8) NOT NULL,
    
    -- Trade quantity
    quantity DECIMAL(20, 8) NOT NULL,
    
    -- Trade side: 'BUY' or 'SELL'
    side VARCHAR(4) NOT NULL CHECK (side IN ('BUY', 'SELL')),
    
    -- Exchange trade ID (for deduplication)
    trade_id VARCHAR(50) NOT NULL,
    
    -- Whether buyer is maker
    is_buyer_maker BOOLEAN NOT NULL,
    
    -- Exchange name (for multi-exchange support)
    exchange VARCHAR(20) DEFAULT 'gateio'
);

-- Indexes for tick_data
CREATE INDEX IF NOT EXISTS idx_tick_symbol_time ON tick_data(symbol, timestamp DESC);
CREATE UNIQUE INDEX IF NOT EXISTS idx_tick_unique ON tick_data(symbol, trade_id, timestamp, exchange);
CREATE INDEX IF NOT EXISTS idx_tick_timestamp ON tick_data(timestamp);

-- =================================================================
-- Table 2: OHLCV Data (Candlestick data)
-- =================================================================
CREATE TABLE IF NOT EXISTS ohlcv_data (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    interval VARCHAR(10) NOT NULL, -- '1m', '5m', '15m', '1h', '1d', etc.
    open DECIMAL(20, 8) NOT NULL,
    high DECIMAL(20, 8) NOT NULL,
    low DECIMAL(20, 8) NOT NULL,
    close DECIMAL(20, 8) NOT NULL,
    volume DECIMAL(20, 8) NOT NULL,
    exchange VARCHAR(20) DEFAULT 'gateio'
);

-- Indexes for ohlcv_data
CREATE UNIQUE INDEX IF NOT EXISTS idx_ohlcv_unique ON ohlcv_data(symbol, interval, timestamp, exchange);
CREATE INDEX IF NOT EXISTS idx_ohlcv_symbol_interval_time ON ohlcv_data(symbol, interval, timestamp DESC);

-- =================================================================
-- Table 3: Backtest Results
-- =================================================================
CREATE TABLE IF NOT EXISTS backtest_results (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Strategy info
    strategy_name VARCHAR(50) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    
    -- Initial conditions
    initial_balance DECIMAL(20, 8) NOT NULL,
    leverage INTEGER DEFAULT 1,
    
    -- Performance metrics
    final_balance DECIMAL(20, 8) NOT NULL,
    total_pnl DECIMAL(20, 8) NOT NULL,
    total_fees DECIMAL(20, 8) NOT NULL,
    
    -- Trade statistics
    total_trades INTEGER NOT NULL DEFAULT 0,
    winning_trades INTEGER NOT NULL DEFAULT 0,
    losing_trades INTEGER NOT NULL DEFAULT 0,
    win_rate DECIMAL(10, 4) NOT NULL, -- Percentage
    
    -- Advanced metrics
    roi DECIMAL(10, 4) NOT NULL, -- Return on Investment (percentage)
    profit_factor DECIMAL(10, 4), -- Gross profit / Gross loss
    max_drawdown DECIMAL(10, 4), -- Maximum drawdown (percentage)
    sharpe_ratio DECIMAL(10, 4), -- Risk-adjusted return
    
    -- Time range
    start_time TIMESTAMP WITH TIME ZONE,
    end_time TIMESTAMP WITH TIME ZONE,
    
    -- Configuration (JSON)
    config JSONB,
    
    -- Notes
    notes TEXT
);

-- Indexes for backtest_results
CREATE INDEX IF NOT EXISTS idx_backtest_strategy_symbol ON backtest_results(strategy_name, symbol);
CREATE INDEX IF NOT EXISTS idx_backtest_created_at ON backtest_results(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_backtest_roi ON backtest_results(roi DESC);

-- =================================================================
-- Table 4: Strategy Logs (Detailed trade-by-trade execution)
-- =================================================================
CREATE TABLE IF NOT EXISTS strategy_logs (
    id BIGSERIAL PRIMARY KEY,
    backtest_id BIGINT REFERENCES backtest_results(id) ON DELETE CASCADE,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    
    strategy_name VARCHAR(50) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    
    -- Signal information
    signal_type VARCHAR(50) NOT NULL, -- 'BUY', 'SELL', 'HOLD', 'UPDATE_ORDERS', etc.
    signal_data JSONB, -- Additional signal-specific data
    
    -- Price information
    current_price DECIMAL(20, 8) NOT NULL,
    
    -- Position information
    position_size DECIMAL(20, 8),
    entry_price DECIMAL(20, 8),
    unrealized_pnl DECIMAL(20, 8),
    
    -- Portfolio information
    portfolio_value DECIMAL(20, 8) NOT NULL,
    total_pnl DECIMAL(20, 8) NOT NULL,
    
    -- Performance metrics at this point
    win_rate DECIMAL(10, 4),
    profit_factor DECIMAL(10, 4),
    
    -- Additional metadata
    metadata JSONB
);

-- Indexes for strategy_logs
CREATE INDEX IF NOT EXISTS idx_strategy_logs_backtest_id ON strategy_logs(backtest_id);
CREATE INDEX IF NOT EXISTS idx_strategy_logs_timestamp ON strategy_logs(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_strategy_logs_strategy_symbol ON strategy_logs(strategy_name, symbol);

-- =================================================================
-- Table 5: Account History (Gate.io account snapshots)
-- =================================================================
CREATE TABLE IF NOT EXISTS account_history (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    exchange VARCHAR(20) DEFAULT 'gateio',
    settle VARCHAR(10) NOT NULL, -- 'usdt', 'btc', etc.
    
    -- Balance information
    total DECIMAL(20, 8) NOT NULL,
    available DECIMAL(20, 8) NOT NULL,
    locked DECIMAL(20, 8) NOT NULL,
    
    -- Additional account info (JSON)
    account_data JSONB
);

-- Indexes for account_history
CREATE INDEX IF NOT EXISTS idx_account_history_timestamp ON account_history(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_account_history_exchange_settle ON account_history(exchange, settle);

-- =================================================================
-- Comments for documentation
-- =================================================================
COMMENT ON TABLE tick_data IS 'Real-time tick-by-tick trade data from exchanges';
COMMENT ON TABLE ohlcv_data IS 'Candlestick/OHLCV data aggregated from tick data';
COMMENT ON TABLE backtest_results IS 'Summary results of backtesting runs';
COMMENT ON TABLE strategy_logs IS 'Detailed execution logs for each strategy decision point';
COMMENT ON TABLE account_history IS 'Historical account balance snapshots from exchanges';

