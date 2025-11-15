# 🧪 Руководство по тестированию Dip Buy Strategy

## 📍 Где отображается стратегия

### 1. В терминале (CLI) - Рекомендуется для начала

```bash
cd trading-core
cargo run backtest
```

**Вы увидите:**

- Список стратегий (включая "Dip Buy Strategy (Low Frequency)")
- Интерактивный выбор параметров
- Результаты бэктестинга с метриками
- Детальный анализ сделок

### 2. В веб-интерфейсе (требует Desktop приложение)

Веб-интерфейс работает только в Tauri Desktop приложении:

```bash
cd frontend
npx tauri dev
```

**После запуска:**

- Откройте раздел "Backtest"
- Выберите "Dip Buy Strategy (Low Frequency)"
- Выберите ETHUSDT
- Запустите бэктестинг

---

## 📥 Загрузка исторических данных

### Быстрый способ: Использование Premium Data Provider

```bash
# 1. Установите зависимости
pip install psycopg2-binary pandas requests

# 2. Установите переменные окружения
export DATABASE_URL="postgresql://user:password@localhost/trading_core"
export KAIKO_API_KEY="ec47c618-04bb-4eff-a962-dad3fab8ca45"

# 3. Загрузите данные ETH 1m за 30 дней
cd /home/crypto/sites/cryptotrader.com/rust-trade/scripts
python3 import_freqtrade_data.py --download --days 30
```

**Что делает скрипт:**

1. Использует `/home/crypto/sites/cryptotrader.com/freqtrade/premium_data_provider.py`
2. Скачивает данные ETH 1m с Kaiko API
3. Конвертирует OHLCV в tick data (4 тика на свечу)
4. Импортирует в базу данных rust-trade

### Альтернатива: Импорт из файла

```bash
python3 import_freqtrade_data.py --file /path/to/ETH_USDT-1m.json --symbol ETHUSDT
```

---

## 🧪 Полный процесс тестирования

### Шаг 1: Загрузите данные

```bash
cd /home/crypto/sites/cryptotrader.com/rust-trade/scripts
python3 import_freqtrade_data.py --download --days 30
```

Ожидаемый вывод:

```
📥 Загрузка данных с Kaiko для ETH/USDT (1m)...
✅ Загружено 43200 свечей с Kaiko
📊 Конвертация 43200 свечей в tick data...
✅ Создано 172800 тиков
✅ Импортировано 172800 новых тиков
```

### Шаг 2: Запустите бэктестинг

```bash
cd /home/crypto/sites/cryptotrader.com/rust-trade/trading-core
cargo run backtest
```

**Интерактивный процесс:**

1. Выберите стратегию:

   ```
   🎯 Available Strategies:
     1) SMA Strategy
     2) RSI Strategy
     3) Dip Buy Strategy (Low Frequency)  ← Выберите это
   
   Select strategy (1-3): 3
   ```

2. Выберите символ:

   ```
   Select symbol: ETHUSDT
   ```

3. Укажите количество записей:

   ```
   Enter number of records to backtest (default: 10000): 50000
   ```

4. Начальный капитал:

   ```
   Enter initial capital (default: $10000): $10000
   ```

5. Комиссия:

   ```
   Enter commission rate % (default: 0.1%): 0.1
   ```

### Шаг 3: Анализ результатов

Вы увидите отчет:

```
BACKTEST RESULTS SUMMARY
============================================================
Strategy: Dip Buy Strategy (Low Frequency)
Initial Capital: $10000.00
Final Value: $10500.00
Total P&L: $500.00
Return: 5.00%

TRADING STATISTICS
------------------------------
Total Trades: 45
Winning Trades: 28 (62.2%)
Losing Trades: 17 (37.8%)
Profit Factor: 1.85

RISK METRICS
------------------------------
Max Drawdown: 2.50%
Sharpe Ratio: 1.25
Volatility: 15.30%
```

---

## 📊 Параметры стратегии

Текущие параметры (в `trading-core/src/backtest/strategy/dip_buy.rs`):

- **buy_dip_percent**: 0.2% - покупка при просадке от максимума
- **take_profit_percent**: 0.6% - продажа при прибыли
- **stop_loss_percent**: 0.22% - стоп-лосс

### Изменение параметров

Отредактируйте файл `trading-core/src/backtest/strategy/dip_buy.rs`:

```rust
pub fn new() -> Self {
    Self {
        buy_dip_percent: dec!(0.002),      // 0.2%
        take_profit_percent: dec!(0.006),   // 0.6%
        stop_loss_percent: dec!(0.0022),    // 0.22%
        // ...
    }
}
```

Затем пересоберите:

```bash
cd trading-core
cargo build --release
```

---

## 🔍 Проверка данных в базе

```sql
-- Проверка количества данных
SELECT 
    symbol,
    COUNT(*) as tick_count,
    MIN(timestamp) as first_tick,
    MAX(timestamp) as last_tick
FROM tick_data
WHERE symbol = 'ETHUSDT'
GROUP BY symbol;

-- Последние тики
SELECT timestamp, price, quantity, side
FROM tick_data
WHERE symbol = 'ETHUSDT'
ORDER BY timestamp DESC
LIMIT 20;
```

---

## ⚠️ Важные замечания

1. **Формат данных**: Скрипт создает 4 тика на каждую OHLCV свечу
2. **Символы**: Используйте формат `ETHUSDT` (без разделителя)
3. **База данных**: Убедитесь, что PostgreSQL запущен
4. **API ключи**: Kaiko ключ уже в скрипте, но можно использовать свой

---

## 🐛 Решение проблем

### "Module not found: premium_data_provider"

Проверьте путь в скрипте:

```python
sys.path.insert(0, '/home/crypto/sites/cryptotrader.com/freqtrade')
```

### "Database connection failed"

Проверьте DATABASE_URL:

```bash
export DATABASE_URL="postgresql://user:password@localhost/trading_core"
psql $DATABASE_URL -c "SELECT 1;"
```

### "No data imported"

Проверьте:

- Существуют ли данные в файле
- Правильный ли формат (Freqtrade JSON)
- Доступна ли база данных

---

## 📚 Дополнительная информация

- [BACKTEST_DIP_BUY_STRATEGY.md](BACKTEST_DIP_BUY_STRATEGY.md) - детальное руководство
- [DIP_BUY_STRATEGY.md](DIP_BUY_STRATEGY.md) - описание стратегии
- [REPORTS_GUIDE.md](REPORTS_GUIDE.md) - просмотр отчетов

---

**Готово! Теперь вы можете загрузить данные и протестировать стратегию! 🚀**
