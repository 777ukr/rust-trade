# 🔑 Настройка API ключей и переменных окружения

## Обязательные переменные окружения

### 1. PostgreSQL Database
```bash
export DATABASE_URL="postgresql://user:password@localhost:5432/cryptotrader"
```

### 2. Gate.io API (опционально, для загрузки данных)
```bash
export GATE_API_KEY="your_api_key_here"
export GATE_API_SECRET="your_api_secret_here"
```

**Примечание**: API ключи не обязательны для публичных endpoints (загрузка исторических данных). 
Но для полного доступа и live торговли они нужны.

## Как получить Gate.io API ключи

1. Зарегистрируйтесь на [Gate.io](https://www.gate.io/)
2. Перейдите в **API Management** → **Create API Key**
3. Выберите права:
   - **Read Only** - для загрузки данных (достаточно)
   - **Trade** - для live торговли
   - **Withdraw** - для вывода средств (не рекомендуется для ботов)
4. Скопируйте `API Key` и `API Secret`
5. Установите в `.env` файл или экспортируйте переменные

## Файл .env (рекомендуется)

Создайте файл `.env` в корне проекта:

```bash
# PostgreSQL
DATABASE_URL=postgresql://postgres:password@localhost:5432/cryptotrader

# Gate.io API (опционально)
GATE_API_KEY=your_key_here
GATE_API_SECRET=your_secret_here

# Логирование (опционально)
RUST_LOG=info
# Для отладки используйте:
# RUST_LOG=debug
# RUST_LOG=rust_test::backtest=debug,rust_test::strategy=info
```

## Проверка настройки

### 1. Проверка БД
```bash
cargo run --bin investor_portal --features dashboard,database,gate_exec
# Должно быть: ✅ Подключено к PostgreSQL
```

### 2. Проверка API ключей
```bash
cargo run --bin load_historical_data --features database,gate_exec
# Если ключи не установлены - будет предупреждение, но продолжит работу
```

## Логирование

### Уровни логирования через RUST_LOG:

- `RUST_LOG=error` - только ошибки
- `RUST_LOG=warn` - предупреждения и ошибки
- `RUST_LOG=info` - информационные сообщения (по умолчанию)
- `RUST_LOG=debug` - отладочная информация
- `RUST_LOG=trace` - максимальная детализация

### Для конкретных модулей:
```bash
RUST_LOG=rust_test::backtest=debug,rust_test::strategy=info
```

### Примеры использования:

```bash
# Только ошибки
RUST_LOG=error cargo run --bin investor_portal

# Отладка бэктестера
RUST_LOG=rust_test::backtest=debug cargo run --bin investor_portal

# Полная отладка
RUST_LOG=debug cargo run --bin investor_portal
```

## Безопасность

⚠️ **ВАЖНО**: Никогда не коммитьте `.env` файл в Git!

```bash
# Добавьте в .gitignore
echo ".env" >> .gitignore
```

## Следующие шаги

1. ✅ Настройте `DATABASE_URL`
2. ✅ (Опционально) Получите Gate.io API ключи
3. ✅ Загрузите исторические данные:
   ```bash
   cargo run --bin load_historical_data --features database,gate_exec
   ```
4. ✅ Запустите Investor Portal:
   ```bash
   cargo run --bin investor_portal --features dashboard,database,gate_exec
   ```

