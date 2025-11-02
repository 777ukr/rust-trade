//! Анализатор логов стратегии
//! Загружает CSV файлы с историей торговли и анализирует эффективность
//! Используйте через Cursor для анализа ваших стратегий

#![cfg(feature = "gate_exec")]

use anyhow::Result;
use clap::Parser;
use rust_test::analytics::log_analyzer::LogAnalyzer;

#[derive(Parser)]
#[command(name = "analyze-strategy", about = "Analyze trading strategy from CSV logs")]
struct Args {
    /// Path to CSV file with trade history
    #[arg(short, long)]
    log_file: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    println!("📊 Analyzing strategy from: {}\n", args.log_file);
    
    match LogAnalyzer::analyze_strategy_from_log(&args.log_file) {
        Ok(analysis) => {
            analysis.print();
            
            // Сохраняем отчет
            let report_path = format!("data/strategy_report_{}.txt", 
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs());
            
            std::fs::write(&report_path, format!("{:?}", analysis))
                .unwrap_or_else(|_| println!("⚠️ Could not save report"));
            
            println!("\n💾 Report saved to: {}", report_path);
        }
        Err(e) => {
            eprintln!("❌ Error analyzing log: {}", e);
        }
    }
    
    Ok(())
}

