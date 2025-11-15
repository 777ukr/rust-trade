# ⚡ Быстрая шпаргалка - Просмотр отчетов

## 🔬 Бэктестинг

```bash
# Запуск и просмотр отчета в консоли
cd trading-core
cargo run backtest

# Сохранение отчета в файл
cargo run backtest 2>&1 | tee backtest_report.txt
```

**Где смотреть:** Прямо в терминале - полный отчет появится после завершения

---

## 📝 Paper Trading

### В реальном времени

```bash
# Запуск и просмотр сигналов в реальном времени
cd trading-core
cargo run live --paper-trading

# Сохранение логов в файл
cargo run live --paper-trading 2>&1 | tee paper_trading_$(date +%Y%m%d_%H%M%S).log
```

**Где смотреть:** В терминале - сигналы BUY/SELL появляются в реальном времени

### История из базы данных

```sql
-- Последние 50 сигналов
SELECT timestamp, symbol, signal_type, current_price, portfolio_value, total_pnl
FROM live_strategy_log
WHERE strategy_id = 'RSI Strategy'
ORDER BY timestamp DESC
LIMIT 50;

-- Статистика
SELECT 
    COUNT(*) as total_signals,
    SUM(CASE WHEN signal_type = 'BUY' THEN 1 ELSE 0 END) as buys,
    SUM(CASE WHEN signal_type = 'SELL' THEN 1 ELSE 0 END) as sells,
    MAX(total_pnl) as max_pnl,
    AVG(portfolio_value) as avg_portfolio
FROM live_strategy_log
WHERE strategy_id = 'RSI Strategy';
```

---

## 📡 Live Trading (сбор данных)

```bash
# Запуск и просмотр логов
cd trading-core
cargo run live
```

**Где смотреть:** В терминале - логи подключения и статус

### Проверка собранных данных

```sql
-- Количество тиков за последний час
SELECT symbol, COUNT(*) as tick_count
FROM tick_data
WHERE timestamp >= NOW() - INTERVAL '1 hour'
GROUP BY symbol;

-- Последние тики
SELECT timestamp, symbol, price, quantity
FROM tick_data
WHERE symbol = 'BTCUSDT'
ORDER BY timestamp DESC
LIMIT 20;
```

---

## 📊 Полезные SQL запросы

### Paper Trading - все сделки

```sql
SELECT timestamp, symbol, signal_type, current_price, portfolio_value, total_pnl
FROM live_strategy_log
WHERE signal_type IN ('BUY', 'SELL')
ORDER BY timestamp DESC;
```

### Paper Trading - производительность

```sql
SELECT 
    AVG(processing_time_us) as avg_time_us,
    AVG(CASE WHEN cache_hit THEN 1 ELSE 0 END) * 100 as cache_hit_rate
FROM live_strategy_log
WHERE strategy_id = 'RSI Strategy';
```

### Экспорт в CSV

```bash
psql $DATABASE_URL -c "COPY (SELECT * FROM live_strategy_log WHERE strategy_id = 'RSI Strategy') TO STDOUT WITH CSV HEADER" > report.csv
```

---

## 📚 Полная документация

- **Подробное руководство:** [REPORTS_GUIDE.md](REPORTS_GUIDE.md)
- **Руководство по Gate.io:** [GATEIO_TRADING_GUIDE.md](GATEIO_TRADING_GUIDE.md)
