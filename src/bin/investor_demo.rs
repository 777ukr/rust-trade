//! Демонстрация для инвестора: 3 стратегии на Gate.io
//! 1. Канальная стратегия (с дроблением на 3 ордера)
//! 2. Market Making
//! 3. HFT
//! Тестирование на SOL, ETH, BTC с плечом x100

#![cfg(feature = "gate_exec")]

use anyhow::Result;
use dotenvy::dotenv;
use rust_test::config::runner::{load_gate_credentials, load_runner_config};
use rust_test::execution::GateClient;
use rust_test::strategy::channel_split::{ChannelSplitStrategy, ChannelSplitSignal, OrderPart};
use rust_test::strategy::market_making::{MarketMakingStrategy, MarketMakingSignal};
use rust_test::strategy::hft::{HFTStrategy, HFTSignal};
use reqwest::Client;
use serde_json::Value;
use std::fs::File;
use std::io::Write;

#[cfg(feature = "database")]
use rust_test::database::{DatabaseRepository, BacktestResult as DbBacktestResult};
#[cfg(feature = "database")]
use rust_decimal::Decimal;
#[cfg(feature = "database")]
use chrono::{Utc, Duration as ChronoDuration};

#[derive(Debug, Clone)]
struct StrategyResult {
    name: String,
    symbol: String,
    initial_balance: f64,
    final_balance: f64,
    total_pnl: f64,
    total_fees: f64,
    trades: usize,
    wins: usize,
    losses: usize,
    win_rate: f64,
    roi: f64,
    max_drawdown: f64,
    profit_factor: f64,
}

impl StrategyResult {
    fn print(&self) {
        println!("\n  📊 {} на {}", self.name, self.symbol);
        println!("    Начальный баланс: ${:.2}", self.initial_balance);
        println!("    Финальный баланс: ${:.2}", self.final_balance);
        println!("    Total P&L: ${:.2}", self.total_pnl);
        println!("    Комиссии: ${:.2}", self.total_fees);
        println!("    Сделки: {} (Wins: {}, Losses: {})", self.trades, self.wins, self.losses);
        println!("    Win Rate: {:.1}%", self.win_rate);
        println!("    ROI: {:.2}%", self.roi);
        println!("    Profit Factor: {:.2}", self.profit_factor);
        println!("    Max Drawdown: {:.2}%", self.max_drawdown);
    }

    fn to_csv_line(&self) -> String {
        format!(
            "{},{},{:.2},{:.2},{:.2},{:.2},{},{},{},{:.1},{:.2},{:.2},{:.2}\n",
            self.name,
            self.symbol,
            self.initial_balance,
            self.final_balance,
            self.total_pnl,
            self.total_fees,
            self.trades,
            self.wins,
            self.losses,
            self.win_rate,
            self.roi,
            self.profit_factor,
            self.max_drawdown
        )
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    println!("🚀 Investor Demo - 3 Strategies Test\n");
    println!("{}", "=".repeat(70));
    println!("GATE.IO ACCOUNT ANALYSIS & STRATEGY COMPARISON");
    println!("{}", "=".repeat(70));

    // 1. Получение данных Gate.io
    println!("\n📡 Step 1: Fetching Gate.io Account Data\n");
    
    // Попытка загрузить реальные credentials, если не получается - используем демо-режим
    let (deposit_info, commission) = match load_runner_config("config/gate_mvp.yaml")
        .and_then(|config| load_gate_credentials(&config))
    {
        Ok(creds) => {
            let client = GateClient::new(creds);
            println!("✅ Using real Gate.io API credentials");
            let deposit = get_deposit_info(&client).await?;
            let comm = get_commission_rate(&client).await?;
            (deposit, comm)
        }
        Err(_) => {
            println!("⚠️  Real API credentials not found, using DEMO mode");
            println!("   💡 To use real data, set environment variables:");
            println!("      export gateio_api_key=your_key");
            println!("      export gateio_secret_key=your_secret");
            println!("   Or add them to .env file");
            
            // Демо-режим: фиксированные значения
            (
                DepositInfo {
                    total: 1250.0,
                    available: 1250.0,
                    locked: 0.0,
                },
                CommissionInfo {
                    maker: 0.00015,  // 0.015%
                    taker: 0.0005,   // 0.05%
                }
            )
        }
    };

    println!("💰 Current Deposit: ${:.2}", deposit_info.total);
    println!("   Available: ${:.2}", deposit_info.available);

    let avg_commission = (commission.maker + commission.taker) / 2.0;
    println!("💳 Commission: {:.4}% (average)", avg_commission * 100.0);

    // 2. Тестирование на трех монетах
    let symbols = vec!["SOL_USDT", "ETH_USDT", "BTC_USDT"];
    let leverage = 100.0;
    let initial_balance = deposit_info.total;
    
    let mut all_results = Vec::new();

    for symbol in &symbols {
        println!("\n\n{}", "=".repeat(70));
        println!("TESTING ON: {}", symbol);
        println!("{}", "=".repeat(70));

        // Получаем исторические данные
        println!("\n📊 Fetching historical data (last 72 hours)...");
        let prices = fetch_historical_prices(symbol, 72).await?;
        println!("   Loaded {} price points", prices.len());

        if prices.len() < 100 {
            println!("   ⚠️  Not enough data, skipping...");
            continue;
        }

        // Тест 1: Канальная стратегия с дроблением
        println!("\n1️⃣ Testing Channel Split Strategy...");
        let channel_result = test_channel_split_strategy(
            &prices,
            symbol,
            initial_balance,
            leverage,
            avg_commission,
        ).await?;
        channel_result.print();
        all_results.push(channel_result);

        // Тест 2: Market Making
        println!("\n2️⃣ Testing Market Making Strategy...");
        let mm_result = test_market_making_strategy(
            &prices,
            symbol,
            initial_balance,
            leverage,
            avg_commission,
        ).await?;
        mm_result.print();
        all_results.push(mm_result);

        // Тест 3: HFT
        println!("\n3️⃣ Testing HFT Strategy...");
        let hft_result = test_hft_strategy(
            &prices,
            symbol,
            initial_balance,
            leverage,
            avg_commission,
        ).await?;
        hft_result.print();
        all_results.push(hft_result);
    }

    // 3. Сводка результатов
    println!("\n\n{}", "=".repeat(70));
    println!("📈 FINAL SUMMARY");
    println!("{}", "=".repeat(70));

    // Сохраняем в CSV
    save_results_csv(&all_results)?;

    // Сохраняем в PostgreSQL, если доступно
    #[cfg(feature = "database")]
    {
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            println!("\n💾 Saving results to PostgreSQL...");
            match save_results_to_database(&all_results, &database_url).await {
                Ok(count) => {
                    println!("   ✅ Saved {} backtest results to database", count);
                }
                Err(e) => {
                    eprintln!("   ⚠️  Failed to save to database: {}", e);
                    eprintln!("   💡 Results still saved to CSV");
                }
            }
        } else {
            println!("\n💡 Database not configured (DATABASE_URL not set)");
            println!("   Results saved to CSV only");
        }
    }

    // Показываем лучшие результаты
    print_summary(&all_results);

    println!("\n✅ Results saved to: data/investor_demo_results.csv");
    #[cfg(feature = "database")]
    {
        if std::env::var("DATABASE_URL").is_ok() {
            println!("💾 Results also saved to PostgreSQL");
        }
    }
    println!("🌐 Start dashboard: cargo run --bin investor_dashboard --features dashboard");

    Ok(())
}

async fn test_channel_split_strategy(
    prices: &[(u64, f64)],
    symbol: &str,
    initial_balance: f64,
    leverage: f64,
    commission: f64,
) -> Result<StrategyResult> {
    let mut strategy = ChannelSplitStrategy::new(
        20,    // окно канала
        1.0,   // ширина канала 1%
        2.0,   // стоп-лосс 2%
        4.0,   // тейк-профит 4%
        3,     // дробление на 3 части
    );

    let mut balance = initial_balance;
    let mut trades = 0;
    let mut wins = 0;
    let mut losses = 0;
    let mut total_pnl = 0.0;
    let mut total_fees = 0.0;
    let mut max_balance = balance;
    let mut max_drawdown = 0.0;
    let mut position: Option<(f64, f64)> = None; // (avg_entry_price, position_size_usd)

    for (timestamp, price) in prices {
        let signal = strategy.update(*timestamp, *price, balance);

        match signal {
            ChannelSplitSignal::EnterSplit { parts } => {
                // Рассчитываем среднюю цену входа и размер позиции
                let total_size_usd: f64 = parts.iter().map(|p| p.size).sum();
                let avg_entry_price: f64 = parts.iter()
                    .zip(parts.iter().map(|p| p.size))
                    .map(|(part, size)| part.price * size)
                    .sum::<f64>() / total_size_usd.max(0.001);
                
                // Сохраняем позицию: используем 30% от баланса (как в стратегии)
                let position_size_usd = total_size_usd.min(balance * 0.3);
                position = Some((avg_entry_price, position_size_usd));
            }
            ChannelSplitSignal::Exit { price: exit_price, .. } => {
                if let Some((entry_price, position_size_usd)) = position {
                    // Реальный расчет P&L: изменение цены * размер позиции * leverage
                    let price_change_pct = (exit_price - entry_price) / entry_price;
                    
                    // P&L = изменение цены * размер позиции * leverage
                    let pnl_before_fee = price_change_pct * position_size_usd * leverage;
                    
                    // Комиссии: на вход и выход от размера позиции
                    let entry_fee = position_size_usd * commission;
                    let exit_fee = position_size_usd * (1.0 + price_change_pct.abs() * leverage) * commission;
                    let total_fee = entry_fee + exit_fee;
                    
                    let pnl_after_fee = pnl_before_fee - total_fee;

                    balance += pnl_after_fee;
                    total_pnl += pnl_after_fee;
                    total_fees += total_fee;
                    trades += 1;

                    if pnl_after_fee > 0.0 {
                        wins += 1;
                    } else {
                        losses += 1;
                    }

                    position = None;

                    if balance > max_balance {
                        max_balance = balance;
                    }

                    let drawdown = ((max_balance - balance) / max_balance) * 100.0;
                    if drawdown > max_drawdown {
                        max_drawdown = drawdown;
                    }
                }
            }
            _ => {}
        }
    }

    let win_rate = if trades > 0 { wins as f64 / trades as f64 * 100.0 } else { 0.0 };
    let roi = ((balance - initial_balance) / initial_balance) * 100.0;
    let profit_factor = if losses > 0 {
        (wins as f64 * total_pnl.max(0.0) / trades as f64) / (losses as f64 * total_pnl.min(0.0).abs() / trades as f64).max(0.001)
    } else {
        999.0
    };

    Ok(StrategyResult {
        name: "Channel Split".to_string(),
        symbol: symbol.to_string(),
        initial_balance,
        final_balance: balance,
        total_pnl,
        total_fees,
        trades,
        wins,
        losses,
        win_rate,
        roi,
        max_drawdown,
        profit_factor,
    })
}

async fn test_market_making_strategy(
    prices: &[(u64, f64)],
    symbol: &str,
    initial_balance: f64,
    leverage: f64,
    commission: f64,
) -> Result<StrategyResult> {
    let mut strategy = MarketMakingStrategy::new(
        0.1,   // спред 0.1%
        5.0,   // 5% от баланса на ордер
        1000.0, // макс позиция
        20,    // окно
    );

    let mut balance = initial_balance;
    let mut trades = 0;
    let mut wins = 0;
    let mut losses = 0;
    let mut total_pnl = 0.0;
    let mut total_fees = 0.0;
    let mut max_balance = balance;
    let mut max_drawdown = 0.0;
    let mut last_order_update: Option<u64> = None;
    let min_order_interval = 300; // Минимум 5 минут между ордерами (реалистично)

    for (timestamp, price) in prices {
        let signal = strategy.update(*price, balance);

        match signal {
            MarketMakingSignal::UpdateOrders { bid, ask, bid_size, ask_size } => {
                // MM получает прибыль от спреда без leverage (это не направленная торговля)
                // Проверяем интервал между ордерами для реалистичности
                if last_order_update.is_none() || timestamp - last_order_update.unwrap() >= min_order_interval {
                    // Размер каждой позиции (bid и ask)
                    let order_size = bid_size.min(ask_size).min(balance * 0.05); // Максимум 5% от баланса на ордер
                    
                    // Спред в процентах
                    let spread_pct = (ask - bid) / bid;
                    
                    // Прибыль от спреда (maker комиссия обычно меньше, получаем rebate)
                    // Упрощенно: прибыль = спред - комиссии (maker обычно 0.015%, получаем часть спреда)
                    let maker_rebate = 0.0001; // Небольшой rebate за maker ордер
                    let spread_profit = spread_pct * order_size - (order_size * commission * 2.0) + (order_size * maker_rebate * 2.0);
                    
                    // Реалистично: не каждая пара ордеров заполняется
                    // Вероятность заполнения обеих сторон ~30% в спокойном рынке
                    if spread_profit > 0.0 {
                        let pnl = spread_profit * 0.3; // 30% вероятность заполнения

                        balance += pnl;
                        total_pnl += pnl;
                        total_fees += order_size * commission * 2.0 * 0.3;
                        trades += 1;

                        if pnl > 0.0 {
                            wins += 1;
                        } else {
                            losses += 1;
                        }
                        
                        last_order_update = Some(*timestamp);
                    }
                }
            }
            _ => {}
        }

        if balance > max_balance {
            max_balance = balance;
        }

        let drawdown = ((max_balance - balance) / max_balance) * 100.0;
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
    }

    let win_rate = if trades > 0 { wins as f64 / trades as f64 * 100.0 } else { 0.0 };
    let roi = ((balance - initial_balance) / initial_balance) * 100.0;
    let profit_factor = if losses > 0 {
        (wins as f64 * total_pnl.max(0.0) / trades as f64) / (losses as f64 * total_pnl.min(0.0).abs() / trades as f64).max(0.001)
    } else {
        999.0
    };

    Ok(StrategyResult {
        name: "Market Making".to_string(),
        symbol: symbol.to_string(),
        initial_balance,
        final_balance: balance,
        total_pnl,
        total_fees,
        trades,
        wins,
        losses,
        win_rate,
        roi,
        max_drawdown,
        profit_factor,
    })
}

async fn test_hft_strategy(
    prices: &[(u64, f64)],
    symbol: &str,
    initial_balance: f64,
    leverage: f64,
    commission: f64,
) -> Result<StrategyResult> {
    let mut strategy = HFTStrategy::new(
        0.01,  // порог входа 0.01%
        0.02,  // тейк-профит 0.02%
        60,    // макс удержание 60 сек
        10.0,  // 10% от баланса
    );

    let mut balance = initial_balance;
    let mut trades = 0;
    let mut wins = 0;
    let mut losses = 0;
    let mut total_pnl = 0.0;
    let mut total_fees = 0.0;
    let mut max_balance = balance;
    let mut max_drawdown = 0.0;
    let mut position: Option<(u64, f64, String, f64)> = None; // (time, price, side, size)

    for (timestamp, price) in prices {
        // Симулируем ордербук (упрощенно)
        let bid_volume = 100.0;
        let ask_volume = 100.0;

        let signal = strategy.update(*timestamp, *price, bid_volume, ask_volume, balance);

        match signal {
            HFTSignal::Enter { side, price: entry_price, size, timestamp: entry_time } => {
                position = Some((entry_time, entry_price, side, size));
            }
            _ => {}
        }

        // Проверка выхода
        if let Some((entry_time, entry_price, ref side, size)) = position {
            if strategy.check_exit(entry_price, entry_time, *price, *timestamp, side) {
                // size уже является суммой в USDT (10% от баланса на момент входа)
                let price_change_pct = if side == "buy" {
                    (*price - entry_price) / entry_price
                } else {
                    (entry_price - *price) / entry_price
                };

                // P&L = изменение цены * размер позиции * leverage
                let pnl_before_fee = price_change_pct * size * leverage;
                
                // Комиссии: на вход и выход
                let entry_fee = size * commission;
                let exit_value = size * (1.0 + price_change_pct.abs() * leverage);
                let exit_fee = exit_value * commission;
                let total_fee = entry_fee + exit_fee;
                
                let pnl_after_fee = pnl_before_fee - total_fee;

                balance += pnl_after_fee;
                total_pnl += pnl_after_fee;
                total_fees += total_fee;
                trades += 1;

                if pnl_after_fee > 0.0 {
                    wins += 1;
                } else {
                    losses += 1;
                }

                position = None;
            }
        }

        if balance > max_balance {
            max_balance = balance;
        }

        let drawdown = ((max_balance - balance) / max_balance) * 100.0;
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
    }

    let win_rate = if trades > 0 { wins as f64 / trades as f64 * 100.0 } else { 0.0 };
    let roi = ((balance - initial_balance) / initial_balance) * 100.0;
    let profit_factor = if losses > 0 {
        (wins as f64 * total_pnl.max(0.0) / trades as f64) / (losses as f64 * total_pnl.min(0.0).abs() / trades as f64).max(0.001)
    } else {
        999.0
    };

    Ok(StrategyResult {
        name: "HFT".to_string(),
        symbol: symbol.to_string(),
        initial_balance,
        final_balance: balance,
        total_pnl,
        total_fees,
        trades,
        wins,
        losses,
        win_rate,
        roi,
        max_drawdown,
        profit_factor,
    })
}

async fn fetch_historical_prices(symbol: &str, hours: u32) -> Result<Vec<(u64, f64)>> {
    let client = Client::new();
    let interval = "15m"; // 15 минут
    let limit = (hours * 60) / 15;

    let url = format!(
        "https://api.gateio.ws/api/v4/futures/usdt/candlesticks?contract={}&interval={}&limit={}",
        symbol, interval, limit
    );

    let resp = client.get(&url).send().await?;
    let json: Value = resp.json().await?;

    let mut prices = Vec::new();

    if let Some(candles) = json.as_array() {
        for candle in candles {
            // Формат Gate.io: объект с полями t (timestamp), c (close), o, h, l, v
            // Или массив: [t, v, c, h, l, o, sum]
            if candle.is_object() {
                // Объект формат
                if let Some(ts) = candle.get("t").and_then(|v| v.as_u64()) {
                    if let Some(close_str) = candle.get("c").and_then(|v| v.as_str()) {
                        if let Ok(close) = close_str.parse::<f64>() {
                            prices.push((ts, close));
                        }
                    }
                }
            } else if let Some(arr) = candle.as_array() {
                // Массив формат: [t, v, c, h, l, o, sum] или [t, o, h, l, c, v, sum]
                if arr.len() >= 6 {
                    if let Some(ts) = arr[0].as_u64() {
                        // Пробуем разные позиции для close price
                        let close = arr.get(4).and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok()))
                            .or_else(|| arr.get(2).and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())))
                            .or_else(|| arr.get(4).and_then(|v| v.as_f64()))
                            .or_else(|| arr.get(2).and_then(|v| v.as_f64()));
                        
                        if let Some(close) = close {
                            prices.push((ts, close));
                        }
                    }
                }
            }
        }
    }

    prices.sort_by_key(|(t, _)| *t);
    Ok(prices)
}

async fn get_deposit_info(client: &GateClient) -> Result<DepositInfo> {
    let accounts = client.fetch_futures_accounts("usdt").await?;
    
    let total = accounts.get("total")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    
    let available = accounts.get("available")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    
    let locked = total - available;

    Ok(DepositInfo {
        total,
        available,
        locked,
    })
}

async fn get_commission_rate(_client: &GateClient) -> Result<CommissionInfo> {
    // Gate.io стандартные комиссии
    // Для фьючерсов обычно: maker 0.015%, taker 0.05%
    Ok(CommissionInfo {
        maker: 0.00015,  // 0.015%
        taker: 0.0005,   // 0.05%
    })
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

fn save_results_csv(results: &[StrategyResult]) -> Result<()> {
    std::fs::create_dir_all("data")?;
    let mut file = File::create("data/investor_demo_results.csv")?;
    
    writeln!(file, "strategy,symbol,initial_balance,final_balance,total_pnl,total_fees,trades,wins,losses,win_rate,roi,profit_factor,max_drawdown")?;
    
    for result in results {
        file.write_all(result.to_csv_line().as_bytes())?;
    }
    
    Ok(())
}

#[cfg(feature = "database")]
async fn save_results_to_database(results: &[StrategyResult], database_url: &str) -> Result<usize> {
    use std::str::FromStr;
    
    let pool = DatabaseRepository::create_pool(database_url).await?;
    let repo = DatabaseRepository::new(pool);
    
    // Проверяем соединение
    repo.test_connection().await?;
    
    let mut saved = 0;
    let start_time = Utc::now() - ChronoDuration::hours(72);
    let end_time = Utc::now();
    
    for result in results {
        let db_result = DbBacktestResult {
            strategy_name: result.name.clone(),
            symbol: result.symbol.clone(),
            initial_balance: Decimal::from_str(&format!("{:.8}", result.initial_balance))?,
            leverage: 100, // x100 leverage
            final_balance: Decimal::from_str(&format!("{:.8}", result.final_balance))?,
            total_pnl: Decimal::from_str(&format!("{:.8}", result.total_pnl))?,
            total_fees: Decimal::from_str(&format!("{:.8}", result.total_fees))?,
            total_trades: result.trades as i32,
            winning_trades: result.wins as i32,
            losing_trades: result.losses as i32,
            win_rate: Decimal::from_str(&format!("{:.4}", result.win_rate / 100.0))?,
            roi: Decimal::from_str(&format!("{:.4}", result.roi / 100.0))?,
            profit_factor: Some(Decimal::from_str(&format!("{:.4}", result.profit_factor))?),
            max_drawdown: Some(Decimal::from_str(&format!("{:.4}", result.max_drawdown / 100.0))?),
            sharpe_ratio: None, // Можно рассчитать позже
            start_time: Some(start_time),
            end_time: Some(end_time),
            config: Some(serde_json::json!({
                "leverage": 100,
                "commission_maker": 0.00015,
                "commission_taker": 0.0005,
            })),
            notes: Some(format!("Automated backtest for investor demo")),
        };
        
        match repo.insert_backtest_result(&db_result).await {
            Ok(_) => saved += 1,
            Err(e) => eprintln!("   ⚠️  Failed to save {} on {}: {}", result.name, result.symbol, e),
        }
    }
    
    Ok(saved)
}

fn print_summary(results: &[StrategyResult]) {
    // Группируем по стратегиям
    let mut by_strategy: std::collections::HashMap<String, Vec<&StrategyResult>> = std::collections::HashMap::new();
    
    for result in results {
        by_strategy.entry(result.name.clone())
            .or_insert_with(Vec::new)
            .push(result);
    }

    for (name, strategy_results) in by_strategy {
        println!("\n📊 {} Strategy:", name);
        let avg_roi: f64 = strategy_results.iter().map(|r| r.roi).sum::<f64>() / strategy_results.len() as f64;
        let total_trades: usize = strategy_results.iter().map(|r| r.trades).sum();
        let avg_win_rate: f64 = strategy_results.iter().map(|r| r.win_rate).sum::<f64>() / strategy_results.len() as f64;
        
        println!("   Average ROI: {:.2}%", avg_roi);
        println!("   Total Trades: {}", total_trades);
        println!("   Average Win Rate: {:.1}%", avg_win_rate);
    }

    // Лучшая стратегия
    let best = results.iter()
        .max_by(|a, b| a.roi.partial_cmp(&b.roi).unwrap());

    if let Some(best) = best {
        println!("\n🏆 Best Strategy:");
        println!("   {} on {}: {:.2}% ROI", best.name, best.symbol, best.roi);
    }
}

