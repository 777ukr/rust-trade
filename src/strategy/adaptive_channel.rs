//! Адаптивная канальная стратегия с 3 вариантами исполнения
//! 1. Trailing stop - с отпуском вне канала при прорыве
//! 2. Early exit - закрытие при намеке на разворот
//! 3. Extended target - бесконечное оттягивание цели

use std::collections::VecDeque;
use crate::base_classes::types::Side;
use crate::execution::{QuoteIntent, TimeInForce, Venue, ClientOrderId};
use crate::models::Position;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StrategyVariant {
    TrailingStop,    // Трейлинг с отпуском при прорыве
    EarlyExit,       // Ранний выход при развороте
    ExtendedTarget,  // Бесконечное оттягивание цели
}

pub struct AdaptiveChannelStrategy {
    variant: StrategyVariant,
    channel_window: usize,
    channel_size: f64,
    stop_loss_percent: f64,
    take_profit_percent: f64,
    price_history: VecDeque<f64>,
    current_position: Option<Position>,
    entry_price: Option<f64>,
    highest_price: Option<f64>,  // Для trailing stop
    lowest_price: Option<f64>,
    reversal_detected: bool,
}

impl AdaptiveChannelStrategy {
    pub fn new(
        variant: StrategyVariant,
        channel_window: usize,
        channel_size: f64,
        stop_loss: f64,
        take_profit: f64,
    ) -> Self {
        Self {
            variant,
            channel_window,
            channel_size,
            stop_loss_percent: stop_loss,
            take_profit_percent: take_profit,
            price_history: VecDeque::with_capacity(100),
            current_position: None,
            entry_price: None,
            highest_price: None,
            lowest_price: None,
            reversal_detected: false,
        }
    }

    pub fn update_price(&mut self, price: f64) {
        self.price_history.push_back(price);
        if self.price_history.len() > 100 {
            self.price_history.pop_front();
        }

        if let Some(entry) = self.entry_price {
            match self.variant {
                StrategyVariant::TrailingStop => self.update_trailing(price, entry),
                StrategyVariant::EarlyExit => self.detect_reversal(price),
                StrategyVariant::ExtendedTarget => self.update_extended_target(price, entry),
            }
        }
    }

    fn update_trailing(&mut self, price: f64, entry: f64) {
        // Обновляем максимальную цену для trailing stop
        if price > self.highest_price.unwrap_or(price) {
            self.highest_price = Some(price);
        }

        // Если цена прорвала канал вверх - отпускаем trailing
        if let Some(channel_max) = self.channel_max() {
            if price > channel_max * 1.05 {
                // Прорыв канала - отпускаем trailing stop
                self.highest_price = Some(price);
            }
        }
    }

    fn detect_reversal(&mut self, price: f64) {
        // Простая детекция разворота: смена тренда
        if self.price_history.len() >= 5 {
            let recent: Vec<f64> = self.price_history.iter().rev().take(5).copied().collect();
            let trend_up = recent[0] > recent[4];
            let current_trend = price > recent[0];
            
            // Разворот детектирован
            if trend_up && !current_trend {
                self.reversal_detected = true;
            }
        }
    }

    fn update_extended_target(&mut self, price: f64, entry: f64) {
        // Бесконечное оттягивание: цель всегда выше текущей цены на take_profit_percent
        // Но никогда не уменьшается
        let current_target = entry * (1.0 + self.take_profit_percent / 100.0);
        let price_based_target = price * (1.0 + self.take_profit_percent / 100.0);
        
        // Используем максимум из обоих
        if let Some(existing) = self.highest_price {
            self.highest_price = Some(existing.max(price_based_target));
        } else {
            self.highest_price = Some(current_target.max(price_based_target));
        }
    }

    pub fn should_enter(&self) -> bool {
        if self.current_position.is_some() {
            return false;
        }

        if self.price_history.len() < self.channel_window {
            return false;
        }

        let current_price = *self.price_history.back().unwrap();
        let channel_min = self.channel_min().unwrap_or(current_price);
        
        // Вход в нижней части канала
        current_price <= channel_min * (1.0 + self.channel_size / 2.0)
    }

    pub fn should_exit(&self) -> bool {
        if self.current_position.is_none() {
            return false;
        }

        let current_price = *self.price_history.back().unwrap();
        let entry = self.entry_price.unwrap();

        match self.variant {
            StrategyVariant::TrailingStop => {
                // Выход по trailing stop
                if let Some(high) = self.highest_price {
                    let trailing_stop = high * (1.0 - self.stop_loss_percent / 100.0);
                    current_price <= trailing_stop
                } else {
                    // Базовый стоп-лосс
                    current_price <= entry * (1.0 - self.stop_loss_percent / 100.0)
                }
            }
            StrategyVariant::EarlyExit => {
                // Выход при развороте ИЛИ достижении цели
                if self.reversal_detected {
                    return true;
                }
                // Или достигли цели
                current_price >= entry * (1.0 + self.take_profit_percent / 100.0)
            }
            StrategyVariant::ExtendedTarget => {
                // Выход только по стоп-лоссу ИЛИ по расширенной цели
                let stop_loss = entry * (1.0 - self.stop_loss_percent / 100.0);
                if current_price <= stop_loss {
                    return true;
                }
                if let Some(target) = self.highest_price {
                    current_price >= target
                } else {
                    false
                }
            }
        }
    }

    fn channel_min(&self) -> Option<f64> {
        if self.price_history.len() < self.channel_window {
            return None;
        }
        let window: Vec<f64> = self.price_history.iter()
            .rev()
            .take(self.channel_window)
            .copied()
            .collect();
        window.iter().fold(f64::INFINITY, |a, &b| a.min(b)).into()
    }

    fn channel_max(&self) -> Option<f64> {
        if self.price_history.len() < self.channel_window {
            return None;
        }
        let window: Vec<f64> = self.price_history.iter()
            .rev()
            .take(self.channel_window)
            .copied()
            .collect();
        window.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)).into()
    }

    pub fn create_entry_intent(&mut self, symbol: &str, size: f64) -> QuoteIntent {
        let price = *self.price_history.back().unwrap();
        self.entry_price = Some(price);
        self.highest_price = Some(price);
        self.lowest_price = Some(price);
        
        QuoteIntent::new(
            Venue::Gate,
            symbol.to_string(),
            Side::Bid,
            price,
            size,
            TimeInForce::Gtc,
            ClientOrderId::new(format!("entry-{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs())),
        )
    }

    pub fn create_exit_intent(&mut self, symbol: &str, size: f64) -> QuoteIntent {
        let price = *self.price_history.back().unwrap();
        
        // Сбрасываем состояние
        self.current_position = None;
        self.entry_price = None;
        self.highest_price = None;
        self.reversal_detected = false;
        
        QuoteIntent::new(
            Venue::Gate,
            symbol.to_string(),
            Side::Ask,
            price,
            size,
            TimeInForce::Ioc,
            ClientOrderId::new(format!("exit-{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs())),
        )
    }

    pub fn set_position(&mut self, position: Position) {
        let entry_price = position.entry_price;
        self.current_position = Some(position);
        self.entry_price = Some(entry_price);
        self.highest_price = Some(entry_price);
        self.lowest_price = Some(entry_price);
    }

    pub fn variant(&self) -> StrategyVariant {
        self.variant
    }
}

