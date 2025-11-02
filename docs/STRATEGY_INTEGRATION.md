# Интеграция вашей логики в стратегию Gate.io

## Архитектура стратегии

Текущая система использует трейт-подход для стратегий. Основные компоненты:

1. **SimpleQuoteStrategy** (`src/strategy/simple_quote.rs`) - базовая стратегия маркет-мейкинга
2. **Gate Runner** (`src/bin/gate_runner.rs`) - основной цикл выполнения
3. **Execution Gateway** - отправка ордеров на Gate.io

## Где добавить вашу логику

### Вариант 1: Расширить SimpleQuoteStrategy

Создайте новую стратегию, расширяющую или заменяющую `SimpleQuoteStrategy`:

```rust
// src/strategy/my_strategy.rs
use crate::strategy::{QuoteConfig, QuotePlan, ReferenceMeta};
use crate::execution::{QuoteIntent, ClientOrderId};
use std::time::Instant;

pub struct MyCustomStrategy {
    base_config: QuoteConfig,
    // Добавьте ваши поля для логики
    my_state: f64,
    indicators: Vec<f64>,
}

impl MyCustomStrategy {
    pub fn new(config: QuoteConfig) -> Self {
        Self {
            base_config: config,
            my_state: 0.0,
            indicators: Vec::new(),
        }
    }

    // Ваша логика обработки обновлений рынка
    pub fn on_market_update(
        &mut self,
        price: f64,
        meta: Option<ReferenceMeta>,
        now: Instant,
    ) -> Vec<ClientOrderId> {
        // ВАША ЛОГИКА ЗДЕСЬ
        // Например, анализ индикаторов, вычисление справедливой цены
        
        // Пример: вычисление вашей целевой цены
        let my_fair_value = self.calculate_fair_value(price);
        
        // Определение, нужно ли пересчитать котировки
        if self.should_reprice(price, my_fair_value) {
            // Возвращаем ID ордеров для отмены
            Vec::new() // или список ClientOrderId для отмены
        } else {
            Vec::new()
        }
    }

    // Ваша логика планирования котировок
    pub fn plan_quotes(&mut self, now: Instant) -> Option<QuotePlan> {
        // ВАША ЛОГИКА РАСЧЕТА КОТИРОВОК
        // Определите bid/ask цены и размеры на основе вашей логики
        
        let fair_value = self.calculate_fair_value(...)?;
        let spread = self.calculate_spread(fair_value);
        
        let intents = self.build_intents(fair_value, spread);
        
        Some(QuotePlan {
            reference_price: fair_value,
            cancels: Vec::new(),
            intents,
            planned_at: now,
            reference_meta: None,
        })
    }

    // Вспомогательные методы для вашей логики
    fn calculate_fair_value(&self, current_price: f64) -> f64 {
        // Ваша логика расчета справедливой цены
        // Например, на основе индикаторов, других бирж и т.д.
        current_price
    }

    fn calculate_spread(&self, fair_value: f64) -> f64 {
        // Ваша логика расчета спреда
        fair_value * 0.001 // 10 bps пример
    }

    fn build_intents(&mut self, fair_value: f64, spread: f64) -> Vec<QuoteIntent> {
        // Построение QuoteIntent на основе ваших расчетов
        // См. пример в SimpleQuoteStrategy::build_intents
        todo!()
    }

    pub fn handle_report(&mut self, report: &ExecutionReport) {
        // Обработка исполнения ордеров
        // Обновите ваше состояние на основе fills
    }
}
```

### Вариант 2: Создать трейт для стратегии

```rust
// src/strategy/trait.rs
pub trait TradingStrategy {
    fn on_market_update(&mut self, price: f64, meta: Option<ReferenceMeta>, now: Instant) -> Vec<ClientOrderId>;
    fn plan_quotes(&mut self, now: Instant) -> Option<QuotePlan>;
    fn commit_plan(&mut self, plan: &QuotePlan);
    fn handle_report(&mut self, report: &ExecutionReport);
}
```

## Интеграция в Gate Runner

Измените `src/bin/gate_runner.rs`:

```rust
// Замените:
let strategy = Arc::new(Mutex::new(SimpleQuoteStrategy::new(
    config.strategy.clone(),
)));

// На:
let strategy = Arc::new(Mutex::new(MyCustomStrategy::new(
    config.strategy.clone(),
)));
```

## Доступ к рыночным данным

В `gate_runner.rs` стратегия получает данные через каналы:

- `reference_rx` - обновления цен от Gate.io
- `fast_ref_rx` - быстрые обновления для отмены ордеров

Данные приходят в формате `ReferenceEvent`:
```rust
pub struct ReferenceEvent {
    pub price: f64,
    pub source: String,  // "gate", "binance", "bybit", etc.
    pub ts_ns: Option<u64>,
    pub received_at: Instant,
}
```

## Доступ к данным других бирж

Если включены фиды других бирж в `config/gate_mvp.yaml`:
```yaml
feeds:
  gate: true
  binance: auto
  bybit: auto
  bitget: auto
  okx: auto
```

Данные других бирж также приходят через `ReferenceEvent` с разными `source`.

## Пример: Использование данных нескольких бирж

```rust
pub struct MultiExchangeStrategy {
    gate_prices: VecDeque<(f64, Instant)>,
    binance_prices: VecDeque<(f64, Instant)>,
    bybit_prices: VecDeque<(f64, Instant)>,
}

impl MultiExchangeStrategy {
    pub fn on_market_update(&mut self, price: f64, meta: Option<ReferenceMeta>, now: Instant) {
        if let Some(meta) = &meta {
            match meta.source.as_str() {
                "gate" => self.gate_prices.push_back((price, now)),
                "binance" => self.binance_prices.push_back((price, now)),
                "bybit" => self.bybit_prices.push_back((price, now)),
                _ => {}
            }
        }
    }

    fn calculate_fair_value(&self) -> f64 {
        // Ваша логика объединения цен с разных бирж
        // Например, взвешенное среднее по латентности
        let gate_price = self.gate_prices.back()?.0;
        let binance_price = self.binance_prices.back()?.0;
        // ...
        (gate_price + binance_price) / 2.0
    }
}
```

## Бэктестинг

### Текущие инструменты

1. **Python скрипты** в `scripts/`:
   - `evaluate_fair_value.py` - оценка стратегий справедливой цены
   - Использует исторические данные из CSV логов

2. **Логирование** - Gate Runner автоматически логирует:
   - Котировки (quotes)
   - Исполнения (fills)
   - Отмены (cancels)
   - Рыночные снапшоты

### Создание бэктестера

Для полноценного бэктестинга нужно:

1. **Записать исторические данные**:
   ```bash
   # Запустите gate_runner с логированием
   cargo run --bin gate_runner --features gate_exec
   # Логи сохраняются в logs/gate_activity.csv
   ```

2. **Создать бэктестер** (пример структуры):
```rust
// src/backtest/mod.rs
pub struct Backtester {
    strategy: Box<dyn TradingStrategy>,
    historical_data: Vec<HistoricalTick>,
}

pub struct HistoricalTick {
    pub timestamp: Instant,
    pub price: f64,
    pub source: String,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
}

impl Backtester {
    pub fn run(&mut self) -> BacktestResult {
        // Проиграть исторические данные через стратегию
        // Симулировать исполнение ордеров
        // Вычислить P&L, Sharpe ratio и т.д.
    }
}
```

3. **Использовать Python для анализа**:
   - Загрузите CSV логи в pandas
   - Симулируйте исполнение по историческим данным
   - Оцените производительность стратегии

### Пример анализа бэктеста (Python)

```python
import pandas as pd

# Загрузить логи
df = pd.read_csv('logs/gate_activity.csv')

# Симулировать исполнение ордеров
# (пример - нужно доработать под вашу логику)
def simulate_execution(quotes_df, market_data_df):
    positions = []
    for quote in quotes_df:
        # Найти исполнение на основе рыночных данных
        fill = find_matching_fill(quote, market_data_df)
        if fill:
            positions.append(calculate_pnl(fill))
    return positions

# Вычислить метрики
def calculate_metrics(positions):
    return {
        'total_pnl': sum(p['pnl'] for p in positions),
        'win_rate': sum(1 for p in positions if p['pnl'] > 0) / len(positions),
        'sharpe': calculate_sharpe(positions),
    }
```

## Следующие шаги

1. **Добавьте API ключи** в `.env` (см. `docs/GATE_API_SETUP.md`)
2. **Создайте свою стратегию** расширяя или заменяя `SimpleQuoteStrategy`
3. **Запустите в dry-run режиме** для проверки логики
4. **Соберите исторические данные** для бэктестинга
5. **Создайте бэктестер** или используйте Python скрипты для анализа

## Полезные файлы для изучения

- `src/strategy/simple_quote.rs` - пример стратегии
- `src/bin/gate_runner.rs` - основной цикл выполнения
- `src/execution/` - модуль исполнения ордеров
- `config/gate_mvp.yaml` - конфигурация
- `scripts/evaluate_fair_value.py` - пример анализа стратегий

