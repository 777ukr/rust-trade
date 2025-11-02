// Logger utilities
use log::{info, warn, error, debug};

pub fn init_logger() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
}
