use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Database {
    #[serde(default = "default_database_url")]
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime: u64,
}

fn default_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://cryptotrader:cryptotrader@localhost/trading_core".to_string())
}

fn default_max_connections() -> u32 {
    5
}

fn default_min_connections() -> u32 {
    1
}

fn default_max_lifetime() -> u64 {
    1800
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u32,
}

#[derive(Debug, Deserialize)]
pub struct MemoryCache {
    pub max_ticks_per_symbol: usize,
    pub ttl_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct RedisCache {
    #[serde(default = "default_redis_url")]
    pub url: String,
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
    #[serde(default = "default_redis_ttl")]
    pub ttl_seconds: u64,
    #[serde(default = "default_redis_max_ticks")]
    pub max_ticks_per_symbol: usize,
}

fn default_redis_url() -> String {
    std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string())
}

fn default_pool_size() -> u32 {
    10
}

fn default_redis_ttl() -> u64 {
    3600
}

fn default_redis_max_ticks() -> usize {
    10000
}

#[derive(Debug, Deserialize)]
pub struct Cache {
    pub memory: MemoryCache,
    pub redis: RedisCache,
}

#[derive(Debug, Deserialize)]
pub struct PaperTrading {
    pub enabled: bool,
    pub strategy: String,
    pub initial_capital: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExchangeConfig {
    #[serde(default = "default_exchange")]
    pub provider: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub api_secret: Option<String>,
}

fn default_exchange() -> String {
    "binance".to_string()
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: Database,
    pub server: Server,
    pub cache: Cache,
    pub symbols: Vec<String>,
    pub paper_trading: PaperTrading,
    #[serde(default = "default_exchange_config")]
    pub exchange: ExchangeConfig,
}

fn default_exchange_config() -> ExchangeConfig {
    ExchangeConfig {
        provider: default_exchange(),
        api_key: None,
        api_secret: None,
    }
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        // Try to find config file in multiple locations
        let config_file = format!("{}.toml", run_mode);
        let possible_paths = vec![
            format!("../config/{}", config_file),
            format!("config/{}", config_file),
            format!("../../config/{}", config_file),
        ];
        
        let mut config_path = None;
        for path in &possible_paths {
            if std::path::Path::new(path).exists() {
                config_path = Some(path.clone());
                break;
            }
        }
        
        let mut builder = Config::builder();
        if let Some(path) = config_path {
            builder = builder.add_source(File::with_name(&path.replace(".toml", "")).required(true));
        } else {
            // Fallback to default path
            builder = builder.add_source(File::with_name(&format!("../config/{}", run_mode)).required(true));
        }

        // Always set database.url from environment if available, or use default
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://cryptotrader:cryptotrader@localhost/trading_core".to_string());
        builder = builder.set_override("database.url", database_url)?;

        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            builder = builder.set_override("cache.redis.url", redis_url)?;
        }

        let s = builder.build()?;
        s.try_deserialize()
    }
}
