use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tower_http::cors::{CorsLayer, Any};

mod api;
mod backtest;
mod config;
mod data;
mod exchange;
mod live_trading;
mod service;

use config::Settings;
use data::{cache::TieredCache, repository::TickDataRepository};
use exchange::create_exchange;
use live_trading::PaperTradingProcessor;
use service::MarketDataService;

use crate::data::cache::TickDataCache;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("backtest") => run_backtest_mode().await,
        Some("live") => {
            // Check if paper trading is enabled
            if args.contains(&"--paper-trading".to_string()) {
                run_live_with_paper_trading().await
            } else {
                run_live_mode().await
            }
        }
        Some("api") | Some("server") => run_api_server().await,
        None => run_live_mode().await,
        Some("--help") | Some("-h") => {
            print_usage();
            Ok(())
        }
        _ => {
            eprintln!("❌ Unknown command: {}", args[1]);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    println!("Trading Core - Cryptocurrency Data Collection & Backtesting System");
    println!();
    println!("Usage:");
    println!("  cargo run                # Run live data collection");
    println!("  cargo run live            # Run live data collection");
    println!("  cargo run backtest        # Run backtesting mode");
    println!("  cargo run api             # Run HTTP API server (for web interface)");
    println!("  cargo run --help          # Show this help message");
    println!();
}

/// HTTP API Server mode
async fn run_api_server() -> Result<(), Box<dyn std::error::Error>> {
    init_application().await?;

    info!("🌐 Starting Trading Core HTTP API Server");

    let settings = Settings::new()?;
    info!("📋 Configuration loaded successfully");

    // Create database connection pool
    info!("🔌 Connecting to database...");
    let pool = create_database_pool(&settings).await?;
    test_database_connection(&pool).await?;
    info!("✅ Database connection established");

    // Create cache
    info!("💾 Initializing cache...");
    let cache = create_cache(&settings).await?;
    info!("✅ Cache initialized");

    // Create repository
    let repository = Arc::new(TickDataRepository::new(pool, cache));

    // Create API router
    let app = api::create_router(repository)
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any));

    // Start server
    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    info!("🚀 Starting HTTP API server on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await
        .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;
    
    info!("✅ HTTP API server listening on http://{}", addr);
    info!("📡 Available endpoints:");
    info!("   GET /api/strategies - List available strategies");
    info!("   GET /api/data/info - Get database information");
    info!("   GET /api/backtest/validate?symbol=ETHUSDT&data_count=10000 - Validate backtest config");

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}

async fn run_live_with_paper_trading() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize application environment
    init_application().await?;

    info!("🎯 Starting Trading Core Application (Live Mode + Paper Trading)");

    // Load configuration
    let settings = Settings::new()?;

    // Check if paper trading is enabled
    if !settings.paper_trading.enabled {
        warn!("⚠️ Paper trading is disabled in config. Set paper_trading.enabled = true");
        warn!("⚠️ Falling back to live data collection only...");
        return run_live_mode().await;
    }

    info!("📋 Configuration loaded successfully");
    info!("📊 Monitoring symbols: {:?}", settings.symbols);
    info!(
        "🎯 Paper Trading Strategy: {}",
        settings.paper_trading.strategy
    );
    info!(
        "💰 Initial Capital: ${}",
        settings.paper_trading.initial_capital
    );
    info!(
        "🗄️  Database: {} connections",
        settings.database.max_connections
    );
    info!(
        "💾 Cache: Memory({} ticks/{}s) + Redis({} ticks/{}s)",
        settings.cache.memory.max_ticks_per_symbol,
        settings.cache.memory.ttl_seconds,
        settings.cache.redis.max_ticks_per_symbol,
        settings.cache.redis.ttl_seconds
    );

    // Verify strategy exists
    let strategy_id = &settings.paper_trading.strategy;
    match crate::backtest::strategy::create_strategy(strategy_id) {
        Ok(_) => {
            info!("✅ Strategy '{}' verified", strategy_id);
        }
        Err(e) => {
            error!("❌ Invalid strategy '{}': {}", strategy_id, e);
            error!("Available strategies:");
            for s in crate::backtest::strategy::list_strategies() {
                error!("  - {} ({})", s.id, s.name);
            }
            std::process::exit(1);
        }
    }

    // Create database connection pool
    info!("🔌 Connecting to database...");
    let pool = create_database_pool(&settings).await?;
    test_database_connection(&pool).await?;
    info!("✅ Database connection established");

    // Create cache
    info!("💾 Initializing cache...");
    let cache = create_cache(&settings).await?;
    info!("✅ Cache initialized");

    // Create repository
    let repository = Arc::new(TickDataRepository::new(pool, cache));

    // Create exchange
    info!("📡 Initializing exchange connection...");
    info!("🔌 Exchange provider: {}", settings.exchange.provider);
    let exchange = create_exchange(&settings.exchange.provider)
        .map_err(|e| format!("Failed to create exchange: {}", e))?;
    info!("✅ Exchange connection ready");

    // Create strategy for paper trading
    let strategy = crate::backtest::strategy::create_strategy(strategy_id)
        .map_err(|e| format!("Failed to create strategy: {}", e))?;
    
    // Create paper trading processor
    let paper_trading = PaperTradingProcessor::new(
        strategy,
        repository.clone(),
        Decimal::from_str(&settings.paper_trading.initial_capital.to_string())
            .unwrap_or(Decimal::from(10000)),
    );

    // Create market data service with paper trading callback
    let service = MarketDataService::new(exchange, repository.clone(), settings.symbols.clone())
        .with_paper_trading(Arc::new(Mutex::new(paper_trading)));

    info!("🎯 Starting paper trading for {} symbols", settings.symbols.len());

    // Setup signal forwarding to service
    let service_shutdown_tx = service.get_shutdown_tx();
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl-c");
        println!("\nReceived Ctrl+C signal, forwarding to service...");
        info!("Received Ctrl+C signal, forwarding to service");
        let _ = service_shutdown_tx.send(());
    });

    // Start service and wait for completion
    match service.start().await {
        Ok(()) => {
            info!("✅ Service stopped gracefully");
        }
        Err(e) => {
            error!("❌ Service error: {}", e);
            std::process::exit(1);
        }
    }

    info!("✅ Application stopped gracefully");
    Ok(())
}

async fn run_live_application_with_service(
    settings: Settings,
    repository: Arc<TickDataRepository>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create exchange
    info!("📡 Initializing exchange connection...");
    info!("🔌 Exchange provider: {}", settings.exchange.provider);
    let exchange = create_exchange(&settings.exchange.provider)
        .map_err(|e| format!("Failed to create exchange: {}", e))?;
    info!("✅ Exchange connection ready");

    // Create market data service
    let service = MarketDataService::new(exchange, repository, settings.symbols.clone());

    info!(
        "🎯 Starting market data collection for {} symbols",
        settings.symbols.len()
    );

    // Setup signal forwarding to service
    let service_shutdown_tx = service.get_shutdown_tx();
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl-c");
        println!("\nReceived Ctrl+C signal, forwarding to service...");
        info!("Received Ctrl+C signal, forwarding to service");
        let _ = service_shutdown_tx.send(());
    });

    // Start service and wait for completion
    match service.start().await {
        Ok(()) => {
            info!("✅ Service stopped gracefully");
        }
        Err(e) => {
            error!("❌ Service error: {}", e);
            std::process::exit(1);
        }
    }

    info!("✅ Application stopped gracefully");
    Ok(())
}

async fn run_live_mode() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize application environment
    init_application().await?;

    info!("🔬 Starting Trading Core Application (Live Mode)");

    // Load configuration
    let settings = Settings::new()?;
    info!("📋 Configuration loaded successfully");
    info!("📊 Monitoring symbols: {:?}", settings.symbols);
    info!(
        "🗄️  Database: {} connections",
        settings.database.max_connections
    );
    info!(
        "💾 Cache: Memory({} ticks/{}s) + Redis({} ticks/{}s)",
        settings.cache.memory.max_ticks_per_symbol,
        settings.cache.memory.ttl_seconds,
        settings.cache.redis.max_ticks_per_symbol,
        settings.cache.redis.ttl_seconds
    );

    run_live_application(settings).await
}

/// Backtesting mode entry
async fn run_backtest_mode() -> Result<(), Box<dyn std::error::Error>> {
    init_application().await?;

    info!("🔬 Starting Trading Core Application (Backtest Mode)");

    let settings = Settings::new()?;
    info!("📋 Configuration loaded successfully");

    let pool = create_database_pool(&settings).await?;
    test_database_connection(&pool).await?;
    info!("✅ Database connection established");

    let cache = create_backtest_cache(&settings).await?;
    info!("✅ Cache initialized for backtest");

    let repository = TickDataRepository::new(pool, cache);

    run_backtest_interactive(repository).await?;

    info!("✅ Backtest completed successfully");
    Ok(())
}

/// Backtesting interactive interface
async fn run_backtest_interactive(
    repository: TickDataRepository,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::backtest::{
        engine::{BacktestConfig, BacktestEngine},
        strategy::{create_strategy, list_strategies},
    };
    use rust_decimal::Decimal;
    use std::io::{self, Write};
    use std::str::FromStr;

    println!("{}", "=".repeat(60));
    println!("🎯 TRADING CORE BACKTESTING SYSTEM");
    println!("{}", "=".repeat(60));

    // Display statistics
    println!("📊 Loading data statistics...");
    let data_info = repository.get_backtest_data_info().await?;

    println!("\n📈 Available Data:");
    println!("   Total Records: {}", data_info.total_records);
    println!("   Symbols: {}", data_info.symbols_count);
    if let Some(earliest) = data_info.earliest_time {
        println!("   Earliest: {}", earliest.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    if let Some(latest) = data_info.latest_time {
        println!("   Latest: {}", latest.format("%Y-%m-%d %H:%M:%S UTC"));
    }

    println!("\n📊 Available Symbols:");
    for symbol_info in &data_info.symbol_info {
        println!(
            "   {}: {} records",
            symbol_info.symbol, symbol_info.records_count
        );
    }

    // Strategy selection
    println!("\n{}", "=".repeat(60));
    println!("🎯 Available Strategies:");
    let strategies = list_strategies();
    for (idx, strategy) in strategies.iter().enumerate() {
        println!("   {}) {}", idx + 1, strategy.name);
        println!("      ID: {}", strategy.id);
        println!("      Description: {}", strategy.description);
    }

    print!("\nSelect strategy (1-{}): ", strategies.len());
    io::stdout().flush()?;
    let mut strategy_input = String::new();
    io::stdin().read_line(&mut strategy_input)?;
    let strategy_idx: usize = strategy_input.trim().parse().map_err(|_| {
        "Invalid strategy selection".to_string()
    })?;

    if strategy_idx < 1 || strategy_idx > strategies.len() {
        return Err("Invalid strategy selection".into());
    }

    let selected_strategy = &strategies[strategy_idx - 1];
    println!("✅ Selected: {}", selected_strategy.name);

    // Symbol selection
    print!("\nSelect symbol: ");
    io::stdout().flush()?;
    let mut symbol_input = String::new();
    io::stdin().read_line(&mut symbol_input)?;
    let symbol = symbol_input.trim().to_uppercase();

    // Check if symbol exists
    let symbol_info = data_info
        .symbol_info
        .iter()
        .find(|s| s.symbol == symbol)
        .ok_or_else(|| format!("Symbol {} not found in database", symbol))?;

    println!("✅ Selected: {} ({} records)", symbol, symbol_info.records_count);

    // Data count
    print!("\nEnter number of records to backtest (default: 10000): ");
    io::stdout().flush()?;
    let mut count_input = String::new();
    io::stdin().read_line(&mut count_input)?;
    let data_count: u64 = if count_input.trim().is_empty() {
        10000
    } else {
        count_input.trim().parse().map_err(|_| "Invalid number")?
    };

    if data_count > symbol_info.records_count {
        warn!(
            "⚠️  Requested {} records, but only {} available. Using {} records.",
            data_count, symbol_info.records_count, symbol_info.records_count
        );
    }

    // Initial capital
    print!("\nEnter initial capital (default: $10000): $");
    io::stdout().flush()?;
    let mut capital_input = String::new();
    io::stdin().read_line(&mut capital_input)?;
    let initial_capital = if capital_input.trim().is_empty() {
        Decimal::from(10000)
    } else {
        Decimal::from_str(capital_input.trim())
            .map_err(|_| "Invalid capital amount")?
    };

    // Commission rate
    print!("\nEnter commission rate % (default: 0.1%): ");
    io::stdout().flush()?;
    let mut commission_input = String::new();
    io::stdin().read_line(&mut commission_input)?;
    let commission_percent = if commission_input.trim().is_empty() {
        Decimal::from_str("0.1").unwrap()
    } else {
        Decimal::from_str(commission_input.trim())
            .map_err(|_| "Invalid commission rate")?
    };
    let commission_rate = commission_percent / Decimal::from(100);

    println!("\n{}", "=".repeat(60));
    println!("🚀 Starting Backtest");
    println!("{}", "=".repeat(60));
    println!("Strategy: {}", selected_strategy.name);
    println!("Symbol: {}", symbol);
    println!("Data Points: {}", data_count);
    println!("Initial Capital: ${}", initial_capital);
    println!("Commission Rate: {:.4}%", commission_rate * Decimal::from(100));
    println!("{}", "=".repeat(60));

    // Create strategy
    let mut strategy = create_strategy(&selected_strategy.id)
        .map_err(|e| format!("Failed to create strategy: {}", e))?;

    // Initialize strategy with empty params for now
    strategy.initialize(std::collections::HashMap::new())
        .map_err(|e| format!("Failed to initialize strategy: {}", e))?;

    // Check if strategy supports OHLC (before creating engine)
    let supports_ohlc = strategy.supports_ohlc();
    let preferred_timeframe = strategy.preferred_timeframe();

    // Create backtest config
    let config = BacktestConfig::new(initial_capital)
        .with_commission_rate(commission_rate);

    // Create backtest engine
    let mut engine = BacktestEngine::new(strategy, config)?;

    // Check if strategy supports OHLC
    if supports_ohlc {
        if let Some(timeframe) = preferred_timeframe {
            info!(
                "\n🔄 Strategy supports OHLC, using {} timeframe for better performance",
                timeframe.as_str()
            );

            // Get OHLC data
            let ohlc_data = repository
                .generate_recent_ohlc_for_backtest(&symbol, timeframe, data_count as u32)
                .await?;

            if ohlc_data.is_empty() {
                return Err(format!(
                    "No OHLC data available for {} with timeframe {}",
                    symbol,
                    timeframe.as_str()
                )
                .into());
            }

            info!("📊 Loaded {} OHLC candles", ohlc_data.len());

            // Run backtest with OHLC
            let result = engine.run_with_ohlc(ohlc_data);
            display_backtest_results(&result);
        } else {
            // Fallback to tick data
            let ticks = repository
                .get_recent_ticks_for_backtest(&symbol, data_count as i64)
                .await?;

            if ticks.is_empty() {
                return Err(format!("No data available for symbol: {}", symbol).into());
            }

            info!("📊 Loaded {} ticks", ticks.len());

            // Run backtest
            let result = engine.run(ticks);
            display_backtest_results(&result);
        }
    } else {
        // Get tick data
        let ticks = repository
            .get_recent_ticks_for_backtest(&symbol, data_count as i64)
            .await?;

        if ticks.is_empty() {
            return Err(format!("No data available for symbol: {}", symbol).into());
        }

        info!("📊 Loaded {} ticks", ticks.len());

        // Run backtest
        let result = engine.run(ticks);
        display_backtest_results(&result);
    }

    Ok(())
}

fn display_backtest_results(result: &crate::backtest::engine::BacktestResult) {
    println!("\n{}", "=".repeat(60));
    println!("📊 BACKTEST RESULTS SUMMARY");
    println!("{}", "=".repeat(60));
    println!("Strategy: {}", result.strategy_name);
    println!("Initial Capital: ${:.2}", result.initial_capital);
    println!("Final Value: ${:.2}", result.final_value);
    println!("Total P&L: ${:.2}", result.total_pnl);
    println!("Return: {:.2}%", result.return_percentage);

    println!("\n{}", "-".repeat(60));
    println!("TRADING STATISTICS");
    println!("{}", "-".repeat(60));
    println!("Total Trades: {}", result.total_trades);
    println!(
        "Winning Trades: {} ({:.1}%)",
        result.winning_trades,
        if result.total_trades > 0 {
            (result.winning_trades as f64 / result.total_trades as f64) * 100.0
        } else {
            0.0
        }
    );
    println!(
        "Losing Trades: {} ({:.1}%)",
        result.losing_trades,
        if result.total_trades > 0 {
            (result.losing_trades as f64 / result.total_trades as f64) * 100.0
        } else {
            0.0
        }
    );
    println!("Profit Factor: {:.2}", result.profit_factor);

    println!("\n{}", "-".repeat(60));
    println!("RISK METRICS");
    println!("{}", "-".repeat(60));
    println!("Max Drawdown: {:.2}%", result.max_drawdown);
    println!("Sharpe Ratio: {:.2}", result.sharpe_ratio);
    println!("Volatility: {:.2}%", result.volatility);
}

// Helper functions
async fn init_application() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    if dotenv::dotenv().is_err() {
        warn!("⚠️  No .env file found, using environment variables");
    }

    // Initialize tracing
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("trading_core=info"));

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false))
        .with(filter)
        .init();

    info!("🔧 Application environment initialized");
    Ok(())
}

async fn create_database_pool(settings: &Settings) -> Result<PgPool, Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| settings.database.url.clone());

    let pool = sqlx::PgPool::connect_with(
        sqlx::postgres::PgConnectOptions::from_str(&database_url)?
    )
    .await?;

    Ok(pool)
}

async fn test_database_connection(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(|e| format!("Database connection test failed: {}", e))?;
    Ok(())
}

async fn create_cache(settings: &Settings) -> Result<data::cache::TieredCache, Box<dyn std::error::Error>> {
    use data::cache::TieredCache;

    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    match TieredCache::new(
        (settings.cache.memory.max_ticks_per_symbol, settings.cache.memory.ttl_seconds),
        (&redis_url, settings.cache.redis.max_ticks_per_symbol, settings.cache.redis.ttl_seconds),
    ).await {
        Ok(cache) => {
            info!("✅ Using tiered cache (Memory + Redis)");
            Ok(cache)
        }
        Err(e) => {
            warn!("⚠️  Redis unavailable ({}), but continuing with memory cache", e);
            // Try to create with minimal Redis config
            TieredCache::new(
                (settings.cache.memory.max_ticks_per_symbol, settings.cache.memory.ttl_seconds),
                ("redis://127.0.0.1:6379", 0, 0),
            ).await.map_err(|e| format!("Failed to create cache: {}", e).into())
        }
    }
}

async fn create_backtest_cache(settings: &Settings) -> Result<data::cache::TieredCache, Box<dyn std::error::Error>> {
    // For backtesting, we can use a simpler cache configuration
    use data::cache::TieredCache;

    let cache = TieredCache::new(
        (settings.cache.memory.max_ticks_per_symbol, settings.cache.memory.ttl_seconds),
        ("", 0, 0), // No Redis for backtesting
    ).await.map_err(|_| "Failed to create backtest cache")?;

    Ok(cache)
}

/// Main application runtime (original live mode)
async fn run_live_application(settings: Settings) -> Result<(), Box<dyn std::error::Error>> {
    // Validate basic configuration
    if settings.symbols.is_empty() {
        error!("❌ No symbols configured for monitoring");
        std::process::exit(1);
    }

    if settings.database.max_connections == 0 {
        error!("❌ Database max_connections must be greater than 0");
        std::process::exit(1);
    }

    // Create database connection pool
    info!("🔌 Connecting to database...");
    let pool = create_database_pool(&settings).await?;

    // Test database connectivity
    test_database_connection(&pool).await?;
    info!("✅ Database connection established");

    // Create cache
    info!("💾 Initializing cache...");
    let cache = create_cache(&settings).await?;
    info!("✅ Cache initialized");

    // Create repository
    let repository = Arc::new(TickDataRepository::new(pool, cache));

    // Create exchange
    info!("📡 Initializing exchange connection...");
    info!("🔌 Exchange provider: {}", settings.exchange.provider);
    let exchange = create_exchange(&settings.exchange.provider)
        .map_err(|e| format!("Failed to create exchange: {}", e))?;
    info!("✅ Exchange connection ready");

    // Create market data service
    let service = MarketDataService::new(exchange, repository, settings.symbols.clone());

    info!(
        "🎯 Starting market data collection for {} symbols",
        settings.symbols.len()
    );

    // Setup signal forwarding to service
    let service_shutdown_tx = service.get_shutdown_tx();
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl-c");
        println!("\nReceived Ctrl+C signal, forwarding to service...");
        info!("Received Ctrl+C signal, forwarding to service");
        let _ = service_shutdown_tx.send(());
    });

    // Start service and wait for completion
    match service.start().await {
        Ok(()) => {
            info!("✅ Service stopped gracefully");
        }
        Err(e) => {
            error!("❌ Service error: {}", e);
            std::process::exit(1);
        }
    }

    info!("✅ Application stopped gracefully");
    Ok(())
}
