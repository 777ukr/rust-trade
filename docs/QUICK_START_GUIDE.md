# 🚀 Quick Start Guide - Как запустить проект

## 📦 Что у вас есть:

### Основные модули:
1. **Gate.io Trading** - Интеграция с Gate.io API
2. **Strategy Backtesting** - 3 стратегии (Channel Split, Market Making, HFT)
3. **PostgreSQL Database** - Хранение результатов бэктестов
4. **Web Dashboard** - Визуализация результатов

## 🔧 Шаг 1: Настройка окружения

### 1.1 Установите зависимости (если еще не установлены):
```bash
# Rust (если не установлен)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# PostgreSQL (если нужна база данных)
sudo apt-get install postgresql postgresql-contrib
```

### 1.2 Создайте файл `.env`:
```bash
cd /home/crypto/sites/cryptotrader.com

# Создайте .env файл
cat > .env << 'EOF'
# Gate.io API (опционально, для реальной торговли)
gateio_api_key=your_api_key_here
gateio_secret_key=your_secret_key_here

# PostgreSQL (опционально, для сохранения в БД)
DATABASE_URL=postgresql://user:password@localhost:5432/cryptotrader
EOF
```

## 🗄️ Шаг 2: Настройка PostgreSQL (опционально)

### 2.1 Создайте базу данных:
```bash
# Создайте пользователя (если нужно)
sudo -u postgres createuser --interactive cryptotrader

# Создайте базу данных
createdb cryptotrader

# Или с паролем:
sudo -u postgres psql -c "CREATE USER cryptotrader WITH PASSWORD 'your_password';"
sudo -u postgres psql -c "CREATE DATABASE cryptotrader OWNER cryptotrader;"
```

### 2.2 Примените схему:
```bash
psql cryptotrader < database/schema.sql
```

### 2.3 Обновите `.env`:
```bash
# Добавьте в .env:
DATABASE_URL=postgresql://cryptotrader:your_password@localhost:5432/cryptotrader
```

## 🎯 Шаг 3: Запуск основных команд

### 3.1 Инвесторская демонстрация (ГЛАВНАЯ КОМАНДА):
```bash
# С реальными данными Gate.io (если настроены API ключи)
cargo run --bin investor_demo --features gate_exec

# Без API ключей (демо режим)
cargo run --bin investor_demo --features gate_exec
# Автоматически использует демо-режим если ключи не найдены
```

**Что делает:**
- Тестирует 3 стратегии на SOL, ETH, BTC
- Использует x100 плечо
- Реалистичные расчеты P&L и комиссий
- Сохраняет результаты в CSV: `data/investor_demo_results.csv`
- Сохраняет в PostgreSQL (если `DATABASE_URL` установлен)

### 3.2 Веб-дашборд для визуализации:
```bash
# Запустите сервер
cargo run --bin investor_dashboard --features dashboard

# Откройте в браузере:
# http://localhost:3000
```

### 3.3 Другие полезные команды:

```bash
# Анализ реальной торговли Gate.io
cargo run --bin gate_real_analysis --features gate_exec

# Простой просмотр результатов
cargo run --bin view_results

# Демо стратегий
cargo run --bin demo_strategies

# Бэктест SOL
cargo run --bin sol_backtest
```

## 📊 Шаг 4: Просмотр результатов

### 4.1 CSV файлы:
```bash
# Основные результаты
cat data/investor_demo_results.csv

# Анализ канальной торговли
cat data/channel_analysis.csv
```

### 4.2 PostgreSQL:
```bash
# Подключитесь к базе
psql cryptotrader

# Посмотрите результаты бэктестов
SELECT strategy_name, symbol, roi, total_pnl, total_trades 
FROM backtest_results 
ORDER BY created_at DESC 
LIMIT 10;

# Посмотрите логи стратегий
SELECT * FROM strategy_logs LIMIT 10;
```

## 🔍 Шаг 5: Проверка работы

### 5.1 Проверка компиляции:
```bash
# Проверка всех features
cargo check --features gate_exec,database

# Должно показать: "Finished"
```

### 5.2 Тестирование стратегий:
```bash
# Запустите тесты
cargo test --features gate_exec

# Тесты стратегий
cargo test --features gate_exec strategy_tests
```

## 🐛 Troubleshooting

### Проблема: "DATABASE_URL not found"
**Решение**: База данных опциональна. Результаты сохраняются в CSV в любом случае.

### Проблема: "Failed to connect to database"
**Решение**: 
1. Проверьте, что PostgreSQL запущен: `sudo systemctl status postgresql`
2. Проверьте `DATABASE_URL` в `.env`
3. Проверьте права пользователя: `psql -U cryptotrader -d cryptotrader`

### Проблема: "Feature `database` requires `gate_exec`"
**Решение**: Используйте оба features:
```bash
cargo run --bin investor_demo --features gate_exec,database
```

### Проблема: "API credentials not found"
**Решение**: Это нормально! Программа автоматически использует демо-режим с фиксированными значениями ($1250 баланс).

## 📝 Следующие шаги:

1. ✅ **Запустите investor_demo** - Основная демонстрация
2. ✅ **Настройте PostgreSQL** (опционально) - Для хранения результатов
3. ✅ **Откройте dashboard** - Для визуализации
4. 🔄 **Настройте API ключи Gate.io** (опционально) - Для реальных данных

## 🎯 Готовые команды (копируйте и запускайте):

```bash
# 1. Проверка компиляции
cargo check --features gate_exec,database

# 2. Запуск инвесторской демонстрации
cargo run --bin investor_demo --features gate_exec

# 3. Запуск дашборда
cargo run --bin investor_dashboard --features dashboard

# 4. Просмотр результатов
cat data/investor_demo_results.csv
```

## 📚 Дополнительная документация:

- `docs/DATABASE_SETUP.md` - Подробная настройка PostgreSQL
- `docs/INVESTOR_DEMO_GUIDE.md` - Руководство по investor_demo
- `docs/COMMAND_REFERENCE.md` - Все команды проекта
- `.cursorrules` - Правила разработки и архитектура

