//! Анализатор канальной торговли
//! Автоматический расчет прибыли/убытка при торговле в канале
//! Учитывает комиссию, плечо, стоп-лоссы

use crate::analytics::trade_analyzer::TradeRecord;

#[derive(Debug, Clone)]
pub struct ChannelTrade {
    pub entry_time: u64,
    pub entry_price: f64,
    pub exit_time: u64,
    pub exit_price: f64,
    pub side: String,
    pub size: f64,
    pub pnl_before_fee: f64,
    pub fee: f64,
    pub pnl_after_fee: f64,
    pub pnl_percent: f64,
    pub stop_loss_hit: bool,
    pub channel_exit: bool,
}

#[derive(Debug)]
pub struct ChannelAnalysis {
    pub trades: Vec<ChannelTrade>,
    pub total_pnl_before_fee: f64,
    pub total_fees: f64,
    pub total_pnl_after_fee: f64,
    pub wins: usize,
    pub losses: usize,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub stop_loss_triggers: usize,
    pub max_drawdown: f64,
    pub initial_deposit: f64,
    pub final_balance: f64,
    pub roi: f64,
}

pub struct ChannelAnalyzer {
    pub commission_rate: f64,        // Комиссия Gate.io (например, 0.0003 = 0.03%)
    pub leverage: f64,               // Плечо (например, 100)
    pub channel_width_percent: f64, // Ширина канала в %
    pub stop_loss_percent: f64,      // Стоп-лосс в %
    pub take_profit_percent: f64,    // Тейк-профит в %
    pub initial_deposit: f64,        // Начальный депозит
}

impl ChannelAnalyzer {
    pub fn new(
        commission_rate: f64,
        leverage: f64,
        channel_width: f64,
        stop_loss: f64,
        take_profit: f64,
        initial_deposit: f64,
    ) -> Self {
        Self {
            commission_rate,
            leverage,
            channel_width_percent: channel_width,
            stop_loss_percent: stop_loss,
            take_profit_percent: take_profit,
            initial_deposit,
        }
    }

    /// Анализ торговли в канале на исторических данных
    pub fn analyze_channel_trading(
        &self,
        prices: &[(u64, f64)],
        channel_lower: &[(u64, f64)],
        channel_upper: &[(u64, f64)],
    ) -> ChannelAnalysis {
        let window_size = 20.min(prices.len());
        let mut trades = Vec::new();
        let mut current_position: Option<(u64, f64, String, f64)> = None; // (time, price, side, size)
        let mut balance = self.initial_deposit;
        let mut max_balance = balance;
        let mut max_drawdown = 0.0;

        for i in window_size..prices.len() {
            let (timestamp, price) = prices[i];
            
            // Находим канал для текущего момента
            let channel_min = self.find_channel_value(channel_lower, timestamp);
            let channel_max = self.find_channel_value(channel_upper, timestamp);
            
            if channel_min.is_none() || channel_max.is_none() {
                continue;
            }
            
            let min = channel_min.unwrap();
            let max = channel_max.unwrap();
            
            // Логика входа/выхода
            if current_position.is_none() {
                // Вход в нижней части канала (покупка)
                let entry_threshold = min * (1.0 + self.channel_width_percent / 4.0);
                if price <= entry_threshold {
                    let size = self.calculate_position_size(balance, price);
                    current_position = Some((timestamp, price, "long".to_string(), size));
                }
            } else {
                let (entry_time, entry_price, side, size) = current_position.as_ref().unwrap();
                
                // Проверка стоп-лосса
                let stop_loss_price = if side == "long" {
                    entry_price * (1.0 - self.stop_loss_percent / 100.0)
                } else {
                    entry_price * (1.0 + self.stop_loss_percent / 100.0)
                };
                
                let stop_loss_hit = if side == "long" {
                    price <= stop_loss_price
                } else {
                    price >= stop_loss_price
                };
                
                // Проверка тейк-профита
                let take_profit_price = if side == "long" {
                    entry_price * (1.0 + self.take_profit_percent / 100.0)
                } else {
                    entry_price * (1.0 - self.take_profit_percent / 100.0)
                };
                
                let take_profit_hit = if side == "long" {
                    price >= take_profit_price
                } else {
                    price <= take_profit_price
                };
                
                // Выход при достижении верха канала или стоп/тейк
                let channel_exit = if side == "long" {
                    price >= max * (1.0 - self.channel_width_percent / 4.0)
                } else {
                    price <= min * (1.0 + self.channel_width_percent / 4.0)
                };
                
                let should_exit = stop_loss_hit || take_profit_hit || channel_exit;
                
                if should_exit {
                    let pnl_before_fee = if side == "long" {
                        (price - entry_price) * size * self.leverage
                    } else {
                        (entry_price - price) * size * self.leverage
                    };
                    
                    // Комиссия: вход + выход
                    let entry_fee = entry_price * size * self.commission_rate;
                    let exit_fee = price * size * self.commission_rate;
                    let total_fee = entry_fee + exit_fee;
                    
                    let pnl_after_fee = pnl_before_fee - total_fee;
                    let pnl_percent = (pnl_after_fee / (entry_price * size)) * 100.0;
                    
                    balance += pnl_after_fee;
                    
                    if balance > max_balance {
                        max_balance = balance;
                    }
                    
                    let drawdown = ((max_balance - balance) / max_balance) * 100.0;
                    if drawdown > max_drawdown {
                        max_drawdown = drawdown;
                    }
                    
                    trades.push(ChannelTrade {
                        entry_time: *entry_time,
                        entry_price: *entry_price,
                        exit_time: timestamp,
                        exit_price: price,
                        side: side.clone(),
                        size: *size,
                        pnl_before_fee,
                        fee: total_fee,
                        pnl_after_fee,
                        pnl_percent,
                        stop_loss_hit,
                        channel_exit,
                    });
                    
                    current_position = None;
                }
            }
        }
        
        // Закрываем открытую позицию
        if let Some((entry_time, entry_price, side, size)) = current_position {
            if let Some((exit_time, exit_price)) = prices.last() {
                let pnl_before_fee = if side == "long" {
                    (exit_price - entry_price) * size * self.leverage
                } else {
                    (entry_price - exit_price) * size * self.leverage
                };
                
                let entry_fee = entry_price * size * self.commission_rate;
                let exit_fee = exit_price * size * self.commission_rate;
                let total_fee = entry_fee + exit_fee;
                
                let pnl_after_fee = pnl_before_fee - total_fee;
                
                balance += pnl_after_fee;
                
                trades.push(ChannelTrade {
                    entry_time,
                    entry_price,
                    exit_time: *exit_time,
                    exit_price: *exit_price,
                    side,
                    size,
                    pnl_before_fee,
                    fee: total_fee,
                    pnl_after_fee,
                    pnl_percent: (pnl_after_fee / (entry_price * size)) * 100.0,
                    stop_loss_hit: false,
                    channel_exit: false,
                });
            }
        }
        
        let total_pnl_before_fee: f64 = trades.iter().map(|t| t.pnl_before_fee).sum();
        let total_fees: f64 = trades.iter().map(|t| t.fee).sum();
        let total_pnl_after_fee: f64 = trades.iter().map(|t| t.pnl_after_fee).sum();
        
        let wins = trades.iter().filter(|t| t.pnl_after_fee > 0.0).count();
        let losses = trades.iter().filter(|t| t.pnl_after_fee < 0.0).count();
        let win_rate = if !trades.is_empty() {
            wins as f64 / trades.len() as f64 * 100.0
        } else {
            0.0
        };
        
        let win_sum: f64 = trades.iter().filter(|t| t.pnl_after_fee > 0.0).map(|t| t.pnl_after_fee).sum();
        let loss_sum: f64 = trades.iter().filter(|t| t.pnl_after_fee < 0.0).map(|t| t.pnl_after_fee.abs()).sum();
        let profit_factor = if loss_sum > 0.0 {
            win_sum / loss_sum
        } else if wins > 0 {
            f64::INFINITY
        } else {
            0.0
        };
        
        let stop_loss_triggers = trades.iter().filter(|t| t.stop_loss_hit).count();
        let roi = ((balance - self.initial_deposit) / self.initial_deposit) * 100.0;
        
        ChannelAnalysis {
            trades,
            total_pnl_before_fee,
            total_fees,
            total_pnl_after_fee,
            wins,
            losses,
            win_rate,
            profit_factor: if profit_factor.is_finite() { profit_factor } else { 999.0 },
            stop_loss_triggers,
            max_drawdown,
            initial_deposit: self.initial_deposit,
            final_balance: balance,
            roi,
        }
    }

    fn calculate_position_size(&self, balance: f64, price: f64) -> f64 {
        // Используем 10% от баланса для каждой позиции
        let risk_amount = balance * 0.1;
        risk_amount / price
    }

    fn find_channel_value(&self, channel_data: &[(u64, f64)], timestamp: u64) -> Option<f64> {
        // Находим ближайшее значение канала для timestamp
        channel_data
            .iter()
            .min_by_key(|(t, _)| (*t as i64 - timestamp as i64).abs() as u64)
            .map(|(_, price)| *price)
    }
}

impl ChannelAnalysis {
    pub fn print(&self) {
        println!("\n📊 Channel Trading Analysis:");
        println!("  Initial Deposit: ${:.2}", self.initial_deposit);
        println!("  Final Balance: ${:.2}", self.final_balance);
        println!("  ROI: {:.2}%", self.roi);
        println!("\n  Total Trades: {}", self.trades.len());
        println!("  Wins: {} | Losses: {}", self.wins, self.losses);
        println!("  Win Rate: {:.1}%", self.win_rate);
        println!("\n  P&L Before Fees: ${:.2}", self.total_pnl_before_fee);
        println!("  Total Fees: ${:.2}", self.total_fees);
        println!("  P&L After Fees: ${:.2}", self.total_pnl_after_fee);
        println!("\n  Profit Factor: {:.2}", self.profit_factor);
        println!("  Max Drawdown: {:.2}%", self.max_drawdown);
        println!("  Stop-Loss Triggers: {}", self.stop_loss_triggers);
        
        if !self.trades.is_empty() {
            println!("\n  Recent Trades:");
            for (i, trade) in self.trades.iter().rev().take(10).enumerate() {
                let sign = if trade.pnl_after_fee >= 0.0 { "✅" } else { "❌" };
                println!("    {} Trade {}: {} {}→{} | P&L: ${:.2} | Fee: ${:.4}", 
                    sign, i + 1, trade.side, trade.entry_price, trade.exit_price,
                    trade.pnl_after_fee, trade.fee);
            }
        }
    }
}

