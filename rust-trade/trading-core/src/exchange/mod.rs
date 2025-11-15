// exchange/mod.rs
pub mod binance;
pub mod gateio;
pub mod errors;
pub mod traits;
pub mod types;
pub mod utils;

// Re-export main interfaces for easy access
pub use binance::BinanceExchange;
pub use gateio::GateioExchange;
pub use errors::ExchangeError;
pub use traits::Exchange;
pub use types::*;

use std::sync::Arc;

/// Factory function to create an exchange instance based on provider name
/// 
/// # Arguments
/// * `provider` - Exchange provider name: "binance" or "gateio" (case-insensitive)
/// 
/// # Returns
/// * `Ok(Arc<dyn Exchange>)` - Exchange instance wrapped in Arc
/// * `Err(String)` - Error message if provider is unknown
/// 
/// # Examples
/// ```
/// use trading_core::exchange::create_exchange;
/// 
/// let exchange = create_exchange("gateio")?;
/// ```
pub fn create_exchange(provider: &str) -> Result<Arc<dyn Exchange>, String> {
    match provider.to_lowercase().as_str() {
        "binance" => Ok(Arc::new(BinanceExchange::new())),
        "gateio" | "gate.io" => Ok(Arc::new(GateioExchange::new())),
        _ => Err(format!(
            "Unknown exchange provider: {}. Supported providers: binance, gateio",
            provider
        )),
    }
}
