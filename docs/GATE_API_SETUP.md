# Настройка Gate.io API для торговли

## Где добавить API ключи

### Способ 1: Файл .env (Рекомендуется)

**📍 ПРЯМОЙ ПУТЬ К ФАЙЛУ:**
```
/home/crypto/sites/cryptotrader.com/.env
```

Или относительно корня проекта:
```
./.env
```

1. Создайте файл `.env` в корне проекта:
```bash
cd /home/crypto/sites/cryptotrader.com
nano .env
# или
vim .env
# или любой другой редактор
```

2. Добавьте ваши ключи в файл `.env`:
```bash
GATEIO_API_KEY=ваш_ключ_здесь
GATEIO_SECRET_KEY=ваш_секрет_здесь
```

**Важно:** Файл должен находиться именно в корне проекта, рядом с `Cargo.toml`.

3. Проект автоматически загрузит переменные из `.env` (через `dotenvy::dotenv()`)

### Способ 2: Переменные окружения системы

```bash
export GATEIO_API_KEY=your_api_key
export GATEIO_SECRET_KEY=your_secret_key
```

### Способ 3: Кастомные имена переменных

Если хотите использовать другие имена, измените `config/gate_mvp.yaml`:

```yaml
credentials:
  api_key_env: my_custom_api_key_name
  api_secret_env: my_custom_secret_name
```

Тогда установите:
```bash
export my_custom_api_key_name=your_key
export my_custom_secret_name=your_secret
```

## Где хранятся ключи в коде

Ключи загружаются в файле `src/config/runner.rs` функцией `load_gate_credentials()`:

```93:109:src/config/runner.rs
pub fn load_gate_credentials(config: &RunnerConfig) -> Result<GateCredentials> {
    let creds = config.credentials.clone().unwrap_or_default();
    let key_env = creds
        .api_key_env
        .unwrap_or_else(|| "GATEIO_API_KEY".to_string());
    let secret_env = creds
        .api_secret_env
        .unwrap_or_else(|| "GATEIO_SECRET_KEY".to_string());

    let api_key = std::env::var(&key_env).with_context(|| format!("missing env var {key_env}"))?;
    let api_secret =
        std::env::var(&secret_env).with_context(|| format!("missing env var {secret_env}"))?;
    Ok(GateCredentials {
        api_key,
        api_secret,
    })
}
```

## Режимы работы

В `config/gate_mvp.yaml` есть режим `dry_run`:

```yaml
mode:
  dry_run: true   # true = только логирование, без реальных сделок
  dry_run: false  # false = реальная торговля (требуются API ключи)
```

## Проверка подключения

Запустите тест с `dry_run: true` для проверки без реальных сделок:
```bash
cargo run --bin gate_runner --features gate_exec
```

