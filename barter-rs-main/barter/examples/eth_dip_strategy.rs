//! Example: Ethereum Dip Strategy with Paper Trading
//!
//! Strategy parameters:
//! - Entry: 0.2% dip from local maximum
//! - Take profit: 0.6% from entry price
//! - Stop loss: 0.22% from entry price

use barter::{
    engine::{
        clock::LiveClock,
        state::{
            instrument::{
                data::DefaultInstrumentMarketData,
                filter::InstrumentFilter,
            },
            trading::TradingState,
        },
    },
    logging::init_logging,
    risk::DefaultRiskManager,
    statistic::time::Daily,
    system::{
        builder::{AuditMode, EngineFeedMode, SystemArgs, SystemBuilder},
        config::SystemConfig,
    },
};
use barter_data::{
    streams::builder::dynamic::indexed::init_indexed_multi_exchange_market_stream,
    subscription::SubKind,
};
use barter_instrument::index::IndexedInstruments;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::{fs::File, io::BufReader};
use tracing::info;
use chrono::Utc;

const FILE_PATH_SYSTEM_CONFIG: &str = "barter/examples/config/eth_gateio_config.json";
const RISK_FREE_RETURN: Decimal = dec!(0.05);

// Import the strategy module
mod strategy;
mod global_data;

use strategy::EthDipStrategy;
use global_data::EthDipGlobalData;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get current working directory and create logs directory
    let current_dir = std::env::current_dir()?;
    let logs_dir = current_dir.join("logs");
    std::fs::create_dir_all(&logs_dir)?;
    
    // Setup file logging with absolute path
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let log_file = logs_dir.join(format!("eth_strategy_{}.log", timestamp));
    let log_file_str = log_file.to_string_lossy().to_string();
    let file = std::fs::File::create(&log_file)?;
    
    // Initialise Tracing with both console and file output
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};
    
    let env_filter = tracing_subscriber::filter::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env_lossy();
    
    let console_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true);
    
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file)
        .with_ansi(false);
    
    Registry::default()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    let separator = "=".repeat(80);
    info!("{}", separator);
    info!("Starting Ethereum Dip Strategy with Paper Trading");
    info!("{}", separator);
    info!("Parameters: Entry dip 0.2%, Take profit 0.6%, Stop loss 0.22%");
    info!("Log file: {}", log_file_str);
    info!("Working directory: {:?}", current_dir);
    info!("{}", separator);

    // Load SystemConfig
    let SystemConfig {
        instruments,
        executions,
    } = load_config()?;

    // Construct IndexedInstruments
    let instruments = IndexedInstruments::new(instruments);

    // Initialise MarketData Stream
    // Note: Gate.io Spot only supports PublicTrades, not OrderBooksL1
    let market_stream = init_indexed_multi_exchange_market_stream(
        &instruments,
        &[SubKind::PublicTrades],
    )
    .await?;

    // Construct System Args with our custom strategy
    let args = SystemArgs::new(
        &instruments,
        executions,
        LiveClock,
        EthDipStrategy::default(),
        DefaultRiskManager::default(),
        market_stream,
        EthDipGlobalData::default(),
        |_| DefaultInstrumentMarketData::default(),
    );

    // Build & run full system:
    let system = SystemBuilder::new(args)
        // Engine feed in Sync mode (Iterator input)
        .engine_feed_mode(EngineFeedMode::Iterator)
        // Audit feed is enabled (Engine sends audits)
        .audit_mode(AuditMode::Enabled)
        // Engine starts with TradingState::Disabled
        .trading_state(TradingState::Disabled)
        // Build System, but don't start spawning tasks yet
        .build()?
        // Init System, spawning component tasks on the current runtime
        .init_with_runtime(tokio::runtime::Handle::current())
        .await?;

    info!("System initialized. Enabling trading...");

    // Enable trading
    system.trading_state(TradingState::Enabled);

    info!("Trading enabled. Strategy is now active.");
    info!("Press Ctrl+C to stop...");

    // Run until interrupted
    tokio::signal::ctrl_c().await?;

    info!("Shutting down...");

    // Before shutting down, CancelOrders and then ClosePositions
    system.cancel_orders(InstrumentFilter::None);
    system.close_positions(InstrumentFilter::None);

    // Shutdown
    let (engine, _shutdown_audit) = system.shutdown().await?;

    // Generate TradingSummary<Daily>
    let trading_summary = engine
        .trading_summary_generator(RISK_FREE_RETURN)
        .generate(Daily);

    // Print TradingSummary<Daily> to terminal
    let separator = "=".repeat(80);
    info!("{}", separator);
    info!("TRADING SUMMARY");
    info!("{}", separator);
    trading_summary.print_summary();

    // Save summary to file
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let summary_file = format!("eth_strategy_summary_{}.txt", timestamp);
    save_summary_to_file(&trading_summary, &summary_file)?;
    info!("Summary saved to: {}", summary_file);

    Ok(())
}

fn save_summary_to_file(summary: &barter::statistic::summary::TradingSummary<barter::statistic::time::Daily>, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    let mut file = std::fs::File::create(filename)?;
    let separator = "=".repeat(80);
    writeln!(file, "{}", separator)?;
    writeln!(file, "ETHEREUM DIP STRATEGY - TRADING SUMMARY")?;
    writeln!(file, "Generated: {}", Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))?;
    writeln!(file, "{}", separator)?;
    writeln!(file, "")?;
    
    // Convert summary to string and write
    let summary_str = format!("{:#?}", summary);
    writeln!(file, "{}", summary_str)?;
    
    Ok(())
}

fn load_config() -> Result<SystemConfig, Box<dyn std::error::Error>> {
    let file = File::open(FILE_PATH_SYSTEM_CONFIG)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}

