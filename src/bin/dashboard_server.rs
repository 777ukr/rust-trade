//! Веб-дашборд для инвестора и трейдера
//! Доступен по IP адресу для просмотра результатов в браузере

use axum::{
    extract::Query,
    response::{Html, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize)]
struct TradeRecord {
    entry_time: u64,
    entry_price: f64,
    exit_time: u64,
    exit_price: f64,
    side: String,
    pnl: f64,
    pnl_percent: f64,
}

#[derive(Serialize, Deserialize)]
struct PriceRecord {
    timestamp: u64,
    price: f64,
}

#[derive(Serialize)]
struct DashboardData {
    trades: Vec<TradeRecord>,
    prices: Vec<PriceRecord>,
    summary: PerformanceSummary,
}

#[derive(Serialize)]
struct PerformanceSummary {
    total_trades: usize,
    wins: usize,
    losses: usize,
    win_rate: f64,
    total_pnl: f64,
    avg_win: f64,
    avg_loss: f64,
    profit_factor: f64,
    max_drawdown: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", get(index))
        .route("/api/data", get(get_data))
        .route("/api/files", get(list_files))
        .route("/api/backtest", get(get_backtest))
        .route("/api/prices", get(get_prices));

    let addr = "0.0.0.0:8080";
    println!("🚀 Dashboard server starting on http://{}", addr);
    println!("📊 Access from browser: http://localhost:8080");
    println!("🌐 Access from network: http://<your-ip>:8080");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index() -> Html<String> {
    let html = std::fs::read_to_string("templates/dashboard.html")
        .unwrap_or_else(|_| include_str!("../../templates/dashboard.html").to_string());
    Html(html)
}

#[derive(Deserialize)]
struct FileQuery {
    file: Option<String>,
}

async fn get_data(Query(params): Query<FileQuery>) -> Json<DashboardData> {
    let file_name = params.file.clone();
    let backtest_file = file_name.clone().unwrap_or_else(|| {
        get_latest_file("data", "backtest").unwrap_or_else(|| "data/sol_backtest.csv".to_string())
    });
    
    let prices_file = params.file.as_ref().map(|f| f.replace("backtest", "prices"))
        .unwrap_or_else(|| {
            get_latest_file("data", "prices").unwrap_or_else(|| "data/sol_prices.csv".to_string())
        });

    let trades = load_backtest(&backtest_file).unwrap_or_default();
    let prices = load_prices(&prices_file).unwrap_or_default();
    let summary = calculate_summary(&trades);

    Json(DashboardData {
        trades,
        prices,
        summary,
    })
}

async fn list_files() -> Json<Vec<String>> {
    let files = get_all_files("data").unwrap_or_default();
    Json(files)
}

async fn get_backtest(Query(params): Query<FileQuery>) -> Json<Vec<TradeRecord>> {
    let file = params.file.unwrap_or_else(|| {
        get_latest_file("data", "backtest").unwrap_or_else(|| "data/sol_backtest.csv".to_string())
    });
    Json(load_backtest(&file).unwrap_or_default())
}

async fn get_prices(Query(params): Query<FileQuery>) -> Json<Vec<PriceRecord>> {
    let file = params.file.unwrap_or_else(|| {
        get_latest_file("data", "prices").unwrap_or_else(|| "data/sol_prices.csv".to_string())
    });
    Json(load_prices(&file).unwrap_or_default())
}

fn load_backtest(path: &str) -> Result<Vec<TradeRecord>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut trades = Vec::new();
    
    for line in content.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 7 {
            trades.push(TradeRecord {
                entry_time: parts[0].parse()?,
                entry_price: parts[1].parse()?,
                exit_time: parts[2].parse()?,
                exit_price: parts[3].parse()?,
                side: parts[4].to_string(),
                pnl: parts[5].parse()?,
                pnl_percent: parts[6].parse()?,
            });
        }
    }
    
    Ok(trades)
}

fn load_prices(path: &str) -> Result<Vec<PriceRecord>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut prices = Vec::new();
    
    for line in content.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 2 {
            prices.push(PriceRecord {
                timestamp: parts[0].parse()?,
                price: parts[1].parse()?,
            });
        }
    }
    
    Ok(prices)
}

fn calculate_summary(trades: &[TradeRecord]) -> PerformanceSummary {
    if trades.is_empty() {
        return PerformanceSummary {
            total_trades: 0,
            wins: 0,
            losses: 0,
            win_rate: 0.0,
            total_pnl: 0.0,
            avg_win: 0.0,
            avg_loss: 0.0,
            profit_factor: 0.0,
            max_drawdown: 0.0,
        };
    }

    let wins: Vec<_> = trades.iter().filter(|t| t.pnl > 0.0).collect();
    let losses: Vec<_> = trades.iter().filter(|t| t.pnl < 0.0).collect();
    
    let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();
    let win_pnl: f64 = wins.iter().map(|t| t.pnl).sum();
    let loss_pnl: f64 = losses.iter().map(|t| t.pnl).sum();
    
    let win_rate = wins.len() as f64 / trades.len() as f64 * 100.0;
    let avg_win = if !wins.is_empty() { win_pnl / wins.len() as f64 } else { 0.0 };
    let avg_loss = if !losses.is_empty() { loss_pnl / losses.len() as f64 } else { 0.0 };
    let profit_factor = if avg_loss.abs() > 0.0 {
        win_pnl / avg_loss.abs()
    } else if !wins.is_empty() {
        f64::INFINITY
    } else {
        0.0
    };

    // Простой расчет drawdown
    let mut cumulative = 0.0;
    let mut peak = 0.0;
    let mut max_drawdown = 0.0;
    for trade in trades {
        cumulative += trade.pnl;
        if cumulative > peak {
            peak = cumulative;
        }
        let drawdown = if peak > 0.0 {
            ((peak - cumulative) / peak) * 100.0
        } else {
            0.0
        };
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
    }

    PerformanceSummary {
        total_trades: trades.len(),
        wins: wins.len(),
        losses: losses.len(),
        win_rate,
        total_pnl,
        avg_win,
        avg_loss,
        profit_factor: if profit_factor.is_finite() { profit_factor } else { 999.0 },
        max_drawdown,
    }
}

fn get_latest_file(dir: &str, pattern: &str) -> Option<String> {
    let entries = fs::read_dir(dir).ok()?;
    entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            if !e.path().is_file() {
                return false;
            }
            if let Some(name) = e.path().file_name() {
                name.to_string_lossy().contains(pattern)
            } else {
                false
            }
        })
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok())?
        .path()
        .to_str()
        .map(|s| s.to_string())
}

fn get_all_files(dir: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let entries = fs::read_dir(dir)?;
    Ok(entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.path().file_name()?.to_str().map(|s| s.to_string()))
        .collect())
}

