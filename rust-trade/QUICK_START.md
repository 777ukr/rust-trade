# 🚀 Quick Start Guide - Rust Trade с Gate.io

## ✅ Что уже сделано

1. ✅ Интеграция Gate.io добавлена в проект
2. ✅ WebSocket подписка на trades
3. ✅ REST API для исторических данных
4. ✅ Автоматическая конвертация форматов

## 📦 Структура проекта

```
rust-trade/
├── trading-core/          # Основной движок
│   ├── src/exchange/
│   │   ├── binance.rs     # Binance интеграция
│   │   ├── gateio.rs      # ✨ Gate.io интеграция (НОВОЕ)
│   │   ├── traits.rs      # Интерфейс Exchange
│   │   └── ...
│   └── ...
├── frontend/              # Next.js интерфейс
└── src-tauri/             # Desktop приложение
```

## 🎯 Быстрый старт

### Вариант 1: Использовать в своем коде

```rust
use trading_core::exchange::{GateioExchange, Exchange};

#[tokio::main]
async fn main() -> Result<()> {
    let exchange = GateioExchange::new();
    
    // Подписка на real-time данные
    exchange.subscribe_trades(
        &["BTCUSDT".to_string()],
        Box::new(|tick_data| {
            println!("Trade: {} @ {}", tick_data.price, tick_data.symbol);
        }),
        shutdown_rx,
    ).await?;
    
    Ok(())
}
```

### Вариант 2: Заменить Binance на Gate.io в main.rs

В файле `trading-core/src/main.rs` замените:

```rust
// Было:
use exchange::BinanceExchange;
let exchange = BinanceExchange::new();

// Стало:
use exchange::GateioExchange;
let exchange = GateioExchange::new();
```

### Вариант 3: Использовать в backtest

Backtest engine автоматически работает с любым Exchange:

```rust
use trading_core::exchange::GateioExchange;

let exchange = GateioExchange::new();
let params = HistoricalTradeParams::new("BTCUSDT".to_string())
    .with_time_range(start, end)
    .with_limit(1000);

let data = exchange.get_historical_trades(params).await?;
```

## 📝 Настройка

### 1. Переменные окружения

Создайте `.env` в `rust-trade/trading-core/`:

```bash
DATABASE_URL=postgresql://user:password@localhost/trading_core
RUN_MODE=development
```

### 2. Конфигурация символов

В `rust-trade/config/development.toml`:

```toml
symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"]
```

## 🧪 Тестирование

```bash
cd rust-trade/trading-core

# Проверка компиляции
cargo check

# Запуск с Gate.io
cargo run live
```

## ⚠️ Важные замечания

1. **Формат символов**: Gate.io принимает `BTCUSDT` (как Binance), но внутри использует `BTC_USDT` - конвертация автоматическая.

2. **SQLx макросы**: Если видите ошибки `set DATABASE_URL`, либо:
   - Подключите PostgreSQL
   - Или используйте `cargo sqlx prepare` для offline режима

3. **WebSocket URL**: `wss://fx-ws.gateio.ws/v4/ws/usdt` - уже настроен.

## 📚 Документация

- [Полная инструкция Gate.io](GATEIO_INTEGRATION.md)
- [Оригинальный README](README.md)

## 🎉 Готово!

Теперь можно использовать Gate.io в rust-trade точно так же, как Binance!

