//! Веб-дашборд для инвестора
//! Показывает результаты всех 3 стратегий с визуализацией

#![cfg(feature = "dashboard")]

use axum::{
    response::{Html, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize)]
struct StrategyResult {
    strategy: String,
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
    profit_factor: f64,
    max_drawdown: f64,
}

#[derive(Serialize)]
struct DashboardData {
    results: Vec<StrategyResult>,
    summary: SummaryData,
}

#[derive(Serialize)]
struct SummaryData {
    total_strategies: usize,
    best_strategy: String,
    best_roi: f64,
    average_roi: f64,
    total_pnl: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", get(index))
        .route("/api/data", get(get_data))
        .route("/api/results", get(get_results));

    let addr = "0.0.0.0:8080";
    println!("🌐 Investor Dashboard запущен!");
    println!("📊 Откройте в браузере:");
    println!("   http://localhost:8080");
    println!("   http://0.0.0.0:8080");
    println!("   http://<your-ip>:8080");
    println!("\n💡 Для остановки нажмите Ctrl+C\n");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn index() -> Html<String> {
    let html = std::fs::read_to_string("templates/investor_dashboard.html")
        .unwrap_or_else(|_| include_str!("../../templates/investor_dashboard.html").to_string());
    Html(html)
}

async fn get_data() -> Json<DashboardData> {
    let results = load_investor_results().unwrap_or_default();
    let summary = calculate_summary(&results);
    
    Json(DashboardData { results, summary })
}

async fn get_results() -> Json<Vec<StrategyResult>> {
    let results = load_investor_results().unwrap_or_default();
    Json(results)
}

fn load_investor_results() -> Result<Vec<StrategyResult>, Box<dyn std::error::Error>> {
    let file_path = "data/investor_demo_results.csv";
    
    if !fs::metadata(file_path).is_ok() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(file_path)?;
    let lines: Vec<&str> = content.lines().collect();
    
    if lines.len() < 2 {
        return Ok(vec![]);
    }

    let mut results = Vec::new();
    
    for line in lines.iter().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 13 {
            continue;
        }

        let result = StrategyResult {
            strategy: parts[0].to_string(),
            symbol: parts[1].to_string(),
            initial_balance: parts[2].parse().unwrap_or(0.0),
            final_balance: parts[3].parse().unwrap_or(0.0),
            total_pnl: parts[4].parse().unwrap_or(0.0),
            total_fees: parts[5].parse().unwrap_or(0.0),
            trades: parts[6].parse().unwrap_or(0),
            wins: parts[7].parse().unwrap_or(0),
            losses: parts[8].parse().unwrap_or(0),
            win_rate: parts[9].parse().unwrap_or(0.0),
            roi: parts[10].parse().unwrap_or(0.0),
            profit_factor: parts[11].parse().unwrap_or(0.0),
            max_drawdown: parts[12].parse().unwrap_or(0.0),
        };
        
        results.push(result);
    }
    
    Ok(results)
}

fn calculate_summary(results: &[StrategyResult]) -> SummaryData {
    if results.is_empty() {
        return SummaryData {
            total_strategies: 0,
            best_strategy: "N/A".to_string(),
            best_roi: 0.0,
            average_roi: 0.0,
            total_pnl: 0.0,
        };
    }

    let best = results.iter()
        .max_by(|a, b| a.roi.partial_cmp(&b.roi).unwrap());
    
    let best_strategy = best.map(|b| format!("{} on {}", b.strategy, b.symbol))
        .unwrap_or_else(|| "N/A".to_string());
    let best_roi = best.map(|b| b.roi).unwrap_or(0.0);
    
    let average_roi = results.iter().map(|r| r.roi).sum::<f64>() / results.len() as f64;
    let total_pnl = results.iter().map(|r| r.total_pnl).sum();
    
    SummaryData {
        total_strategies: results.len(),
        best_strategy,
        best_roi,
        average_roi,
        total_pnl,
    }
}

