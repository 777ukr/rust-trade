//! Система логирования с настройкой уровней через переменные окружения
//! Использует env_logger для гибкого управления логами

use std::env;

/// Инициализация системы логирования
/// 
/// Уровни логирования настраиваются через переменную окружения RUST_LOG:
/// - RUST_LOG=error - только ошибки
/// - RUST_LOG=warn - предупреждения и ошибки
/// - RUST_LOG=info - информационные сообщения (по умолчанию)
/// - RUST_LOG=debug - отладочная информация
/// - RUST_LOG=trace - максимальная детализация
/// 
/// Можно указать для конкретного модуля:
/// - RUST_LOG=rust_test::backtest=debug,rust_test::strategy=info
/// 
/// Примеры:
/// ```bash
/// # Только ошибки
/// RUST_LOG=error cargo run
/// 
/// # Отладка для бэктестера
/// RUST_LOG=rust_test::backtest=debug cargo run
/// 
/// # Полная отладка
/// RUST_LOG=debug cargo run
/// ```
pub fn init_logging() {
    // Устанавливаем уровень по умолчанию если не указан
    if env::var("RUST_LOG").is_err() {
        unsafe {
            env::set_var("RUST_LOG", "info");
        }
    }
    
    // Инициализируем env_logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_secs()
        .format_module_path(true)
        .format_target(false)
        .init();
    
    log::info!("✅ Система логирования инициализирована");
    log::info!("📝 Уровень логирования: {}", env::var("RUST_LOG").unwrap_or_default());
}

/// Проверка включено ли логирование
pub fn is_logging_enabled() -> bool {
    env::var("RUST_LOG").is_ok()
}

/// Получить текущий уровень логирования
pub fn get_log_level() -> String {
    env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
}

/// Включить логирование для конкретного модуля
pub fn enable_module_logging(module: &str, level: &str) {
    let current = env::var("RUST_LOG").unwrap_or_default();
    let new = if current.is_empty() {
        format!("{}={}", module, level)
    } else {
        format!("{},{}={}", current, module, level)
    };
    unsafe {
        env::set_var("RUST_LOG", new);
    }
}

