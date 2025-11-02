// =================================================================
// exchange/gateio.rs - Gate.io Exchange Implementation
// =================================================================

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use super::{
    errors::ExchangeError,
    traits::Exchange,
    types::{HistoricalTradeParams, GateioTradeMessage, GateioStreamMessage, GateioSubscribeMessage},
    utils::{build_gateio_trade_streams, convert_gateio_to_tick_data},
};
use crate::data::types::TickData;

// Constants
const GATEIO_WS_URL: &str = "wss://fx-ws.gateio.ws/v4/ws/usdt";
const GATEIO_API_URL: &str = "https://api.gateio.ws/api/v4";
const RECONNECT_DELAY: Duration = Duration::from_secs(5);
const PING_INTERVAL: Duration = Duration::from_secs(30);

/// Gate.io exchange implementation
pub struct GateioExchange {
    ws_url: String,
    api_url: String,
    client: reqwest::Client,
}

impl GateioExchange {
    /// Create a new Gate.io exchange instance
    pub fn new() -> Self {
        Self {
            ws_url: GATEIO_WS_URL.to_string(),
            api_url: GATEIO_API_URL.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Parse WebSocket message and extract trade data
    fn parse_trade_message(&self, text: &str) -> Result<TickData, ExchangeError> {
        // First try to parse as stream message
        if let Ok(stream_msg) = serde_json::from_str::<GateioStreamMessage>(text) {
            return convert_gateio_to_tick_data(stream_msg.result);
        }

        // Fallback: try to parse as direct trade message
        if let Ok(trade_msg) = serde_json::from_str::<GateioTradeMessage>(text) {
            return convert_gateio_to_tick_data(trade_msg);
        }

        // Check if it's a subscription confirmation or other control message
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
            if value.get("error").is_some() {
                let error_msg = value["error"]["message"]
                    .as_str()
                    .unwrap_or("Unknown Gate.io error");
                return Err(ExchangeError::ApiError(error_msg.to_string()));
            }
            if value.get("id").is_some() || value.get("result").is_some() {
                // This is a subscription confirmation or ping/pong
                debug!("Received control message: {}", text);
                return Err(ExchangeError::ParseError(
                    "Control message, not trade data".to_string(),
                ));
            }
        }

        Err(ExchangeError::ParseError(format!(
            "Unable to parse Gate.io message: {}",
            text
        )))
    }

    /// Handle WebSocket connection with reconnection logic
    async fn handle_websocket_connection(
        &self,
        symbols: &[String],
        callback: Box<dyn Fn(TickData) + Send + Sync>,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<(), ExchangeError> {
        let channels = build_gateio_trade_streams(symbols)?;
        info!(
            "Connecting to Gate.io WebSocket with {} channels",
            channels.len()
        );

        let mut reconnect_attempts = 0;
        const MAX_RECONNECT_ATTEMPTS: u32 = 10;

        loop {
            // Check for shutdown signal before each connection attempt
            if shutdown_rx.try_recv().is_ok() {
                info!("Shutdown signal received, stopping WebSocket connection attempts");
                return Ok(());
            }

            match self
                .connect_and_subscribe(&channels, &callback, shutdown_rx.resubscribe())
                .await
            {
                Ok(()) => {
                    // Reset reconnect attempts on successful connection
                    reconnect_attempts = 0;
                    info!(
                        "WebSocket connection ended normally - checking if shutdown was requested"
                    );
                    return Ok(());
                }
                Err(e) => {
                    reconnect_attempts += 1;
                    error!(
                        "WebSocket connection failed (attempt {}): {}",
                        reconnect_attempts, e
                    );

                    if reconnect_attempts >= MAX_RECONNECT_ATTEMPTS {
                        return Err(ExchangeError::NetworkError(format!(
                            "Max reconnection attempts ({}) exceeded",
                            MAX_RECONNECT_ATTEMPTS
                        )));
                    }

                    warn!("Attempting to reconnect in {:?}...", RECONNECT_DELAY);

                    // Wait for reconnect delay or shutdown signal
                    tokio::select! {
                        _ = sleep(RECONNECT_DELAY) => {
                            continue;
                        }
                        _ = shutdown_rx.recv() => {
                            info!("Shutdown signal received during reconnect delay");
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    /// Connect to WebSocket and handle subscription
    async fn connect_and_subscribe(
        &self,
        channels: &[String],
        callback: &Box<dyn Fn(TickData) + Send + Sync>,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<(), ExchangeError> {
        // Establish WebSocket connection
        let (ws_stream, _) = connect_async(&self.ws_url)
            .await
            .map_err(|e| ExchangeError::WebSocketError(format!("Failed to connect: {}", e)))?;

        debug!("WebSocket connected to {}", self.ws_url);

        let (mut write, mut read) = ws_stream.split();

        // Send subscription message for each channel
        // Gate.io requires one subscription per channel
        for channel in channels {
            let subscribe_msg = GateioSubscribeMessage::new(channel.clone());
            let subscribe_json = serde_json::to_string(&subscribe_msg).map_err(|e| {
                ExchangeError::ParseError(format!("Failed to serialize subscription: {}", e))
            })?;

            write
                .send(Message::Text(subscribe_json))
                .await
                .map_err(|e| {
                    ExchangeError::WebSocketError(format!("Failed to send subscription: {}", e))
                })?;

            // Small delay between subscriptions to avoid rate limiting
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        info!("Subscription sent for {} channels", channels.len());

        // Message processing loop
        loop {
            tokio::select! {
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            match self.parse_trade_message(&text) {
                                Ok(tick_data) => callback(tick_data),
                                Err(e) => {
                                    // Only warn on actual parse errors, not control messages
                                    if !e.to_string().contains("Control message") {
                                        warn!("Parse error: {}", e);
                                    } else {
                                        debug!("Control message: {}", e);
                                    }
                                }
                            }
                        }
                        Some(Ok(Message::Ping(ping))) => {
                            write.send(Message::Pong(ping)).await?;
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!("WebSocket closed by server");
                            break;
                        }
                        Some(Err(e)) => {
                            return Err(ExchangeError::WebSocketError(e.to_string()));
                        }
                        None => {
                            info!("WebSocket stream ended");
                            break;
                        }
                        _ => continue,
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Shutdown signal received, closing WebSocket gracefully");
                    if let Err(e) = write.send(Message::Close(None)).await {
                        warn!("Failed to send close frame: {}", e);
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    /// Fetch historical trades using REST API
    async fn fetch_historical_trades_api(
        &self,
        params: &HistoricalTradeParams,
    ) -> Result<Vec<TickData>, ExchangeError> {
        // Gate.io uses different format: futures/{settle}/trades
        // Convert symbol format: BTCUSDT -> BTC_USDT
        let symbol = params.symbol.replace("USDT", "_USDT");
        
        let mut url = format!("{}/futures/usdt/trades", self.api_url);
        url.push_str(&format!("?contract={}", symbol));

        if let Some(limit) = params.limit {
            // Gate.io API typically allows up to 1000
            let limit = limit.min(1000);
            url.push_str(&format!("&limit={}", limit));
        }

        if let Some(start_time) = params.start_time {
            url.push_str(&format!("&from={}", start_time.timestamp()));
        }

        if let Some(end_time) = params.end_time {
            url.push_str(&format!("&to={}", end_time.timestamp()));
        }

        debug!("Fetching historical trades from: {}", url);

        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(30))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ExchangeError::ApiError(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        let trades_json = response.text().await?;
        let trades: Vec<serde_json::Value> = serde_json::from_str(&trades_json)?;

        let mut tick_data_vec = Vec::with_capacity(trades.len());

        for trade in trades {
            // Parse trade data from Gate.io API
            // Gate.io format: { "id": 123, "create_time": 1234567890, "contract": "BTC_USDT", "price": "50000", "size": 1, "role": "maker" }
            let trade_id = trade["id"].as_u64().unwrap_or(0);
            let create_time = trade["create_time"].as_u64().unwrap_or(0);
            let contract = trade["contract"].as_str().unwrap_or("");
            let price_str = trade["price"].as_str().unwrap_or("0");
            let size = trade["size"].as_i64().unwrap_or(0);
            let role = trade["role"].as_str().unwrap_or("maker");

            // Convert contract format back: BTC_USDT -> BTCUSDT
            let symbol = contract.replace("_", "");

            let trade_msg = GateioTradeMessage {
                id: trade_id,
                create_time: create_time,
                contract: contract.to_string(),
                price: price_str.to_string(),
                size: size,
                role: role.to_string(),
            };

            match convert_gateio_to_tick_data(trade_msg) {
                Ok(tick_data) => {
                    // Override symbol with normalized format
                    let mut tick_data = tick_data;
                    tick_data.symbol = symbol;
                    tick_data_vec.push(tick_data);
                }
                Err(e) => warn!("Failed to convert historical trade: {}", e),
            }
        }

        info!(
            "Successfully fetched {} historical trades for {}",
            tick_data_vec.len(),
            params.symbol
        );
        Ok(tick_data_vec)
    }
}

#[async_trait]
impl Exchange for GateioExchange {
    async fn subscribe_trades(
        &self,
        symbols: &[String],
        callback: Box<dyn Fn(TickData) + Send + Sync>,
        shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<(), ExchangeError> {
        if symbols.is_empty() {
            return Err(ExchangeError::InvalidSymbol(
                "No symbols provided".to_string(),
            ));
        }

        info!(
            "Starting Gate.io trade subscription for symbols: {:?}",
            symbols
        );

        self.handle_websocket_connection(symbols, callback, shutdown_rx.resubscribe())
            .await
    }

    async fn get_historical_trades(
        &self,
        params: HistoricalTradeParams,
    ) -> Result<Vec<TickData>, ExchangeError> {
        if params.symbol.is_empty() {
            return Err(ExchangeError::InvalidSymbol(
                "Symbol cannot be empty".to_string(),
            ));
        }

        info!("Fetching historical trades for symbol: {}", params.symbol);

        self.fetch_historical_trades_api(&params).await
    }
}

impl Default for GateioExchange {
    fn default() -> Self {
        Self::new()
    }
}

