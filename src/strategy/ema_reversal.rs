//! EMA Reversal Strategy - Стратегия пользователя
//! Ловит развороты на просадках с тройными ордерами
//! 
//! Логика:
//! 1. Мониторинг резких проливов (просадок)
//! 2. Покупка на развороте на уровнях 0.3%, 0.5%, 0.8%
//! 3. Чем больше просадка, тем больше время ожидания разворота
//! 4. После покупки - короткий стоп, трейлинг стоп
//! 5. Продажа по целям с отпуском цены и подтягиванием стопа в безубыток
//! 6. Три ордера (как в channel_split стратегии)

// Простой trait для сброса состояния стратегии
pub trait StrategyReset {
    fn reset_strategy(&mut self);
}

#[derive(Debug, Clone)]
pub struct EmaReversalStrategy {
    // Уровни просадки для входа
    dip_levels: Vec<f64>,  // [0.3%, 0.5%, 0.8%]
    
    // Время ожидания разворота (секунды) для каждого уровня
    reversal_wait_times: Vec<u64>,  // [30, 60, 120]
    
    // EMA параметры
    ema_fast_period: usize,  // Быстрая EMA
    ema_slow_period: usize,  // Медленная EMA
    
    // Стоп-лосс и тейк-профит
    initial_stop_loss_pct: f64,  // Короткий стоп после входа (например, 0.5%)
    trailing_stop_pct: f64,      // Трейлинг стоп (например, 1.0%)
    take_profit_targets: Vec<f64>, // Цели для тейк-профита (например, [1.0%, 2.0%, 3.0%])
    
    // Управление позицией
    min_reversal_confirmation: f64,  // Минимальный % разворота для входа (например, 0.1%)
    price_release_threshold: f64,    // Порог "отпускания" цены перед подтягиванием стопа (например, 0.5%)
    
    // Состояние
    price_history: Vec<f64>,
    ema_fast: Vec<f64>,
    ema_slow: Vec<f64>,
    orders: Vec<OrderPart>,  // Три ордера
    entry_price: Option<f64>,
    highest_price: Option<f64>,
    current_stop_loss: Option<f64>,
    reversal_wait_start: Option<(u64, f64)>, // (timestamp, dip_level)
}

#[derive(Debug, Clone)]
pub struct OrderPart {
    pub entry_price: f64,
    pub size_pct: f64,      // Процент от общей позиции (33.33% для каждого)
    pub stop_loss: f64,
    pub take_profit: Option<f64>,
    pub filled: bool,
}

#[derive(Debug, Clone)]
pub enum EmaReversalSignal {
    WaitForReversal { dip_level: f64, wait_time: u64 },
    EnterLong { price: f64, orders: Vec<OrderPart> },
    UpdateStopLoss { new_stop: f64 },
    PartialExit { price: f64, target: f64 },
    ExitAll { price: f64, reason: String },
    Hold,
}

impl EmaReversalStrategy {
    pub fn new(
        dip_levels: Vec<f64>,
        reversal_wait_times: Vec<u64>,
        ema_fast_period: usize,
        ema_slow_period: usize,
        initial_stop_loss_pct: f64,
        trailing_stop_pct: f64,
        take_profit_targets: Vec<f64>,
        min_reversal_confirmation: f64,
        price_release_threshold: f64,
    ) -> Self {
        assert_eq!(dip_levels.len(), reversal_wait_times.len(), "Dip levels and wait times must match");
        
        Self {
            dip_levels,
            reversal_wait_times,
            ema_fast_period,
            ema_slow_period,
            initial_stop_loss_pct,
            trailing_stop_pct,
            take_profit_targets,
            min_reversal_confirmation,
            price_release_threshold,
            price_history: Vec::new(),
            ema_fast: Vec::new(),
            ema_slow: Vec::new(),
            orders: Vec::new(),
            entry_price: None,
            highest_price: None,
            current_stop_loss: None,
            reversal_wait_start: None,
        }
    }

    pub fn default() -> Self {
        Self::new(
            vec![0.3, 0.5, 0.8],  // Просадки 0.3%, 0.5%, 0.8%
            vec![30, 60, 120],     // Время ожидания 30, 60, 120 секунд
            12,                    // Быстрая EMA 12
            26,                    // Медленная EMA 26
            0.5,                   // Начальный стоп 0.5%
            1.0,                   // Трейлинг стоп 1.0%
            vec![1.0, 2.0, 3.0],   // Цели: 1%, 2%, 3%
            0.1,                   // Минимальное подтверждение разворота 0.1%
            0.5,                   // Порог отпускания цены 0.5%
        )
    }

    fn calculate_ema(&mut self, period: usize) -> Option<f64> {
        if self.price_history.len() < period {
            return None;
        }

        let prices = &self.price_history[self.price_history.len() - period..];
        let multiplier = 2.0 / (period as f64 + 1.0);
        
        let mut ema = prices[0];
        for price in prices.iter().skip(1) {
            ema = (price * multiplier) + (ema * (1.0 - multiplier));
        }
        
        Some(ema)
    }

    fn detect_dip(&self, current_price: f64) -> Option<(usize, f64)> {
        if self.price_history.len() < 10 {
            return None;
        }

        // Находим локальный максимум за последние 10 свечей
        let recent_prices = &self.price_history[self.price_history.len().saturating_sub(10)..];
        let local_high = recent_prices.iter().fold(0.0f64, |a, &b| a.max(b));

        if local_high <= 0.0 {
            return None;
        }

        let dip_pct = ((local_high - current_price) / local_high) * 100.0;

        // Находим ближайший уровень просадки
        for (i, &level) in self.dip_levels.iter().enumerate() {
            if dip_pct >= level * 0.9 && dip_pct <= level * 1.1 {
                return Some((i, dip_pct));
            }
        }

        None
    }

    fn check_reversal(&self, current_price: f64, dip_level: f64) -> bool {
        if self.price_history.len() < 3 {
            return false;
        }

        // Проверяем разворот: цена должна начать расти
        let last_prices = &self.price_history[self.price_history.len().saturating_sub(3)..];
        let was_falling = last_prices.windows(2).all(|w| w[0] > w[1]);
        
        if !was_falling {
            return false;
        }

        // Текущая цена должна быть выше предыдущей
        let price_increase = ((current_price - last_prices.last().unwrap()) / last_prices.last().unwrap()) * 100.0;
        
        price_increase >= self.min_reversal_confirmation
    }

    pub fn update(&mut self, price: f64, timestamp: u64) -> EmaReversalSignal {
        self.price_history.push(price);

        // Ограничиваем размер истории
        if self.price_history.len() > 1000 {
            self.price_history.remove(0);
        }

        // Вычисляем EMA
        if let Some(ema_fast_val) = self.calculate_ema(self.ema_fast_period) {
            self.ema_fast.push(ema_fast_val);
            if self.ema_fast.len() > 100 {
                self.ema_fast.remove(0);
            }
        }

        if let Some(ema_slow_val) = self.calculate_ema(self.ema_slow_period) {
            self.ema_slow.push(ema_slow_val);
            if self.ema_slow.len() > 100 {
                self.ema_slow.remove(0);
            }
        }

        // Если есть позиция - управляем стопами и тейк-профитами
        if let Some(entry) = self.entry_price {
            return self.manage_position(price, timestamp);
        }

        // Нет позиции - ищем возможность входа
        self.manage_entry(price, timestamp)
    }

    fn manage_entry(&mut self, price: f64, timestamp: u64) -> EmaReversalSignal {
        // Проверяем просадку
        if let Some((dip_idx, dip_pct)) = self.detect_dip(price) {
            let wait_time = self.reversal_wait_times[dip_idx];

            // Если еще не начали ждать - начинаем
            if self.reversal_wait_start.is_none() {
                self.reversal_wait_start = Some((timestamp, dip_pct));
                return EmaReversalSignal::WaitForReversal {
                    dip_level: self.dip_levels[dip_idx],
                    wait_time,
                };
            }

            // Проверяем, прошло ли достаточно времени
            if let Some((wait_start, _)) = self.reversal_wait_start {
                let elapsed = timestamp.saturating_sub(wait_start);
                
                if elapsed >= wait_time {
                    // Время вышло - проверяем разворот
                    if self.check_reversal(price, dip_pct) {
                        // ВХОД!
                        return self.create_entry_orders(price);
                    } else {
                        // Разворота нет - сбрасываем
                        self.reversal_wait_start = None;
                    }
                }
            }
        } else {
            // Нет просадки - сбрасываем ожидание
            self.reversal_wait_start = None;
        }

        EmaReversalSignal::Hold
    }

    fn create_entry_orders(&mut self, price: f64) -> EmaReversalSignal {
        let size_per_order = 100.0 / 3.0; // 33.33% на каждый ордер

        let orders: Vec<OrderPart> = self.dip_levels.iter().enumerate().map(|(i, &dip_level)| {
            // Цена входа каждого ордера немного отличается (лестница)
            let entry_price = price * (1.0 - (dip_level / 100.0) * 0.5);
            let stop_loss = entry_price * (1.0 - self.initial_stop_loss_pct / 100.0);
            let take_profit = if i < self.take_profit_targets.len() {
                Some(entry_price * (1.0 + self.take_profit_targets[i] / 100.0))
            } else {
                None
            };

            OrderPart {
                entry_price,
                size_pct: size_per_order,
                stop_loss,
                take_profit,
                filled: false,
            }
        }).collect();

        self.orders = orders.clone();
        self.entry_price = Some(price);
        self.highest_price = Some(price);
        self.current_stop_loss = Some(price * (1.0 - self.initial_stop_loss_pct / 100.0));
        self.reversal_wait_start = None;

        EmaReversalSignal::EnterLong { price, orders }
    }

    fn manage_position(&mut self, price: f64, _timestamp: u64) -> EmaReversalSignal {
        let entry = self.entry_price.unwrap();

        // Обновляем максимальную цену
        if price > self.highest_price.unwrap_or(price) {
            self.highest_price = Some(price);
        }

        let highest = self.highest_price.unwrap();

        // Проверяем стоп-лосс
        if let Some(stop) = self.current_stop_loss {
            if price <= stop {
                self.reset();
                return EmaReversalSignal::ExitAll {
                    price,
                    reason: "Stop loss hit".to_string(),
                };
            }
        }

        // Проверяем тейк-профиты для частичных выходов
        for (i, order) in self.orders.iter().enumerate() {
            if !order.filled {
                if let Some(tp) = order.take_profit {
                    if price >= tp {
                        // Частичный выход
                        let pct = ((price - entry) / entry) * 100.0;
                        self.orders[i].filled = true;
                        return EmaReversalSignal::PartialExit {
                            price,
                            target: self.take_profit_targets.get(i).copied().unwrap_or(1.0),
                        };
                    }
                }
            }
        }

        // Обновляем трейлинг стоп
        let profit_pct = ((highest - entry) / entry) * 100.0;
        
        // Активируем трейлинг только если прибыль больше порога "отпускания"
        if profit_pct >= self.price_release_threshold {
            let new_stop = highest * (1.0 - self.trailing_stop_pct / 100.0);
            
            if let Some(current_stop) = self.current_stop_loss {
                // Подтягиваем стоп только вверх (в безубыток или выше)
                if new_stop > current_stop {
                    // Убеждаемся что стоп не ниже цены входа (безубыток)
                    let break_even_stop = entry * 1.001; // +0.1% над входом
                    let final_stop = new_stop.max(break_even_stop as f64);
                    
                    self.current_stop_loss = Some(final_stop);
                    return EmaReversalSignal::UpdateStopLoss { new_stop: final_stop };
                }
            } else {
                self.current_stop_loss = Some(new_stop);
            }
        }

        EmaReversalSignal::Hold
    }

    pub fn reset(&mut self) {
        self.price_history.clear();
        self.ema_fast.clear();
        self.ema_slow.clear();
        self.orders.clear();
        self.entry_price = None;
        self.highest_price = None;
        self.current_stop_loss = None;
        self.reversal_wait_start = None;
    }

    pub fn get_orders(&self) -> &[OrderPart] {
        &self.orders
    }
}

impl StrategyReset for EmaReversalStrategy {
    fn reset_strategy(&mut self) {
        self.reset();
    }
}

