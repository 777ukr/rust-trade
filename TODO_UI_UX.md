# TODO: Доработка UI/UX Investor Portal

## ✅ Что уже есть:
1. HTML интерфейс (investor_portal.html)
2. Выбор стратегий, символов, плеча
3. Фильтрация прибыльных стратегий в UI
4. Базовые API endpoints
5. Визуализация результатов (cards, summary)

## ❌ Что нужно добавить (из истории чата):

### Критично (Приоритет 1):
1. **WebSocket стриминг прогресса** - `/ws/backtest/:id` для real-time прогресса
2. **Реальный запуск бэктеста** - интеграция с `BacktestEngine`, сейчас заглушка
3. **Сохранение в PostgreSQL** - через `DatabaseRepository.insert_backtest_result()`
4. **Рейтинг стратегий** - добавить `rating` с `stars` в ответы API

### Важно (Приоритет 2):
5. **Исторические данные** - скрипт для заполнения BTC/ETH/SOL за 180 дней
6. **Equity Curve график** - Chart.js визуализация P&L по времени
7. **Таблица всех сделок** - детальная таблица trades
8. **Фоновый запуск** - tokio spawn для неблокирующего выполнения

### Опционально (Приоритет 3):
9. **Авторизация JWT** - защита endpoints
10. **Фильтры API** - `?only_profitable=true&sort_by=roi`
11. **Экспорт результатов** - CSV/JSON download

## Текущий статус:
- `investor_portal.rs` line 157: `// TODO: Вызов реального бэктеста`
- Результаты только в памяти (`Arc<Mutex<Vec>>`)
- Нет WebSocket поддержки в Axum (нужно добавить feature "ws")
- Нет рейтинга в `StrategyResult`

