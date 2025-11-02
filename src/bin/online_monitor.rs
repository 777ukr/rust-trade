//! Онлайн мониторинг стратегии - отслеживание в реальном времени
//! Адаптивная методология с автоматическим выбором лучшей стратегии

#![cfg(feature = "gate_exec")]

use std::time::Duration;
use anyhow::Result;
use tokio::time::interval;

use rust_test::strategy::adaptive_channel::{AdaptiveChannelStrategy, StrategyVariant};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔄 Online Strategy Monitor Starting...\n");
    
    dotenvy::dotenv().ok();
    
    // Создаем 3 стратегии для сравнения
    let mut trailing = AdaptiveChannelStrategy::new(
        StrategyVariant::TrailingStop,
        20,
        0.02,
        2.0,
        4.0,
    );
    
    let mut early = AdaptiveChannelStrategy::new(
        StrategyVariant::EarlyExit,
        20,
        0.02,
        2.0,
        4.0,
    );
    
    let mut extended = AdaptiveChannelStrategy::new(
        StrategyVariant::ExtendedTarget,
        20,
        0.02,
        2.0,
        4.0,
    );
    
    let mut monitor = StrategyMonitor::new();
    
    // Обновляем цены каждые 5 секунд
    let mut ticker = interval(Duration::from_secs(5));
    
    println!("📊 Monitoring strategies in real-time...");
    println!("   Update interval: 5 seconds\n");
    
    loop {
        ticker.tick().await;
        
        // Получаем текущую цену BTC
        match fetch_current_price().await {
            Ok(price) => {
                // Обновляем все стратегии
                trailing.update_price(price);
                early.update_price(price);
                extended.update_price(price);
                
                // Мониторинг и рекомендации
                monitor.update(price, &trailing, &early, &extended).await?;
            }
            Err(e) => {
                eprintln!("Error fetching price: {}", e);
            }
        }
    }
}

async fn fetch_current_price() -> Result<f64> {
    let client = reqwest::Client::new();
    let url = "https://api.gateio.ws/api/v4/futures/usdt/tickers?contract=BTC_USDT";
    let resp = client.get(url).send().await?;
    let json: serde_json::Value = resp.json().await?;
    
    if let Some(ticker) = json.as_array().and_then(|a| a.first()) {
        let price = ticker["last"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .or_else(|| ticker["last"].as_f64())
            .ok_or_else(|| anyhow::anyhow!("No price"))?;
        Ok(price)
    } else {
        anyhow::bail!("Invalid response")
    }
}

struct StrategyMonitor {
    price_history: Vec<f64>,
    last_recommendation: Option<StrategyVariant>,
    update_count: u64,
}

impl StrategyMonitor {
    fn new() -> Self {
        Self {
            price_history: Vec::with_capacity(100),
            last_recommendation: None,
            update_count: 0,
        }
    }

    async fn update(
        &mut self,
        price: f64,
        trailing: &AdaptiveChannelStrategy,
        early: &AdaptiveChannelStrategy,
        extended: &AdaptiveChannelStrategy,
    ) -> Result<()> {
        self.price_history.push(price);
        if self.price_history.len() > 100 {
            self.price_history.remove(0);
        }
        
        self.update_count += 1;
        
        // Каждые 10 обновлений (50 секунд) делаем рекомендацию
        if self.update_count % 10 == 0 {
            let recommendation = self.select_best_strategy(trailing, early, extended);
            
            if Some(recommendation) != self.last_recommendation {
                println!("\n🎯 Recommendation: Use {:?} strategy", recommendation);
                println!("   Current BTC: ${:.2}", price);
                self.last_recommendation = Some(recommendation);
            }
        } else {
            // Каждое обновление показываем статус
            println!("💰 BTC: ${:.2} | T:{} E:{} X:{} | Entry signals: trailing={} early={} extended={}",
                price,
                if trailing.should_enter() { "✓" } else { "-" },
                if early.should_enter() { "✓" } else { "-" },
                if extended.should_enter() { "✓" } else { "-" },
                trailing.should_enter(),
                early.should_enter(),
                extended.should_enter(),
            );
        }
        
        Ok(())
    }

    fn select_best_strategy(
        &self,
        trailing: &AdaptiveChannelStrategy,
        early: &AdaptiveChannelStrategy,
        extended: &AdaptiveChannelStrategy,
    ) -> StrategyVariant {
        // Простая логика выбора на основе текущего состояния рынка
        // В реальности здесь будет более сложная аналитика
        
        let volatility = self.calculate_volatility();
        
        if volatility > 0.03 {
            // Высокая волатильность - используем trailing stop
            StrategyVariant::TrailingStop
        } else if volatility < 0.01 {
            // Низкая волатильность - используем extended target
            StrategyVariant::ExtendedTarget
        } else {
            // Средняя - используем early exit
            StrategyVariant::EarlyExit
        }
    }

    fn calculate_volatility(&self) -> f64 {
        if self.price_history.len() < 10 {
            return 0.0;
        }
        
        let recent: Vec<f64> = self.price_history.iter().rev().take(10).copied().collect();
        let avg = recent.iter().sum::<f64>() / recent.len() as f64;
        let variance = recent.iter()
            .map(|p| (p - avg).powi(2))
            .sum::<f64>() / recent.len() as f64;
        variance.sqrt() / avg
    }
}

