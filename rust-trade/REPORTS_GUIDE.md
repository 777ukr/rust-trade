# 📊 Руководство по просмотру отчетов - Gate.io Trading

Это руководство объясняет, где и как просматривать отчеты для бэктестинга, paper trading и live trading.

## 📋 Содержание

1. [Бэктестинг - Отчеты в консоли](#бэктестинг---отчеты-в-консоли)
2. [Paper Trading - Отчеты в реальном времени](#paper-trading---отчеты-в-реальном-времени)
3. [Paper Trading - Отчеты из базы данных](#paper-trading---отчеты-из-базы-данных)
4. [Live Trading - Логи и мониторинг](#live-trading---логи-и-мониторинг)
5. [SQL запросы для анализа](#sql-запросы-для-анализа)
6. [Экспорт данных](#экспорт-данных)

---

## 🔬 Бэктестинг - Отчеты в консоли

### Где смотреть

**Все отчеты бэктестинга отображаются прямо в терминале/консоли** при запуске команды:

```bash
cd trading-core
cargo run backtest
```

### Что вы увидите

#### 1. Интерактивный выбор параметров

```
🎯 TRADING CORE BACKTESTING SYSTEM
============================================================
📊 Loading data statistics...

📈 Available Data:
  Total Records: 150000
  Available Symbols: 3
  Earliest Data: 2024-01-01 00:00:00 UTC
  Latest Data: 2024-12-15 23:59:59 UTC

🎯 Available Strategies:
  1) SMA Strategy - Trading strategy based on short and long-term moving average crossover
  2) RSI Strategy - Trading strategy based on Relative Strength Index (RSI)

Select strategy (1-2): 2
✅ Selected Strategy: RSI Strategy
```

#### 2. Прогресс выполнения

Во время выполнения бэктестинга вы увидите прогресс:

```
Starting backtest...
Strategy: RSI Strategy
Initial capital: $10000
Data points: 10000
Commission rate: 0.1%
============================================================
Progress: 10% (1000/10000) | Portfolio Value: $10050.00 | P&L: $50.00
Progress: 20% (2000/10000) | Portfolio Value: $10100.00 | P&L: $100.00
Progress: 30% (3000/10000) | Portfolio Value: $9950.00 | P&L: $-50.00
...
BUY BTCUSDT 0.1 @ $45000.00
SELL BTCUSDT 0.1 @ $45500.00
BUY BTCUSDT 0.1 @ $45200.00
...
```

#### 3. Итоговый отчет

После завершения бэктестинга вы увидите полный отчет:

```
============================================================

BACKTEST RESULTS SUMMARY
============================================================
Strategy: RSI Strategy
Initial Capital: $10000.00
Final Value: $10500.00
Total P&L: $500.00
Return: 5.00%
Total Commission: $45.00

TRADING STATISTICS
------------------------------
Total Trades: 45
Winning Trades: 28 (62.2%)
Losing Trades: 17 (37.8%)
Profit Factor: 1.85
Avg Trade Duration: 3600 seconds

RISK METRICS
------------------------------
Max Drawdown: 2.50%
Sharpe Ratio: 1.25
Volatility: 15.30%

CURRENT POSITIONS
------------------------------
BTCUSDT: 0.1 @ $45000.00 (Unrealized P&L: $50.00)

RECENT TRADES (Last 5)
------------------------------
2024-12-15 10:30:00 BUY  BTCUSDT @ $45000.00
2024-12-15 11:00:00 SELL BTCUSDT @ $45500.00 (P&L: $50.00)
2024-12-15 11:30:00 BUY  BTCUSDT @ $45200.00
2024-12-15 12:00:00 SELL BTCUSDT @ $44800.00 (P&L: $-40.00)
2024-12-15 12:30:00 BUY  BTCUSDT @ $45000.00
============================================================
```

#### 4. Детальный анализ сделок (опционально)

После основного отчета система спросит:

```
Show detailed trade analysis? (y/N): y
```

Если ответите `y`, увидите:

```
DETAILED TRADE ANALYSIS
================================================================================
Buy Trades: 23
Sell Trades: 22
Profitable Sells: 15 (68.2%)
Total Gross Profit: $850.00
Total Gross Loss: $-350.00
Average Profit per Winning Trade: $56.67
Average Loss per Losing Trade: $-50.00
================================================================================
```

### Сохранение отчета в файл

Чтобы сохранить отчет в файл:

```bash
# Linux/Mac
cargo run backtest 2>&1 | tee backtest_report.txt

# Windows PowerShell
cargo run backtest 2>&1 | Tee-Object -FilePath backtest_report.txt
```

---

## 📝 Paper Trading - Отчеты в реальном времени

### Где смотреть

**Все отчеты paper trading отображаются в реальном времени в терминале** при запуске:

```bash
cd trading-core
cargo run live --paper-trading
```

### Что вы увидите

#### 1. Стартовая информация

```
🚀 Paper trading is now active! Watch for trading signals below...
📈 Strategy: RSI Strategy | Initial Capital: $10000
================================================================================
```

#### 2. Торговые сигналы в реальном времени

При каждом сигнале BUY/SELL вы увидите:

```
🎯 BUY BTCUSDT @ $45000.00 | Portfolio: $10000.00 | P&L: $0.00 (0.00%) | Position: 0.2222 | Cash: $0.00 | Trades: 1 | Cache: HIT (45μs) | Total: 120μs
🎯 SELL BTCUSDT @ $45500.00 | Portfolio: $10111.11 | P&L: $111.11 (1.11%) | Position: 0.0000 | Cash: $10111.11 | Trades: 2 | Cache: HIT (42μs) | Total: 115μs
🎯 BUY ETHUSDT @ $3000.00 | Portfolio: $10111.11 | P&L: $111.11 (1.11%) | Position: 1.0000 | Cash: $7111.11 | Trades: 3 | Cache: HIT (38μs) | Total: 110μs
```

**Расшифровка полей:**

- `🎯 BUY/SELL` - тип сигнала
- `BTCUSDT @ $45000.00` - символ и цена
- `Portfolio: $10000.00` - текущая стоимость портфеля
- `P&L: $0.00 (0.00%)` - прибыль/убыток в долларах и процентах
- `Position: 0.2222` - текущая позиция (количество)
- `Cash: $0.00` - доступные средства
- `Trades: 1` - общее количество сделок
- `Cache: HIT (45μs)` - использование кэша и время доступа
- `Total: 120μs` - общее время обработки

#### 3. Периодические обновления (HOLD сигналы)

Каждые 10 секунд (при HOLD сигналах) вы увидите:

```
📊 BTCUSDT @ $45400.00 | Portfolio: $10111.11 | P&L: $111.11 | Cache: ✓ (38μs)
📊 ETHUSDT @ $3010.00 | Portfolio: $10121.11 | P&L: $121.11 | Cache: ✓ (35μs)
```

### Сохранение логов в файл

```bash
# Linux/Mac
cargo run live --paper-trading 2>&1 | tee paper_trading_log.txt

# Windows PowerShell
cargo run live --paper-trading 2>&1 | Tee-Object -FilePath paper_trading_log.txt
```

---

## 💾 Paper Trading - Отчеты из базы данных

### Где хранятся данные

Все данные paper trading сохраняются в таблице `live_strategy_log` в PostgreSQL.

### Подключение к базе данных

```bash
# Используя psql
psql -h localhost -U your_user -d your_database

# Или через переменную окружения
psql $DATABASE_URL
```

### Основные SQL запросы

#### 1. Общая статистика по стратегии

```sql
SELECT 
    strategy_id,
    symbol,
    COUNT(*) as total_signals,
    SUM(CASE WHEN signal_type = 'BUY' THEN 1 ELSE 0 END) as buy_signals,
    SUM(CASE WHEN signal_type = 'SELL' THEN 1 ELSE 0 END) as sell_signals,
    SUM(CASE WHEN signal_type = 'HOLD' THEN 1 ELSE 0 END) as hold_signals,
    AVG(portfolio_value) as avg_portfolio_value,
    MAX(portfolio_value) as max_portfolio_value,
    MIN(portfolio_value) as min_portfolio_value,
    MAX(total_pnl) as max_pnl,
    MIN(total_pnl) as min_pnl,
    AVG(total_pnl) as avg_pnl
FROM live_strategy_log
WHERE strategy_id = 'RSI Strategy'
GROUP BY strategy_id, symbol
ORDER BY symbol;
```

#### 2. Последние сигналы

```sql
SELECT 
    timestamp,
    strategy_id,
    symbol,
    current_price,
    signal_type,
    portfolio_value,
    total_pnl,
    ROUND((total_pnl / 10000.0) * 100, 2) as return_pct
FROM live_strategy_log
WHERE strategy_id = 'RSI Strategy'
ORDER BY timestamp DESC
LIMIT 50;
```

#### 3. Все сделки (BUY/SELL)

```sql
SELECT 
    timestamp,
    symbol,
    signal_type,
    current_price,
    portfolio_value,
    total_pnl,
    ROUND((total_pnl / 10000.0) * 100, 2) as return_pct
FROM live_strategy_log
WHERE strategy_id = 'RSI Strategy'
  AND signal_type IN ('BUY', 'SELL')
ORDER BY timestamp DESC;
```

#### 4. Статистика по времени

```sql
SELECT 
    DATE_TRUNC('hour', timestamp) as hour,
    COUNT(*) as signals_count,
    SUM(CASE WHEN signal_type = 'BUY' THEN 1 ELSE 0 END) as buys,
    SUM(CASE WHEN signal_type = 'SELL' THEN 1 ELSE 0 END) as sells,
    AVG(portfolio_value) as avg_portfolio,
    MAX(total_pnl) as max_pnl
FROM live_strategy_log
WHERE strategy_id = 'RSI Strategy'
  AND timestamp >= NOW() - INTERVAL '24 hours'
GROUP BY hour
ORDER BY hour DESC;
```

#### 5. Производительность (cache hit rate)

```sql
SELECT 
    strategy_id,
    COUNT(*) as total_requests,
    SUM(CASE WHEN cache_hit = true THEN 1 ELSE 0 END) as cache_hits,
    SUM(CASE WHEN cache_hit = false THEN 1 ELSE 0 END) as cache_misses,
    ROUND(
        (SUM(CASE WHEN cache_hit = true THEN 1 ELSE 0 END)::numeric / COUNT(*)::numeric) * 100, 
        2
    ) as cache_hit_rate_pct,
    AVG(processing_time_us) as avg_processing_time_us,
    MAX(processing_time_us) as max_processing_time_us
FROM live_strategy_log
WHERE strategy_id = 'RSI Strategy'
GROUP BY strategy_id;
```

#### 6. График портфеля (для построения графиков)

```sql
SELECT 
    timestamp,
    portfolio_value,
    total_pnl,
    ROUND((total_pnl / 10000.0) * 100, 2) as return_pct
FROM live_strategy_log
WHERE strategy_id = 'RSI Strategy'
  AND timestamp >= NOW() - INTERVAL '24 hours'
ORDER BY timestamp ASC;
```

#### 7. Анализ прибыльности по символам

```sql
WITH trade_pairs AS (
    SELECT 
        symbol,
        signal_type,
        current_price,
        total_pnl,
        LAG(total_pnl) OVER (PARTITION BY symbol ORDER BY timestamp) as prev_pnl,
        LAG(signal_type) OVER (PARTITION BY symbol ORDER BY timestamp) as prev_signal
    FROM live_strategy_log
    WHERE strategy_id = 'RSI Strategy'
      AND signal_type IN ('BUY', 'SELL')
)
SELECT 
    symbol,
    COUNT(*) as total_trades,
    SUM(CASE WHEN signal_type = 'SELL' AND total_pnl > prev_pnl THEN 1 ELSE 0 END) as profitable_trades,
    SUM(CASE WHEN signal_type = 'SELL' AND total_pnl < prev_pnl THEN 1 ELSE 0 END) as losing_trades,
    ROUND(
        (SUM(CASE WHEN signal_type = 'SELL' AND total_pnl > prev_pnl THEN 1 ELSE 0 END)::numeric / 
         NULLIF(SUM(CASE WHEN signal_type = 'SELL' THEN 1 ELSE 0 END), 0)::numeric) * 100, 
        2
    ) as win_rate_pct
FROM trade_pairs
WHERE signal_type = 'SELL'
GROUP BY symbol;
```

---

## 📡 Live Trading - Логи и мониторинг

### Где смотреть

**Логи live trading отображаются в терминале** при запуске:

```bash
cd trading-core
cargo run live
```

### Что вы увидите

#### 1. Стартовая информация

```
🚀 Starting Trading Core Application (Live Mode)
📋 Configuration loaded successfully
📊 Monitoring symbols: ["BTCUSDT", "ETHUSDT", "ADAUSDT"]
🔌 Exchange provider: gateio
🔌 Connecting to database...
✅ Database connection established
💾 Initializing cache...
✅ Cache initialized
📡 Initializing exchange connection...
🔌 Exchange provider: gateio
✅ Exchange connection ready
🎯 Starting market data collection for 3 symbols
Connecting to Gate.io WebSocket with 3 channels
Subscription sent for 3 channels
```

#### 2. Логи подключения

```
WebSocket connected to wss://fx-ws.gateio.ws/v4/ws/usdt
Subscription sent for 3 channels
```

#### 3. Ошибки и переподключения

При проблемах с подключением:

```
WebSocket connection failed (attempt 1): Network error
Attempting to reconnect in 5s...
WebSocket connected to wss://fx-ws.gateio.ws/v4/ws/usdt
```

### Мониторинг данных в базе

#### Проверка количества собранных данных

```sql
SELECT 
    symbol,
    COUNT(*) as tick_count,
    MIN(timestamp) as first_tick,
    MAX(timestamp) as last_tick,
    MAX(timestamp) - MIN(timestamp) as duration
FROM tick_data
WHERE timestamp >= NOW() - INTERVAL '24 hours'
GROUP BY symbol
ORDER BY tick_count DESC;
```

#### Последние собранные тики

```sql
SELECT 
    timestamp,
    symbol,
    price,
    quantity,
    side
FROM tick_data
WHERE symbol = 'BTCUSDT'
ORDER BY timestamp DESC
LIMIT 20;
```

#### Статистика по символам

```sql
SELECT 
    symbol,
    COUNT(*) as total_ticks,
    AVG(price) as avg_price,
    MIN(price) as min_price,
    MAX(price) as max_price,
    SUM(quantity) as total_volume
FROM tick_data
WHERE timestamp >= NOW() - INTERVAL '1 hour'
GROUP BY symbol
ORDER BY total_ticks DESC;
```

---

## 📊 SQL запросы для анализа

### Универсальные запросы для всех режимов

#### 1. Экспорт данных для анализа в Excel/CSV

```sql
-- Экспорт paper trading данных
COPY (
    SELECT 
        timestamp,
        strategy_id,
        symbol,
        current_price,
        signal_type,
        portfolio_value,
        total_pnl
    FROM live_strategy_log
    WHERE strategy_id = 'RSI Strategy'
    ORDER BY timestamp
) TO '/tmp/paper_trading_export.csv' WITH CSV HEADER;
```

#### 2. Сравнение стратегий

```sql
SELECT 
    strategy_id,
    COUNT(*) as total_signals,
    SUM(CASE WHEN signal_type = 'BUY' THEN 1 ELSE 0 END) as buys,
    SUM(CASE WHEN signal_type = 'SELL' THEN 1 ELSE 0 END) as sells,
    AVG(portfolio_value) as avg_portfolio,
    MAX(total_pnl) as max_pnl,
    MIN(total_pnl) as min_pnl
FROM live_strategy_log
WHERE timestamp >= NOW() - INTERVAL '7 days'
GROUP BY strategy_id
ORDER BY max_pnl DESC;
```

#### 3. Анализ производительности по времени суток

```sql
SELECT 
    EXTRACT(HOUR FROM timestamp) as hour,
    COUNT(*) as signals,
    AVG(processing_time_us) as avg_processing_time,
    AVG(CASE WHEN cache_hit THEN 1 ELSE 0 END) * 100 as cache_hit_rate
FROM live_strategy_log
WHERE strategy_id = 'RSI Strategy'
  AND timestamp >= NOW() - INTERVAL '7 days'
GROUP BY hour
ORDER BY hour;
```

---

## 📤 Экспорт данных

### Экспорт через psql

```bash
# Экспорт в CSV
psql $DATABASE_URL -c "COPY (SELECT * FROM live_strategy_log WHERE strategy_id = 'RSI Strategy') TO STDOUT WITH CSV HEADER" > paper_trading.csv

# Экспорт в JSON (требует дополнительных инструментов)
psql $DATABASE_URL -t -A -F"," -c "SELECT json_agg(row_to_json(t)) FROM (SELECT * FROM live_strategy_log WHERE strategy_id = 'RSI Strategy') t" > paper_trading.json
```

### Экспорт через Python скрипт

Создайте файл `export_reports.py`:

```python
import psycopg2
import pandas as pd
from datetime import datetime
import os

# Подключение к БД
conn = psycopg2.connect(os.getenv('DATABASE_URL'))

# Экспорт paper trading данных
df = pd.read_sql_query("""
    SELECT 
        timestamp,
        strategy_id,
        symbol,
        current_price,
        signal_type,
        portfolio_value,
        total_pnl
    FROM live_strategy_log
    WHERE strategy_id = 'RSI Strategy'
    ORDER BY timestamp
""", conn)

# Сохранение в CSV
df.to_csv('paper_trading_report.csv', index=False)

# Сохранение в Excel
df.to_excel('paper_trading_report.xlsx', index=False)

print(f"Экспортировано {len(df)} записей")
```

---

## 🎨 Визуализация (опционально)

### Использование веб-интерфейса

Если у вас настроен веб-интерфейс (Tauri app), вы можете просматривать отчеты визуально:

1. Запустите веб-приложение
2. Перейдите в раздел "Backtest" или "Paper Trading"
3. Просматривайте графики и таблицы

### Создание графиков через Python

```python
import pandas as pd
import matplotlib.pyplot as plt
import psycopg2
import os

conn = psycopg2.connect(os.getenv('DATABASE_URL'))

# Загрузка данных
df = pd.read_sql_query("""
    SELECT timestamp, portfolio_value, total_pnl
    FROM live_strategy_log
    WHERE strategy_id = 'RSI Strategy'
    ORDER BY timestamp
""", conn)

# График портфеля
plt.figure(figsize=(12, 6))
plt.plot(df['timestamp'], df['portfolio_value'])
plt.title('Portfolio Value Over Time')
plt.xlabel('Time')
plt.ylabel('Value ($)')
plt.grid(True)
plt.savefig('portfolio_chart.png')
plt.show()

# График P&L
plt.figure(figsize=(12, 6))
plt.plot(df['timestamp'], df['total_pnl'])
plt.title('Profit & Loss Over Time')
plt.xlabel('Time')
plt.ylabel('P&L ($)')
plt.grid(True)
plt.axhline(y=0, color='r', linestyle='--')
plt.savefig('pnl_chart.png')
plt.show()
```

---

## 📝 Резюме

### Где смотреть отчеты

1. **Бэктестинг**: Прямо в консоли при запуске `cargo run backtest`
2. **Paper Trading (реальное время)**: В консоли при запуске `cargo run live --paper-trading`
3. **Paper Trading (история)**: В базе данных, таблица `live_strategy_log`
4. **Live Trading (логи)**: В консоли при запуске `cargo run live`
5. **Live Trading (данные)**: В базе данных, таблица `tick_data`

### Полезные команды

```bash
# Сохранение логов в файл
cargo run live --paper-trading 2>&1 | tee paper_trading_$(date +%Y%m%d_%H%M%S).log

# Просмотр последних логов
tail -f paper_trading.log

# Экспорт из БД
psql $DATABASE_URL -c "SELECT * FROM live_strategy_log ORDER BY timestamp DESC LIMIT 100;"
```

---

**Теперь вы знаете, где и как просматривать все отчеты! 📊**
