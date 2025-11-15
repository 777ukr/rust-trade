# ⚡ Быстрый старт - HTTP API для веб-интерфейса

## ✅ HTTP API сервер готов

Веб-интерфейс теперь может работать с реальными данными из базы через HTTP API.

## 🚀 Запуск

### 1. Запустите API сервер (в отдельном терминале)

```bash
cd /home/crypto/sites/cryptotrader.com/rust-trade/trading-core
export DATABASE_URL="postgresql://cryptotrader:cryptotrader@localhost/trading_core"
cargo run api
```

Вы увидите:

```
🌐 Starting Trading Core HTTP API Server
✅ Database connection established
✅ Cache initialized
🚀 Starting HTTP API server on http://0.0.0.0:8080
✅ HTTP API server listening on http://0.0.0.0:8080
📡 Available endpoints:
   GET /api/strategies - List available strategies
   GET /api/data/info - Get database information
   GET /api/backtest/validate?symbol=ETHUSDT&data_count=10000 - Validate backtest config
```

### 2. Запустите веб-интерфейс (в другом терминале)

```bash
cd /home/crypto/sites/cryptotrader.com/rust-trade/frontend
npm run dev
```

### 3. Откройте в браузере

```
http://localhost:3000/backtest
```

## ✅ Что будет работать

- ✅ **Список стратегий** - загружается из Rust backend (включая EMA BTC Week)
- ✅ **Список символов** - загружается из базы данных (ETHUSDT с реальными данными)
- ✅ **Валидация конфигурации** - проверка доступности данных

## 📊 Проверка API

```bash
# Проверка списка стратегий
curl http://localhost:8080/api/strategies

# Проверка данных
curl http://localhost:8080/api/data/info

# Валидация
curl "http://localhost:8080/api/backtest/validate?symbol=ETHUSDT&data_count=10000"
```

---

**Готово! Веб-интерфейс теперь работает с реальными данными! 🎉**
