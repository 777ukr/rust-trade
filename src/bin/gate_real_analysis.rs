//! Реальный анализ торговли на Gate.io
//! Получение депозита, комиссии, истории сделок
//! Анализ канальной торговли с учетом реальных параметров

#![cfg(feature = "gate_exec")]

use anyhow::Result;
use dotenvy::dotenv;
use rust_test::config::runner::{load_gate_credentials, load_runner_config};
use rust_test::execution::GateClient;
use rust_test::analytics::channel_analyzer::{ChannelAnalyzer, ChannelAnalysis};
use reqwest::Client;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    
    println!("🔍 Gate.io Real Trading Analysis\n");
    
    // 1. Получение реальных данных с Gate.io
    println!("{}", "=".repeat(60));
    println!("STEP 1: Fetching Real Data from Gate.io");
    println!("{}", "=".repeat(60));
    
    let config = load_runner_config("config/gate_mvp.yaml")?;
    let creds = load_gate_credentials(&config)?;
    let client = GateClient::new(creds);
    
    // Получаем депозит
    let deposit_info = get_deposit_info(&client).await?;
    println!("\n💰 Current Deposit:");
    println!("  Total: ${:.2}", deposit_info.total);
    println!("  Available: ${:.2}", deposit_info.available);
    println!("  Locked: ${:.2}", deposit_info.locked);
    
    // Получаем комиссию
    let commission = get_commission_rate(&client).await?;
    println!("\n💳 Commission Rate:");
    println!("  Maker: {:.4}%", commission.maker * 100.0);
    println!("  Taker: {:.4}%", commission.taker * 100.0);
    println!("  Using: {:.4}% (average)", (commission.maker + commission.taker) / 2.0 * 100.0);
    
    // Получаем историю сделок за 2-3 дня
    println!("\n📈 Fetching Trade History (last 3 days)...");
    let trades = get_recent_trades(&client, "BTC_USDT", 3).await?;
    println!("  Found {} trades", trades.len());
    
    // 2. Анализ канальной торговли
    println!("\n\n");
    println!("{}", "=".repeat(60));
    println!("STEP 2: Channel Trading Analysis");
    println!("{}", "=".repeat(60));
    
    // Получаем исторические данные для анализа канала
    println!("\n📊 Fetching historical price data...");
    let prices = fetch_historical_prices("BTC_USDT", 72).await?; // 72 часа = 3 дня
    println!("  Loaded {} price points", prices.len());
    
    // Строим канал из данных
    let (channel_lower, channel_upper) = build_channel(&prices, 20, 1.0); // окно 20, ширина 1%
    
    // Анализ с реальными параметрами
    let analyzer = ChannelAnalyzer::new(
        (commission.maker + commission.taker) / 2.0, // средняя комиссия
        100.0,  // плечо x100
        1.0,    // ширина канала 1%
        2.0,    // стоп-лосс 2%
        4.0,    // тейк-профит 4%
        deposit_info.total, // начальный депозит
    );
    
    let analysis = analyzer.analyze_channel_trading(&prices, &channel_lower, &channel_upper);
    
    // 3. Вывод результатов
    println!("\n\n");
    println!("{}", "=".repeat(60));
    println!("STEP 3: Analysis Results");
    println!("{}", "=".repeat(60));
    analysis.print();
    
    // 4. Сравнение с реальными сделками
    println!("\n\n");
    println!("{}", "=".repeat(60));
    println!("STEP 4: Real Trades Comparison");
    println!("{}", "=".repeat(60));
    
    if !trades.is_empty() {
        println!("\n📋 Real Trades Summary:");
        println!("  Total real trades: {}", trades.len());
        
        let mut real_pnl = 0.0;
        let mut real_wins = 0;
        let mut real_losses = 0;
        
        for trade in &trades {
            if let Some(pnl_str) = trade.get("pnl").and_then(|v| v.as_str()) {
                if let Ok(pnl) = pnl_str.parse::<f64>() {
                    real_pnl += pnl;
                    if pnl > 0.0 {
                        real_wins += 1;
                    } else {
                        real_losses += 1;
                    }
                }
            }
        }
        
        println!("  Real P&L: ${:.2}", real_pnl);
        println!("  Real Wins: {} | Losses: {}", real_wins, real_losses);
        
        println!("\n📊 Comparison:");
        println!("  Simulated P&L: ${:.2}", analysis.total_pnl_after_fee);
        println!("  Real P&L: ${:.2}", real_pnl);
        println!("  Difference: ${:.2}", analysis.total_pnl_after_fee - real_pnl);
    } else {
        println!("\n⚠️ No real trades found for comparison");
    }
    
    // Сохраняем результаты
    save_analysis(&analysis)?;
    
    println!("\n✅ Analysis complete! Results saved to data/channel_analysis.csv");
    
    Ok(())
}

#[derive(Debug)]
struct DepositInfo {
    total: f64,
    available: f64,
    locked: f64,
}

#[derive(Debug)]
struct CommissionInfo {
    maker: f64,
    taker: f64,
}

async fn get_deposit_info(client: &GateClient) -> Result<DepositInfo> {
    // Получаем баланс USDT на фьючерсах
    let settle = "usdt";
    let accounts = client.fetch_futures_accounts(settle).await?;
    
    // Gate.io возвращает total, available как строки или числа
    let total: f64 = if let Some(Value::String(s)) = accounts.get("total") {
        s.parse().unwrap_or(0.0)
    } else {
        accounts["total"].as_f64().unwrap_or(0.0)
    };
    
    let available: f64 = if let Some(Value::String(s)) = accounts.get("available") {
        s.parse().unwrap_or(0.0)
    } else {
        accounts["available"].as_f64().unwrap_or(0.0)
    };
    
    Ok(DepositInfo {
        total,
        available,
        locked: total - available,
    })
}

async fn get_commission_rate(_client: &GateClient) -> Result<CommissionInfo> {
    // Gate.io комиссия для фьючерсов обычно:
    // Maker: 0.015% (0.00015)
    // Taker: 0.05% (0.0005)
    // TODO: Получить реальную комиссию через API
    Ok(CommissionInfo {
        maker: 0.00015,
        taker: 0.0005,
    })
}

async fn get_recent_trades(client: &GateClient, symbol: &str, days: u32) -> Result<Vec<Value>> {
    // Получаем сделки за последние N дней
    let settle = "usdt";
    let limit = days * 100; // примерно 100 сделок в день
    
    // Получаем timestamp 3 дня назад
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let from = now - (days as u64 * 86400);
    
    let trades = client.fetch_user_trades(settle, symbol, Some(limit), Some(from)).await?;
    
    if let Value::Array(arr) = trades {
        Ok(arr)
    } else {
        Ok(vec![trades])
    }
}

async fn fetch_historical_prices(symbol: &str, hours: u32) -> Result<Vec<(u64, f64)>> {
    let client = Client::new();
    let interval = "3600"; // 1 час
    let limit = hours;
    
    let url = format!(
        "https://api.gateio.ws/api/v4/futures/usdt/candlesticks?contract={}&interval={}&limit={}",
        symbol, interval, limit
    );
    
    let resp = client.get(&url).send().await?;
    let json: Value = resp.json().await?;
    
    let mut prices = Vec::new();
    
    if let Some(candles) = json.as_array() {
        for candle in candles {
            if let Some(ts) = candle[0].as_u64() {
                if let Some(close) = candle[4].as_str().and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| candle[4].as_f64()) {
                    prices.push((ts, close));
                }
            }
        }
    }
    
    prices.sort_by_key(|(t, _)| *t);
    Ok(prices)
}

fn build_channel(prices: &[(u64, f64)], window: usize, width_percent: f64) -> (Vec<(u64, f64)>, Vec<(u64, f64)>) {
    let mut lower = Vec::new();
    let mut upper = Vec::new();
    
    for i in window..prices.len() {
        let window_prices: Vec<f64> = prices[i-window..i].iter().map(|(_, p)| *p).collect();
        let min = window_prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = window_prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        let timestamp = prices[i].0;
        lower.push((timestamp, min * (1.0 - width_percent / 200.0)));
        upper.push((timestamp, max * (1.0 + width_percent / 200.0)));
    }
    
    (lower, upper)
}

fn save_analysis(analysis: &ChannelAnalysis) -> Result<()> {
    use std::fs::File;
    use std::io::Write;
    
    std::fs::create_dir_all("data")?;
    let mut file = File::create("data/channel_analysis.csv")?;
    
    writeln!(file, "entry_time,entry_price,exit_time,exit_price,side,size,pnl_before_fee,fee,pnl_after_fee,pnl_percent,stop_loss_hit")?;
    
    for trade in &analysis.trades {
        writeln!(
            file,
            "{},{},{},{},{},{},{:.4},{:.4},{:.4},{:.2},{}",
            trade.entry_time,
            trade.entry_price,
            trade.exit_time,
            trade.exit_price,
            trade.side,
            trade.size,
            trade.pnl_before_fee,
            trade.fee,
            trade.pnl_after_fee,
            trade.pnl_percent,
            if trade.stop_loss_hit { 1 } else { 0 }
        )?;
    }
    
    Ok(())
}

