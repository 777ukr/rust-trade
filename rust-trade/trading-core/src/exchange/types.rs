// =================================================================
// exchange/types.rs - Data Structures
// =================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Parameters for querying historical trade data
#[derive(Debug, Clone)]
pub struct HistoricalTradeParams {
    pub symbol: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
}

impl HistoricalTradeParams {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol: symbol.to_uppercase(),
            start_time: None,
            end_time: None,
            limit: None,
        }
    }

    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Binance specific trade message format
#[derive(Debug, Deserialize, Clone)]
pub struct BinanceTradeMessage {
    /// Symbol
    #[serde(rename = "s")]
    pub symbol: String,

    /// Trade ID
    #[serde(rename = "t")]
    pub trade_id: u64,

    /// Price
    #[serde(rename = "p")]
    pub price: String,

    /// Quantity
    #[serde(rename = "q")]
    pub quantity: String,

    /// Trade time
    #[serde(rename = "T")]
    pub trade_time: u64,

    /// Is the buyer the market maker?
    #[serde(rename = "m")]
    pub is_buyer_maker: bool,
}

/// Binance WebSocket stream wrapper for combined streams
#[derive(Debug, Deserialize)]
pub struct BinanceStreamMessage {
    /// Stream name (e.g., "btcusdt@trade")
    pub stream: String,

    /// The actual trade data
    pub data: BinanceTradeMessage,
}

/// Binance subscription message format
#[derive(Debug, Serialize)]
pub struct BinanceSubscribeMessage {
    pub method: String,
    pub params: Vec<String>,
    pub id: u32,
}

impl BinanceSubscribeMessage {
    pub fn new(streams: Vec<String>) -> Self {
        Self {
            method: "SUBSCRIBE".to_string(),
            params: streams,
            id: 1,
        }
    }
}

/// Gate.io specific trade message format
#[derive(Debug, Deserialize, Clone)]
pub struct GateioTradeMessage {
    /// Trade ID
    pub id: u64,
    
    /// Create time (Unix timestamp in seconds)
    #[serde(rename = "create_time")]
    pub create_time: u64,
    
    /// Contract symbol (e.g., "BTC_USDT")
    pub contract: String,
    
    /// Price as string
    pub price: String,
    
    /// Size (quantity)
    pub size: i64,
    
    /// Role: "maker" or "taker"
    pub role: String,
}

/// Gate.io WebSocket stream wrapper
#[derive(Debug, Deserialize)]
pub struct GateioStreamMessage {
    /// Time (Unix timestamp)
    pub time: Option<u64>,
    
    /// Channel name
    pub channel: Option<String>,
    
    /// Event type
    pub event: Option<String>,
    
    /// The actual trade data
    pub result: GateioTradeMessage,
}

/// Gate.io subscription message format
#[derive(Debug, Serialize)]
pub struct GateioSubscribeMessage {
    pub time: u64,
    pub channel: String,
    pub event: String,
    pub payload: Vec<String>,
}

impl GateioSubscribeMessage {
    pub fn new(channel: String) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            time: now,
            channel: channel.clone(),
            event: "subscribe".to_string(),
            payload: vec![channel],
        }
    }
}
