//! Full Order Book Reconstruction (L2/L3)
//! Поддержка скрытых ордеров, айсбергов, очередей исполнения

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderLevel {
    pub price: f64,
    pub visible_quantity: f64,
    pub hidden_quantity: f64, // Скрытые ордера
    pub iceberg_quantity: f64, // Айсберг ордера
    pub orders: Vec<OrderQueueItem>, // L3: отдельные ордера
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderQueueItem {
    pub order_id: u64,
    pub quantity: f64,
    pub is_hidden: bool,
    pub is_iceberg: bool,
    pub timestamp: i64, // Для FIFO очереди
}

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub symbol: String,
    pub bids: BTreeMap<i64, OrderLevel>, // price * 1e8 как i64 для точности
    pub asks: BTreeMap<i64, OrderLevel>,
    pub best_bid: Option<f64>,
    pub best_ask: Option<f64>,
}

impl OrderBook {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            best_bid: None,
            best_ask: None,
        }
    }
    
    /// Конвертация цены в ключ (price * 1e8)
    fn price_to_key(price: f64) -> i64 {
        (price * 1_000_000_000.0) as i64
    }
    
    /// Конвертация ключа обратно в цену
    fn key_to_price(key: i64) -> f64 {
        key as f64 / 1_000_000_000.0
    }
    
    /// Обновление уровня в стакане (L2)
    pub fn update_level(&mut self, price: f64, quantity: f64, is_bid: bool) {
        let key = Self::price_to_key(price);
        
        let levels = if is_bid {
            &mut self.bids
        } else {
            &mut self.asks
        };
        
        if quantity > 0.0 {
            levels.entry(key).or_insert_with(|| OrderLevel {
                price,
                visible_quantity: 0.0,
                hidden_quantity: 0.0,
                iceberg_quantity: 0.0,
                orders: Vec::new(),
            }).visible_quantity = quantity;
        } else {
            levels.remove(&key);
        }
        
        // Обновляем лучшие цены
        self.update_best_prices();
    }
    
    /// Добавление ордера в очередь (L3)
    pub fn add_order_to_queue(
        &mut self,
        price: f64,
        quantity: f64,
        order_id: u64,
        is_bid: bool,
        is_hidden: bool,
        is_iceberg: bool,
        timestamp: i64,
    ) {
        let key = Self::price_to_key(price);
        
        let levels = if is_bid {
            &mut self.bids
        } else {
            &mut self.asks
        };
        
        let level = levels.entry(key).or_insert_with(|| OrderLevel {
            price,
            visible_quantity: 0.0,
            hidden_quantity: 0.0,
            iceberg_quantity: 0.0,
            orders: Vec::new(),
        });
        
        level.orders.push(OrderQueueItem {
            order_id,
            quantity,
            is_hidden,
            is_iceberg,
            timestamp,
        });
        
        // Обновляем видимые/скрытые количества
        if is_hidden {
            level.hidden_quantity += quantity;
        } else {
            level.visible_quantity += quantity;
        }
        
        if is_iceberg {
            level.iceberg_quantity += quantity;
        }
        
        self.update_best_prices();
    }
    
    /// Исполнение ордера с учетом позиции в очереди
    pub fn fill_order(
        &mut self,
        price: f64,
        quantity: f64,
        is_bid: bool,
        fill_model: FillModel,
    ) -> Vec<FilledOrder> {
        let mut filled_orders = Vec::new();
        let mut remaining = quantity;
        
        let levels = if is_bid {
            // Исполняем по ASK (покупаем)
            &mut self.asks
        } else {
            // Исполняем по BID (продаем)
            &mut self.bids
        };
        
        let price_key = Self::price_to_key(price);
        
        // Находим все уровни, которые должны исполниться
        // Сначала собираем ключи (immutable borrow)
        let keys_vec: Vec<i64> = levels.keys().copied().collect();
        let keys_to_process: Vec<i64> = if is_bid {
            // Покупаем - берем ASK от самой низкой цены до price
            keys_vec.into_iter()
                .filter(|&k| k <= price_key)
                .collect()
        } else {
            // Продаем - берем BID от самой высокой цены до price
            let mut filtered: Vec<i64> = keys_vec.into_iter()
                .filter(|&k| k >= price_key)
                .collect();
            filtered.sort_by(|a, b| b.cmp(a)); // Сортировка по убыванию
            filtered
        };
        
        for key in keys_to_process {
            if remaining <= 0.0 {
                break;
            }
            
            if let Some(level) = levels.get_mut(&key) {
                let filled = match fill_model {
                    FillModel::FIFO => OrderBook::fill_fifo_static(level, remaining, price),
                    FillModel::ProRata => OrderBook::fill_prorata_static(level, remaining, price),
                    FillModel::TimePriority => OrderBook::fill_time_priority_static(level, remaining, price),
                };
                
                filled_orders.extend(filled.0);
                remaining -= filled.1;
                
                // Удаляем уровень если весь исполнен
                if level.visible_quantity <= 0.0 && level.hidden_quantity <= 0.0 {
                    levels.remove(&key);
                }
            }
        }
        
        self.update_best_prices();
        filled_orders
    }
    
    fn fill_fifo_static(
        level: &mut OrderLevel,
        max_quantity: f64,
        execution_price: f64,
    ) -> (Vec<FilledOrder>, f64) {
        let mut filled = Vec::new();
        let mut remaining = max_quantity;
        
        // Сортируем по времени (FIFO)
        level.orders.sort_by_key(|o| o.timestamp);
        
        let mut to_remove = Vec::new();
        for (idx, order) in level.orders.iter_mut().enumerate() {
            if remaining <= 0.0 {
                break;
            }
            
            let fill_qty = order.quantity.min(remaining);
            order.quantity -= fill_qty;
            remaining -= fill_qty;
            
            if order.is_hidden {
                level.hidden_quantity -= fill_qty;
            } else {
                level.visible_quantity -= fill_qty;
            }
            
            if order.is_iceberg {
                level.iceberg_quantity -= fill_qty;
            }
            
            filled.push(FilledOrder {
                order_id: order.order_id,
                price: level.price,
                execution_price,
                quantity: fill_qty,
            });
            
            if order.quantity <= 0.0 {
                to_remove.push(idx);
            }
        }
        
        // Удаляем исполненные ордера
        for &idx in to_remove.iter().rev() {
            level.orders.remove(idx);
        }
        
        (filled, max_quantity - remaining)
    }
    
    fn fill_prorata_static(
        level: &mut OrderLevel,
        max_quantity: f64,
        execution_price: f64,
    ) -> (Vec<FilledOrder>, f64) {
        // PRO RATA: распределение пропорционально размеру ордеров
        let total_qty: f64 = level.orders.iter().map(|o| o.quantity).sum();
        if total_qty == 0.0 {
            return (Vec::new(), 0.0);
        }
        
        let mut filled = Vec::new();
        let mut remaining = max_quantity.min(total_qty);
        
        for order in &mut level.orders {
            if remaining <= 0.0 {
                break;
            }
            
            let proportion = order.quantity / total_qty;
            let fill_qty = (max_quantity * proportion).min(order.quantity).min(remaining);
            
            order.quantity -= fill_qty;
            remaining -= fill_qty;
            
            if order.is_hidden {
                level.hidden_quantity -= fill_qty;
            } else {
                level.visible_quantity -= fill_qty;
            }
            
            if order.is_iceberg {
                level.iceberg_quantity -= fill_qty;
            }
            
            filled.push(FilledOrder {
                order_id: order.order_id,
                price: level.price,
                execution_price,
                quantity: fill_qty,
            });
        }
        
        // Удаляем полностью исполненные ордера
        level.orders.retain(|o| o.quantity > 0.0);
        
        (filled, max_quantity - remaining)
    }
    
    fn fill_time_priority_static(
        level: &mut OrderLevel,
        max_quantity: f64,
        execution_price: f64,
    ) -> (Vec<FilledOrder>, f64) {
        // Время имеет приоритет, но размер тоже важен
        // Сортируем: сначала по времени, потом по размеру
        level.orders.sort_by(|a, b| {
            a.timestamp.cmp(&b.timestamp)
                .then_with(|| b.quantity.partial_cmp(&a.quantity).unwrap())
        });
        
        OrderBook::fill_fifo_static(level, max_quantity, execution_price)
    }
    
    fn update_best_prices(&mut self) {
        self.best_bid = self.bids
            .iter()
            .next_back()
            .map(|(k, _)| Self::key_to_price(*k));
        
        self.best_ask = self.asks
            .iter()
            .next()
            .map(|(k, _)| Self::key_to_price(*k));
    }
    
    /// Получить глубину стакана
    pub fn get_depth(&self, levels: usize) -> (Vec<(f64, f64)>, Vec<(f64, f64)>) {
        let mut bids: Vec<(f64, f64)> = self.bids
            .iter()
            .rev()
            .take(levels)
            .map(|(_, level)| (level.price, level.visible_quantity + level.hidden_quantity))
            .collect();
        bids.reverse(); // Чтобы были отсортированы правильно
        
        let asks: Vec<(f64, f64)> = self.asks
            .iter()
            .take(levels)
            .map(|(_, level)| (level.price, level.visible_quantity + level.hidden_quantity))
            .collect();
        
        (bids, asks)
    }
    
    /// Получить спред
    pub fn get_spread(&self) -> Option<f64> {
        if let (Some(bid), Some(ask)) = (self.best_bid, self.best_ask) {
            Some(ask - bid)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillModel {
    FIFO,         // First In First Out
    ProRata,      // Пропорциональное распределение
    TimePriority, // Приоритет времени + размера
}

#[derive(Debug, Clone)]
pub struct FilledOrder {
    pub order_id: u64,
    pub price: f64,
    pub execution_price: f64,
    pub quantity: f64,
}

