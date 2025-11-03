# 📡 API Endpoints - Полная документация

## Статус реализации

### ✅ Реализовано
- `GET /` - Главная страница (investor_portal.html)
- `GET /api/strategies` - Список доступных стратегий
- `GET /api/leverages` - Список доступных плеч
- `GET /api/symbols` - Список доступных символов
- `POST /api/backtest` - Запуск бэктеста
- `GET /api/results` - Получить результаты бэктестов
- `GET /api/results/latest` - Последние результаты

### 🚧 В разработке
- `POST /api/auth/register` - Регистрация пользователя
- `POST /api/auth/login` - Вход в систему
- `GET /api/auth/me` - Текущий пользователь
- `POST /api/auth/logout` - Выход

### 📋 Запланировано

#### Управление стратегиями
- `GET /api/strategies/my` - Мои стратегии (требует auth)
- `POST /api/strategies` - Создать стратегию
- `GET /api/strategies/:id` - Получить стратегию
- `PUT /api/strategies/:id` - Обновить стратегию
- `DELETE /api/strategies/:id` - Удалить стратегию
- `GET /api/strategies/top` - Топ стратегий по рейтингу
- `GET /api/strategies/public` - Публичные стратегии
- `POST /api/strategies/parse` - Парсить конфиг `##Begin_Strategy...##End_Strategy`

#### Бэктесты
- `POST /api/backtest` - Запустить бэктест (✅ частично)
- `POST /api/backtest/stream` - WebSocket стрим прогресса
- `GET /api/backtest/:id` - Получить результаты бэктеста
- `GET /api/backtest/history` - История бэктестов пользователя
- `DELETE /api/backtest/:id` - Удалить бэктест

#### Рейтинги и метрики
- `GET /api/strategies/:id/rating` - Рейтинг стратегии
- `GET /api/strategies/:id/metrics` - Детальные метрики
- `POST /api/strategies/:id/rate` - Оценить стратегию (user rating)
- `GET /api/metrics/compare` - Сравнить несколько стратегий

#### Управление API ключами
- `POST /api/keys/exchange` - Добавить API ключи биржи
- `GET /api/keys/exchange` - Список ключей
- `PUT /api/keys/exchange/:id` - Обновить ключи
- `DELETE /api/keys/exchange/:id` - Удалить ключи

#### Заявки клиентов
- `POST /api/requests` - Создать заявку (custom strategy, feature request)
- `GET /api/requests` - Мои заявки
- `GET /api/requests/:id` - Детали заявки
- `PUT /api/requests/:id` - Обновить заявку (admin)

#### Live торговля
- `POST /api/trading/start` - Запустить live торговлю
- `POST /api/trading/stop` - Остановить торговлю
- `GET /api/trading/sessions` - Активные сессии
- `GET /api/trading/sessions/:id` - Детали сессии
- `WebSocket /ws/trading/:session_id` - Live обновления

## Детальная спецификация

### POST /api/backtest

**Request:**
```json
{
  "strategies": ["channel_split", "market_making"],
  "symbols": ["BTC_USDT", "ETH_USDT"],
  "leverage": 100.0,
  "initial_balance": 1000.0,
  "use_rebate": true,
  "settings": {
    "tick_interval_ms": 2,
    "latency_ms_range": [10, 20],
    "use_orderbook_l3": true,
    "fill_model": "FIFO"
  }
}
```

**Response:**
```json
{
  "success": true,
  "message": "Backtest completed",
  "results": [
    {
      "strategy_name": "channel_split",
      "symbol": "BTC_USDT",
      "initial_balance": 1000.0,
      "final_balance": 1250.0,
      "total_pnl": 250.0,
      "total_fees": 5.0,
      "fees_after_rebate": 2.0,
      "trades": 150,
      "wins": 120,
      "losses": 30,
      "win_rate": 80.0,
      "roi": 25.0,
      "profit_factor": 2.5,
      "max_drawdown": 5.0,
      "leverage": 100.0,
      "profitable": true,
      "rating": {
        "profitability_score": 8.5,
        "stability_score": 7.2,
        "risk_score": 9.0,
        "fill_rate_score": 8.0,
        "overall_rating": 8.2,
        "stars": 4
      }
    }
  ]
}
```

### POST /api/auth/login

**Request:**
```json
{
  "username": "user123",
  "password": "secure_password"
}
```

**Response:**
```json
{
  "success": true,
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "uuid",
    "username": "user123",
    "email": "user@example.com",
    "is_admin": false
  }
}
```

### POST /api/strategies

**Request:**
```json
{
  "strategy_name": "my_custom_strategy",
  "description": "Custom EMA reversal",
  "config_text": "##Begin_Strategy...##End_Strategy",
  "initial_balance": 1000.0,
  "leverage": 10,
  "tags": ["scalping", "long", "ema"],
  "category": "Custom",
  "is_public": false
}
```

**Response:**
```json
{
  "success": true,
  "strategy": {
    "id": "uuid",
    "strategy_name": "my_custom_strategy",
    "created_at": "2024-01-15T10:30:00Z",
    "rating": {
      "overall_rating": 0.0,
      "stars": 0
    }
  }
}
```

## WebSocket Endpoints

### /ws/backtest/:backtest_id

Стрим прогресса бэктеста:
```json
{
  "type": "progress",
  "backtest_id": "uuid",
  "progress": 45.5,
  "current_tick": 45000,
  "total_ticks": 100000,
  "estimated_time_remaining_ms": 120000
}
```

Завершение:
```json
{
  "type": "complete",
  "backtest_id": "uuid",
  "result": { ... }
}
```

## Аутентификация

Большинство endpoints требуют JWT токен в заголовке:
```
Authorization: Bearer <token>
```

Исключения:
- `POST /api/auth/register`
- `POST /api/auth/login`
- `GET /api/strategies/public`
- `GET /api/strategies/top`

## Коды ошибок

- `200 OK` - Успешно
- `201 Created` - Ресурс создан
- `400 Bad Request` - Неверный запрос
- `401 Unauthorized` - Не авторизован
- `403 Forbidden` - Нет доступа
- `404 Not Found` - Ресурс не найден
- `409 Conflict` - Конфликт (например, стратегия уже существует)
- `500 Internal Server Error` - Ошибка сервера

## Rate Limiting

- Бэктесты: 5 одновременных на пользователя
- API запросы: 100 запросов в минуту
- WebSocket: без ограничений

