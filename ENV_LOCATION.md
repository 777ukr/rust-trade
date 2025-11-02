# 📍 Расположение файла .env

## Прямой путь:

```
/home/crypto/sites/cryptotrader.com/.env
```

## Быстрая команда для редактирования:

```bash
nano /home/crypto/sites/cryptotrader.com/.env
```

или из корня проекта:

```bash
cd /home/crypto/sites/cryptotrader.com
nano .env
```

## Формат файла:

```
GATEIO_API_KEY=ваш_api_ключ_здесь
GATEIO_SECRET_KEY=ваш_secret_ключ_здесь
```

## Проверка:

```bash
# Проверить существует ли файл
ls -la /home/crypto/sites/cryptotrader.com/.env

# Посмотреть содержимое (БЕЗ ПЕЧАТИ СЕКРЕТОВ)
head -2 /home/crypto/sites/cryptotrader.com/.env
```

**⚠️ ВАЖНО:** Файл `.env` уже в `.gitignore`, ваши ключи не попадут в git.

