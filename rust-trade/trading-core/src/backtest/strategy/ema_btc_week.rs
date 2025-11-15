use super::base::{Signal, Strategy};
use crate::data::types::{OHLCData, TickData, Timeframe};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::{HashMap, VecDeque};

/// EMA BTC Week Strategy - адаптированная из Jesse
/// 
/// Логика стратегии:
/// - Интервал: 4 часа (использует OHLC данные)
/// - Вход: если цена упала на 10% от недельного максимума
/// - Стоп-лосс: 20% ниже цены входа
/// - Тейк-профит: 50% выше цены входа
/// - Трейлинг-стоп: активируется после роста на 5%, отступ 5%
pub struct EmaBtcWeekStrategy {
    // Параметры стратегии
    week_drop_percent: Decimal,      // 10% просадка от недельного максимума
    stop_loss_percent: Decimal,      // 20% стоп-лосс
    take_profit_percent: Decimal,    // 50% тейк-профит
    trailing_activation_percent: Decimal, // 5% для активации трейлинга
    trailing_stop_percent: Decimal,  // 5% отступ для трейлинг-стопа
    
    // Состояние
    highest_weekly_price: Option<Decimal>,
    entry_price: Option<Decimal>,
    trailing_active: bool,
    highest_after_entry: Option<Decimal>,
    position: bool,
    symbol: Option<String>,
    
    // История цен для расчета недельного максимума (42 свечи по 4 часа ≈ 1 неделя)
    price_history: VecDeque<Decimal>,
    max_history_size: usize,
}

impl EmaBtcWeekStrategy {
    pub fn new() -> Self {
        Self {
            week_drop_percent: dec!(0.10),      // 10%
            stop_loss_percent: dec!(0.20),      // 20%
            take_profit_percent: dec!(0.50),    // 50%
            trailing_activation_percent: dec!(0.05), // 5%
            trailing_stop_percent: dec!(0.05),  // 5%
            highest_weekly_price: None,
            entry_price: None,
            trailing_active: false,
            highest_after_entry: None,
            position: false,
            symbol: None,
            price_history: VecDeque::new(),
            max_history_size: 42, // ~1 неделя 4H свечей
        }
    }
    
    fn calculate_week_high(&self) -> Option<Decimal> {
        if self.price_history.is_empty() {
            return None;
        }
        Some(*self.price_history.iter().max().unwrap())
    }
    
    fn update_week_high(&mut self, price: Decimal) {
        if let Some(current_high) = self.highest_weekly_price {
            if price > current_high {
                self.highest_weekly_price = Some(price);
            }
        } else {
            self.highest_weekly_price = Some(price);
        }
    }
}

impl Strategy for EmaBtcWeekStrategy {
    fn name(&self) -> &str {
        "EMA BTC Week Strategy"
    }

    fn initialize(&mut self, params: HashMap<String, String>) -> Result<(), String> {
        if let Some(week_drop) = params.get("week_drop_percent") {
            let value: f64 = week_drop.parse().map_err(|_| "Invalid week_drop_percent")?;
            self.week_drop_percent = Decimal::try_from(value / 100.0)
                .map_err(|_| "week_drop_percent must be a valid decimal")?;
        }
        
        if let Some(stop_loss) = params.get("stop_loss_percent") {
            let value: f64 = stop_loss.parse().map_err(|_| "Invalid stop_loss_percent")?;
            self.stop_loss_percent = Decimal::try_from(value / 100.0)
                .map_err(|_| "stop_loss_percent must be a valid decimal")?;
        }
        
        if let Some(take_profit) = params.get("take_profit_percent") {
            let value: f64 = take_profit.parse().map_err(|_| "Invalid take_profit_percent")?;
            self.take_profit_percent = Decimal::try_from(value / 100.0)
                .map_err(|_| "take_profit_percent must be a valid decimal")?;
        }

        println!(
            "EMA BTC Week Strategy initialized: week_drop={:.2}%, stop_loss={:.2}%, take_profit={:.2}%",
            self.week_drop_percent * dec!(100),
            self.stop_loss_percent * dec!(100),
            self.take_profit_percent * dec!(100)
        );
        Ok(())
    }

    fn reset(&mut self) {
        self.highest_weekly_price = None;
        self.entry_price = None;
        self.trailing_active = false;
        self.highest_after_entry = None;
        self.position = false;
        self.symbol = None;
        self.price_history.clear();
    }

    fn on_tick(&mut self, tick: &TickData) -> Signal {
        // Стратегия работает на OHLC данных, не на тиках
        // Для тиков просто обновляем историю
        if self.symbol.is_none() {
            self.symbol = Some(tick.symbol.clone());
        }
        
        // Обновляем историю цен
        self.price_history.push_back(tick.price);
        if self.price_history.len() > self.max_history_size {
            self.price_history.pop_front();
        }
        
        // Обновляем недельный максимум
        self.update_week_high(tick.price);
        
        // Если есть позиция, проверяем стоп-лосс и тейк-профит
        if self.position {
            if let Some(entry) = self.entry_price {
                let price_change = (tick.price - entry) / entry;
                
                // Тейк-профит
                if price_change >= self.take_profit_percent {
                    self.position = false;
                    self.entry_price = None;
                    self.trailing_active = false;
                    self.highest_after_entry = None;
                    return Signal::Sell {
                        symbol: tick.symbol.clone(),
                        quantity: Decimal::from(100),
                    };
                }
                
                // Стоп-лосс
                if price_change <= -self.stop_loss_percent {
                    self.position = false;
                    self.entry_price = None;
                    self.trailing_active = false;
                    self.highest_after_entry = None;
                    return Signal::Sell {
                        symbol: tick.symbol.clone(),
                        quantity: Decimal::from(100),
                    };
                }
                
                // Трейлинг-стоп
                if let Some(highest) = self.highest_after_entry {
                    if tick.price > highest {
                        self.highest_after_entry = Some(tick.price);
                    }
                    
                    // Активация трейлинга после 5% роста
                    if price_change >= self.trailing_activation_percent && !self.trailing_active {
                        self.trailing_active = true;
                    }
                    
                    // Трейлинг-стоп: если цена упала на 5% от максимума после входа
                    if self.trailing_active {
                        let drop_from_high = (highest - tick.price) / highest;
                        if drop_from_high >= self.trailing_stop_percent {
                            self.position = false;
                            self.entry_price = None;
                            self.trailing_active = false;
                            self.highest_after_entry = None;
                            return Signal::Sell {
                                symbol: tick.symbol.clone(),
                                quantity: Decimal::from(100),
                            };
                        }
                    }
                } else {
                    self.highest_after_entry = Some(tick.price);
                }
            }
        } else {
            // Нет позиции - проверяем условие входа
            if let Some(week_high) = self.highest_weekly_price {
                let drop = (week_high - tick.price) / week_high;
                
                if drop >= self.week_drop_percent {
                    // Вход в позицию
                    self.position = true;
                    self.entry_price = Some(tick.price);
                    self.trailing_active = false;
                    self.highest_after_entry = Some(tick.price);
                    return Signal::Buy {
                        symbol: tick.symbol.clone(),
                        quantity: Decimal::from(100),
                    };
                }
            }
        }

        Signal::Hold
    }

    fn on_ohlc(&mut self, ohlc: &OHLCData) -> Signal {
        // Стратегия оптимизирована для OHLC данных
        if self.symbol.is_none() {
            self.symbol = Some(ohlc.symbol.clone());
        }
        
        // Используем high для расчета недельного максимума
        self.price_history.push_back(ohlc.high);
        if self.price_history.len() > self.max_history_size {
            self.price_history.pop_front();
        }
        
        // Обновляем недельный максимум
        self.update_week_high(ohlc.high);
        
        // Если есть позиция, проверяем стоп-лосс и тейк-профит
        if self.position {
            if let Some(entry) = self.entry_price {
                let price_change = (ohlc.close - entry) / entry;
                
                // Тейк-профит
                if price_change >= self.take_profit_percent {
                    self.position = false;
                    self.entry_price = None;
                    self.trailing_active = false;
                    self.highest_after_entry = None;
                    return Signal::Sell {
                        symbol: ohlc.symbol.clone(),
                        quantity: Decimal::from(100),
                    };
                }
                
                // Стоп-лосс
                if price_change <= -self.stop_loss_percent {
                    self.position = false;
                    self.entry_price = None;
                    self.trailing_active = false;
                    self.highest_after_entry = None;
                    return Signal::Sell {
                        symbol: ohlc.symbol.clone(),
                        quantity: Decimal::from(100),
                    };
                }
                
                // Трейлинг-стоп
                if let Some(highest) = self.highest_after_entry {
                    if ohlc.high > highest {
                        self.highest_after_entry = Some(ohlc.high);
                    }
                    
                    // Активация трейлинга после 5% роста
                    if price_change >= self.trailing_activation_percent && !self.trailing_active {
                        self.trailing_active = true;
                    }
                    
                    // Трейлинг-стоп: если цена упала на 5% от максимума после входа
                    if self.trailing_active {
                        let current_high = self.highest_after_entry.unwrap();
                        let drop_from_high = (current_high - ohlc.close) / current_high;
                        if drop_from_high >= self.trailing_stop_percent {
                            self.position = false;
                            self.entry_price = None;
                            self.trailing_active = false;
                            self.highest_after_entry = None;
                            return Signal::Sell {
                                symbol: ohlc.symbol.clone(),
                                quantity: Decimal::from(100),
                            };
                        }
                    }
                } else {
                    self.highest_after_entry = Some(ohlc.high);
                }
            }
        } else {
            // Нет позиции - проверяем условие входа
            if let Some(week_high) = self.highest_weekly_price {
                let drop = (week_high - ohlc.close) / week_high;
                
                if drop >= self.week_drop_percent {
                    // Вход в позицию
                    self.position = true;
                    self.entry_price = Some(ohlc.close);
                    self.trailing_active = false;
                    self.highest_after_entry = Some(ohlc.high);
                    return Signal::Buy {
                        symbol: ohlc.symbol.clone(),
                        quantity: Decimal::from(100),
                    };
                }
            }
        }

        Signal::Hold
    }

    fn supports_ohlc(&self) -> bool {
        true
    }

    fn preferred_timeframe(&self) -> Option<Timeframe> {
        Some(Timeframe::FourHours) // 4 часа, как в оригинальной стратегии
    }
}

