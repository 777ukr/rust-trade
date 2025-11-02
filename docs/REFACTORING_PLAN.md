# План рефакторинга: Разбиение больших файлов

## Цель
Разбить файлы, превышающие 800 строк, на модульные компоненты для лучшей поддерживаемости и регулярных коммитов.

## Критерии
- **Идеал**: 300-500 строк
- **Приемлемо**: 800 строк
- **Максимум**: 1200 строк
- **Требует рефакторинга**: > 800 строк

## Файлы, требующие внимания

### Критические (> 1200 строк)

#### 1. `src/base_classes/engine.rs` - **1992 строки**
**Статус**: Критично - требует немедленного рефакторинга

**План разбиения**:
```
src/base_classes/engine/
├── mod.rs              # Публичный API и реэкспорты
├── state.rs            # Управление состоянием (EngineState, etc.)
├── handlers.rs         # Обработчики событий
├── feeds.rs            # Управление фидами (spawn_state_engine, etc.)
├── snapshot.rs         # Генерация снапшотов
└── helpers.rs          # Вспомогательные функции
```

**Коммиты**:
1. `refactor: extract engine state management to separate module`
2. `refactor: split engine event handlers into handlers.rs`
3. `refactor: extract feed management from engine.rs`
4. `refactor: move snapshot generation to snapshot.rs`
5. `refactor: complete engine.rs modularization`

#### 2. `src/logging/quote_raw.rs` - **1428 строк**
**Статус**: Критично

**План разбиения**:
```
src/logging/quote_raw/
├── mod.rs              # Основной трейт и реэкспорты
├── writer.rs           # Запись в файл
├── formatter.rs        # Форматирование записей
├── types.rs            # Типы для логирования
└── csv.rs              # CSV-специфичная логика
```

**Коммиты**:
1. `refactor: extract quote_raw writer logic`
2. `refactor: split quote_raw formatter into separate module`
3. `refactor: complete quote_raw.rs modularization`

#### 3. `src/logging/quote_block.rs` - **1425 строк**
**Статус**: Критично

**План разбиения**: Аналогично `quote_raw.rs`

### Высокий приоритет (800-1200 строк)

#### 4. `src/execution/gate_ws.rs` - **1034 строки**
**Статус**: Близко к лимиту, но пока приемлемо

**План разбиения** (если вырастет):
```
src/execution/gate_ws/
├── mod.rs              # Gateway и Worker структуры
├── worker.rs            # GateWsWorker логика
├── messages.rs          # Обработка сообщений
├── orders.rs            # Управление ордерами
└── auth.rs              # Аутентификация и подписи
```

#### 5. `src/bin/gate_runner.rs` - **953 строки**
**Статус**: Можно оставить, но рассмотреть разбиение

**План разбиения**:
```
src/bin/gate_runner/
├── main.rs             # Точка входа
├── config.rs           # Загрузка конфигурации
├── handlers.rs         # Обработчики событий (handle_market_update, etc.)
├── quotes.rs           # Логика котировок
└── lifecycle.rs        # Управление жизненным циклом
```

## Стратегия рефакторинга

### Принципы
1. **Инкрементальный подход**: Один модуль за раз
2. **Рабочие коммиты**: Каждый шаг должен компилироваться и проходить тесты
3. **Сохранить функциональность**: Не ломать существующий код
4. **Тесты первыми**: Добавлять тесты для извлекаемых модулей

### Процесс

#### Шаг 1: Подготовка
```bash
# Создать новую ветку
git checkout -b refactor/engine-modularization

# Убедиться, что тесты проходят
cargo test
```

#### Шаг 2: Извлечение типов
```rust
// Создать src/base_classes/engine/types.rs
pub struct EngineState { ... }
// Переместить связанные типы
```

**Коммит**: `refactor: extract engine types to types.rs`

#### Шаг 3: Извлечение функций
```rust
// Создать src/base_classes/engine/helpers.rs
pub fn helper_function() { ... }
```

**Коммит**: `refactor: extract helper functions from engine.rs`

#### Шаг 4: Обновление импортов
```rust
// В engine/mod.rs
mod types;
mod helpers;
pub use types::*;
```

**Коммит**: `refactor: update engine.rs imports after extraction`

#### Шаг 5: Тестирование
```bash
cargo test
cargo build
```

**Коммит**: `test: verify engine refactoring with integration tests`

### Чеклист перед коммитом

- [ ] Код компилируется без ошибок
- [ ] Все тесты проходят
- [ ] Нет нарушений существующего API (или изменения задокументированы)
- [ ] Импорты обновлены
- [ ] Документация обновлена при необходимости

## Пример: Рефакторинг engine.rs

### Текущая структура
```rust
// engine.rs (1992 строки)
pub struct EngineState { ... }
pub fn spawn_state_engine() { ... }
// ... много кода ...
```

### Целевая структура

**Шаг 1: Создать директорию**
```bash
mkdir -p src/base_classes/engine
mv src/base_classes/engine.rs src/base_classes/engine/mod.rs
```

**Шаг 2: Извлечь типы**
```rust
// src/base_classes/engine/types.rs
pub struct EngineState { ... }
pub struct FeedConfig { ... }
```

**Шаг 3: Извлечь обработчики**
```rust
// src/base_classes/engine/handlers.rs
pub async fn handle_market_update() { ... }
pub async fn handle_orderbook_update() { ... }
```

**Шаг 4: Обновить mod.rs**
```rust
// src/base_classes/engine/mod.rs
mod types;
mod handlers;
mod state;
mod feeds;

pub use types::*;
pub use handlers::*;
// ...
```

## График рефакторинга

### Фаза 1: Критические файлы (2-3 недели)
- [ ] `engine.rs` → модульная структура
- [ ] `quote_raw.rs` → модульная структура  
- [ ] `quote_block.rs` → модульная структура

### Фаза 2: Высокий приоритет (1-2 недели)
- [ ] `gate_ws.rs` → разделение если вырастет
- [ ] `gate_runner.rs` → опциональное разбиение

### Фаза 3: Поддержание (ongoing)
- Мониторинг размера новых файлов
- Рефакторинг при достижении 800 строк
- Регулярные проверки: `find src -name "*.rs" -exec wc -l {} + | sort -rn`

## Инструменты для мониторинга

### Проверка размера файлов
```bash
# Найти большие файлы
find src -name "*.rs" -exec wc -l {} + | sort -rn | head -10

# Проверить конкретный файл
wc -l src/base_classes/engine.rs
```

### Git hooks для проверки
```bash
# .git/hooks/pre-commit
#!/bin/bash
find src -name "*.rs" -exec wc -l {} + | awk '$1 > 1200 {print "WARNING: " $2 " has " $1 " lines (max 1200)"; exit 1}'
```

## Рекомендации

1. **Начните с самого большого**: `engine.rs`
2. **Делайте маленькие коммиты**: Один модуль = один коммит
3. **Тестируйте часто**: После каждого извлечения
4. **Документируйте изменения**: В коммитах и PR
5. **Не торопитесь**: Качественный рефакторинг лучше быстрого

## Контрольные точки

После каждого большого рефакторинга:
- [ ] Все тесты проходят
- [ ] Производительность не деградировала (benchmarks)
- [ ] Документация обновлена
- [ ] Code review пройден

