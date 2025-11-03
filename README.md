# 🚀 Crypto Trader - Professional HFT Backtesting Platform

Полнофункциональная платформа для бэктестинга криптовалютных торговых стратегий с поддержкой HFT (High-Frequency Trading) уровня точности.

## ✨ Основные возможности

### 🎯 HFT Backtest Engine
- **Tick-by-Tick Simulation**: Симуляция каждого тика с миллисекундной точностью
- **Full Order Book (L2/L3)**: Полная реконструкция стакана с очередями исполнения
- **Latency Modeling**: Реалистичные задержки сети (10-20 мс)
- **Multiple Fill Models**: FIFO, ProRata, TimePriority
- **Monte Carlo Simulation**: Многократные прогоны для статистики
- **MoonBot Compatibility**: Поддержка .bin формата данных

### 📊 Фильтры и селекторы рынков
- Дельта фильтры (1м, 3м, 5м, 15м, 30м, 1ч, 24ч)
- Объемные фильтры и ликвидность
- Фильтр ставки финансирования
- Топ-N рынков по критерию

### ⭐ Система рейтинга стратегий
- Автоматический расчет рейтинга (0-10) по 4 критериям:
  - Profitability (P&L, profit factor, win rate)
  - Stability (Sharpe ratio)
  - Risk (max drawdown)
  - Fill Rate
- Автоматическое присвоение звезд (0-5)

### 🎮 Торговые стратегии
- **Channel Split**: Канальная стратегия с дроблением ордеров
- **Market Making**: Маркет-мейкинг
- **HFT**: High-frequency trading
- **Long/Short Trailing**: Трейлинг стоп на лонг/шорт
- **EMA Reversal**: EMA развороты
- **MShot**: MoonBot MShot (базовая реализация)
- **MStrike, Hook, Spread**: В разработке

### 🌐 SaaS Platform
- JWT аутентификация
- Управление стратегиями через UI
- Индивидуальные настройки для каждого клиента
- Система заявок клиентов
- API key management

## 🚀 Быстрый старт

### Требования
- Rust 1.70+
- PostgreSQL 14+
- Tokio runtime

### Установка

```bash
# Клонировать репозиторий
git clone <repo-url>
cd cryptotrader.com

# Установить зависимости
cargo build --features gate_exec,dashboard,database

# Настроить БД
createdb cryptotrader
psql cryptotrader < database/schema.sql
psql cryptotrader < database/saas_schema.sql

# Установить переменные окружения
export DATABASE_URL="postgresql://user:password@localhost:5432/cryptotrader"
export JWT_SECRET="your-secret-key-change-in-production"
```

### Запуск

```bash
# Investor Portal (веб-интерфейс)
cargo run --bin investor_portal --features dashboard,database

# Открыть в браузере
# http://localhost:8080
```

## 📖 Документация

- **[HFT Backtest Spec](docs/HFT_BACKTEST_SPEC.md)** - Полная спецификация бэктестера
- **[API Endpoints](docs/API_ENDPOINTS.md)** - Документация по API
- **[Checklist](docs/CHECKLIST.md)** - Чеклист разработки
- **[Architecture Review](docs/ARCHITECTURE_REVIEW.md)** - Обзор архитектуры

## 🏗️ Архитектура

```
src/
├── backtest/           # HFT бэктестер
│   ├── engine.rs      # Основной движок
│   ├── orderbook.rs    # L2/L3 стакан
│   ├── filters.rs      # Фильтры рынков
│   └── metrics.rs      # Метрики и рейтинг
├── strategy/           # Торговые стратегии
│   └── moon_strategies/ # MoonBot стратегии
├── database/          # Работа с БД
├── auth/              # JWT аутентификация
└── saas/              # SaaS функционал
```

## 🧪 Пример использования

```rust
use rust_test::backtest::*;

// Настройки бэктеста
let settings = BacktestSettings {
    tick_interval_ms: 2,
    latency_ms_range: (10, 20),
    use_orderbook_l3: true,
    fill_model: FillModel::FIFO,
    ..Default::default()
};

// Создаем движок
let mut engine = BacktestEngine::new(settings);

// Загружаем данные
let mut replay = ReplayEngine::new(Default::default());
replay.load_bin_file("data/BTC_2024_01.bin")?;

for stream in replay.take_streams() {
    engine.add_stream(stream);
}

// Запускаем бэктест
let result = engine.run()?;

println!("Rating: {:.2}/10 ⭐: {}", 
    result.rating.overall_rating, result.rating.stars);
```

## 📊 Метрики бэктеста

После выполнения бэктеста получаете:
- Total P&L
- Win Rate, Profit Factor
- Max Drawdown, Sharpe Ratio
- Fill Rate
- Средняя длительность сделок
- **Рейтинг стратегии** (0-10) и **звезды** (0-5)

## 🔒 Безопасность

- JWT токены для авторизации
- Argon2/Bcrypt для хеширования паролей
- Шифрование API ключей
- Rate limiting (в разработке)

## 🎯 Roadmap

- [x] HFT Backtest Engine
- [x] Order Book (L2/L3)
- [x] Система рейтинга
- [ ] Полная реализация MoonBot стратегий
- [ ] WebSocket стриминг
- [ ] ИИ оптимизация параметров
- [ ] Live trading integration

## 🤝 Contributing

См. `.cursorrules` для guidelines по разработке.

## 📝 Лицензия

[Указать лицензию]

## 🙏 Благодарности

- MoonBot за вдохновение и спецификацию стратегий
- Rust community за отличные crates (Tokio, Axum, SQLx)
