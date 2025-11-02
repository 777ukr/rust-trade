# Быстрый старт: Подключение Gate.io и добавление стратегии

## Шаг 1: Добавьте API ключи Gate.io

**Создайте файл `.env` в корне проекта:**

```bash
# В корне проекта
nano .env
```

**Добавьте ваши ключи:**
```
GATEIO_API_KEY=ваш_api_ключ_здесь
GATEIO_SECRET_KEY=ваш_secret_ключ_здесь
```

**Важно:** Файл `.env` автоматически игнорируется git, поэтому ваши ключи не попадут в репозиторий.

## Шаг 2: Настройте конфигурацию

Откройте `config/gate_mvp.yaml` и проверьте настройки:

```yaml
strategy:
  symbol: SOL_USDT      # Ваша торговая пара
  size: 1                # Размер ордера
  spread_bps: 40.0       # Спред в базисных пунктах

mode:
  dry_run: true          # ВАЖНО: сначала true для теста без реальных сделок
```

## Шаг 3: Где добавить вашу логику

### Вариант A: Быстрое изменение существующей стратегии

Откройте `src/strategy/simple_quote.rs` и найдите метод `build_intents()`:

```rust
fn build_intents(&mut self, mid: f64) -> Vec<QuoteIntent> {
    // ИЗМЕНИТЕ ЭТУ ЛОГИКУ:
    let mut spread = mid * self.config.spread_bps / 10_000.0;
    
    // ВАША ЛОГИКА ЗДЕСЬ:
    // Например, динамический спред на основе волатильности
    // или справедливая цена с учетом других бирж
    
    let (bid_px, ask_px) = self.quote_levels(mid, half);
    // ...
}
```

### Вариант B: Создать новую стратегию

1. Создайте `src/strategy/my_strategy.rs` (см. `docs/STRATEGY_INTEGRATION.md`)
2. Добавьте в `src/strategy/mod.rs`:
   ```rust
   pub mod my_strategy;
   pub use my_strategy::MyStrategy;
   ```
3. Измените `src/bin/gate_runner.rs` - замените `SimpleQuoteStrategy` на `MyStrategy`

## Шаг 4: Тестовый запуск (dry-run)

```bash
# Убедитесь, что в config/gate_mvp.yaml: dry_run: true
cargo run --bin gate_runner --features gate_exec
```

Вы увидите логи котировок, но реальные сделки не будут отправляться.

## Шаг 5: Бэктестинг

### Способ 1: Использовать существующие логи

1. Запустите стратегию и соберите данные:
   ```bash
   cargo run --bin gate_runner --features gate_exec
   # Логи сохраняются в logs/gate_activity.csv
   ```

2. Проанализируйте в Python:
   ```bash
   cd scripts
   python3 evaluate_fair_value.py
   ```

### Способ 2: Создать собственный бэктестер

См. подробности в `docs/STRATEGY_INTEGRATION.md`, раздел "Бэктестинг"

## Шаг 6: Реальная торговля

**ВНИМАНИЕ:** Только после тщательного тестирования!

1. Измените `config/gate_mvp.yaml`:
   ```yaml
   mode:
     dry_run: false  # Включить реальную торговлю
   ```

2. Начните с минимальных размеров:
   ```yaml
   strategy:
     size: 0.1  # Начните с малого
   ```

3. Запустите:
   ```bash
   cargo run --bin gate_runner --features gate_exec
   ```

## Структура файлов

```
.
├── .env                          # ← ВАШИ API КЛЮЧИ ЗДЕСЬ (не в git)
├── config/
│   └── gate_mvp.yaml             # ← Конфигурация стратегии
├── src/
│   ├── strategy/
│   │   ├── simple_quote.rs      # ← Базовая стратегия (можно изменить)
│   │   └── mod.rs
│   └── bin/
│       └── gate_runner.rs        # ← Основной цикл выполнения
└── docs/
    ├── GATE_API_SETUP.md         # ← Детали настройки API
    └── STRATEGY_INTEGRATION.md   # ← Подробная документация
```

## Где что находится

| Что нужно сделать | Где это находится |
|------------------|-------------------|
| Добавить API ключи | `.env` файл в корне |
| Настроить параметры стратегии | `config/gate_mvp.yaml` |
| Изменить логику расчета цен | `src/strategy/simple_quote.rs` метод `build_intents()` |
| Добавить обработку данных других бирж | `src/strategy/simple_quote.rs` метод `on_market_update()` |
| Создать новую стратегию | `src/strategy/my_strategy.rs` (создать новый файл) |

## Полезные команды

```bash
# Тестовый запуск
cargo run --bin gate_runner --features gate_exec

# Проверка конфигурации
cat config/gate_mvp.yaml

# Просмотр логов в реальном времени
tail -f logs/gate_activity.csv

# Анализ логов Python
cd scripts && python3 evaluate_fair_value.py
```

## Нужна помощь?

- Детали настройки API: `docs/GATE_API_SETUP.md`
- Интеграция стратегий: `docs/STRATEGY_INTEGRATION.md`
- План тестирования: `docs/gate_testing_plan.md`

