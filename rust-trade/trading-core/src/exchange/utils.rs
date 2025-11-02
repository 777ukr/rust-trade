// =================================================================
// exchange/utils.rs - Utility Functions
// =================================================================

use super::{BinanceTradeMessage, ExchangeError, GateioTradeMessage};
use crate::data::types::{TickData, TradeSide};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;

/// Convert Binance trade message to standard TickData format
pub fn convert_binance_to_tick_data(msg: BinanceTradeMessage) -> Result<TickData, ExchangeError> {
    // Convert timestamp from milliseconds to DateTime
    let timestamp = DateTime::from_timestamp_millis(msg.trade_time as i64)
        .ok_or_else(|| ExchangeError::ParseError("Invalid timestamp".to_string()))?;

    // Parse price and quantity as Decimal for precision
    let price = Decimal::from_str(&msg.price)
        .map_err(|e| ExchangeError::ParseError(format!("Invalid price '{}': {}", msg.price, e)))?;

    let quantity = Decimal::from_str(&msg.quantity).map_err(|e| {
        ExchangeError::ParseError(format!("Invalid quantity '{}': {}", msg.quantity, e))
    })?;

    // Validate parsed values
    if price <= Decimal::ZERO {
        return Err(ExchangeError::ParseError(
            "Price must be positive".to_string(),
        ));
    }

    if quantity <= Decimal::ZERO {
        return Err(ExchangeError::ParseError(
            "Quantity must be positive".to_string(),
        ));
    }

    // Determine trade side based on maker flag
    // If buyer is maker, it means a sell order was filled (seller was taker)
    // If buyer is not maker, it means a buy order was filled (buyer was taker)
    let side = if msg.is_buyer_maker {
        TradeSide::Sell
    } else {
        TradeSide::Buy
    };

    Ok(TickData::new(
        timestamp,
        msg.symbol,
        price,
        quantity,
        side,
        msg.trade_id.to_string(),
        msg.is_buyer_maker,
    ))
}

/// Validate symbol format for Binance
pub fn validate_binance_symbol(symbol: &str) -> Result<String, ExchangeError> {
    if symbol.is_empty() {
        return Err(ExchangeError::InvalidSymbol(
            "Symbol cannot be empty".to_string(),
        ));
    }

    let symbol = symbol.to_uppercase();

    // Basic validation: should be alphanumeric and reasonable length
    if !symbol.chars().all(char::is_alphanumeric) {
        return Err(ExchangeError::InvalidSymbol(format!(
            "Symbol '{}' contains invalid characters",
            symbol
        )));
    }

    if symbol.len() < 3 || symbol.len() > 20 {
        return Err(ExchangeError::InvalidSymbol(format!(
            "Symbol '{}' has invalid length",
            symbol
        )));
    }

    Ok(symbol)
}

/// Build WebSocket subscription streams for Binance
pub fn build_binance_trade_streams(symbols: &[String]) -> Result<Vec<String>, ExchangeError> {
    if symbols.is_empty() {
        return Err(ExchangeError::InvalidSymbol(
            "No symbols provided".to_string(),
        ));
    }

    let mut streams = Vec::with_capacity(symbols.len());

    for symbol in symbols {
        let validated_symbol = validate_binance_symbol(symbol)?;
        streams.push(format!("{}@trade", validated_symbol.to_lowercase()));
    }

    Ok(streams)
}

/// Convert Gate.io trade message to standard TickData format
pub fn convert_gateio_to_tick_data(msg: GateioTradeMessage) -> Result<TickData, ExchangeError> {
    // Convert timestamp from seconds to DateTime
    let timestamp = DateTime::from_timestamp(msg.create_time as i64, 0)
        .ok_or_else(|| ExchangeError::ParseError("Invalid timestamp".to_string()))?;

    // Parse price as Decimal for precision
    let price = Decimal::from_str(&msg.price)
        .map_err(|e| ExchangeError::ParseError(format!("Invalid price '{}': {}", msg.price, e)))?;

    // Size can be negative for sells, use absolute value for quantity
    let quantity = Decimal::from(msg.size.abs());

    // Validate parsed values
    if price <= Decimal::ZERO {
        return Err(ExchangeError::ParseError(
            "Price must be positive".to_string(),
        ));
    }

    if quantity <= Decimal::ZERO {
        return Err(ExchangeError::ParseError(
            "Quantity must be positive".to_string(),
        ));
    }

    // Determine trade side:
    // If size > 0: buy, if size < 0: sell
    // Role "maker" means maker order, "taker" means taker order
    let side = if msg.size > 0 {
        TradeSide::Buy
    } else {
        TradeSide::Sell
    };

    // is_buyer_maker: true if buyer is maker (role == "maker" and side == Buy)
    let is_buyer_maker = msg.role == "maker" && side == TradeSide::Buy;

    // Convert contract format: BTC_USDT -> BTCUSDT
    let symbol = msg.contract.replace("_", "");

    Ok(TickData::new(
        timestamp,
        symbol,
        price,
        quantity,
        side,
        msg.id.to_string(),
        is_buyer_maker,
    ))
}

/// Validate symbol format for Gate.io
pub fn validate_gateio_symbol(symbol: &str) -> Result<String, ExchangeError> {
    if symbol.is_empty() {
        return Err(ExchangeError::InvalidSymbol(
            "Symbol cannot be empty".to_string(),
        ));
    }

    let symbol = symbol.to_uppercase();

    // Gate.io uses format like BTCUSDT, ETHUSDT
    if !symbol.ends_with("USDT") {
        return Err(ExchangeError::InvalidSymbol(format!(
            "Gate.io symbol '{}' must end with USDT",
            symbol
        )));
    }

    if symbol.len() < 7 || symbol.len() > 20 {
        return Err(ExchangeError::InvalidSymbol(format!(
            "Symbol '{}' has invalid length",
            symbol
        )));
    }

    Ok(symbol)
}

/// Build WebSocket subscription channels for Gate.io
pub fn build_gateio_trade_streams(symbols: &[String]) -> Result<Vec<String>, ExchangeError> {
    if symbols.is_empty() {
        return Err(ExchangeError::InvalidSymbol(
            "No symbols provided".to_string(),
        ));
    }

    let mut channels = Vec::with_capacity(symbols.len());

    for symbol in symbols {
        let validated_symbol = validate_gateio_symbol(symbol)?;
        // Gate.io format: futures.trades.BTC_USDT
        let contract = validated_symbol.replace("USDT", "_USDT");
        channels.push(format!("futures.trades.{}", contract));
    }

    Ok(channels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_validation() {
        assert!(validate_binance_symbol("BTCUSDT").is_ok());
        assert!(validate_binance_symbol("btcusdt").is_ok());
        assert!(validate_binance_symbol("").is_err());
        assert!(validate_binance_symbol("BTC-USDT").is_err());
    }

    #[test]
    fn test_stream_building() {
        let symbols = vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()];
        let streams = build_binance_trade_streams(&symbols).unwrap();

        assert_eq!(streams.len(), 2);
        assert_eq!(streams[0], "btcusdt@trade");
        assert_eq!(streams[1], "ethusdt@trade");
    }
}
