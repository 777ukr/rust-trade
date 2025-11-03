//! Шортовая стратегия с трейлинг стопом
//! Входит в шорт при тренде вниз, защищает прибыль трейлинг стопом

pub trait StrategyReset {
    fn reset(&mut self);
}

#[derive(Debug, Clone)]
pub struct ShortTrailingStrategy {
    trailing_stop_percent: f64,
    trailing_activation_percent: f64,
    entry_threshold: f64,
    lookback_period: usize,
    entry_price: Option<f64>,
    lowest_price: Option<f64>,
    trailing_stop_price: Option<f64>,
}

#[derive(Debug, Clone)]
pub enum ShortTrailingSignal {
    EnterShort { price: f64 },
    ExitShort { price: f64, reason: String },
    Hold,
}

impl ShortTrailingStrategy {
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
            lowest_price: None,
            trailing_stop_price: None,
        }
    }

    pub fn update(&mut self, prices: &[f64], current_price: f64) -> ShortTrailingSignal {
        if prices.len() < self.lookback_period {
            return ShortTrailingSignal::Hold;
        }

        // Определяем нисходящий тренд
        let recent_avg: f64 = prices[prices.len() - self.lookback_period..]
            .iter()
            .sum::<f64>() / self.lookback_period as f64;
        let older_avg: f64 = prices[prices.len() - self.lookback_period * 2..prices.len() - self.lookback_period]
            .iter()
            .sum::<f64>() / self.lookback_period as f64;

        let trend_down = recent_avg < older_avg * (1.0 - self.entry_threshold / 100.0);

        match self.entry_price {
            None => {
                if trend_down {
                    self.entry_price = Some(current_price);
                    self.lowest_price = Some(current_price);
                    ShortTrailingSignal::EnterShort { price: current_price }
                } else {
                    ShortTrailingSignal::Hold
                }
            }
            Some(entry) => {
                if let Some(lowest) = self.lowest_price {
                    if current_price < lowest {
                        self.lowest_price = Some(current_price);
                    }
                } else {
                    self.lowest_price = Some(current_price);
                }

                let lowest = self.lowest_price.unwrap();
                // Для шорта: профит = (entry - lowest) / entry
                let profit_pct = ((entry - lowest) / entry) * 100.0;

                if profit_pct >= self.trailing_activation_percent {
                    // Для шорта: стоп выше минимальной цены
                    let new_stop = lowest * (1.0 + self.trailing_stop_percent / 100.0);
                    
                    if let Some(stop) = self.trailing_stop_price {
                        if new_stop < stop {
                            self.trailing_stop_price = Some(new_stop);
                        }
                    } else {
                        self.trailing_stop_price = Some(new_stop);
                    }

                    if let Some(stop) = self.trailing_stop_price {
                        if current_price >= stop {
                            self.reset();
                            return ShortTrailingSignal::ExitShort {
                                price: current_price,
                                reason: format!("Trailing stop hit at {:.2}% profit", profit_pct),
                            };
                        }
                    }
                }

                let current_loss = ((entry - current_price) / entry) * 100.0;
                if current_loss <= -5.0 {
                    self.reset();
                    return ShortTrailingSignal::ExitShort {
                        price: current_price,
                        reason: format!("Stop loss at {:.2}% loss", current_loss),
                    };
                }

                ShortTrailingSignal::Hold
            }
        }
    }

    pub fn reset(&mut self) {
        self.entry_price = None;
        self.lowest_price = None;
        self.trailing_stop_price = None;
    }
}

impl TradingStrategy for ShortTrailingStrategy {
    fn reset(&mut self) {
        self.reset();
    }
}

