//! Простой бэктестер для SOL - сбор цен и симуляция канальной стратегии
//! Минимализм, скорость, чистота кода

use std::fs::File;
use std::io::Write;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use reqwest::Client;
use serde_json::Value;

const GATE_API_BASE: &str = "https://api.gateio.ws/api/v4";
const SYMBOL: &str = "SOL_USDT";
const OUTPUT_FILE: &str = "data/sol_prices.csv";
const BACKTEST_FILE: &str = "data/sol_backtest.csv";

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 SOL Backtest: Starting price collection...");
    
    let client = Client::new();
    let mut prices = Vec::new();
    let start_time = SystemTime::now();
    
    // Собираем цены каждые 10 секунд в течение 1 часа
    let duration = Duration::from_secs(3600);
    let interval = Duration::from_secs(10);
    let mut last_price = None;
    
    while start_time.elapsed().unwrap_or(Duration::ZERO) < duration {
        match fetch_price(&client).await {
            Ok(price) => {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                
                prices.push((timestamp, price));
                last_price = Some(price);
                
                println!("{} | SOL: ${:.2}", 
                    format_time(timestamp),
                    price
                );
            }
            Err(e) => {
                eprintln!("Error fetching price: {}", e);
            }
        }
        
        tokio::time::sleep(interval).await;
    }
    
    // Сохраняем цены
    save_prices(&prices)?;
    
    // Симулируем торговлю
    if prices.len() > 0 {
        let backtest_results = simulate_trading(&prices);
        save_backtest(&backtest_results)?;
        print_summary(&backtest_results);
    }
    
    println!("\n✅ Done! Results saved to:");
    println!("  - {}", OUTPUT_FILE);
    println!("  - {}", BACKTEST_FILE);
    
    Ok(())
}

async fn fetch_price(client: &Client) -> Result<f64> {
    let url = format!("{}/futures/usdt/tickers?contract={}", GATE_API_BASE, SYMBOL);
    let resp = client.get(&url).send().await?;
    let json: Value = resp.json().await?;
    
    if let Some(ticker) = json.as_array().and_then(|a| a.first()) {
        let price = ticker["last"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .or_else(|| ticker["last"].as_f64())
            .ok_or_else(|| anyhow::anyhow!("No price in response"))?;
        Ok(price)
    } else {
        anyhow::bail!("Invalid response format")
    }
}

fn format_time(ts: u64) -> String {
    let hours = (ts % 86400) / 3600;
    let minutes = (ts % 3600) / 60;
    let seconds = ts % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn save_prices(prices: &[(u64, f64)]) -> Result<()> {
    std::fs::create_dir_all("data")?;
    let mut file = File::create(OUTPUT_FILE)?;
    writeln!(file, "timestamp,price")?;
    
    for (ts, price) in prices {
        writeln!(file, "{},{}", ts, price)?;
    }
    
    Ok(())
}

#[derive(Debug)]
struct Trade {
    entry_time: u64,
    entry_price: f64,
    exit_time: u64,
    exit_price: f64,
    side: String,
    pnl: f64,
    pnl_percent: f64,
}

#[derive(Debug)]
struct BacktestResults {
    trades: Vec<Trade>,
    total_pnl: f64,
    win_count: usize,
    loss_count: usize,
    max_drawdown: f64,
}

fn simulate_trading(prices: &[(u64, f64)]) -> BacktestResults {
    let mut trades = Vec::new();
    let mut position: Option<(u64, f64, String)> = None;
    let mut total_pnl = 0.0;
    let mut win_count = 0;
    let mut loss_count = 0;
    let mut max_price = f64::MIN;
    let mut min_price = f64::MAX;
    let mut max_drawdown = 0.0;
    
    // Простая канальная стратегия: покупаем на дне, продаем на верху
    let window = 20.min(prices.len());
    let channel_size = 0.02; // 2% канал
    
    for i in window..prices.len() {
        let window_prices: Vec<f64> = prices[i-window..i].iter().map(|(_, p)| *p).collect();
        let min = window_prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = window_prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let current_price = prices[i].1;
        
        // Обновляем максимум/минимум для drawdown
        if current_price > max_price {
            max_price = current_price;
            min_price = current_price;
        }
        if current_price < min_price {
            min_price = current_price;
        }
        let drawdown = (max_price - min_price) / max_price * 100.0;
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
        
        // Логика входа/выхода
        if position.is_none() {
            // Покупка на дне канала
            if current_price <= min * (1.0 + channel_size / 2.0) {
                position = Some((prices[i].0, current_price, "long".to_string()));
            }
        } else if let Some((entry_time, entry_price, side)) = position.as_ref() {
            // Выход на верху канала или стоп-лосс 2%
            let profit = if *side == "long" {
                (current_price - entry_price) / entry_price
            } else {
                (entry_price - current_price) / entry_price
            };
            
            let should_exit = current_price >= max * (1.0 - channel_size / 2.0) 
                || profit <= -0.02; // стоп-лосс 2%
            
            if should_exit {
                let pnl = if *side == "long" {
                    current_price - entry_price
                } else {
                    entry_price - current_price
                };
                let pnl_percent = pnl / entry_price * 100.0;
                
                total_pnl += pnl;
                if pnl > 0.0 {
                    win_count += 1;
                } else {
                    loss_count += 1;
                }
                
                let side_clone = side.clone();
                trades.push(Trade {
                    entry_time: *entry_time,
                    entry_price: *entry_price,
                    exit_time: prices[i].0,
                    exit_price: current_price,
                    side: side_clone,
                    pnl,
                    pnl_percent,
                });
                
                position = None;
            }
        }
    }
    
    // Закрываем открытую позицию
    if let Some((entry_time, entry_price, side)) = position.as_ref() {
        if let Some((exit_time, exit_price)) = prices.last().map(|(t, p)| (*t, *p)) {
            let pnl = if *side == "long" {
                exit_price - entry_price
            } else {
                entry_price - exit_price
            };
            let pnl_percent = pnl / entry_price * 100.0;
            
            total_pnl += pnl;
            if pnl > 0.0 {
                win_count += 1;
            } else {
                loss_count += 1;
            }
            
            trades.push(Trade {
                entry_time: *entry_time,
                entry_price: *entry_price,
                exit_time,
                exit_price,
                side: side.clone(),
                pnl,
                pnl_percent,
            });
        }
    }
    
    BacktestResults {
        trades,
        total_pnl,
        win_count,
        loss_count,
        max_drawdown,
    }
}

fn save_backtest(results: &BacktestResults) -> Result<()> {
    let mut file = File::create(BACKTEST_FILE)?;
    writeln!(file, "entry_time,entry_price,exit_time,exit_price,side,pnl,pnl_percent")?;
    
    for trade in &results.trades {
        writeln!(
            file,
            "{},{},{},{},{},{:.4},{:.2}",
            trade.entry_time,
            trade.entry_price,
            trade.exit_time,
            trade.exit_price,
            trade.side,
            trade.pnl,
            trade.pnl_percent
        )?;
    }
    
    Ok(())
}

fn print_summary(results: &BacktestResults) {
    println!("\n📊 Backtest Summary:");
    println!("  Total trades: {}", results.trades.len());
    println!("  Wins: {}", results.win_count);
    println!("  Losses: {}", results.loss_count);
    if results.trades.len() > 0 {
        let win_rate = results.win_count as f64 / results.trades.len() as f64 * 100.0;
        println!("  Win rate: {:.1}%", win_rate);
    }
    println!("  Total P&L: ${:.2}", results.total_pnl);
    println!("  Max drawdown: {:.2}%", results.max_drawdown);
}

