//! High Frequency Trading (HFT) стратегия
//! Быстрые скальпинг сделки на микро-движениях цены
//! Низкий спред, быстрый вход/выход

use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct HFTStrategy {
    entry_threshold: f64,      // Порог входа (например, 0.01% изменения)
    exit_threshold: f64,       // Порог выхода (например, 0.02% прибыль)
    max_hold_time: u64,        // Максимальное время удержания (секунды)
    min_volume: f64,           // Минимальный объем для входа
    order_size_percent: f64,   // Размер ордера от баланса
    price_history: VecDeque<(u64, f64)>, // (timestamp, price)
    order_book_imbalance: VecDeque<f64>, // Дисбаланс ордербука
}

impl HFTStrategy {
    pub fn new(
        entry_threshold: f64,
        exit_threshold: f64,
        max_hold_time: u64,
        order_size_percent: f64,
    ) -> Self {
        Self {
            entry_threshold,
            exit_threshold,
            max_hold_time,
            min_volume: 0.0,
            order_size_percent,
            price_history: VecDeque::with_capacity(100),
            order_book_imbalance: VecDeque::with_capacity(10),
        }
    }

    /// Обновить цену и получить сигнал
    pub fn update(
        &mut self,
        timestamp: u64,
        price: f64,
        bid_volume: f64,
        ask_volume: f64,
        balance: f64,
    ) -> HFTSignal {
        self.price_history.push_back((timestamp, price));
        if self.price_history.len() > 100 {
            self.price_history.pop_front();
        }

        // Рассчитываем дисбаланс ордербука
        let imbalance = if bid_volume + ask_volume > 0.0 {
            (bid_volume - ask_volume) / (bid_volume + ask_volume)
        } else {
            0.0
        };
        
        self.order_book_imbalance.push_back(imbalance);
        if self.order_book_imbalance.len() > 10 {
            self.order_book_imbalance.pop_front();
        }

        if self.price_history.len() < 5 {
            return HFTSignal::Wait;
        }

        // Анализ микро-тренда
        let micro_trend = self.detect_micro_trend();
        
        // Анализ импульса
        let _momentum = self.calculate_momentum();

        // Сигнал на основе дисбаланса и тренда
        let signal_strength = micro_trend * 0.6 + imbalance * 0.4;

        if signal_strength.abs() > self.entry_threshold {
            let side = if signal_strength > 0.0 { "buy" } else { "sell" };
            let size = balance * self.order_size_percent / 100.0;

            HFTSignal::Enter {
                side: side.to_string(),
                price,
                size,
                timestamp,
            }
        } else {
            HFTSignal::Wait
        }
    }

    /// Проверить условия выхода для открытой позиции
    pub fn check_exit(
        &self,
        entry_price: f64,
        entry_time: u64,
        current_price: f64,
        current_time: u64,
        side: &str,
    ) -> bool {
        // Проверка времени удержания
        if current_time - entry_time > self.max_hold_time {
            return true; // Время истекло - выход
        }

        // Проверка тейк-профита
        let price_change = if side == "buy" {
            (current_price - entry_price) / entry_price
        } else {
            (entry_price - current_price) / entry_price
        };

        // exit_threshold в процентах (0.02 = 2%), преобразуем в доли
        let threshold_decimal = self.exit_threshold / 100.0;
        if price_change >= threshold_decimal {
            return true; // Тейк-профит достигнут
        }

        // Проверка стоп-лосса (быстрый выход при развороте)
        let stop_threshold = -self.exit_threshold / 100.0 * 0.5; // Половина тейк-профита в обратную сторону
        if price_change <= stop_threshold {
            return true; // Быстрый стоп при развороте
        }

        false
    }

    fn detect_micro_trend(&self) -> f64 {
        if self.price_history.len() < 5 {
            return 0.0;
        }

        let recent: Vec<f64> = self.price_history
            .iter()
            .rev()
            .take(5)
            .map(|(_, p)| *p)
            .collect();

        // Простая линейная регрессия для тренда
        let n = recent.len() as f64;
        let sum_x: f64 = (0..recent.len()).map(|i| i as f64).sum();
        let sum_y: f64 = recent.iter().sum();
        let sum_xy: f64 = recent.iter()
            .enumerate()
            .map(|(i, p)| i as f64 * p)
            .sum();
        let sum_x2: f64 = (0..recent.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        
        // Нормализуем по цене
        let avg_price = sum_y / n;
        if avg_price > 0.0 {
            (slope / avg_price) * 100.0 // Возвращаем в процентах
        } else {
            0.0
        }
    }

    fn calculate_momentum(&self) -> f64 {
        if self.price_history.len() < 3 {
            return 0.0;
        }

        let prices: Vec<f64> = self.price_history
            .iter()
            .rev()
            .take(3)
            .map(|(_, p)| *p)
            .collect();

        let momentum = (prices[0] - prices[2]) / prices[2] * 100.0;
        momentum
    }

    pub fn reset(&mut self) {
        self.price_history.clear();
        self.order_book_imbalance.clear();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HFTSignal {
    Wait,
    Enter {
        side: String,
        price: f64,
        size: f64,
        timestamp: u64,
    },
}

