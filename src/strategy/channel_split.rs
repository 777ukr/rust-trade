//! Улучшенная канальная стратегия с дроблением ордеров на 3 части
//! Ваша стратегия: вход в нижней части канала, выход при прорыве

use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct ChannelSplitStrategy {
    channel_window: usize,
    channel_size: f64,          // Ширина канала в %
    stop_loss_percent: f64,
    take_profit_percent: f64,
    price_history: VecDeque<f64>,
    current_position: Option<ChannelPosition>,
    order_split_count: usize,    // На сколько частей дробить ордер (3)
    split_prices: Vec<f64>,      // Цены для каждой части ордера
}

#[derive(Debug, Clone)]
pub struct ChannelPosition {
    entry_time: u64,
    entry_prices: Vec<f64>,       // Цены входа для каждой части (3 части)
    sizes: Vec<f64>,              // Размеры каждой части
    filled_parts: usize,          // Сколько частей уже заполнено
    total_size: f64,
    channel_min: f64,
    channel_max: f64,
}

impl ChannelSplitStrategy {
    pub fn new(
        channel_window: usize,
        channel_size: f64,
        stop_loss: f64,
        take_profit: f64,
        order_split_count: usize,
    ) -> Self {
        Self {
            channel_window,
            channel_size,
            stop_loss_percent: stop_loss,
            take_profit_percent: take_profit,
            price_history: VecDeque::with_capacity(channel_window * 2),
            current_position: None,
            order_split_count,
            split_prices: Vec::new(),
        }
    }

    /// Обновить цену и получить сигналы
    pub fn update(&mut self, timestamp: u64, price: f64, balance: f64) -> ChannelSplitSignal {
        self.price_history.push_back(price);
        if self.price_history.len() > self.channel_window * 2 {
            self.price_history.pop_front();
        }

        if self.price_history.len() < self.channel_window {
            return ChannelSplitSignal::Wait;
        }

        // Вычисляем канал
        let (channel_min, channel_max) = self.calculate_channel();

        if let Some(ref mut _position) = self.current_position {
            // Управление открытой позицией
            return self.manage_position(timestamp, price, balance, channel_min, channel_max);
        }

        // Проверка входа
        let entry_threshold = channel_min * (1.0 + self.channel_size / 200.0); // Вход в нижней части
        
        if price <= entry_threshold {
            // Дробим ордер на 3 части
            let total_size = balance * 0.3; // 30% от баланса для позиции
            let size_per_part = total_size / self.order_split_count as f64;
            
            // Распределяем цены входа: немного разные цены для каждой части
            let price_step = entry_threshold * 0.001; // 0.1% разница между частями
            let mut split_prices = Vec::new();
            let mut sizes = Vec::new();
            
            for i in 0..self.order_split_count {
                // Первая часть по лучшей цене, последняя немного выше
                let entry_price = entry_threshold + (i as f64 * price_step);
                split_prices.push(entry_price);
                sizes.push(size_per_part);
            }

            self.current_position = Some(ChannelPosition {
                entry_time: timestamp,
                entry_prices: split_prices.clone(),
                sizes: sizes.clone(),
                filled_parts: 0,
                total_size,
                channel_min,
                channel_max,
            });

            return ChannelSplitSignal::EnterSplit {
                parts: split_prices.iter().zip(sizes.iter())
                    .map(|(p, s)| OrderPart { price: *p, size: *s })
                    .collect(),
            };
        }

        ChannelSplitSignal::Wait
    }

    fn manage_position(
        &mut self,
        _timestamp: u64,
        price: f64,
        _balance: f64,
        channel_min: f64,
        channel_max: f64,
    ) -> ChannelSplitSignal {
        if let Some(position) = &mut self.current_position {
            // Обновляем канал
            position.channel_min = channel_min;
            position.channel_max = channel_max;

            // Вычисляем avg_entry - клонируем данные для расчета
            let entry_prices = position.entry_prices.clone();
            let sizes = position.sizes.clone();
            let avg_entry = calculate_avg_entry_internal(&entry_prices, &sizes);

            // Проверяем условия выхода
            // 1. Прорыв канала вверх - выход
            if price >= channel_max * (1.0 - self.channel_size / 200.0) {
                let signal = ChannelSplitSignal::Exit {
                    price,
                    reason: "channel_breakout".to_string(),
                    avg_entry_price: avg_entry,
                };
                self.current_position = None;
                return signal;
            }

            // 2. Стоп-лосс
            let stop_price = avg_entry * (1.0 - self.stop_loss_percent / 100.0);
            if price <= stop_price {
                let signal = ChannelSplitSignal::Exit {
                    price,
                    reason: "stop_loss".to_string(),
                    avg_entry_price: avg_entry,
                };
                self.current_position = None;
                return signal;
            }

            // 3. Тейк-профит
            let take_price = avg_entry * (1.0 + self.take_profit_percent / 100.0);
            if price >= take_price {
                let signal = ChannelSplitSignal::Exit {
                    price,
                    reason: "take_profit".to_string(),
                    avg_entry_price: avg_entry,
                };
                self.current_position = None;
                return signal;
            }
        }

        ChannelSplitSignal::Hold
    }

    fn calculate_channel(&self) -> (f64, f64) {
        let window: Vec<f64> = self.price_history
            .iter()
            .rev()
            .take(self.channel_window)
            .copied()
            .collect();

        let min = window.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = window.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        // Добавляем размер канала
        let channel_size = (max - min) * self.channel_size / 100.0;
        (min - channel_size / 2.0, max + channel_size / 2.0)
    }

    fn calculate_avg_entry(&self, position: &ChannelPosition) -> f64 {
        calculate_avg_entry_internal(&position.entry_prices, &position.sizes)
    }

    pub fn reset(&mut self) {
        self.current_position = None;
        self.price_history.clear();
        self.split_prices.clear();
    }
}

fn calculate_avg_entry_internal(entry_prices: &[f64], sizes: &[f64]) -> f64 {
    let total_value: f64 = entry_prices.iter()
        .zip(sizes.iter())
        .map(|(p, s)| p * s)
        .sum();
    let total_size: f64 = sizes.iter().sum();
    
    if total_size > 0.0 {
        total_value / total_size
    } else {
        0.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderPart {
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChannelSplitSignal {
    Wait,
    Hold,
    EnterSplit {
        parts: Vec<OrderPart>,
    },
    Exit {
        price: f64,
        reason: String,
        avg_entry_price: f64,
    },
}

