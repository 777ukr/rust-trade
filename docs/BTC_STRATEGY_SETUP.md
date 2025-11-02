# Настройка и запуск BTC стратегии

## Что мы используем из существующего проекта:

### ✅ Готовые компоненты (не нужно писать с нуля):

1. **Парсер Gate.io** (`src/exchanges/gate/parser.rs`)
   - Уже парсит WebSocket данные от Gate.io
   - Извлекает цены, orderbook, trades

2. **API Gate.io** (`src/execution/gate_client.rs`, `gate_ws.rs`)
   - Полностью готовое подключение к Gate.io
   - REST API для получения позиций
   - WebSocket для исполнения ордеров

3. **OrderManager** (`src/execution/order_manager.rs`)
   - Управление жизненным циклом ордеров
   - Отправка, отмена, отслеживание статусов

4. **InventoryTracker** (`src/execution/inventory.rs`)
   - Автоматическое отслеживание позиций
   - Синхронизация с Gate.io REST API

5. **Стоп-лосс логика** (`src/strategy/stop_loss.rs`)
   - Функции проверки стоп-лосса и тейк-профита

## Что нужно сделать:

### Шаг 1: Добавить API ключи

Создайте `.env` файл:
```bash
GATEIO_API_KEY=ваш_ключ
GATEIO_SECRET_KEY=ваш_секрет
```

### Шаг 2: Настроить конфигурацию

Создайте `config/btc_strategy.yaml`:
```yaml
strategy:
  symbol: BTC_USDT
  entry_size: 0.001
  max_position_size: 0.01
  stop_loss_percent: 2.0
  take_profit_percent: 4.0
  rsi_period: 14
  rsi_oversold: 30.0
  rsi_overbought: 70.0

risk:
  max_order_notional: 1000.0
  max_position_notional: 5000.0

mode:
  dry_run: true  # Начните с dry-run!
  debug_prints: true

credentials:
  api_key_env: GATEIO_API_KEY
  api_secret_env: GATEIO_SECRET_KEY
```

### Шаг 3: Создать runner (уже создан файл стратегии)

BTC стратегия создана в `src/strategy/btc_strategy.rs`

### Шаг 4: Интеграция в runner

Нужно создать `src/bin/btc_runner.rs` на основе `gate_runner.rs`, но с использованием `BtcTradingStrategy`.

## Как работает стратегия:

1. **Получение данных**: Gate.io WebSocket → Парсер (уже есть)
2. **Анализ**: RSI индикатор → Сигналы входа/выхода (новая стратегия)
3. **Стоп-лосс**: Постоянная проверка позиций (интегрировано)
4. **Исполнение**: OrderManager → Gate.io API (уже есть)

## Логика торговли:

- **Вход (LONG)**: RSI < 30 (перепроданность)
- **Выход**: 
  - Тейк-профит: +4% от входа
  - Стоп-лосс: -2% от входа
  - Или RSI > 70 (перекупленность)

## Запуск:

```bash
# 1. Убедитесь, что API ключи в .env
# 2. Создайте конфигурацию config/btc_strategy.yaml
# 3. Скомпилируйте
cargo build --features gate_exec

# 4. Запустите в dry-run режиме
cargo run --bin btc_runner --features gate_exec
```

## Что дальше улучшить:

1. **Доработать индикаторы**: 
   - Реализовать полный RSI расчет
   - Добавить MACD, Bollinger Bands

2. **Расширить скринер**:
   - Фильтрация по волатильности
   - Проверка объема

3. **Улучшить сигналы**:
   - Комбинация нескольких индикаторов
   - Адаптивные уровни стоп-лосса

