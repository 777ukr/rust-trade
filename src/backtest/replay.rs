//! Модуль для воспроизведения исторических данных

use crate::backtest::market::{TradeStream, TradeTick};
use crate::backtest::bin_format::BinFileReader;
use chrono::{DateTime, Utc};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ReplaySettings {
    pub speed_multiplier: f64, // 1.0 = реальное время, 2.0 = 2x быстрее
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

impl Default for ReplaySettings {
    fn default() -> Self {
        ReplaySettings {
            speed_multiplier: 1.0,
            start_time: None,
            end_time: None,
        }
    }
}

pub struct ReplayEngine {
    settings: ReplaySettings,
    streams: Vec<TradeStream>,
}

impl ReplayEngine {
    pub fn new(settings: ReplaySettings) -> Self {
        Self {
            settings,
            streams: Vec::new(),
        }
    }
    
    /// Загрузить .bin файл
    pub fn load_bin_file<P: AsRef<Path>>(&mut self, path: P) -> anyhow::Result<()> {
        let mut reader = BinFileReader::new(path)?;
        let trades = reader.read_all()?;
        
        // Фильтруем по времени если задано
        let filtered_trades: Vec<TradeTick> = trades
            .into_iter()
            .filter(|t| {
                if let Some(start) = self.settings.start_time {
                    if t.timestamp < start {
                        return false;
                    }
                }
                if let Some(end) = self.settings.end_time {
                    if t.timestamp > end {
                        return false;
                    }
                }
                true
            })
            .collect();
        
        if filtered_trades.is_empty() {
            return Err(anyhow::anyhow!("No trades in specified time range"));
        }
        
        // Определяем символ из первого трейда
        let symbol = if !filtered_trades.is_empty() && !filtered_trades[0].symbol.is_empty() {
            filtered_trades[0].symbol.clone()
        } else {
            "UNKNOWN".to_string()
        };
        
        let stream = TradeStream::new(symbol, filtered_trades);
        self.streams.push(stream);
        
        Ok(())
    }
    
    /// Загрузить несколько .bin файлов (для параллельного воспроизведения)
    pub fn load_multiple_bin_files<P: AsRef<Path>>(
        &mut self,
        paths: Vec<P>,
    ) -> anyhow::Result<()> {
        for path in paths {
            self.load_bin_file(path)?;
        }
        Ok(())
    }
    
    pub fn get_streams(&self) -> &[TradeStream] {
        &self.streams
    }
    
    pub fn take_streams(self) -> Vec<TradeStream> {
        self.streams
    }
}

