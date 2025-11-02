// Configuration utilities
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub stop_loss_percent: f64,
    pub take_profit_percent: f64,
    pub min_volume: f64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            stop_loss_percent: 2.0,
            take_profit_percent: 5.0,
            min_volume: 1000000.0,
        }
    }
}

pub fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
