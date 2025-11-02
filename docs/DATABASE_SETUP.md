# Настройка PostgreSQL для CryptoTrader

## Обзор

Проект использует PostgreSQL для хранения:
- **Tick Data** - реальные сделки с биржи
- **OHLCV Data** - свечные данные (candlesticks)
- **Backtest Results** - результаты бэктестов стратегий
- **Strategy Logs** - детальные логи выполнения стратегий
- **Account History** - история балансов аккаунта

## Быстрый старт

### 1. Установка PostgreSQL

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install postgresql postgresql-contrib

# macOS
brew install postgresql@14
brew services start postgresql@14

# Windows - скачайте с https://www.postgresql.org/download/
```

### 2. Создание базы данных

```bash
# Войдите в PostgreSQL
sudo -u postgres psql

# Создайте базу данных и пользователя
CREATE DATABASE cryptotrader;
CREATE USER cryptotrader_user WITH PASSWORD 'your_secure_password';
GRANT ALL PRIVILEGES ON DATABASE cryptotrader TO cryptotrader_user;
\q
```

### 3. Применение схемы

```bash
# Примените SQL схему
psql -U cryptotrader_user -d cryptotrader -f database/schema.sql

# Или из корня проекта
psql postgresql://cryptotrader_user:your_secure_password@localhost/cryptotrader < database/schema.sql
```

### 4. Настройка переменных окружения

Создайте `.env` файл в корне проекта:

```bash
DATABASE_URL=postgresql://cryptotrader_user:your_secure_password@localhost/cryptotrader
```

## Структура таблиц

### `tick_data`
Хранит реальные сделки с биржи:
- `timestamp` - время сделки
- `symbol` - торговый инструмент (BTCUSDT, ETHUSDT)
- `price`, `quantity` - цена и объем
- `side` - BUY/SELL
- `trade_id` - уникальный ID сделки
- `exchange` - биржа (gateio)

### `ohlcv_data`
Хранит свечные данные:
- `timestamp`, `symbol`, `interval` - 1m, 5m, 1h и т.д.
- `open`, `high`, `low`, `close`, `volume`

### `backtest_results`
Результаты бэктестов:
- Метрики производительности (ROI, win rate, profit factor)
- Начальный/конечный баланс
- Количество сделок и статистика

### `strategy_logs`
Детальные логи выполнения:
- Каждый сигнал стратегии
- Состояние позиции
- P&L на каждом шаге

### `account_history`
История балансов:
- Снимки баланса аккаунта
- Доступно/заблокировано средств

## Использование в коде

### Базовое использование

```rust
use rust_test::database::{DatabaseRepository, BacktestResult};
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Создайте connection pool
    let pool = DatabaseRepository::create_pool(
        &std::env::var("DATABASE_URL")?
    ).await?;
    
    let repo = DatabaseRepository::new(pool);
    
    // Проверка соединения
    repo.test_connection().await?;
    
    // Сохранение результата бэктеста
    let result = BacktestResult {
        strategy_name: "ChannelSplit".to_string(),
        symbol: "BTCUSDT".to_string(),
        initial_balance: Decimal::from(1250),
        // ... остальные поля
    };
    
    let id = repo.insert_backtest_result(&result).await?;
    println!("Saved backtest result with ID: {}", id);
    
    Ok(())
}
```

### Запуск с базой данных

```bash
# Убедитесь, что DATABASE_URL установлен
export DATABASE_URL=postgresql://user:password@localhost/cryptotrader

# Запустите investor_demo с feature database
cargo run --bin investor_demo --features gate_exec,database
```

## Интеграция в investor_demo

Модуль `investor_demo` автоматически сохраняет результаты в БД, если:
1. Установлена переменная `DATABASE_URL`
2. Компиляция с feature `database`

Результаты сохраняются как в CSV (для совместимости), так и в PostgreSQL.

## Производительность

### Оптимизация запросов

Таблицы имеют индексы для быстрого поиска:
- По символу и времени для tick_data
- По стратегии и символу для backtest_results
- По backtest_id для strategy_logs

### Batch операции

Для массовой вставки используйте batch методы:
```rust
// Batch insert ticks
repo.insert_ticks_batch(&ticks).await?;

// Batch insert logs
repo.insert_strategy_logs_batch(&logs).await?;
```

## Резервное копирование

```bash
# Создание бэкапа
pg_dump -U cryptotrader_user cryptotrader > backup.sql

# Восстановление
psql -U cryptotrader_user cryptotrader < backup.sql
```

## Мониторинг

### Размер таблиц

```sql
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

### Количество записей

```sql
SELECT 
    'tick_data' as table_name, COUNT(*) as count FROM tick_data
UNION ALL
SELECT 'ohlcv_data', COUNT(*) FROM ohlcv_data
UNION ALL
SELECT 'backtest_results', COUNT(*) FROM backtest_results
UNION ALL
SELECT 'strategy_logs', COUNT(*) FROM strategy_logs;
```

## Troubleshooting

### Ошибка подключения

```bash
# Проверьте, что PostgreSQL запущен
sudo systemctl status postgresql

# Проверьте доступность
psql -U cryptotrader_user -d cryptotrader -c "SELECT 1"
```

### Ошибка прав доступа

```sql
-- Выдайте права
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO cryptotrader_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO cryptotrader_user;
```

### SQLx offline mode

Если не хотите подключаться к БД при компиляции:

```bash
# Подключитесь к БД и подготовьте queries
DATABASE_URL=postgresql://... cargo sqlx prepare --database-url $DATABASE_URL

# Затем компилируйте с offline режимом
cargo sqlx build --offline
```

## Дополнительные ресурсы

- [SQLx документация](https://docs.rs/sqlx/)
- [PostgreSQL документация](https://www.postgresql.org/docs/)
- [Rust Decimal](https://docs.rs/rust_decimal/)

