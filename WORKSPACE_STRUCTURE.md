# Структура Workspace для проектов

## Рекомендуемая организация

Для эффективной работы с несколькими проектами рекомендуется использовать **отдельные workspace** для каждого проекта.

## Преимущества отдельной организации

1. **Изоляция конфигураций** - каждый проект имеет свои настройки
2. **Специфичные cursorrules** - правила для каждого проекта отдельно
3. **Чистое индексирование** - Cursor индексирует только нужные файлы
4. **Избежание конфликтов** - разные зависимости, порты, конфигурации
5. **Быстрая навигация** - легче переключаться между проектами

## Структура проектов

```
/home/crypto/sites/cryptotrader.com/
├── freqtrade/              # Freqtrade стратегии и рейтинг система
│   ├── .cursorrules        # Правила для Freqtrade
│   └── freqtrade.code-workspace
│
├── jesse-test/             # Jesse тестирование
│   ├── .cursorrules        # Правила для Jesse
│   └── jesse.code-workspace
│
├── hummingbot-master/      # Hummingbot архитектура
│   ├── .cursorrules        # Правила для Hummingbot
│   └── hummingbot.code-workspace
│
├── OctoBot-master/         # OctoBot визуализация
│   ├── .cursorrules        # Правила для OctoBot
│   └── octobot.code-workspace
│
└── crypto_trader.code-workspace  # Общий workspace для навигации
```

## Использование

### Для работы над конкретным проектом:

1. Откройте отдельный workspace:
   - `freqtrade/freqtrade.code-workspace` - для работы с Freqtrade
   - `jesse-test/jesse.code-workspace` - для работы с Jesse
   - `hummingbot-master/hummingbot.code-workspace` - для работы с Hummingbot
   - `OctoBot-master/octobot.code-workspace` - для работы с OctoBot

2. Cursor автоматически:
   - Применит `.cursorrules` из корня проекта
   - Проиндексирует только файлы этого проекта
   - Использует правильные настройки

### Для навигации между проектами:

Откройте `crypto_trader.code-workspace` - общий workspace со всеми проектами для быстрого переключения.

## Настройка портов для тестирования

Каждый проект использует свой порт:

- **Freqtrade**: 8080 (по умолчанию)
- **Jesse**: 9001 (настроен в jesse-test)
- **Hummingbot**: 8080 (если есть веб-интерфейс)
- **OctoBot**: 5001 (по умолчанию)

## Правила для каждого проекта

### Freqtrade (.cursorrules)
- Python 3.11+
- FastAPI для API
- Авто-обнаружение стратегий
- Абсолютные пути обязательны

### Jesse (.cursorrules)
- Python 3.10+
- FastAPI + Uvicorn
- Требует Redis и PostgreSQL
- Отдельный порт 9001

### Hummingbot (.cursorrules)
- Python 3.10+
- Модульная архитектура стратегий
- WebSocket для live данных

### OctoBot (.cursorrules)
- Python 3.10+
- Продвинутая визуализация
- Chart.js / Plotly.js

## Переключение между проектами

1. **File → Open Workspace from File...**
2. Выберите нужный `.code-workspace` файл
3. Cursor перезагрузит контекст и правила

## Общий workspace (опционально)

`crypto_trader.code-workspace` содержит все проекты для:
- Быстрого поиска по всем проектам
- Копирования кода между проектами
- Общего обзора структуры

**Но для активной разработки используйте отдельные workspace!**


