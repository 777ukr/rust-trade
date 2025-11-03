# MoonBot Features для добавления в проект

Анализ скриншотов [MoonBot](https://moon-bot.com/en/) и приоритизация функций для реализации.

## ✅ Уже реализовано

1. **Stop Loss / Trailing Stop** - есть в `long_trailing.rs`, `short_trailing.rs`, `ema_reversal.rs`
2. **Take Profit** - есть в стратегиях
3. **Delta Calculations** - `DeltaCalculator` для 15m, 1h, 3h, BTC, Market
4. **Market Filters** - базовая структура в `filters.rs`
5. **Strategy Configuration** - `config_parser.rs` с парсингом параметров
6. **Order Book** - L2/L3 в `orderbook.rs`
7. **HFT Backtest Engine** - tick-by-tick симуляция
8. **SaaS Architecture** - `users`, `user_strategies`, `client_api_keys`

## 🔴 КРИТИЧНО - Приоритет 1

### 1. Global Risk Management (Autostart Settings)
**Из скриншотов:**
- `Stop If loss greater then $X per N trades`
- `Stop If loss greater then $X per N hours AND trades > N`
- `Auto reset session per N hours`
- `Global Panic Sell If BTC rate changed > X%`
- `Global Panic Sell If all markets avg. rate drops > X%`

**Что нужно:**
```rust
// src/risk/global.rs
pub struct GlobalRiskManager {
    max_loss_per_trades: (f64, usize),  // ($, количество трейдов)
    max_loss_per_hours: (f64, u32, usize), // ($, часов, мин. трейдов)
    auto_reset_interval_hours: Option<u32>,
    panic_sell_on_btc_delta: Option<(f64, f64)>, // (drop %, raise %)
    panic_sell_on_market_delta: Option<f64>, // (drop %)
    current_session_loss: f64,
    session_trades: usize,
    session_start_time: DateTime<Utc>,
}

impl GlobalRiskManager {
    pub fn check_stop_conditions(&self, current_pnl: f64) -> RiskAction;
    pub fn check_btc_delta_panic(&self, btc_delta_1h: f64) -> bool;
    pub fn check_market_delta_panic(&self, market_delta_1h: f64) -> bool;
    pub fn should_reset_session(&self) -> bool;
}
```

### 2. Enhanced Session Management
**Из скриншотов:**
- `Stop If loss greater then $X per 28 trades` с Reset
- `Actual: -5.13$ 28 trades` - отслеживание текущих показателей
- `Auto reset session per 1 hours`

**Что нужно:**
```rust
// src/strategy/moon_strategies/sessions.rs - РАСШИРИТЬ
pub struct SessionManager {
    sessions: HashMap<String, SessionState>, // по символу или стратегии
    global_session: GlobalSessionState,
}

pub struct SessionState {
    pub symbol: String,
    pub strategy_id: String,
    pub pnl: f64,
    pub trades_count: usize,
    pub start_time: DateTime<Utc>,
    pub last_reset: DateTime<Utc>,
    pub auto_reset_interval: Option<Duration>,
    pub max_loss_per_trades: Option<(f64, usize)>,
    pub max_loss_per_time: Option<(f64, Duration, usize)>, // ($, время, мин. трейдов)
    pub order_size_multiplier: f64, // Динамическое изменение размера ордера
    pub penalty_until: Option<DateTime<Utc>>, // Время блокировки после убытка
}

impl SessionManager {
    pub fn update_session(&mut self, symbol: &str, pnl_delta: f64);
    pub fn check_stop_conditions(&self, symbol: &str) -> SessionAction;
    pub fn should_reset(&self, symbol: &str) -> bool;
    pub fn get_order_size_multiplier(&self, symbol: &str) -> f64;
}
```

### 3. Auto Stop on Errors / Ping
**Из скриншотов:**
- `Auto Stop if errors level >= 3` (с Panic Sell опцией)
- `Auto Stop if Ping > 1000 ms` (с Panic Sell опцией)
- `Restart in N minutes` после остановки

**Что нужно:**
```rust
// src/risk/auto_stop.rs
pub struct AutoStopManager {
    max_error_level: u32,
    current_error_level: u32,
    max_ping_ms: u64,
    panic_sell_on_stop: bool,
    restart_after_minutes: Option<u32>,
    stopped_at: Option<DateTime<Utc>>,
}

impl AutoStopManager {
    pub fn record_error(&mut self);
    pub fn check_ping(&mut self, ping_ms: u64) -> bool;
    pub fn should_restart(&self) -> bool;
}
```

### 4. Panic Sell System
**Из скриншотов (Main Settings):**
- `Panic Sell drop price to [actual buy] +X%`
- `Panic Sell Spread: X%`
- `Auto Panic Sell If price drops < [actual buy] -X%`
- `Panic Sell If BIDs at [buy] +X% drops`

**Что нужно:**
```rust
// src/risk/panic_sell.rs
pub struct PanicSellManager {
    enabled: bool,
    drop_to_percent: f64, // % от цены покупки
    spread_percent: f64,
    auto_panic_if_drop: Option<f64>, // Автоматический паник при падении < X%
    panic_if_bids_drop: Option<f64>, // Паник если BID упали на X%
}

impl PanicSellManager {
    pub fn should_panic_sell(&self, buy_price: f64, current_price: f64, best_bid: Option<f64>) -> Option<f64>;
    pub fn calculate_panic_price(&self, buy_price: f64) -> f64;
}
```

## 🟠 ВАЖНО - Приоритет 2

### 5. Iceberg Orders
**Из скриншотов (Advanced > Engine settings):**
- `Iceberg Buys` / `Iceberg Sells`
- `Use Iceberg only If Price Step < X%`

**Что нужно:**
```rust
// src/backtest/orderbook.rs - РАСШИРИТЬ Order
pub struct Order {
    // ... существующие поля
    pub is_iceberg: bool,
    pub visible_size: Option<f64>, // Видимый размер для айсберга
    pub hidden_size: f64, // Скрытый размер
}

impl OrderBook {
    fn process_iceberg_order(&mut self, order: Order);
}
```

### 6. Liquidation Control
**Из скриншотов (Advanced > Engine settings):**
- `Liquidation Control` (выделено оранжевым - критично!)

**Что нужно:**
```rust
// src/risk/liquidation.rs
pub struct LiquidationControl {
    enabled: bool,
    max_leverage: u32,
    maintenance_margin_rate: f64,
    liquidation_price_threshold: f64, // % до ликвидации
}

impl LiquidationControl {
    pub fn check_liquidation_risk(&self, position: &Position, mark_price: f64, balance: f64) -> LiquidationWarning;
    pub fn should_reduce_position(&self, warning: &LiquidationWarning) -> Option<f64>;
}
```

### 7. Auto Leverage
**Из скриншотов (Advanced > Engine settings):**
- `Auto Leverage` чекбокс

**Что нужно:**
```rust
// src/risk/leverage.rs
pub struct AutoLeverageManager {
    enabled: bool,
    max_leverage: u32,
    adjustment_factor: f64, // Как агрессивно менять плечо
    volatility_threshold: f64, // При какой волатильности менять
}

impl AutoLeverageManager {
    pub fn calculate_optimal_leverage(&self, volatility: f64, current_leverage: u32) -> u32;
}
```

### 8. Enhanced Market Filters
**Из скриншотов (Main Settings):**
- `Dont buy If price changed > X%` ✅ (есть базовая версия)
- `Dont buy If pump Q < X`
- `Dont buy If Daily Vol. < X BTC`
- `Dont buy If 3h Vol. > X BTC`
- `Dont buy already pumped`
- `Black List` с "Exclude from delta"
- `Dont buy newly added coins N minutes`

**Что нужно:**
```rust
// src/backtest/filters.rs - РАСШИРИТЬ
pub struct MarketFilters {
    // ... существующие
    pub dont_buy_if_price_changed_more: Option<f64>, // %
    pub min_pump_quality: Option<f64>, // "pump Q"
    pub min_daily_volume_btc: Option<f64>,
    pub max_3h_volume_btc: Option<f64>,
    pub dont_buy_pumped: bool,
    pub black_list: Vec<String>,
    pub exclude_blacklist_from_delta: bool,
    pub dont_buy_new_coins_minutes: Option<u32>,
}
```

### 9. Good Pump Detection
**Из скриншотов (Main Settings):**
- `Good pump` conditions:
  - `If price changed < X%`
  - `AND If pump Q > X`
  - `AND If actual buy < X%`

**Что нужно:**
```rust
// src/analytics/pump_detector.rs
pub struct PumpDetector {
    max_price_change_percent: f64,
    min_pump_quality: f64,
    max_buy_percent: f64,
}

impl PumpDetector {
    pub fn detect_good_pump(&self, price_history: &[f64], volume_history: &[f64]) -> Option<PumpSignal>;
    pub fn calculate_pump_quality(&self, price_change: f64, volume: f64, buy_volume: f64) -> f64;
}
```

### 10. Order Management Enhancements
**Из скриншотов (Main Settings):**
- `Cancel buys on sell fills`
- `Fit sell order in best place in the order book`
- `Dont cancel small BUYs`
- `Cancel small SELLS`

**Что нужно:**
```rust
// src/backtest/emulator.rs - РАСШИРИТЬ
pub struct EmulatorSettings {
    // ... существующие
    pub cancel_buys_on_sell_fill: bool,
    pub fit_sell_in_orderbook: bool,
    pub min_buy_size_to_cancel: Option<f64>,
    pub max_sell_size_to_keep: Option<f64>,
}
```

## 🟡 ПОЛЕЗНО - Приоритет 3

### 11. Visual Chart Settings (для investor_portal)
**Из скриншотов (User Interface):**
- `Show profit in $` на графике
- `Draw StopLoss line`
- `Draw Pending orders buy price`
- `Pending orders spread`
- `Order Book zones opacity`
- `Draw Panic Sell zone`

**Реализация:** В веб-интерфейсе (investor_portal) при визуализации бэктестов

### 12. MoonStream-like Streaming
**Из скриншотов (MoonStream):**
- Stream connection с packet loss monitoring
- Stream Server на порту (для SaaS - стриминг результатов)

**Что нужно:**
```rust
// src/streaming/mod.rs
pub struct DataStream {
    server_ip: String,
    port: u16,
    packet_loss: f64,
    error_count: u64,
}

impl DataStream {
    pub fn stream_backtest_progress(&mut self, progress: BacktestProgress) -> Result<()>;
}
```

### 13. VPS Optimization Settings
**Из скриншотов (System Settings):**
- `VDS optimized mode`
- `Use GPU Canvas` (для графиков)
- `Use Direct2D` (для графиков)
- `Use memory for charts: X%`
- Page file recommendations

**Реализация:** Документация и systemd service для деплоя на VPS

### 14. Restart Conditions
**Из скриншотов (Autostart):**
- `Restart when: BTC delta > X AND BTC delta < Y AND market delta > Z`

**Что нужно:**
```rust
// src/risk/restart_conditions.rs
pub struct RestartConditions {
    btc_delta_range: Option<(f64, f64)>,
    market_delta_min: Option<f64>,
}

impl RestartConditions {
    pub fn should_restart(&self, btc_delta: f64, market_delta: f64) -> bool;
}
```

## 📝 Рекомендации по реализации

### Фаза 1 (Критично - сразу):
1. Global Risk Manager
2. Enhanced Session Management  
3. Panic Sell System
4. Auto Stop on Errors/Ping

### Фаза 2 (Важно - следующий спринт):
5. Liquidation Control
6. Enhanced Market Filters
7. Good Pump Detection
8. Order Management Enhancements

### Фаза 3 (Полезно - когда будет время):
9. Iceberg Orders
10. Auto Leverage
11. Visual Chart Settings
12. Streaming Infrastructure

## Заметки

- Многие функции из MoonBot уже частично реализованы в наших стратегиях
- Нужно вынести общую логику в модуль `risk/` для переиспользования
- Session Management - это ключевой компонент для SaaS, где клиенты хотят видеть статистику по сессиям
- Liquidation Control критично для высоких плеч (100x, 125x)

