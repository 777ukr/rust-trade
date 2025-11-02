//! Аналитика BTCUSDT за 3 суток + тестирование всех компонентов
//! Комплексная система проверки и анализа

use anyhow::Result;
#[cfg(feature = "gate_exec")]
use rust_test::tests::api_validation::run_validation_tests;
use rust_test::analytics::trade_analyzer::{TradeAnalyzer, TradeRecord};
use rust_test::analytics::performance::PerformanceMetrics;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 BTC Analytics & Validation Suite\n");

    // 1. Проверка API ключей и баланса
    println!("{}", "=".repeat(50));
    println!("STEP 1: API Validation");
    println!("{}", "=".repeat(50));
    
    #[cfg(feature = "gate_exec")]
    {
        let validation = run_validation_tests().await?;
        if !validation.ready_for_trading {
            eprintln!("\n❌ System not ready for trading. Fix issues above.");
            return Ok(());
        }
    }
    
    #[cfg(not(feature = "gate_exec"))]
    {
        println!("⚠️ gate_exec feature not enabled - skipping API validation");
    }

    // 2. Получение истории торгов за 3 суток
    println!("\n\n");
    println!("{}", "=".repeat(50));
    println!("STEP 2: Fetching Trade History (3 days)");
    println!("{}", "=".repeat(50));
    
    // TODO: Реализовать получение реальных данных с Gate.io
    // Пока используем заглушку для демонстрации структуры
    let trades = fetch_3day_history().await?;

    // 3. Анализ торговли
    println!("\n\n");
    println!("{}", "=".repeat(50));
    println!("STEP 3: Trade Analysis");
    println!("{}", "=".repeat(50));
    let analysis = TradeAnalyzer::analyze_period(&trades, 3);
    analysis.print();

    // 4. Оценка стратегии
    println!("\n\n");
    println!("{}", "=".repeat(50));
    println!("STEP 4: Strategy Evaluation");
    println!("{}", "=".repeat(50));
    evaluate_strategy_performance(&trades);

    println!("\n✅ Analysis complete!");
    Ok(())
}

async fn fetch_3day_history() -> Result<Vec<TradeRecord>> {
    // TODO: Реализовать получение реальных данных
    // Используем структуру для демонстрации
    println!("📥 Fetching BTCUSDT futures trades from Gate.io...");
    
    // Заглушка - в реальности здесь будет запрос к Gate.io API
    Ok(Vec::new())
}

fn evaluate_strategy_performance(trades: &[TradeRecord]) {
    if trades.is_empty() {
        println!("⚠️ No trades to analyze");
        return;
    }

    let metrics = PerformanceMetrics::calculate(trades);
    
    println!("\n📈 Strategy Performance Summary:");
    println!("  Win Rate: {:.1}%", metrics.win_rate);
    println!("  Total P&L: ${:.2}", metrics.total_pnl);
    println!("  Profit Factor: {:.2}", metrics.profit_factor);
    println!("  Sharpe Ratio: {:.2}", metrics.sharpe_ratio);
    
    // Оценка качества
    let score = evaluate_score(&metrics);
    println!("\n🎯 Strategy Score: {:.1}/100", score);
    
    if score >= 70.0 {
        println!("✅ Strategy shows GOOD performance");
    } else if score >= 50.0 {
        println!("⚠️ Strategy shows AVERAGE performance");
    } else {
        println!("❌ Strategy shows POOR performance - needs improvement");
    }
}

fn evaluate_score(metrics: &PerformanceMetrics) -> f64 {
    let mut score = 0.0;
    
    // Win rate (40 points max)
    score += (metrics.win_rate / 100.0) * 40.0;
    
    // Profit factor (30 points max)
    let pf_score = (metrics.profit_factor.min(3.0) / 3.0) * 30.0;
    score += pf_score;
    
    // Sharpe ratio (20 points max)
    let sharpe_score = (metrics.sharpe_ratio.min(2.0) / 2.0) * 20.0;
    score += sharpe_score;
    
    // Drawdown penalty (10 points - inverse)
    let dd_penalty = 10.0 - (metrics.max_drawdown.min(50.0) / 50.0) * 10.0;
    score += dd_penalty;
    
    score
}

