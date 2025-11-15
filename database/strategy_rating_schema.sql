-- =================================================================
-- Strategy Rating System Schema (Ninja-style ranking)
-- Extends existing backtest_results table with rating metrics
-- =================================================================

-- =================================================================
-- Table: strategy_ratings
-- Stores aggregated rating metrics for strategies across multiple backtests
-- =================================================================
CREATE TABLE IF NOT EXISTS strategy_ratings (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Strategy identification
    strategy_name VARCHAR(100) NOT NULL,
    exchange VARCHAR(20) NOT NULL DEFAULT 'gateio',
    stake_currency VARCHAR(10) NOT NULL DEFAULT 'USDT',
    
    -- Aggregated metrics (median values across all backtests)
    total_backtests INTEGER NOT NULL DEFAULT 0,
    
    -- Trade statistics (median)
    median_buys INTEGER,
    median_total_trades INTEGER,
    median_winning_trades INTEGER,
    median_losing_trades INTEGER,
    median_win_rate DECIMAL(10, 4), -- Percentage
    
    -- Profit metrics (median)
    median_avg_profit DECIMAL(10, 4), -- Average profit per trade (%)
    median_total_profit_pct DECIMAL(10, 4), -- Total profit percentage
    median_roi DECIMAL(10, 4),
    
    -- Risk metrics (median)
    median_max_drawdown DECIMAL(10, 4), -- Max drawdown (%)
    median_sharpe_ratio DECIMAL(10, 4),
    median_sortino_ratio DECIMAL(10, 4),
    median_calmar_ratio DECIMAL(10, 4),
    
    -- Advanced metrics (median)
    median_profit_factor DECIMAL(10, 4),
    median_expectancy DECIMAL(10, 4),
    median_cagr DECIMAL(10, 4), -- Compound Annual Growth Rate
    
    -- Rejected signals (median)
    median_rejected_signals INTEGER,
    
    -- Backtest win percentage (how often backtest was profitable)
    backtest_win_percentage DECIMAL(10, 4),
    
    -- Ninja Score (calculated weighted score)
    ninja_score DECIMAL(10, 4) NOT NULL DEFAULT 0,
    
    -- Filters and flags
    has_lookahead_bias BOOLEAN DEFAULT FALSE,
    has_tight_trailing_stop BOOLEAN DEFAULT FALSE,
    leverage INTEGER DEFAULT 1,
    min_trades_threshold INTEGER DEFAULT 10, -- Minimum trades to be considered
    
    -- Strategy metadata
    strategy_hash VARCHAR(64), -- SHA256 hash of strategy file
    strategy_file_path TEXT,
    last_backtest_date TIMESTAMP WITH TIME ZONE,
    
    -- Status
    is_stalled BOOLEAN DEFAULT FALSE,
    stall_reason VARCHAR(50), -- 'negative', '90_percent_negative', 'biased', 'no_trades'
    is_active BOOLEAN DEFAULT TRUE,
    
    -- Additional metadata (JSON)
    metadata JSONB,
    
    -- Constraints
    UNIQUE(strategy_name, exchange, stake_currency)
);

-- Indexes for strategy_ratings
CREATE INDEX IF NOT EXISTS idx_strategy_ratings_ninja_score ON strategy_ratings(ninja_score DESC);
CREATE INDEX IF NOT EXISTS idx_strategy_ratings_strategy_name ON strategy_ratings(strategy_name);
CREATE INDEX IF NOT EXISTS idx_strategy_ratings_active ON strategy_ratings(is_active, is_stalled);
CREATE INDEX IF NOT EXISTS idx_strategy_ratings_exchange_stake ON strategy_ratings(exchange, stake_currency);
CREATE INDEX IF NOT EXISTS idx_strategy_ratings_updated_at ON strategy_ratings(updated_at DESC);

-- =================================================================
-- Table: strategy_backtest_metrics
-- Detailed metrics for each individual backtest (for median calculation)
-- =================================================================
CREATE TABLE IF NOT EXISTS strategy_backtest_metrics (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Link to backtest_results
    backtest_result_id BIGINT REFERENCES backtest_results(id) ON DELETE CASCADE,
    
    -- Link to strategy_ratings
    strategy_rating_id BIGINT REFERENCES strategy_ratings(id) ON DELETE CASCADE,
    
    -- Strategy info
    strategy_name VARCHAR(100) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    exchange VARCHAR(20) NOT NULL,
    stake_currency VARCHAR(10) NOT NULL,
    
    -- Detailed metrics for this backtest
    buys INTEGER,
    total_trades INTEGER NOT NULL,
    winning_trades INTEGER,
    losing_trades INTEGER,
    win_rate DECIMAL(10, 4),
    
    avg_profit DECIMAL(10, 4), -- Average profit per trade (%)
    total_profit_pct DECIMAL(10, 4), -- Total profit percentage
    roi DECIMAL(10, 4),
    
    max_drawdown DECIMAL(10, 4),
    sharpe_ratio DECIMAL(10, 4),
    sortino_ratio DECIMAL(10, 4),
    calmar_ratio DECIMAL(10, 4),
    
    profit_factor DECIMAL(10, 4),
    expectancy DECIMAL(10, 4),
    cagr DECIMAL(10, 4),
    
    rejected_signals INTEGER,
    
    -- Flags
    has_lookahead_bias BOOLEAN DEFAULT FALSE,
    has_tight_trailing_stop BOOLEAN DEFAULT FALSE,
    leverage INTEGER DEFAULT 1,
    
    -- Additional metadata
    metadata JSONB
);

-- Indexes for strategy_backtest_metrics
CREATE INDEX IF NOT EXISTS idx_backtest_metrics_strategy ON strategy_backtest_metrics(strategy_name, symbol);
CREATE INDEX IF NOT EXISTS idx_backtest_metrics_rating_id ON strategy_backtest_metrics(strategy_rating_id);
CREATE INDEX IF NOT EXISTS idx_backtest_metrics_backtest_id ON strategy_backtest_metrics(backtest_result_id);
CREATE INDEX IF NOT EXISTS idx_backtest_metrics_created_at ON strategy_backtest_metrics(created_at DESC);

-- =================================================================
-- Table: strategy_ranking_history
-- Historical ranking snapshots (for tracking changes over time)
-- =================================================================
CREATE TABLE IF NOT EXISTS strategy_ranking_history (
    id BIGSERIAL PRIMARY KEY,
    snapshot_date DATE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    strategy_rating_id BIGINT REFERENCES strategy_ratings(id) ON DELETE CASCADE,
    strategy_name VARCHAR(100) NOT NULL,
    
    -- Ranking position
    rank_position INTEGER,
    ninja_score DECIMAL(10, 4),
    
    -- Key metrics at this snapshot
    total_backtests INTEGER,
    median_win_rate DECIMAL(10, 4),
    median_total_profit_pct DECIMAL(10, 4),
    median_profit_factor DECIMAL(10, 4),
    
    -- Metadata
    metadata JSONB
);

-- Indexes for strategy_ranking_history
CREATE INDEX IF NOT EXISTS idx_ranking_history_date ON strategy_ranking_history(snapshot_date DESC);
CREATE INDEX IF NOT EXISTS idx_ranking_history_strategy ON strategy_ranking_history(strategy_rating_id);
CREATE INDEX IF NOT EXISTS idx_ranking_history_rank ON strategy_ranking_history(snapshot_date, rank_position);

-- =================================================================
-- Function: Update updated_at timestamp
-- =================================================================
CREATE OR REPLACE FUNCTION update_strategy_ratings_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_strategy_ratings_updated_at
    BEFORE UPDATE ON strategy_ratings
    FOR EACH ROW
    EXECUTE FUNCTION update_strategy_ratings_updated_at();

-- =================================================================
-- Comments for documentation
-- =================================================================
COMMENT ON TABLE strategy_ratings IS 'Aggregated rating metrics for strategies (Ninja-style ranking)';
COMMENT ON TABLE strategy_backtest_metrics IS 'Detailed metrics for each backtest (used for median calculation)';
COMMENT ON TABLE strategy_ranking_history IS 'Historical snapshots of strategy rankings';

COMMENT ON COLUMN strategy_ratings.ninja_score IS 'Weighted score calculated using Ninja weights: buys(9), avgprof(26), totprofp(26), winp(24), ddp(-25), stoploss(7), sharpe(7), sortino(7), calmar(7), expectancy(8), profit_factor(9), cagr(10), rejected_signals(-25), backtest_win_percentage(10)';
COMMENT ON COLUMN strategy_ratings.has_lookahead_bias IS 'Detected lookahead bias (e.g., using .iat[-1], .shift(-1), whole dataframe operations)';
COMMENT ON COLUMN strategy_ratings.has_tight_trailing_stop IS 'Tight trailing stop detected (trailing_stop_positive_offset <= 0.05 & trailing_stop_positive <= 0.0025)';
COMMENT ON COLUMN strategy_ratings.leverage IS 'Leverage used (only 1x strategies are ranked)';



