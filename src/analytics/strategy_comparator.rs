//! Сравнение 3 вариантов стратегий для выбора лучшей

use crate::analytics::performance::PerformanceMetrics;
use crate::analytics::trade_analyzer::TradeRecord;
#[cfg(feature = "gate_exec")]
use crate::strategy::adaptive_channel::StrategyVariant;

pub struct StrategyComparator;

impl StrategyComparator {
    /// Сравнение всех 3 вариантов стратегий
    #[cfg(feature = "gate_exec")]
    pub fn compare_all(trades_trailing: &[TradeRecord], 
                      trades_early: &[TradeRecord],
                      trades_extended: &[TradeRecord]) -> ComparisonResult {
        let trailing = PerformanceMetrics::calculate(trades_trailing);
        let early = PerformanceMetrics::calculate(trades_early);
        let extended = PerformanceMetrics::calculate(trades_extended);

        let best = Self::select_best(&trailing, &early, &extended);

        ComparisonResult {
            trailing_metrics: trailing,
            early_metrics: early,
            extended_metrics: extended,
            best_variant: best,
        }
    }

    #[cfg(feature = "gate_exec")]
    fn select_best(t: &PerformanceMetrics, 
                   e: &PerformanceMetrics,
                   x: &PerformanceMetrics) -> StrategyVariant {
        // Комплексная оценка: P&L * Sharpe * (1 - drawdown/100)
        let score_t = t.total_pnl * t.sharpe_ratio * (1.0 - t.max_drawdown / 100.0);
        let score_e = e.total_pnl * e.sharpe_ratio * (1.0 - e.max_drawdown / 100.0);
        let score_x = x.total_pnl * x.sharpe_ratio * (1.0 - x.max_drawdown / 100.0);

        if score_t >= score_e && score_t >= score_x {
            StrategyVariant::TrailingStop
        } else if score_e >= score_x {
            StrategyVariant::EarlyExit
        } else {
            StrategyVariant::ExtendedTarget
        }
    }
}

#[derive(Debug)]
pub struct ComparisonResult {
    pub trailing_metrics: PerformanceMetrics,
    pub early_metrics: PerformanceMetrics,
    pub extended_metrics: PerformanceMetrics,
    #[cfg(feature = "gate_exec")]
    pub best_variant: StrategyVariant,
}

impl ComparisonResult {
    pub fn print(&self) {
        println!("\n📊 Strategy Comparison:");
        println!("\n1️⃣ Trailing Stop:");
        self.trailing_metrics.print();
        
        println!("\n2️⃣ Early Exit:");
        self.early_metrics.print();
        
        println!("\n3️⃣ Extended Target:");
        self.extended_metrics.print();

        #[cfg(feature = "gate_exec")]
        {
            println!("\n🏆 Best Strategy: {:?}", self.best_variant);
            println!("   Recommendation: Use this variant for live trading");
        }
    }
}

