//! Лонговая стратегия с трейлинг стопом
//! Входит в лонг при тренде вверх, защищает прибыль трейлинг стопом

pub trait StrategyReset {
    fn reset_strategy(&mut self);
}

#[derive(Debug, Clone)]
pub struct LongTrailingStrategy {
    trailing_stop_percent: f64,  // Процент для трейлинг стопа (например, 2%)
    trailing_activation_percent: f64, // При каком профите активировать трейлинг (например, 1%)
    entry_threshold: f64,        // Порог входа (например, 0.5% роста)
    lookback_period: usize,      // Период для определения тренда
    entry_price: Option<f64>,
    highest_price: Option<f64>,
    trailing_stop_price: Option<f64>,
}

#[derive(Debug, Clone)]
pub enum LongTrailingSignal {
    EnterLong { price: f64 },
    ExitLong { price: f64, reason: String },
    Hold,
}

impl LongTrailingStrategy {
    pub fn new(
        trailing_stop_percent: f64,
        trailing_activation_percent: f64,
        entry_threshold: f64,
        lookback_period: usize,
    ) -> Self {
        Self {
            trailing_stop_percent,
            trailing_activation_percent,
            entry_threshold,
            lookback_period,
            entry_price: None,
            highest_price: None,
            trailing_stop_price: None,
        }
    }

    pub fn update(&mut self, prices: &[f64], current_price: f64) -> LongTrailingSignal {
        if prices.len() < self.lookback_period {
            return LongTrailingSignal::Hold;
        }

        // Определяем тренд
        let recent_avg: f64 = prices[prices.len() - self.lookback_period..]
            .iter()
            .sum::<f64>() / self.lookback_period as f64;
        let older_avg: f64 = prices[prices.len() - self.lookback_period * 2..prices.len() - self.lookback_period]
            .iter()
            .sum::<f64>() / self.lookback_period as f64;

        let trend_up = recent_avg > older_avg * (1.0 + self.entry_threshold / 100.0);

        match self.entry_price {
            None => {
                // Нет позиции - проверяем вход
                if trend_up {
                    self.entry_price = Some(current_price);
                    self.highest_price = Some(current_price);
                    LongTrailingSignal::EnterLong { price: current_price }
                } else {
                    LongTrailingSignal::Hold
                }
            }
            Some(entry) => {
                // Есть позиция - управляем трейлинг стопом
                if let Some(highest) = self.highest_price {
                    if current_price > highest {
                        self.highest_price = Some(current_price);
                    }
                } else {
                    self.highest_price = Some(current_price);
                }

                let highest = self.highest_price.unwrap();
                let profit_pct = ((highest - entry) / entry) * 100.0;

                // Активируем трейлинг стоп только если профит > activation threshold
                if profit_pct >= self.trailing_activation_percent {
                    let new_stop = highest * (1.0 - self.trailing_stop_percent / 100.0);
                    
                    if let Some(stop) = self.trailing_stop_price {
                        if new_stop > stop {
                            self.trailing_stop_price = Some(new_stop);
                        }
                    } else {
                        self.trailing_stop_price = Some(new_stop);
                    }

                    // Проверяем срабатывание стоп-лосса
                    if let Some(stop) = self.trailing_stop_price {
                        if current_price <= stop {
                            self.reset();
                            return LongTrailingSignal::ExitLong {
                                price: current_price,
                                reason: format!("Trailing stop hit at {:.2}% profit", profit_pct),
                            };
                        }
                    }
                }

                // Защита от больших убытков
                let current_loss = ((current_price - entry) / entry) * 100.0;
                if current_loss <= -5.0 {
                    self.reset();
                    return LongTrailingSignal::ExitLong {
                        price: current_price,
                        reason: format!("Stop loss at {:.2}% loss", current_loss),
                    };
                }

                LongTrailingSignal::Hold
            }
        }
    }

    pub fn reset(&mut self) {
        self.entry_price = None;
        self.highest_price = None;
        self.trailing_stop_price = None;
    }
}

impl StrategyReset for LongTrailingStrategy {
    fn reset_strategy(&mut self) {
        self.reset();
    }
}

