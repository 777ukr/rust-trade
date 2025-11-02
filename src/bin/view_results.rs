//! Просмотр результатов для инвестора и трейдера
//! Отображает все файлы результатов в удобном формате

use anyhow::Result;
use clap::Parser;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "view-results", about = "View trading results for investors and traders")]
struct Args {
    /// Show all result files
    #[arg(short, long)]
    all: bool,
    
    /// Show only summary
    #[arg(short, long)]
    summary: bool,
    
    /// Specific file to view
    #[arg(short, long)]
    file: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    println!("📊 Trading Results Viewer\n");
    
    let data_dir = PathBuf::from("data");
    
    if args.all || args.file.is_none() {
        show_all_files(&data_dir)?;
    }
    
    if let Some(file) = &args.file {
        show_file_content(&data_dir, file, args.summary)?;
    } else if !args.all {
        // Показываем последний файл
        if let Some(latest) = get_latest_file(&data_dir)? {
            println!("\n📄 Latest file: {}\n", latest);
            show_file_content(&data_dir, &latest, false)?;
        }
    }
    
    Ok(())
}

fn show_all_files(data_dir: &PathBuf) -> Result<()> {
    if !data_dir.exists() {
        println!("⚠️ Data directory doesn't exist. Run backtests first.");
        return Ok(());
    }
    
    println!("📁 Available result files:\n");
    
    let entries = fs::read_dir(data_dir)?;
    let mut files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file() && 
            (e.path().extension().map(|s| s == "csv").unwrap_or(false) ||
             e.path().extension().map(|s| s == "txt").unwrap_or(false))
        })
        .collect();
    
    files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
    files.reverse(); // Новые первыми
    
    if files.is_empty() {
        println!("  No result files found.");
        println!("  Run: cargo run --bin sol_backtest");
        return Ok(());
    }
    
    for (i, entry) in files.iter().enumerate() {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy();
        let size = entry.metadata()?.len();
        let modified = entry.metadata()?.modified()?;
        let time_str = format_datetime(modified);
        
        println!("  {}. {} ({:.1} KB) - {}", 
            i + 1, name, size as f64 / 1024.0, time_str);
    }
    
    Ok(())
}

fn get_latest_file(data_dir: &PathBuf) -> Result<Option<String>> {
    if !data_dir.exists() {
        return Ok(None);
    }
    
    let entries = fs::read_dir(data_dir)?;
    let latest = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter_map(|e| {
            e.metadata()
                .and_then(|m| m.modified())
                .map(|t| (t, e.path()))
                .ok()
        })
        .max_by_key(|(t, _)| *t)
        .map(|(_, p)| p.file_name().unwrap().to_string_lossy().to_string());
    
    Ok(latest)
}

fn show_file_content(data_dir: &PathBuf, filename: &str, summary_only: bool) -> Result<()> {
    let file_path = data_dir.join(filename);
    
    if !file_path.exists() {
        eprintln!("❌ File not found: {}", file_path.display());
        return Ok(());
    }
    
    let content = fs::read_to_string(&file_path)?;
    let lines: Vec<&str> = content.lines().collect();
    
    if lines.is_empty() {
        println!("⚠️ File is empty");
        return Ok(());
    }
    
    // Определяем тип файла по имени
    if filename.contains("prices") {
        show_prices_file(&lines, summary_only)?;
    } else if filename.contains("backtest") {
        show_backtest_file(&lines, summary_only)?;
    } else if filename.contains("report") {
        show_report_file(&lines)?;
    } else {
        // Универсальный просмотр
        show_generic_file(&lines, summary_only)?;
    }
    
    Ok(())
}

fn show_prices_file(lines: &[&str], summary: bool) -> Result<()> {
    println!("📈 Price History File\n");
    
    if lines.len() <= 1 {
        println!("  No data");
        return Ok(());
    }
    
    let data_lines: Vec<&str> = lines[1..].iter().filter(|l| !l.trim().is_empty()).copied().collect();
    
    println!("  Total data points: {}", data_lines.len());
    
    if !summary && !data_lines.is_empty() {
        // Показываем первые и последние 5
        println!("\n  First 5 records:");
        for (i, line) in data_lines.iter().take(5).enumerate() {
            if let Some((ts, price)) = parse_price_line(line) {
                println!("    {}: {} = ${:.2}", i + 1, format_timestamp(ts), price);
            }
        }
        
        if data_lines.len() > 10 {
            println!("\n  ... ({} more records) ...\n", data_lines.len() - 10);
        }
        
        if data_lines.len() > 5 {
            println!("  Last 5 records:");
            for (i, line) in data_lines.iter().skip(data_lines.len().saturating_sub(5)).enumerate() {
                if let Some((ts, price)) = parse_price_line(line) {
                    println!("    {}: {} = ${:.2}", 
                        data_lines.len() - 4 + i, format_timestamp(ts), price);
                }
            }
        }
        
        // Статистика
        let prices: Vec<f64> = data_lines.iter()
            .filter_map(|l| parse_price_line(l).map(|(_, p)| p))
            .collect();
        
        if !prices.is_empty() {
            let min = prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let avg = prices.iter().sum::<f64>() / prices.len() as f64;
            let change = ((max - min) / min) * 100.0;
            
            println!("\n  📊 Statistics:");
            println!("    Min: ${:.2}", min);
            println!("    Max: ${:.2}", max);
            println!("    Avg: ${:.2}", avg);
            println!("    Range: {:.2}%", change);
        }
    }
    
    Ok(())
}

fn show_backtest_file(lines: &[&str], summary: bool) -> Result<()> {
    println!("💰 Backtest Results\n");
    
    if lines.len() <= 1 {
        println!("  No trades");
        return Ok(());
    }
    
    let trades: Vec<&str> = lines[1..].iter().filter(|l| !l.trim().is_empty()).copied().collect();
    
    let mut total_pnl = 0.0;
    let mut wins = 0;
    let mut losses = 0;
    let mut win_pnl = 0.0;
    let mut loss_pnl = 0.0;
    
    for line in &trades {
        if let Some((pnl, _)) = parse_backtest_line(line) {
            total_pnl += pnl;
            if pnl > 0.0 {
                wins += 1;
                win_pnl += pnl;
            } else {
                losses += 1;
                loss_pnl += pnl;
            }
        }
    }
    
    let win_rate = if !trades.is_empty() {
        wins as f64 / trades.len() as f64 * 100.0
    } else {
        0.0
    };
    
    println!("  📊 Performance Summary:");
    println!("    Total Trades: {}", trades.len());
    println!("    Wins: {} | Losses: {}", wins, losses);
    println!("    Win Rate: {:.1}%", win_rate);
    println!("    Total P&L: ${:.2}", total_pnl);
    
    if wins > 0 {
        println!("    Avg Win: ${:.2}", win_pnl / wins as f64);
    }
    if losses > 0 {
        println!("    Avg Loss: ${:.2}", loss_pnl / losses as f64);
    }
    
    let profit_factor = if loss_pnl.abs() > 0.0 {
        win_pnl / loss_pnl.abs()
    } else if wins > 0 {
        f64::INFINITY
    } else {
        0.0
    };
    
    if profit_factor.is_finite() {
        println!("    Profit Factor: {:.2}", profit_factor);
    }
    
    if !summary && !trades.is_empty() {
        println!("\n  📋 Recent Trades (last 10):");
        for (i, line) in trades.iter().rev().take(10).enumerate() {
            if let Some((pnl, details)) = parse_backtest_line(line) {
                let sign = if pnl >= 0.0 { "✅" } else { "❌" };
                println!("    {} Trade {}: {} ${:.2}", sign, i + 1, details, pnl);
            }
        }
    }
    
    Ok(())
}

fn show_report_file(lines: &[&str]) -> Result<()> {
    println!("📄 Strategy Report\n");
    for line in lines {
        println!("  {}", line);
    }
    Ok(())
}

fn show_generic_file(lines: &[&str], summary: bool) -> Result<()> {
    if summary {
        println!("  First 10 lines:");
        for (i, line) in lines.iter().take(10).enumerate() {
            println!("    {}: {}", i + 1, line);
        }
        if lines.len() > 10 {
            println!("  ... ({} more lines)", lines.len() - 10);
        }
    } else {
        for line in lines {
            println!("  {}", line);
        }
    }
    Ok(())
}

fn parse_price_line(line: &str) -> Option<(u64, f64)> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() >= 2 {
        let ts = parts[0].parse().ok()?;
        let price = parts[1].parse().ok()?;
        Some((ts, price))
    } else {
        None
    }
}

fn parse_backtest_line(line: &str) -> Option<(f64, String)> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() >= 7 {
        let pnl: f64 = parts[5].parse().ok()?;
        let side = parts[4];
        let entry = parts[1].parse::<f64>().ok()?;
        let exit = parts[3].parse::<f64>().ok()?;
        let details = format!("{} {}→{}", side, entry, exit);
        Some((pnl, details))
    } else {
        None
    }
}

fn format_timestamp(ts: u64) -> String {
    // Простой формат даты
    let secs = ts;
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    format!("Day {} {:02}:{:02}", days, hours, minutes)
}

fn format_datetime(modified: std::time::SystemTime) -> String {
    if let Ok(duration) = modified.duration_since(std::time::SystemTime::UNIX_EPOCH) {
        let ts = duration.as_secs();
        format_timestamp(ts)
    } else {
        "unknown".to_string()
    }
}

