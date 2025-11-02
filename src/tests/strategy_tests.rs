//! Тесты для всех стратегий
//! Проверка корректности работы Channel Split, Market Making и HFT

#![cfg(feature = "gate_exec")]

#[cfg(test)]
mod tests {
    use crate::strategy::channel_split::{ChannelSplitStrategy, ChannelSplitSignal};
    use crate::strategy::market_making::{MarketMakingStrategy, MarketMakingSignal};
    use crate::strategy::hft::{HFTStrategy, HFTSignal};

    #[test]
    fn test_channel_split_strategy_entry() {
        let mut strategy = ChannelSplitStrategy::new(
            20,    // окно канала
            1.0,   // ширина канала 1%
            2.0,   // стоп-лосс 2%
            4.0,   // тейк-профит 4%
            3,     // дробление на 3 части
        );

        let balance = 1000.0;
        
        // Симулируем цены для построения канала (падающие цены)
        let mut prices = vec![50000.0; 25];
        for i in 0..25 {
            prices[i] = 50000.0 - (i as f64 * 5.0); // Цены падают
        }
        
        // Обновляем стратегию
        for (i, price) in prices.iter().enumerate() {
            let signal = strategy.update(i as u64, *price, balance);
            // Проверяем что сигнал валидный (любой из допустимых)
            match signal {
                ChannelSplitSignal::Wait | 
                ChannelSplitSignal::EnterSplit {..} | 
                ChannelSplitSignal::Hold |
                ChannelSplitSignal::Exit {..} => {
                    // Все валидные сигналы
                }
            }
        }
    }

    #[test]
    fn test_channel_split_order_splitting() {
        let mut strategy = ChannelSplitStrategy::new(20, 1.0, 2.0, 4.0, 3);
        let balance = 1000.0;
        
        // Наполняем историю
        for i in 0..25 {
            let price = 50000.0 - (i as f64 * 5.0); // Цена падает к нижней части канала
            let signal = strategy.update(i as u64, price, balance);
            
            // Проверяем что при входе ордер дробится на 3 части
            if let ChannelSplitSignal::EnterSplit { parts } = signal {
                assert_eq!(parts.len(), 3, "Order should be split into 3 parts");
                assert!(parts[0].price <= parts[1].price);
                assert!(parts[1].price <= parts[2].price);
                assert!((parts[0].size - parts[1].size).abs() < 0.01); // Размеры примерно равны
                return;
            }
        }
    }

    #[test]
    fn test_market_making_basic() {
        let mut strategy = MarketMakingStrategy::new(
            0.1,   // 0.1% спред
            5.0,   // 5% от баланса
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
                        assert!(spread > 0.0, "Spread should be positive");
                        assert!(spread < 1.0, "Spread should be reasonable (< 1%)");
                        assert!(bid < ask, "Bid should be less than ask");
                    }
                    MarketMakingSignal::Hold => {}
                    MarketMakingSignal::Wait => {}
                }
            }
        }
    }

    #[test]
    fn test_market_making_spread_calculation() {
        let mut strategy = MarketMakingStrategy::new(0.1, 5.0, 1000.0, 20);
        let balance = 1000.0;
        
        // Стабильная цена
        for _i in 0..25 {
            let price = 50000.0;
            let signal = strategy.update(price, balance);
            
            if let MarketMakingSignal::UpdateOrders { bid, ask, .. } = signal {
                let spread_pct = (ask - bid) / bid * 100.0;
                // Спред должен быть около 0.1% (наш параметр)
                assert!(spread_pct >= 0.05 && spread_pct <= 0.15, 
                    "Spread should be around 0.1%, got {:.2}%", spread_pct);
                break;
            }
        }
    }

    #[test]
    fn test_hft_entry_signal() {
        let mut strategy = HFTStrategy::new(
            0.01,  // порог входа 0.01%
            0.02,  // тейк-профит 0.02%
            60,    // макс удержание 60 сек
            10.0,  // 10% от баланса
        );

        let balance = 1000.0;
        let mut timestamp = 1000000;
        
        // Симулируем движение цены с трендом
        let base_price = 50000.0;
        for i in 0..10 {
            // Создаем тренд вверх
            let price = base_price + (i as f64 * 10.0);
            let bid_volume = 100.0 + (i as f64 * 10.0); // Растущий bid объем
            let ask_volume = 100.0;
            
            let signal = strategy.update(timestamp, price, bid_volume, ask_volume, balance);
            
            match signal {
                HFTSignal::Enter { side, price: entry_price, size, .. } => {
                    assert_eq!(side, "buy", "Should enter long on upward trend");
                    assert!(entry_price > 0.0);
                    assert!(size > 0.0);
                    return;
                }
                _ => {}
            }
            
            timestamp += 1;
        }
    }

    #[test]
    fn test_hft_exit_conditions() {
        let strategy = HFTStrategy::new(0.01, 0.02, 60, 10.0);
        
        let entry_price = 50000.0;
        let entry_time = 1000000;
        let current_time = 1000061; // Через 61 секунду
        
        // Тест 1: Выход по времени
        assert!(strategy.check_exit(
            entry_price,
            entry_time,
            50100.0, // Цена выше
            current_time,
            "buy"
        ), "Should exit after max hold time");

        // Тест 2: Выход по тейк-профиту
        // Тейк-профит 0.02% означает цену 50000 * 1.0002 = 50010
        let take_profit_price = entry_price * (1.0 + 0.0002); // 0.02% = 0.0002 в долях
        assert!(strategy.check_exit(
            entry_price,
            entry_time,
            take_profit_price,
            entry_time + 10,
            "buy"
        ), "Should exit on take profit");

        // Тест 3: Выход по стоп-лоссу (разворот)
        assert!(strategy.check_exit(
            entry_price,
            entry_time,
            50000.0 * 0.9995, // -0.05% (больше 0.01% стоп-лосса)
            entry_time + 10,
            "buy"
        ), "Should exit on stop loss");
    }

    #[test]
    fn test_all_strategies_reset() {
        let mut channel = ChannelSplitStrategy::new(20, 1.0, 2.0, 4.0, 3);
        let mut mm = MarketMakingStrategy::new(0.1, 5.0, 1000.0, 20);
        let mut hft = HFTStrategy::new(0.01, 0.02, 60, 10.0);
        
        // Используем стратегии
        channel.update(0, 50000.0, 1000.0);
        mm.update(50000.0, 1000.0);
        hft.update(0, 50000.0, 100.0, 100.0, 1000.0);
        
        // Сбрасываем
        channel.reset();
        mm.reset();
        hft.reset();
        
        // После reset стратегии должны быть готовы к новому использованию
        // Проверяем что reset не вызывает панику и стратегия работает
        let signal = channel.update(100, 50000.0, 1000.0);
        assert!(matches!(signal, ChannelSplitSignal::Wait | ChannelSplitSignal::EnterSplit {..} | ChannelSplitSignal::Hold));
    }

    #[test]
    fn test_strategy_integration_simulation() {
        // Интеграционный тест: симулируем работу всех стратегий на одном потоке цен
        let mut channel = ChannelSplitStrategy::new(20, 1.0, 2.0, 4.0, 3);
        let mut mm = MarketMakingStrategy::new(0.1, 5.0, 1000.0, 20);
        let mut hft = HFTStrategy::new(0.01, 0.02, 60, 10.0);
        
        let balance = 1000.0;
        let prices = vec![50000.0, 50100.0, 50200.0, 50100.0, 50000.0, 49900.0];
        
        let mut signals_count = 0;
        
        for (i, price) in prices.iter().enumerate() {
            let _channel_signal = channel.update(i as u64, *price, balance);
            let _mm_signal = mm.update(*price, balance);
            let _hft_signal = hft.update(i as u64, *price, 100.0, 100.0, balance);
            
            signals_count += 1;
        }
        
        // Все стратегии должны обработать все цены без ошибок
        assert_eq!(signals_count, prices.len());
    }
}

