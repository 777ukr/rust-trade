//! Анализатор логов стратегии
//! Анализирует файлы с логами торговли и оценивает эффективность стратегии
//! Используется для анализа исторических данных через Cursor

use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::analytics::trade_analyzer::TradeRecord;
use crate::analytics::performance::PerformanceMetrics;

pub struct LogAnalyzer;

impl LogAnalyzer {
    /// Загрузить торговые данные из CSV лога
    pub fn load_from_csv(path: &str) -> Result<Vec<TradeRecord>, String> {
        let file = File::open(path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        
        let mut trades = Vec::new();
        let mut lines = reader.lines();
        
        // Пропускаем заголовок
        lines.next();
        
        for line in lines {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() >= 6 {
                let entry_time = parts[0].parse().unwrap_or(0);
                let entry_price = parts[1].parse().unwrap_or(0.0);
                let exit_time = parts[2].parse().unwrap_or(0);
                let exit_price = parts[3].parse().unwrap_or(0.0);
                let side = parts[4].to_string();
                let size = parts[5].parse().unwrap_or(0.0);
                
                let pnl = if parts.len() > 6 {
                    parts[6].parse().ok()
                } else {
                    Some((exit_price - entry_price) * size)
                };
                
                trades.push(TradeRecord {
                    timestamp: entry_time,
                    entry_time,
                    entry_price,
                    exit_time,
                    exit_price,
                    side,
                    size,
                    pnl,
                });
            }
        }
        
        Ok(trades)
    }

    /// Анализ стратегии по логам
    pub fn analyze_strategy_from_log(log_path: &str) -> Result<StrategyAnalysis, String> {
        let trades = Self::load_from_csv(log_path)?;
        let metrics = PerformanceMetrics::calculate(&trades);
        
        // Анализ паттернов
        let patterns = Self::detect_patterns(&trades);
        
        // Оценка стратегии (создаем до move)
        let evaluation = Self::evaluate_strategy(&metrics, &patterns);
        
        // Генерируем рекомендации (клонируем для использования после move)
        let recommendations = Self::generate_improvements(&metrics, &patterns);
        
        Ok(StrategyAnalysis {
            metrics,
            patterns,
            evaluation,
            recommendations,
        })
    }

    fn detect_patterns(trades: &[TradeRecord]) -> TradingPatterns {
        let mut best_hours = std::collections::HashMap::new();
        let mut worst_hours = std::collections::HashMap::new();
        
        for trade in trades {
            let hour = (trade.entry_time % 86400) / 3600;
            let pnl = trade.pnl.unwrap_or(0.0);
            
            if pnl > 0.0 {
                *best_hours.entry(hour as u32).or_insert(0) += 1;
            } else {
                *worst_hours.entry(hour as u32).or_insert(0) += 1;
            }
        }
        
        TradingPatterns {
            best_trading_hours: best_hours,
            worst_trading_hours: worst_hours,
            avg_trade_duration: Self::calculate_avg_duration(trades),
        }
    }

    fn calculate_avg_duration(trades: &[TradeRecord]) -> f64 {
        if trades.is_empty() {
            return 0.0;
        }
        let total: u64 = trades.iter()
            .map(|t| t.exit_time.saturating_sub(t.entry_time))
            .sum();
        total as f64 / trades.len() as f64
    }

    fn evaluate_strategy(metrics: &PerformanceMetrics, _patterns: &TradingPatterns) -> StrategyEvaluation {
        let score = (metrics.win_rate * 0.4
            + (metrics.profit_factor.min(3.0) / 3.0 * 100.0) * 0.3
            + (metrics.sharpe_ratio.min(2.0) / 2.0 * 100.0) * 0.2
            + (1.0 - metrics.max_drawdown / 50.0) * 10.0) as f64;
        
        let grade = if score >= 80.0 {
            "Excellent"
        } else if score >= 65.0 {
            "Good"
        } else if score >= 50.0 {
            "Average"
        } else {
            "Needs Improvement"
        };
        
        StrategyEvaluation {
            score: score as f64,
            grade: grade.to_string(),
            strengths: Self::identify_strengths(metrics),
            weaknesses: Self::identify_weaknesses(metrics),
        }
    }

    fn identify_strengths(metrics: &PerformanceMetrics) -> Vec<String> {
        let mut strengths = Vec::new();
        
        if metrics.win_rate >= 60.0 {
            strengths.push("High win rate".to_string());
        }
        if metrics.profit_factor >= 2.0 {
            strengths.push("Strong profit factor".to_string());
        }
        if metrics.max_drawdown < 15.0 {
            strengths.push("Low drawdown".to_string());
        }
        if metrics.sharpe_ratio >= 1.5 {
            strengths.push("Good risk-adjusted returns".to_string());
        }
        
        strengths
    }

    fn identify_weaknesses(metrics: &PerformanceMetrics) -> Vec<String> {
        let mut weaknesses = Vec::new();
        
        if metrics.win_rate < 50.0 {
            weaknesses.push("Low win rate - improve entry signals".to_string());
        }
        if metrics.profit_factor < 1.5 {
            weaknesses.push("Weak profit factor - improve risk/reward".to_string());
        }
        if metrics.max_drawdown > 25.0 {
            weaknesses.push("High drawdown - strengthen risk management".to_string());
        }
        if metrics.sharpe_ratio < 1.0 {
            weaknesses.push("Low Sharpe ratio - high volatility of returns".to_string());
        }
        
        weaknesses
    }

    fn generate_improvements(metrics: &PerformanceMetrics, patterns: &TradingPatterns) -> Vec<String> {
        let mut improvements = Vec::new();
        
        if metrics.win_rate < 55.0 {
            improvements.push("Consider tighter entry conditions or better signal filtering".to_string());
        }
        
        if patterns.avg_trade_duration > 3600.0 {
            improvements.push("Trades hold too long - consider faster exit strategy".to_string());
        }
        
        if metrics.max_consecutive_losses > 5 {
            improvements.push("Too many consecutive losses - add position sizing reduction after losses".to_string());
        }
        
        if metrics.profit_factor < 1.8 {
            improvements.push("Improve risk/reward ratio - aim for better exits or tighter stops".to_string());
        }
        
        improvements
    }
}

#[derive(Debug)]
pub struct StrategyAnalysis {
    pub metrics: PerformanceMetrics,
    pub patterns: TradingPatterns,
    pub evaluation: StrategyEvaluation,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TradingPatterns {
    pub best_trading_hours: std::collections::HashMap<u32, usize>,
    pub worst_trading_hours: std::collections::HashMap<u32, usize>,
    pub avg_trade_duration: f64,
}

#[derive(Debug)]
pub struct StrategyEvaluation {
    pub score: f64,
    pub grade: String,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
}

impl StrategyAnalysis {
    pub fn print(&self) {
        println!("\n📊 Strategy Analysis from Logs:");
        self.metrics.print();
        
        println!("\n🎯 Strategy Grade: {} ({:.1}/100)", self.evaluation.grade, self.evaluation.score);
        
        if !self.evaluation.strengths.is_empty() {
            println!("\n✅ Strengths:");
            for s in &self.evaluation.strengths {
                println!("  • {}", s);
            }
        }
        
        if !self.evaluation.weaknesses.is_empty() {
            println!("\n⚠️ Weaknesses:");
            for w in &self.evaluation.weaknesses {
                println!("  • {}", w);
            }
        }
        
        if !self.recommendations.is_empty() {
            println!("\n💡 Recommendations:");
            for r in &self.recommendations {
                println!("  • {}", r);
            }
        }
    }
}

