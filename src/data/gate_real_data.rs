//! Модуль для получения РЕАЛЬНЫХ данных с Gate.io
//! OHLCV свечи, ордербук, дельта объема, история сделок

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Реальная свеча OHLCV с Gate.io
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealCandle {
    pub timestamp: u64,      // u64 = беззнаковое 64-битное целое (время Unix в секундах)
    pub open: f64,           // f64 = число с плавающей точкой двойной точности (цена)
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,         // Объем в базовой валюте
    pub quote_volume: f64,   // Объем в USDT
}

/// Ордербук уровень
#[derive(Debug, Clone)]
pub struct OrderbookLevel {
    pub price: f64,
    pub volume: f64,
}

/// Ордербук snapshot
#[derive(Debug, Clone)]
pub struct OrderbookSnapshot {
    pub bids: Vec<OrderbookLevel>,
    pub asks: Vec<OrderbookLevel>,
    pub timestamp: u64,
}

/// Дельта объема (разница между покупками и продажами)
#[derive(Debug, Clone)]
pub struct VolumeDelta {
    pub buy_volume: f64,
    pub sell_volume: f64,
    pub delta: f64,           // buy_volume - sell_volume
    pub delta_percent: f64,   // delta / total_volume * 100
}

pub struct GateRealDataClient {
    client: Client,
    base_url: String,
}

impl GateRealDataClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.gateio.ws/api/v4".to_string(),
        }
    }

    /// Получить исторические свечи OHLCV
    /// symbol: BTC_USDT, ETH_USDT, SOL_USDT
    /// interval: 1m, 5m, 15m, 1h, 4h, 1d
    /// limit: количество свечей (макс 1000)
    pub async fn fetch_candles(
        &self,
        symbol: &str,
        interval: &str,
        limit: u32,
    ) -> Result<Vec<RealCandle>> {
        let url = format!(
            "{}/futures/usdt/candlesticks?contract={}&interval={}&limit={}",
            self.base_url, symbol, interval, limit
        );

        let resp = self.client.get(&url).send().await?;
        let json: Value = resp.json().await?;

        let mut candles = Vec::new();

        if let Some(candle_array) = json.as_array() {
            for candle in candle_array {
                if candle.is_object() {
                    // Формат Gate.io: {"t": timestamp, "o": open, "h": high, "l": low, "c": close, "v": volume, "sum": quote_volume}
                    if let (Some(t), Some(o), Some(h), Some(l), Some(c), Some(v), Some(sum)) = (
                        candle.get("t").and_then(|v| v.as_u64()),
                        candle.get("o").and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())),
                        candle.get("h").and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())),
                        candle.get("l").and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())),
                        candle.get("c").and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())),
                        candle.get("v").and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())),
                        candle.get("sum").and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())),
                    ) {
                        candles.push(RealCandle {
                            timestamp: t,
                            open: o,
                            high: h,
                            low: l,
                            close: c,
                            volume: v,
                            quote_volume: sum,
                        });
                    }
                }
            }
        }

        // Сортируем по времени (старые первыми)
        candles.sort_by_key(|c| c.timestamp);
        Ok(candles)
    }

    /// Получить последние N свечей для backtest
    pub async fn fetch_recent_candles(
        &self,
        symbol: &str,
        interval: &str,
        hours: u32,
    ) -> Result<Vec<RealCandle>> {
        // Вычисляем сколько свечей нужно
        let candles_per_hour = match interval {
            "1m" => 60,
            "5m" => 12,
            "15m" => 4,
            "1h" => 1,
            "4h" => 1,
            "1d" => 1,
            _ => 4, // По умолчанию 15m
        };
        
        let limit = (hours * candles_per_hour).min(1000); // Gate.io максимум 1000
        self.fetch_candles(symbol, interval, limit).await
    }

    /// Получить snapshot ордербука
    pub async fn fetch_orderbook(&self, symbol: &str, limit: u32) -> Result<OrderbookSnapshot> {
        let url = format!(
            "{}/futures/usdt/order_book?contract={}&limit={}",
            self.base_url, symbol, limit.min(50) // Максимум 50 уровней
        );

        let resp = self.client.get(&url).send().await?;
        let json: Value = resp.json().await?;

        let mut bids = Vec::new();
        let mut asks = Vec::new();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if let (Some(bids_array), Some(asks_array)) = (
            json.get("bids").and_then(|v| v.as_array()),
            json.get("asks").and_then(|v| v.as_array()),
        ) {
            for bid in bids_array.iter().take(limit as usize) {
                if let Some(arr) = bid.as_array() {
                    if arr.len() >= 2 {
                        if let (Some(p), Some(v)) = (
                            arr[0].as_str().and_then(|s| s.parse::<f64>().ok()),
                            arr[1].as_str().and_then(|s| s.parse::<f64>().ok()),
                        ) {
                            bids.push(OrderbookLevel { price: p, volume: v });
                        }
                    }
                }
            }

            for ask in asks_array.iter().take(limit as usize) {
                if let Some(arr) = ask.as_array() {
                    if arr.len() >= 2 {
                        if let (Some(p), Some(v)) = (
                            arr[0].as_str().and_then(|s| s.parse::<f64>().ok()),
                            arr[1].as_str().and_then(|s| s.parse::<f64>().ok()),
                        ) {
                            asks.push(OrderbookLevel { price: p, volume: v });
                        }
                    }
                }
            }
        }

        Ok(OrderbookSnapshot { bids, asks, timestamp })
    }

    /// Рассчитать дельту объема из свечей (упрощенный метод)
    /// Используем разницу между объемами на зеленых и красных свечах
    pub fn calculate_volume_delta(candles: &[RealCandle]) -> Vec<VolumeDelta> {
        candles
            .iter()
            .map(|c| {
                let is_green = c.close >= c.open;
                let buy_volume = if is_green { c.quote_volume } else { c.quote_volume * 0.3 };
                let sell_volume = if !is_green { c.quote_volume } else { c.quote_volume * 0.3 };
                let delta = buy_volume - sell_volume;
                let total = buy_volume + sell_volume;
                let delta_percent = if total > 0.0 { delta / total * 100.0 } else { 0.0 };

                VolumeDelta {
                    buy_volume,
                    sell_volume,
                    delta,
                    delta_percent,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_candles() {
        let client = GateRealDataClient::new();
        let candles = client
            .fetch_candles("BTC_USDT", "15m", 10)
            .await
            .unwrap();
        
        assert!(!candles.is_empty());
        assert_eq!(candles.len(), 10);
    }
}


