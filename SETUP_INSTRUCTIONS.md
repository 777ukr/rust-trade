# 🚀 Однострочная команда для создания проекта

## Быстрый старт

Выполните эту команду для создания всего проекта:

```bash
mkdir -p /home/crypto/sites/cryptotrader.com && cd /home/crypto/sites/cryptotrader.com && (if [ ! -f setup.sh ]; then curl -fsSL https://raw.githubusercontent.com/your-repo/setup.sh -o setup.sh || echo "Download failed, using local script"; fi) && bash setup.sh
```

Или если скрипт уже на месте:

```bash
bash /home/crypto/sites/cryptotrader.com/setup.sh
```

## Что создается:

✅ Rust проект с полной структурой
✅ Модули: API Gateway, Parser, Screener, Indicators, Strategy
✅ Workspace файл для Cursor IDE
✅ Стоп-лосс и стратегия торговли
✅ Все зависимости в Cargo.toml
✅ Gitignore и README

## После установки:

1. Откройте workspace:
```bash
code /home/crypto/sites/cryptotrader.com/crypto_trader.code-workspace
```

2. Соберите проект:
```bash
cd /home/crypto/sites/cryptotrader.com
cargo build
```

3. Запустите:
```bash
cargo run
```

## Структура проекта:

```
crypto_trader/
├── src/
│   ├── api/          # API Gateway для бирж
│   ├── parser/       # Парсер рыночных данных
│   ├── screener/     # Скринер для поиска возможностей
│   ├── indicators/   # Технические индикаторы (RSI, MACD, Bollinger)
│   ├── strategy/     # Торговая стратегия со стоп-лоссом
│   ├── models/       # Модели данных
│   └── utils/        # Утилиты
├── config/           # Конфигурационные файлы
├── data/             # Данные и логи
└── tests/            # Тесты
```

