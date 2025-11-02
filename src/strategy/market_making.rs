//! Market Making стратегия
//! Выставляет bid/ask ордера для получения спреда
//! Подходит для высоколиквидных инструментов

use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct MarketMakingStrategy {
    spread_percent: f64,        // Спред в % (например, 0.1%)
    order_size_percent: f64,    // Размер ордера от баланса (например, 5%)
    max_position_size: f64,     // Максимальный размер позиции
    inventory_target: f64,      // Целевой инвентарь (0 = нейтральный)
    price_history: VecDeque<f64>,
    window_size: usize,
    current_bid: Option<f64>,
    current_ask: Option<f64>,
    position_size: f64,          // Текущая позиция (+ = long, - = short)
}

impl MarketMakingStrategy {
    pub fn new(
        spread_percent: f64,
        order_size_percent: f64,
        max_position_size: f64,
        window_size: usize,
    ) -> Self {
        Self {
            spread_percent,
            order_size_percent,
            max_position_size,
            inventory_target: 0.0,
            price_history: VecDeque::with_capacity(window_size),
            window_size,
            current_bid: None,
            current_ask: None,
            position_size: 0.0,
        }
    }

    /// Обновить цену и получить сигналы
    pub fn update(&mut self, price: f64, balance: f64) -> MarketMakingSignal {
        self.price_history.push_back(price);
        if self.price_history.len() > self.window_size {
            self.price_history.pop_front();
        }

        if self.price_history.len() < 5 {
            return MarketMakingSignal::Wait;
        }

        let mid_price = self.calculate_mid_price();
        
        // Динамический спред на основе волатильности
        let volatility = self.calculate_volatility();
        let dynamic_spread = self.spread_percent * (1.0 + volatility * 0.5);

        // Корректировка для управления инвентарем
        let inventory_skew = self.position_size / self.max_position_size;
        let skew_adjustment = if inventory_skew.abs() > 0.5 {
            // Если позиция слишком большая, расширяем спред на этой стороне
            inventory_skew * 0.002 // 0.2% коррекция
        } else {
            0.0
        };

        let bid_offset = dynamic_spread / 2.0 + skew_adjustment;
        let ask_offset = dynamic_spread / 2.0 - skew_adjustment;

        let new_bid = mid_price * (1.0 - bid_offset / 100.0);
        let new_ask = mid_price * (1.0 + ask_offset / 100.0);

        // Размер ордера
        let order_size = balance * self.order_size_percent / 100.0;

        // Проверяем, нужно ли обновить ордера
        let bid_changed = self.current_bid.map_or(true, |b| (b - new_bid).abs() / b > 0.0001);
        let ask_changed = self.current_ask.map_or(true, |a| (a - new_ask).abs() / a > 0.0001);

        self.current_bid = Some(new_bid);
        self.current_ask = Some(new_ask);

        if bid_changed || ask_changed {
            MarketMakingSignal::UpdateOrders {
                bid: new_bid,
                ask: new_ask,
                bid_size: order_size,
                ask_size: order_size,
            }
        } else {
            MarketMakingSignal::Hold
        }
    }

    /// Обновить размер позиции после исполнения ордера
    pub fn update_position(&mut self, side: &str, size: f64) {
        match side {
            "buy" | "bid_filled" => {
                self.position_size += size;
            }
            "sell" | "ask_filled" => {
                self.position_size -= size;
            }
            _ => {}
        }
    }

    fn calculate_mid_price(&self) -> f64 {
        if self.price_history.is_empty() {
            return 0.0;
        }
        
        // Используем последнюю цену как mid price
        *self.price_history.back().unwrap()
    }

    fn calculate_volatility(&self) -> f64 {
        if self.price_history.len() < 2 {
            return 0.0;
        }

        let returns: Vec<f64> = self.price_history
            .iter()
            .zip(self.price_history.iter().skip(1))
            .map(|(prev, curr)| (curr - prev) / prev)
            .collect();

        if returns.is_empty() {
            return 0.0;
        }

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / returns.len() as f64;

        variance.sqrt() * 100.0 // Возвращаем в процентах
    }

    pub fn reset(&mut self) {
        self.position_size = 0.0;
        self.current_bid = None;
        self.current_ask = None;
        self.price_history.clear();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MarketMakingSignal {
    Wait,
    Hold,
    UpdateOrders {
        bid: f64,
        ask: f64,
        bid_size: f64,
        ask_size: f64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_making_basic() {
        let mut strategy = MarketMakingStrategy::new(
            0.1,  // 0.1% спред
            5.0,  // 5% от баланса
            1000.0,
            20,
        );

        let balance = 1000.0;
        
        // Симулируем несколько обновлений цены
        for i in 0..10 {
            let price = 50000.0 + (i as f64 * 10.0);
            let signal = strategy.update(price, balance);
            
            if i >= 4 {
                // После 5 обновлений должны начаться сигналы
                match signal {
                    MarketMakingSignal::UpdateOrders { bid, ask, .. } => {
                        let spread = (ask - bid) / bid * 100.0;
                        assert!(spread > 0.0);
                        assert!(spread < 1.0); // Спред не должен быть больше 1%
                    }
                    _ => {}
                }
            }
        }
    }
}
