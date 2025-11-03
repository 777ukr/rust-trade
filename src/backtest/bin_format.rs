//! Формат .bin файлов для хранения исторических трейдов
//! Совместимо с MoonBot форматом

use crate::backtest::market::{TradeTick, TradeSide};
use chrono::{DateTime, Utc, NaiveDateTime};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct TradeRecord {
    pub timestamp: i64, // Unix timestamp в миллисекундах
    pub price: f64,
    pub volume: f64,
    pub side: bool,     // true = buy, false = sell
}

pub struct BinFileReader {
    file: BufReader<File>,
    symbol: String,
}

impl BinFileReader {
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        
        // Определяем символ из имени файла
        let filename = path.as_ref().file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("UNKNOWN");
        let symbol = filename.split('_').next().unwrap_or("UNKNOWN").to_string();
        
        Ok(Self { 
            file: reader,
            symbol,
        })
    }
    
    /// Читает все трейды из файла
    pub fn read_all(&mut self) -> anyhow::Result<Vec<TradeTick>> {
        let mut trades = Vec::new();
        
        // MoonBot формат: каждый трейд = 24 байта
        // timestamp (i64), price (f64), volume (f64), side (bool как u8)
        let mut buffer = [0u8; 24];
        
        loop {
            match self.file.read_exact(&mut buffer) {
                Ok(_) => {
                    let timestamp_ms = i64::from_le_bytes([
                        buffer[0], buffer[1], buffer[2], buffer[3],
                        buffer[4], buffer[5], buffer[6], buffer[7],
                    ]);
                    
                    let price = f64::from_le_bytes([
                        buffer[8], buffer[9], buffer[10], buffer[11],
                        buffer[12], buffer[13], buffer[14], buffer[15],
                    ]);
                    
                    let volume = f64::from_le_bytes([
                        buffer[16], buffer[17], buffer[18], buffer[19],
                        buffer[20], buffer[21], buffer[22], buffer[23],
                    ]);
                    
                    // Для side используем последний байт (упрощенно)
                    let side = buffer[23] != 0;
                    
                    let timestamp = DateTime::from_timestamp_millis(timestamp_ms)
                        .unwrap_or_else(Utc::now);
                    
                    trades.push(TradeTick {
                        timestamp,
                        symbol: self.symbol.clone(),
                        price,
                        volume,
                        side: if side { TradeSide::Buy } else { TradeSide::Sell },
                        trade_id: format!("{}", trades.len()),
                        best_bid: None,
                        best_ask: None,
                    });
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        break; // Конец файла
                    } else {
                        return Err(anyhow::anyhow!("Read error: {}", e));
                    }
                }
            }
        }
        
        Ok(trades)
    }
}

pub struct BinFileWriter {
    file: BufWriter<File>,
}

impl BinFileWriter {
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        Ok(Self { file: writer })
    }
    
    /// Записывает трейд в файл
    pub fn write_trade(&mut self, trade: &TradeTick) -> anyhow::Result<()> {
        let timestamp_ms = trade.timestamp.timestamp_millis();
        
        let mut buffer = [0u8; 24];
        
        // Timestamp (i64)
        let ts_bytes = timestamp_ms.to_le_bytes();
        buffer[0..8].copy_from_slice(&ts_bytes);
        
        // Price (f64)
        let price_bytes = trade.price.to_le_bytes();
        buffer[8..16].copy_from_slice(&price_bytes);
        
        // Volume (f64)
        let volume_bytes = trade.volume.to_le_bytes();
        buffer[16..24].copy_from_slice(&volume_bytes);
        
        // Side (bool в последнем байте)
        buffer[23] = if matches!(trade.side, TradeSide::Buy) { 1 } else { 0 };
        
        self.file.write_all(&buffer)?;
        Ok(())
    }
    
    /// Записывает все трейды
    pub fn write_all(&mut self, trades: &[TradeTick]) -> anyhow::Result<()> {
        for trade in trades {
            self.write_trade(trade)?;
        }
        self.file.flush()?;
        Ok(())
    }
}

