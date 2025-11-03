# 🚀 HFT Backtest Engine - Полная спецификация

## Архитектура

### 1. Tick-by-Tick Simulation
- ✅ Каждый тик обрабатывается отдельно
- ✅ Учет порядка исполнения тиков
- ✅ Миллисекундная точность

### 2. Full Order Book Reconstruction

#### L2 (Market-By-Price)
- Уровни цен с видимыми объемами
- Обновление при каждом изменении

#### L3 (Market-By-Order)
- Отдельные ордера в очереди
- FIFO / PRO RATA / Time Priority модели исполнения
- Поддержка скрытых ордеров и айсбергов

### 3. Latency Modeling

```rust
pub struct LatencyModel {
    pub feed_latency_ms: (u64, u64),     // 10-20 мс
    pub order_latency_ms: (u64, u64),    // 5-15 мс
    pub network_jitter: bool,            // Случайные вариации
    pub recalculation_delay_ms: u64,     // 50 мс
}
```

### 4. Order Fill Simulation

#### Модели исполнения:
- **FIFO**: First In First Out - строгая очередь
- **PRO RATA**: Пропорциональное распределение
- **Time Priority**: Время + размер ордера

#### Учет позиции в очереди:
- Позиция ордера на уровне цены
- Частичное исполнение
- Скрытые/айсберг ордера

### 5. Multi-Asset & Multi-Exchange

```rust
pub struct BacktestEngine {
    pub streams: Vec<TradeStream>,        // Несколько пар
    pub exchanges: Vec<ExchangeState>,    // Несколько бирж
    pub cross_exchange_arbitrage: bool,   // Арбитраж
}
```

## Фильтры и селекторы

### Дельта фильтры
- 1м, 3м, 5м, 15м, 30м, 1ч, 24ч окна
- Абсолютные и относительные дельты
- Множественные фильтры на одну валюту

### Объемные фильтры
- Min/Max объем за 24ч
- Ликвидность
- Волатильность

### Фильтр ставки финансирования
- Диапазон ставки
- Время до/после выплаты

### Фильтры цены
- Шаг цены (отсекает "квадратные" монеты)
- Отклонение марк прайса

## Метрики и рейтинг

### Базовые метрики
- Total P&L
- Win Rate
- Profit Factor
- Max Drawdown
- Sharpe Ratio

### Стабильность
- Повторные прогоны (Monte Carlo)
- Разброс P&L
- Стабильность количества сделок

### Рейтинг стратегий
```rust
pub struct StrategyRating {
    pub profitability_score: f64,  // 0-10
    pub stability_score: f64,      // 0-10
    pub risk_score: f64,            // 0-10 (обратный)
    pub fill_rate_score: f64,       // 0-10
    pub overall_rating: f64,        // Средневзвешенное
    pub stars: u8,                  // 0-5
}
```

## Интеграция

### Multi-Exchange Support
- Binance (Spot/Futures)
- Bybit (Spot/Futures)
- Gate.io (Spot/Futures)
- Расширяемо через трейты

### AI Оптимизация
- Hyperparameter tuning
- Reinforcement learning для адаптации
- Feature engineering для ML моделей

### Performance
- Асинхронность (Tokio)
- Многопоточность
- Оптимизация горячих путей

## Использование

```rust
use rust_test::backtest::*;

// Настройки с L2/L3 orderbook
let settings = BacktestSettings {
    use_orderbook_l3: true,
    fill_model: FillModel::FIFO,
    latency_ms_range: (10, 20),
    ..Default::default()
};

// Создаем движок
let mut engine = BacktestEngine::new(settings);

// Добавляем orderbook для символа
engine.add_orderbook("BTCUSDT", OrderBook::new("BTCUSDT".to_string()));

// Загружаем .bin файлы
let mut replay = ReplayEngine::new(Default::default());
replay.load_bin_file("data/BTC_2024_01.bin")?;

// Добавляем фильтры
let filters = MarketFilters {
    delta_filters: vec![DeltaFilter {
        time_window: TimeWindow::Hour1,
        min_delta: Some(-5.0),
        max_delta: Some(5.0),
        is_absolute: false,
    }],
    max_active_markets: 30,
    ..Default::default()
};
engine.set_filters(filters);

// Запуск
let result = engine.run()?;
println!("Rating: {:.2}/10, Stars: {}", 
    result.rating.overall_rating, result.rating.stars);
```

## Формат данных

### .bin файлы
- Timestamp (i64 ms)
- Price (f64)
- Volume (f64)
- Side (bool)
- OrderBook snapshots (опционально)

## Следующие шаги

1. ✅ Full Order Book реконструкция
2. ✅ Фильтры и селекторы
3. ✅ Рейтинг стратегий
4. ⏳ Интеграция с MShot/MStrike/Hook
5. ⏳ AI оптимизация параметров

