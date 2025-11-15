# 🔧 Настройка подключения к базе данных

## ⚠️ Проблема

Ошибка: `password authentication failed for user "user"`

Это означает, что нужно настроить правильные учетные данные для PostgreSQL.

## ✅ Решение

### Вариант 1: Через переменную окружения (Рекомендуется)

```bash
# Установите правильный DATABASE_URL
export DATABASE_URL="postgresql://ваш_пользователь:ваш_пароль@localhost/trading_core"

# Примеры:
# export DATABASE_URL="postgresql://postgres:mypassword@localhost/trading_core"
# export DATABASE_URL="postgresql://crypto:password123@localhost/trading_core"
```

### Вариант 2: Через конфигурационный файл

Отредактируйте `config/development.toml`:

```toml
[database]
url = "postgresql://ваш_пользователь:ваш_пароль@localhost/trading_core"
max_connections = 5
min_connections = 1
max_lifetime = 1800
```

### Вариант 3: Создайте .env файл

Создайте файл `.env` в корне проекта:

```bash
cd /home/crypto/sites/cryptotrader.com/rust-trade
echo 'DATABASE_URL=postgresql://ваш_пользователь:ваш_пароль@localhost/trading_core' > .env
```

---

## 🔍 Как узнать правильные учетные данные

### Проверка существующего подключения

```bash
# Попробуйте подключиться к PostgreSQL
psql -U postgres -d trading_core

# Или
psql -U ваш_пользователь -d trading_core
```

### Если база данных не существует

```bash
# Создайте базу данных
createdb -U postgres trading_core

# Или через psql
psql -U postgres -c "CREATE DATABASE trading_core;"
```

### Если пользователь не существует

```bash
# Создайте пользователя
psql -U postgres -c "CREATE USER ваш_пользователь WITH PASSWORD 'ваш_пароль';"
psql -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE trading_core TO ваш_пользователь;"
```

---

## ✅ Проверка подключения

После настройки проверьте:

```bash
# Проверка через psql
psql $DATABASE_URL -c "SELECT 1;"

# Или через Python скрипт
python3 -c "import psycopg2; import os; conn = psycopg2.connect(os.getenv('DATABASE_URL')); print('✅ Подключение успешно!')"
```

---

## 🚀 После настройки

1. **Запустите миграции** (если нужно):

   ```bash
   psql $DATABASE_URL -f config/schema.sql
   ```

2. **Запустите импорт данных**:

   ```bash
   cd scripts
   python3 import_freqtrade_data.py --download --days 30
   ```

3. **Запустите бэктестинг**:

   ```bash
   cd trading-core
   cargo run backtest
   ```

---

## 📝 Примеры DATABASE_URL

```bash
# Локальный PostgreSQL с пользователем postgres
export DATABASE_URL="postgresql://postgres:postgres@localhost/trading_core"

# С пользователем crypto
export DATABASE_URL="postgresql://crypto:password@localhost/trading_core"

# С указанием порта
export DATABASE_URL="postgresql://user:password@localhost:5432/trading_core"

# Удаленная база данных
export DATABASE_URL="postgresql://user:password@remote-host:5432/trading_core"
```

---

**После настройки DATABASE_URL все должно работать! ✅**
