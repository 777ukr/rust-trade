# Gate.io Integration Guide

## ✅ Что сделано

Интеграция Gate.io добавлена в проект `rust-trade` по аналогии с Binance.

### Структура

1. **`trading-core/src/exchange/gateio.rs`** - основная реализация Gate.io WebSocket и REST API
2. **`trading-core/src/exchange/types.rs`** - добавлены типы для Gate.io сообщений
3. **`trading-core/src/exchange/utils.rs`** - утилиты для конвертации Gate.io данных
4. **`trading-core/src/exchange/mod.rs`** - экспорт `GateioExchange`

### Основные возможности

- ✅ WebSocket подписка на trades (реал-тайм данные)
- ✅ REST API для исторических сделок
- ✅ Автоматическое переподключение
- ✅ Обработка ошибок и валидация
- ✅ Конвертация форматов Gate.io → TickData

## 🚀 Использование

### 1. Выбор биржи в коде

```rust
use trading_core::exchange::{GateioExchange, Exchange};

let exchange = GateioExchange::new();

// Подписка на real-time данные
exchange.subscribe_trades(
    &["BTCUSDT".to_string(), "ETHUSDT".to_string()],
    Box::new(|tick_data| {
        println!("New trade: {:?}", tick_data);
    }),
    shutdown_rx,
).await?;

// Получение исторических данных
let params = HistoricalTradeParams::new("BTCUSDT".to_string())
    .with_time_range(start_time, end_time)
    .with_limit(1000);

let trades = exchange.get_historical_trades(params).await?;
```

### 2. Интеграция с backtest engine

Существующий backtest engine уже поддерживает любой exchange, реализующий трейт `Exchange`. Просто используйте `GateioExchange` вместо `BinanceExchange`:

```rust
// Вместо
let exchange = BinanceExchange::new();

// Используйте
let exchange = GateioExchange::new();
```

### 3. Настройка символов

Gate.io использует формат `BTCUSDT`, `ETHUSDT` и т.д. (тот же, что и Binance).

Автоматическая конвертация в формат Gate.io WebSocket:
- `BTCUSDT` → `futures.trades.BTC_USDT`

## 📋 Форматы данных

### WebSocket формат Gate.io

**Подписка:**
```json
{
  "time": 1234567890,
  "channel": "futures.trades.BTC_USDT",
  "event": "subscribe",
  "payload": ["futures.trades.BTC_USDT"]
}
```

**Trade сообщение:**
```json
{
  "time": 1234567890,
  "channel": "futures.trades.BTC_USDT",
  "event": "update",
  "result": {
    "id": 12345,
    "create_time": 1234567890,
    "contract": "BTC_USDT",
    "price": "50000.00",
    "size": 1,
    "role": "maker"
  }
}
```

### REST API

**Эндпоинт:** `GET /api/v4/futures/usdt/trades?contract=BTC_USDT&limit=1000`

## 🔧 Настройка

### Переменные окружения

Создайте `.env` в `trading-core/`:

```bash
DATABASE_URL=postgresql://user:password@localhost/trading_core
RUN_MODE=development
```

### Конфигурация символов

В `config/development.toml`:

```toml
symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"]
```

## 🧪 Тестирование

```bash
cd trading-core
cargo test exchange::gateio
```

## 📊 Сравнение с Binance

| Функция | Binance | Gate.io |
|---------|---------|---------|
| WebSocket URL | `wss://stream.binance.com:9443/stream` | `wss://fx-ws.gateio.ws/v4/ws/usdt` |
| Формат символа | `BTCUSDT` | `BTCUSDT` (внутри: `BTC_USDT`) |
| Подписка | `btcusdt@trade` | `futures.trades.BTC_USDT` |
| Timestamp | миллисекунды | секунды |
| Trade size | положительное | может быть отрицательным (sell) |

## 🔍 Отладка

Включите детальное логирование:

```bash
RUST_LOG=trading_core::exchange=debug cargo run
```

## 📚 Дополнительно

- [Gate.io Futures API Docs](https://www.gate.io/docs/developers/apiv4/en/#futures)
- [Gate.io WebSocket Docs](https://www.gate.io/docs/developers/apiv4/en/#futures-trades)

## ⚠️ Заметки

1. **Timestamp**: Gate.io использует секунды, Binance - миллисекунды. Конвертация выполняется автоматически.

2. **Size**: В Gate.io `size` может быть отрицательным для продаж. Код автоматически берет абсолютное значение.

3. **Contract format**: Gate.io требует формат `BTC_USDT` в WebSocket, но мы принимаем `BTCUSDT` и конвертируем автоматически.

4. **Rate limits**: Gate.io имеет лимиты на количество символов в одной подписке. Рекомендуется подписываться на не более 10-20 символов одновременно.

