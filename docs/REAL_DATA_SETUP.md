# 🔄 Настройка реальных данных Gate.io

## Архитектура (как в rust-trade)

```
Gate.io API → Data Collector → PostgreSQL → Backtest Engine → Results
                ↓
            Redis Cache
```

## 1. PostgreSQL Схема

```sql
-- OHLCV свечи (для анализа каналов, дельты, индикаторов)
CREATE TABLE ohlcv_data (
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    interval VARCHAR(10) NOT NULL, -- '1m', '5m', '15m', '1h', '4h', '1d'
    open DECIMAL(20, 8) NOT NULL,
    high DECIMAL(20, 8) NOT NULL,
    low DECIMAL(20, 8) NOT NULL,
    close DECIMAL(20, 8) NOT NULL,
    volume DECIMAL(20, 8) NOT NULL,
    quote_volume DECIMAL(20, 8), -- объем в USDT
    trades_count INTEGER,
    PRIMARY KEY (timestamp, symbol, interval)
);

-- Индикаторы (предрассчитанные для ускорения)
CREATE TABLE indicators (
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    interval VARCHAR(10) NOT NULL,
    rsi_14 DECIMAL(10, 4),
    macd DECIMAL(20, 8),
    macd_signal DECIMAL(20, 8),
    macd_histogram DECIMAL(20, 8),
    bollinger_upper DECIMAL(20, 8),
    bollinger_middle DECIMAL(20, 8),
    bollinger_lower DECIMAL(20, 8),
    channel_high DECIMAL(20, 8),
    channel_low DECIMAL(20, 8),
    volume_delta DECIMAL(20, 8), -- дельта объема
    volatility DECIMAL(10, 4), -- волатильность
    PRIMARY KEY (timestamp, symbol, interval)
);

-- Индексы для быстрого поиска
CREATE INDEX idx_ohlcv_symbol_time ON ohlcv_data(symbol, timestamp DESC);
CREATE INDEX idx_ohlcv_symbol_interval ON ohlcv_data(symbol, interval, timestamp DESC);
CREATE INDEX idx_indicators_symbol_time ON indicators(symbol, timestamp DESC);
```

## 2. Переменные окружения

```bash
# .env
DATABASE_URL=postgresql://user:password@localhost:5432/cryptotrader
REDIS_URL=redis://127.0.0.1:6379
GATEIO_API_KEY=your_key
GATEIO_SECRET_KEY=your_secret
```

## 3. Загрузка данных

```bash
# 1. Создать схему БД
psql $DATABASE_URL < config/schema.sql

# 2. Загрузить исторические данные (последние 30 дней)
cargo run --bin data_collector --features real_data -- --symbol BTC_USDT --days 30

# 3. Рассчитать индикаторы
cargo run --bin calculate_indicators --features real_data

# 4. Запустить backtest с реальными данными
cargo run --bin investor_demo --features real_data -- --use-database
```

## 4. Что получаем

- ✅ Реальные цены с Gate.io
- ✅ OHLCV свечи (для анализа каналов)
- ✅ Объем и дельта объема
- ✅ Предрассчитанные индикаторы (RSI, MACD, Bollinger Bands)
- ✅ Быстрый доступ через PostgreSQL + Redis cache
- ✅ Реалистичные backtest результаты


