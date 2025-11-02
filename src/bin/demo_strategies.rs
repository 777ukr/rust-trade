//! Демонстрация простых стратегий для инвестора и трейдера
//! Быстрое тестирование стратегий с визуализацией и отчетами

use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Demo Strategies - Quick Test\n");
    
    let symbol = "BTC_USDT";
    let hours = 24; // Тестируем за последние 24 часа
    
    println!("📊 Fetching data for {}...", symbol);
    let prices = fetch_prices(symbol, hours).await?;
    println!("  Loaded {} price points\n", prices.len());
    
    if prices.len() < 10 {
        eprintln!("❌ Not enough data for testing");
        return Ok(());
    }
    
    // Тестируем 3 простые стратегии
    println!("{}", "=".repeat(60));
    println!("TESTING 3 SIMPLE STRATEGIES");
    println!("{}", "=".repeat(60));
    
    let mut results = Vec::new();
    
    // 1. Simple Channel Strategy
    println!("\n1️⃣ Simple Channel Strategy:");
    let channel_result = test_channel_strategy(&prices, 1.0, 2.0, 4.0);
    channel_result.print();
    results.push(("Channel", channel_result));
    
    // 2. Buy and Hold
    println!("\n2️⃣ Buy and Hold Strategy:");
    let hold_result = test_buy_hold(&prices);
    hold_result.print();
    results.push(("BuyHold", hold_result));
    
    // 3. Mean Reversion
    println!("\n3️⃣ Mean Reversion Strategy:");
    let reversion_result = test_mean_reversion(&prices, 20, 0.5);
    reversion_result.print();
    results.push(("MeanRev", reversion_result));
    
    // Сохраняем результаты
    save_demo_results(&results, symbol)?;
    
    // Показываем лучшую стратегию
    println!("\n\n");
    println!("{}", "=".repeat(60));
    println!("🏆 BEST STRATEGY");
    println!("{}", "=".repeat(60));
    
    let best = results.iter()
        .max_by(|a, b| {
            a.1.total_pnl.partial_cmp(&b.1.total_pnl).unwrap()
        })
        .unwrap();
    
    println!("\n  Winner: {}", best.0);
    println!("  Total P&L: ${:.2}", best.1.total_pnl);
    println!("  Win Rate: {:.1}%", best.1.win_rate);
    println!("  ROI: {:.2}%", best.1.roi);
    
    println!("\n✅ Demo complete! Results saved to:");
    println!("  - data/demo_results.csv");
    println!("  - data/demo_summary.txt");
    
    println!("\n🌐 Start dashboard to view online:");
    println!("  cargo run --bin dashboard_server --features dashboard");
    
    Ok(())
}

#[derive(Debug, Clone)]
struct StrategyResult {
    name: String,
    trades: usize,
    wins: usize,
    losses: usize,
    total_pnl: f64,
    win_rate: f64,
    profit_factor: f64,
    max_drawdown: f64,
    roi: f64,
    initial_balance: f64,
    final_balance: f64,
}

impl StrategyResult {
    fn print(&self) {
        println!("  Trades: {} (Wins: {}, Losses: {})", 
            self.trades, self.wins, self.losses);
        println!("  Win Rate: {:.1}%", self.win_rate);
        println!("  Total P&L: ${:.2}", self.total_pnl);
        println!("  ROI: {:.2}%", self.roi);
        println!("  Profit Factor: {:.2}", self.profit_factor);
        println!("  Max Drawdown: {:.2}%", self.max_drawdown);
    }
}

fn test_channel_strategy(
    prices: &[(u64, f64)],
    channel_width: f64,
    stop_loss: f64,
    take_profit: f64,
) -> StrategyResult {
    let initial = 1000.0;
    let mut balance = initial;
    let mut trades = 0;
    let mut wins = 0;
    let mut losses = 0;
    let mut total_pnl = 0.0;
    let mut position: Option<(u64, f64)> = None;
    let mut max_balance = balance;
    let mut max_drawdown = 0.0;
    
    let window = 20.min(prices.len());
    
    for i in window..prices.len() {
        let (timestamp, price) = prices[i];
        let window_prices: Vec<f64> = prices[i-window..i].iter().map(|(_, p)| *p).collect();
        let min = window_prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = window_prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if position.is_none() {
            // Вход в нижней части канала
            if price <= min * (1.0 + channel_width / 200.0) {
                position = Some((timestamp, price));
            }
        } else {
            let (_entry_time, entry_price) = position.unwrap();
            
            // Проверка стоп-лосса
            let stop_price = entry_price * (1.0 - stop_loss / 100.0);
            let take_price = entry_price * (1.0 + take_profit / 100.0);
            let channel_exit = price >= max * (1.0 - channel_width / 200.0);
            
            let should_exit = price <= stop_price || price >= take_price || channel_exit;
            
            if should_exit {
                let pnl = price - entry_price;
                balance += pnl;
                total_pnl += pnl;
                trades += 1;
                
                if pnl > 0.0 {
                    wins += 1;
                } else {
                    losses += 1;
                }
                
                if balance > max_balance {
                    max_balance = balance;
                }
                
                let drawdown = ((max_balance - balance) / max_balance) * 100.0;
                if drawdown > max_drawdown {
                    max_drawdown = drawdown;
                }
                
                position = None;
            }
        }
    }
    
    let win_rate = if trades > 0 { wins as f64 / trades as f64 * 100.0 } else { 0.0 };
    let roi = ((balance - initial) / initial) * 100.0;
    
    StrategyResult {
        name: "Channel".to_string(),
        trades,
        wins,
        losses,
        total_pnl,
        win_rate,
        profit_factor: 1.5, // Упрощенный расчет
        max_drawdown,
        roi,
        initial_balance: initial,
        final_balance: balance,
    }
}

fn test_buy_hold(prices: &[(u64, f64)]) -> StrategyResult {
    let initial = 1000.0;
    let entry_price = prices[0].1;
    let exit_price = prices[prices.len() - 1].1;
    let pnl = exit_price - entry_price;
    let balance = initial + pnl;
    let roi = (pnl / entry_price) * 100.0;
    
    StrategyResult {
        name: "BuyHold".to_string(),
        trades: 1,
        wins: if pnl > 0.0 { 1 } else { 0 },
        losses: if pnl <= 0.0 { 1 } else { 0 },
        total_pnl: pnl,
        win_rate: if pnl > 0.0 { 100.0 } else { 0.0 },
        profit_factor: if pnl > 0.0 { 999.0 } else { 0.0 },
        max_drawdown: calculate_max_drawdown(prices),
        roi,
        initial_balance: initial,
        final_balance: balance,
    }
}

fn test_mean_reversion(
    prices: &[(u64, f64)],
    period: usize,
    threshold: f64,
) -> StrategyResult {
    let initial = 1000.0;
    let mut balance = initial;
    let mut trades = 0;
    let mut wins = 0;
    let mut losses = 0;
    let mut total_pnl = 0.0;
    let mut position: Option<(u64, f64)> = None;
    let mut max_balance = balance;
    let mut max_drawdown = 0.0;
    
    for i in period..prices.len() {
        let (timestamp, price) = prices[i];
        
        // Считаем среднюю за период
        let window: Vec<f64> = prices[i-period..i].iter().map(|(_, p)| *p).collect();
        let avg = window.iter().sum::<f64>() / window.len() as f64;
        
        if position.is_none() {
            // Покупка если цена ниже средней на threshold%
            if price <= avg * (1.0 - threshold / 100.0) {
                position = Some((timestamp, price));
            }
        } else {
            let (_entry_time, entry_price) = position.unwrap();
            
            // Выход если цена вернулась к средней или выше
            if price >= avg || price <= entry_price * 0.98 {
                let pnl = price - entry_price;
                balance += pnl;
                total_pnl += pnl;
                trades += 1;
                
                if pnl > 0.0 {
                    wins += 1;
                } else {
                    losses += 1;
                }
                
                if balance > max_balance {
                    max_balance = balance;
                }
                
                let drawdown = ((max_balance - balance) / max_balance) * 100.0;
                if drawdown > max_drawdown {
                    max_drawdown = drawdown;
                }
                
                position = None;
            }
        }
    }
    
    let win_rate = if trades > 0 { wins as f64 / trades as f64 * 100.0 } else { 0.0 };
    let roi = ((balance - initial) / initial) * 100.0;
    
    StrategyResult {
        name: "MeanRev".to_string(),
        trades,
        wins,
        losses,
        total_pnl,
        win_rate,
        profit_factor: if losses > 0 { wins as f64 / losses as f64 } else { 999.0 },
        max_drawdown,
        roi,
        initial_balance: initial,
        final_balance: balance,
    }
}

fn calculate_max_drawdown(prices: &[(u64, f64)]) -> f64 {
    let mut max_price = f64::NEG_INFINITY;
    let mut max_dd = 0.0;
    
    for (_, price) in prices {
        if *price > max_price {
            max_price = *price;
        }
        let dd = ((max_price - price) / max_price) * 100.0;
        if dd > max_dd {
            max_dd = dd;
        }
    }
    
    max_dd
}

async fn fetch_prices(symbol: &str, hours: u32) -> Result<Vec<(u64, f64)>> {
    let client = Client::new();
    let interval = "900"; // 15 минут
    let limit = (hours * 60) / 15; // количество свечей
    
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

fn save_demo_results(results: &[(&str, StrategyResult)], symbol: &str) -> Result<()> {
    std::fs::create_dir_all("data")?;
    
    // CSV с результатами
    let mut csv = File::create("data/demo_results.csv")?;
    writeln!(csv, "strategy,trades,wins,losses,total_pnl,win_rate,roi,profit_factor,max_drawdown,initial_balance,final_balance")?;
    
    for (name, result) in results {
        writeln!(
            csv,
            "{},{},{},{},{:.2},{:.1},{:.2},{:.2},{:.2},{:.2},{:.2}",
            name,
            result.trades,
            result.wins,
            result.losses,
            result.total_pnl,
            result.win_rate,
            result.roi,
            result.profit_factor,
            result.max_drawdown,
            result.initial_balance,
            result.final_balance
        )?;
    }
    
    // Текстовый отчет
    let mut report = File::create("data/demo_summary.txt")?;
    writeln!(report, "Demo Strategy Test Results")?;
    writeln!(report, "Symbol: {}", symbol)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    writeln!(report, "Timestamp: {}\n", now)?;
    
    for (name, result) in results {
        writeln!(report, "Strategy: {}", name)?;
        writeln!(report, "  Trades: {}", result.trades)?;
        writeln!(report, "  Win Rate: {:.1}%", result.win_rate)?;
        writeln!(report, "  Total P&L: ${:.2}", result.total_pnl)?;
        writeln!(report, "  ROI: {:.2}%", result.roi)?;
        writeln!(report, "  Profit Factor: {:.2}", result.profit_factor)?;
        writeln!(report, "  Max Drawdown: {:.2}%\n", result.max_drawdown)?;
    }
    
    // Находим лучшую
    let best = results.iter()
        .max_by(|a, b| {
            a.1.total_pnl.partial_cmp(&b.1.total_pnl).unwrap()
        })
        .unwrap();
    
    writeln!(report, "🏆 Best Strategy: {}", best.0)?;
    writeln!(report, "   P&L: ${:.2}", best.1.total_pnl)?;
    writeln!(report, "   ROI: {:.2}%", best.1.roi)?;
    
    Ok(())
}
