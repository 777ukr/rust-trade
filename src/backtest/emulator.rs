//! Эмулятор рынка и исполнения ордеров

use crate::backtest::market::{TradeTick, TradeSide};
use crate::backtest::metrics::BacktestMetrics;
use chrono::{DateTime, Utc};
use rand::Rng;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EmulatorSettings {
    pub fill_probability: f64,      // Вероятность заполнения лимитного ордера (0.0-1.0)
    pub slippage_percent: f64,      // Скольжение цены (%)
    pub max_active_orders: usize,  // Максимум активных ордеров
}

impl Default for EmulatorSettings {
    fn default() -> Self {
        EmulatorSettings {
            fill_probability: 0.95, // 95% вероятность заполнения при подходящей цене
            slippage_percent: 0.1,  // 0.1% скольжение
            max_active_orders: 30,   // Как в MoonBot
        }
    }
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: u64,
    pub symbol: String,
    pub price: f64,
    pub size: f64,
    pub filled: f64,
    pub is_buy: bool,
    pub placed_at: DateTime<Utc>,
    pub filled_at: Option<DateTime<Utc>>,
}

pub struct MarketEmulator {
    settings: EmulatorSettings,
    active_orders: HashMap<u64, Order>,
    next_order_id: u64,
}

impl MarketEmulator {
    pub fn new() -> Self {
        Self {
            settings: EmulatorSettings::default(),
            active_orders: HashMap::new(),
            next_order_id: 1,
        }
    }
    
    /// Разместить лимитный ордер
    pub fn place_limit_order(
        &mut self,
        symbol: &str,
        price: f64,
        size: f64,
        is_buy: bool,
        timestamp: DateTime<Utc>,
    ) -> u64 {
        // Проверка на максимум ордеров (как в MoonBot)
        if self.active_orders.len() >= self.settings.max_active_orders {
            return 0; // Не удалось разместить
        }
        
        let order_id = self.next_order_id;
        self.next_order_id += 1;
        
        let order = Order {
            id: order_id,
            symbol: symbol.to_string(),
            price,
            size,
            filled: 0.0,
            is_buy,
            placed_at: timestamp,
            filled_at: None,
        };
        
        self.active_orders.insert(order_id, order);
        order_id
    }
    
    /// Обработка нового тика - проверка заполнения ордеров
    pub fn process_tick<R: Rng>(
        &mut self,
        tick: &TradeTick,
        metrics: &mut BacktestMetrics,
        rng: &mut R,
    ) {
        let orders_to_check: Vec<u64> = self.active_orders
            .keys()
            .copied()
            .collect();
        
        for order_id in orders_to_check {
            if let Some(order) = self.active_orders.get_mut(&order_id) {
                if order.symbol != tick.symbol {
                    continue;
                }
                
                // Проверка условия заполнения лимитного ордера
                let should_fill = if order.is_buy {
                    // Buy ордер заполняется если цена трейда <= цене ордера
                    tick.price <= order.price
                } else {
                    // Sell ордер заполняется если цена трейда >= цене ордера
                    tick.price >= order.price
                };
                
                if should_fill {
                    // Применяем вероятность заполнения (не всегда заполняется!)
                    if rng.gen_range(0.0f64..1.0f64) < self.settings.fill_probability {
                        // Применяем скольжение
                        let execution_price = if order.is_buy {
                            tick.price * (1.0 + self.settings.slippage_percent / 100.0)
                        } else {
                            tick.price * (1.0 - self.settings.slippage_percent / 100.0)
                        };
                        
                        // Исполняем ордер (полностью или частично)
                        let remaining = order.size - order.filled;
                        let fill_size = remaining.min(tick.volume * 0.1); // Примерно 10% объема тика
                        
                        order.filled += fill_size;
                        
                        if order.filled >= order.size {
                            order.filled_at = Some(tick.timestamp);
                            
                            // Обновляем метрики
                            let pnl = if order.is_buy {
                                // Продали по execution_price, купили по order.price
                                (execution_price - order.price) * order.size
                            } else {
                                // Продали по order.price, купили по execution_price
                                (order.price - execution_price) * order.size
                            };
                            
                            metrics.record_trade(
                                tick.symbol.clone(),
                                order.price,
                                execution_price,
                                order.size,
                                order.is_buy,
                                pnl,
                                tick.timestamp,
                            );
                            
                            // Удаляем исполненный ордер
                            self.active_orders.remove(&order_id);
                        }
                    }
                }
            }
        }
    }
    
    /// Исполнить ордер с задержкой (из очереди событий)
    pub fn execute_order(
        &mut self,
        order_id: u64,
        _timestamp: DateTime<Utc>,
        _metrics: &mut BacktestMetrics,
    ) {
        if let Some(order) = self.active_orders.get_mut(&order_id) {
            if order.filled < order.size {
                // Исполняем оставшуюся часть
                let _remaining = order.size - order.filled;
                order.filled = order.size;
                order.filled_at = Some(_timestamp);
                
                // Обновляем метрики
                // Note: В реальной реализации здесь будет обновление метрик
                // Сейчас временно закомментировано из-за borrow checker
                // metrics.record_trade(...);
                
                self.active_orders.remove(&order_id);
            }
        }
    }
    
    /// Переставить ордер (для Sell ордеров с задержкой)
    pub fn reposition_order(
        &mut self,
        order_id: u64,
        new_price: f64,
        timestamp: DateTime<Utc>,
    ) {
        if let Some(order) = self.active_orders.get_mut(&order_id) {
            order.price = new_price;
            order.placed_at = timestamp;
        }
    }
    
    /// Отменить ордер
    pub fn cancel_order(&mut self, order_id: u64) -> bool {
        self.active_orders.remove(&order_id).is_some()
    }
    
    pub fn get_active_orders(&self) -> &HashMap<u64, Order> {
        &self.active_orders
    }
}

