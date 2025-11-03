//! MShot Strategy - стратегия ловли прострелов с переставлением ордеров в коридоре
//! 
//! Задача: поймать "прострел" цены вниз и выставить ордер как можно быстрее
//! Ордер переставляется в коридоре между MShotPrice и MShotPriceMin

use crate::strategy::trading_strategy::{Strategy, StrategyContext, Signal};
use chrono::{DateTime, Utc, Duration};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct MShotConfig {
    // Основные параметры
    pub mshot_price: f64,              // Цена ордера в % от текущей (положительная, ниже рынка)
    pub mshot_price_min: f64,          // Минимум цены в % от текущей
    pub mshot_minus_satoshi: bool,      // Не ближе 2 сатоши от ASK
    pub mshot_replace_delay: f64,       // Задержка перестановки после падения (сек, может быть <1)
    pub mshot_raise_wait: f64,          // Задержка перестановки после роста (сек)
    
    // Дельты для корректировки
    pub mshot_add_3h_delta: f64,        // Коэффициент добавления за каждый % 3ч дельты
    pub mshot_add_hourly_delta: f64,    // Коэффициент добавления за каждый % часовой дельты
    pub mshot_add_15min_delta: f64,     // Коэффициент добавления за каждый % 15м дельты
    pub mshot_add_market_delta: f64,    // Коэффициент добавления за каждый % дельты маркета
    pub mshot_add_btc_delta: f64,       // Коэффициент добавления за каждый % дельты BTC
    pub mshot_add_btc_5m_delta: f64,    // Для учета 5-минутной дельты BTC
    
    pub mshot_add_distance: f64,        // Коэффициент расширения дальней границы (%)
    pub mshot_add_price_bug: f64,        // Модификатор в зависимости от PriceBug (рекоменд. 0.2)
    
    // Продажа
    pub mshot_sell_at_last_price: bool,  // Продавать по максимальной из (sell price, last ASK)
    pub mshot_sell_price_adjust: f64,    // Поправка к цене ASK (%)
    
    // Сортировка и фильтры
    pub mshot_sort_by: String,           // LastNhDelta, DVolToHVolAsc, OrderBook, DailyVol, MinuteVol
    pub mshot_sort_desc: bool,          // Сортировка по убыванию
    
    // Использование цены
    pub mshot_use_price: String,         // BID, ASK, Trade
    
    // Повтор
    pub mshot_repeat_after_buy: bool,    // Повторять после покупки
    pub mshot_repeat_if_profit: f64,     // Повторять если прибыль >= %
    pub mshot_repeat_wait: f64,         // Ожидание перед повтором (сек)
    pub mshot_repeat_delay: f64,        // Задержка повтора (сек)
    
    pub fast_shot_algo: bool,           // Ускорение работы (требует +5-10% CPU)
    
    // Размер ордера
    pub order_size: f64,
}

impl Default for MShotConfig {
    fn default() -> Self {
        MShotConfig {
            mshot_price: 10.0,
            mshot_price_min: 7.0,
            mshot_minus_satoshi: false,
            mshot_replace_delay: 0.0,
            mshot_raise_wait: 0.0,
            mshot_add_3h_delta: 0.0,
            mshot_add_hourly_delta: 0.0,
            mshot_add_15min_delta: 0.0,
            mshot_add_market_delta: 0.0,
            mshot_add_btc_delta: 0.0,
            mshot_add_btc_5m_delta: 0.0,
            mshot_add_distance: 0.0,
            mshot_add_price_bug: 0.2,
            mshot_sell_at_last_price: false,
            mshot_sell_price_adjust: 0.0,
            mshot_sort_by: "LastNhDelta".to_string(),
            mshot_sort_desc: true,
            mshot_use_price: "BID".to_string(),
            mshot_repeat_after_buy: false,
            mshot_repeat_if_profit: 0.0,
            mshot_repeat_wait: 0.0,
            mshot_repeat_delay: 0.0,
            fast_shot_algo: false,
            order_size: 100.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MShotStrategy {
    config: MShotConfig,
    
    // Состояние
    active_order_id: Option<u64>,
    current_price: Option<f64>,
    last_ask_price: Option<f64>,
    last_ask_4s_ago: Option<f64>, // Цена ASK 4 секунды назад (для MShotSellAtLastPrice)
    
    // Дельты (обновляются извне)
    delta_3h: f64,
    delta_hourly: f64,
    delta_15min: f64,
    delta_market: f64,
    delta_btc: f64,
    delta_btc_5m: f64,
    
    // Таймеры для задержек
    last_price_drop_time: Option<DateTime<Utc>>,
    last_price_rise_time: Option<DateTime<Utc>>,
    
    // История цен для расчета дельт
    price_history: VecDeque<(DateTime<Utc>, f64)>,
    ask_history: VecDeque<(DateTime<Utc>, f64)>,
    
    // Позиция
    buy_price: Option<f64>,
    position_size: f64,
}

impl MShotStrategy {
    pub fn new(config: MShotConfig) -> Self {
        Self {
            config,
            active_order_id: None,
            current_price: None,
            last_ask_price: None,
            last_ask_4s_ago: None,
            delta_3h: 0.0,
            delta_hourly: 0.0,
            delta_15min: 0.0,
            delta_market: 0.0,
            delta_btc: 0.0,
            delta_btc_5m: 0.0,
            last_price_drop_time: None,
            last_price_rise_time: None,
            price_history: VecDeque::new(),
            ask_history: VecDeque::new(),
            buy_price: None,
            position_size: 0.0,
        }
    }
    
    /// Обновить дельты (вызывается извне)
    pub fn update_deltas(
        &mut self,
        delta_3h: f64,
        delta_hourly: f64,
        delta_15min: f64,
        delta_market: f64,
        delta_btc: f64,
        delta_btc_5m: f64,
    ) {
        self.delta_3h = delta_3h;
        self.delta_hourly = delta_hourly;
        self.delta_15min = delta_15min;
        self.delta_market = delta_market;
        self.delta_btc = delta_btc;
        self.delta_btc_5m = delta_btc_5m;
    }
    
    /// Рассчитать скорректированный MShotPrice с учетом дельт
    fn calculate_adjusted_price(&self) -> (f64, f64) {
        let mut price = self.config.mshot_price;
        let mut price_min = self.config.mshot_price_min;
        
        // Добавляем дельты
        price += self.delta_3h * self.config.mshot_add_3h_delta;
        price_min += self.delta_3h * self.config.mshot_add_3h_delta;
        
        price += self.delta_hourly * self.config.mshot_add_hourly_delta;
        price_min += self.delta_hourly * self.config.mshot_add_hourly_delta;
        
        price += self.delta_15min * self.config.mshot_add_15min_delta;
        price_min += self.delta_15min * self.config.mshot_add_15min_delta;
        
        price += self.delta_market * self.config.mshot_add_market_delta;
        price_min += self.delta_market * self.config.mshot_add_market_delta;
        
        price += self.delta_btc * self.config.mshot_add_btc_delta;
        price_min += self.delta_btc * self.config.mshot_add_btc_delta;
        
        price += self.delta_btc_5m * self.config.mshot_add_btc_5m_delta;
        price_min += self.delta_btc_5m * self.config.mshot_add_btc_5m_delta;
        
        // MShotAddDistance - расширение дальней границы
        if self.config.mshot_add_distance > 0.0 {
            let adjustment = (price_min - self.config.mshot_price_min) * 
                (1.0 + self.config.mshot_add_distance / 100.0);
            price = self.config.mshot_price + adjustment;
        }
        
        (price, price_min)
    }
    
    /// Получить текущую цену для расчета (BID, ASK или Trade)
    fn get_reference_price(&self, bid: f64, ask: f64, trade: f64) -> f64 {
        match self.config.mshot_use_price.as_str() {
            "BID" => bid,
            "ASK" => ask,
            "Trade" => trade,
            _ => trade,
        }
    }
    
    /// Проверить нужно ли переставить ордер
    fn should_replace_order(&self, current_time: DateTime<Utc>, current_price: f64, ask: f64) -> bool {
        if let Some(active_id) = self.active_order_id {
            if let Some(order_price) = self.current_price {
                let (adjusted_price, adjusted_price_min) = self.calculate_adjusted_price();
                let reference_price = self.get_reference_price(current_price, ask, current_price);
                
                // Верхняя граница коридора
                let upper_bound = reference_price * (1.0 - adjusted_price_min / 100.0);
                // Нижняя граница коридора
                let lower_bound = reference_price * (1.0 - adjusted_price / 100.0);
                
                // Если текущая цена подошла ближе к ордеру чем MShotPriceMin
                if order_price > upper_bound {
                    // Нужно переставить ниже
                    if let Some(drop_time) = self.last_price_drop_time {
                        let delay = Duration::milliseconds((self.config.mshot_replace_delay * 1000.0) as i64);
                        if current_time >= drop_time + delay {
                            return true;
                        }
                    } else {
                        self.last_price_drop_time = Some(current_time);
                    }
                } else if order_price < lower_bound {
                    // Нужно переставить выше
                    if let Some(rise_time) = self.last_price_rise_time {
                        let delay = Duration::milliseconds((self.config.mshot_raise_wait * 1000.0) as i64);
                        if current_time >= rise_time + delay {
                            return true;
                        }
                    } else {
                        self.last_price_rise_time = Some(current_time);
                    }
                }
            }
        }
        false
    }
}

impl Strategy for MShotStrategy {
    fn on_tick(
        &mut self,
        timestamp: DateTime<Utc>,
        price: f64,
        bid: f64,
        ask: f64,
        volume: f64,
        ctx: &mut StrategyContext,
    ) -> Signal {
        self.current_price = Some(price);
        self.last_ask_price = Some(ask);
        
        // Сохраняем историю цен для дельт и 4-секундной ASK
        self.price_history.push_back((timestamp, price));
        self.ask_history.push_back((timestamp, ask));
        
        // Обновляем last_ask_4s_ago
        let cutoff = timestamp - Duration::seconds(4);
        while let Some(&(ts, _)) = self.ask_history.front() {
            if ts < cutoff {
                self.ask_history.pop_front();
            } else {
                break;
            }
        }
        if let Some(&(_, ask_4s_ago)) = self.ask_history.front() {
            self.last_ask_4s_ago = Some(ask_4s_ago);
        }
        
        // Очищаем старую историю (оставляем последние 3 часа для дельт)
        let max_age = Duration::hours(3);
        let min_time = timestamp - max_age;
        while let Some(&(ts, _)) = self.price_history.front() {
            if ts < min_time {
                self.price_history.pop_front();
            } else {
                break;
            }
        }
        
        // Если нет активного ордера и нет позиции - выставляем ордер
        if self.active_order_id.is_none() && self.position_size == 0.0 {
            let (adjusted_price, _) = self.calculate_adjusted_price();
            let reference_price = self.get_reference_price(bid, ask, price);
            let buy_price = reference_price * (1.0 - adjusted_price / 100.0);
            
            // MShotMinusSatoshi - не ближе 2 сатоши от ASK
            let final_buy_price = if self.config.mshot_minus_satoshi {
                let min_price = ask * 0.98; // 2% от ask (примерно 2 сатоши для монет < 100 сат)
                buy_price.min(min_price)
            } else {
                buy_price
            };
            
            let order = ctx.place_limit_order(
                final_buy_price,
                self.config.order_size,
                true, // buy
            );
            self.active_order_id = Some(order.id);
            self.current_price = Some(final_buy_price);
        }
        
        // Проверяем нужно ли переставить ордер
        if self.should_replace_order(timestamp, price, ask) {
            if let Some(order_id) = self.active_order_id {
                ctx.cancel_order(order_id);
                
                let (adjusted_price, adjusted_price_min) = self.calculate_adjusted_price();
                let reference_price = self.get_reference_price(bid, ask, price);
                let buy_price = reference_price * (1.0 - adjusted_price / 100.0);
                
                let final_buy_price = if self.config.mshot_minus_satoshi {
                    let min_price = ask * 0.98;
                    buy_price.min(min_price)
                } else {
                    buy_price
                };
                
                let order = ctx.place_limit_order(
                    final_buy_price,
                    self.config.order_size,
                    true,
                );
                self.active_order_id = Some(order.id);
                self.current_price = Some(final_buy_price);
            }
        }
        
        Signal::Hold
    }
    
    fn on_fill(&mut self, order_id: u64, price: f64, size: f64, is_buy: bool) {
        if Some(order_id) == self.active_order_id && is_buy {
            self.buy_price = Some(price);
            self.position_size = size;
            self.active_order_id = None;
        }
    }
    
    fn get_position(&self) -> Option<(f64, f64)> {
        if self.position_size > 0.0 {
            self.buy_price.map(|p| (p, self.position_size))
        } else {
            None
        }
    }
    
    fn calculate_sell_price(&self, current_price: f64, bid: f64, ask: f64) -> Option<f64> {
        if let Some(buy_price) = self.buy_price {
            if self.config.mshot_sell_at_last_price {
                // Максимум из: стратегическая цена продажи и last_ask_4s_ago с поправкой
                let strategy_sell_price = buy_price * 1.01; // Пример, должна быть из конфига
                
                let last_ask_sell_price = if let Some(ask_4s) = self.last_ask_4s_ago {
                    ask_4s * (1.0 - self.config.mshot_sell_price_adjust / 100.0)
                } else {
                    ask * (1.0 - self.config.mshot_sell_price_adjust / 100.0)
                };
                
                Some(strategy_sell_price.max(last_ask_sell_price))
            } else {
                // Обычная логика продажи
                Some(buy_price * 1.01) // Должно быть из конфига SellPrice
            }
        } else {
            None
        }
    }
}

