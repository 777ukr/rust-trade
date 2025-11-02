#!/bin/bash
set -e

PROJECT_DIR="/home/crypto/sites/cryptotrader.com"
WORKSPACE_FILE="$PROJECT_DIR/crypto_trader.code-workspace"

echo "🚀 Setting up Crypto Trader project..."

# Create project directory
mkdir -p "$PROJECT_DIR"
cd "$PROJECT_DIR"

# Check and install Rust if needed
if ! command -v cargo &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Initialize Rust project
if [ ! -f "Cargo.toml" ]; then
    echo "🔧 Initializing Rust project..."
    cargo init --name crypto_trader
fi

# Create directory structure
echo "📁 Creating directory structure..."
mkdir -p src/api src/indicators src/screener src/parser src/strategy src/models src/utils
mkdir -p tests config data/logs

# Create module files
echo "📝 Creating module files..."

# Create API module stub files
touch src/api/gateway.rs src/api/client.rs
echo "pub mod gateway;" > src/api/gateway.rs
echo "pub mod client;" > src/api/client.rs

cat > src/api/mod.rs << 'EOF'
pub mod gateway;
pub mod client;

use crate::models::MarketData;

pub trait ExchangeAPI {
    fn get_price(&self, symbol: &str) -> Result<f64, String>;
    fn place_order(&self, order: &OrderRequest) -> Result<String, String>;
}

pub struct OrderRequest {
    pub symbol: String,
    pub side: String, // "buy" or "sell"
    pub amount: f64,
    pub price: Option<f64>, // None for market orders
}
EOF

# Create parser module stub files
touch src/parser/market_data.rs src/parser/price_parser.rs
echo "pub mod market_data;" > src/parser/market_data.rs
echo "pub mod price_parser;" > src/parser/price_parser.rs

cat > src/parser/mod.rs << 'EOF'
pub mod market_data;
pub mod price_parser;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedData {
    pub timestamp: u64,
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
}

pub fn parse_market_data(data: &str) -> Result<ParsedData, String> {
    // Implement parsing logic
    todo!()
}
EOF

# Create screener module stub files
touch src/screener/filters.rs src/screener/scanner.rs
echo "pub mod filters;" > src/screener/filters.rs
echo "pub mod scanner;" > src/screener/scanner.rs

cat > src/screener/mod.rs << 'EOF'
pub mod filters;
pub mod scanner;

use crate::models::CryptoPair;
use crate::indicators::TechnicalIndicator;

pub struct Screener {
    filters: Vec<Box<dyn Filter>>,
}

pub trait Filter {
    fn check(&self, pair: &CryptoPair) -> bool;
}

impl Screener {
    pub fn new() -> Self {
        Screener {
            filters: Vec::new(),
        }
    }

    pub fn add_filter(&mut self, filter: Box<dyn Filter>) {
        self.filters.push(filter);
    }

    pub fn scan(&self, pairs: Vec<CryptoPair>) -> Vec<CryptoPair> {
        pairs.into_iter()
            .filter(|pair| self.filters.iter().all(|f| f.check(pair)))
            .collect()
    }
}
EOF

# Create indicators module stub files
touch src/indicators/rsi.rs src/indicators/macd.rs src/indicators/bollinger.rs src/indicators/sma.rs
for file in rsi macd bollinger sma; do
    echo "pub mod $file;" > "src/indicators/$file.rs"
done

cat > src/indicators/mod.rs << 'EOF'
pub mod rsi;
pub mod macd;
pub mod bollinger;
pub mod sma;

use serde::{Deserialize, Serialize};

pub trait TechnicalIndicator {
    fn calculate(&self, prices: &[f64]) -> Result<IndicatorValue, String>;
    fn name(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndicatorValue {
    Scalar(f64),
    Vector(Vec<f64>),
    Crossover { signal: String, value: f64 },
}

pub struct RSI {
    period: usize,
}

impl RSI {
    pub fn new(period: usize) -> Self {
        RSI { period }
    }
}

impl TechnicalIndicator for RSI {
    fn calculate(&self, prices: &[f64]) -> Result<IndicatorValue, String> {
        if prices.len() < self.period + 1 {
            return Err("Not enough data".to_string());
        }
        // RSI calculation
        todo!()
    }
    
    fn name(&self) -> &str {
        "RSI"
    }
}
EOF

# Create strategy module stub files
touch src/strategy/trading_strategy.rs src/strategy/stop_loss.rs
echo "pub mod trading_strategy;" > src/strategy/trading_strategy.rs
echo "pub mod stop_loss;" > src/strategy/stop_loss.rs

cat > src/strategy/mod.rs << 'EOF'
pub mod trading_strategy;
pub mod stop_loss;

use crate::models::CryptoPair;
use crate::indicators::TechnicalIndicator;

pub struct TradingStrategy {
    entry_conditions: Vec<Box<dyn EntryCondition>>,
    exit_conditions: Vec<Box<dyn ExitCondition>>,
    stop_loss_percent: f64,
    take_profit_percent: f64,
}

pub trait EntryCondition {
    fn check(&self, pair: &CryptoPair) -> bool;
}

pub trait ExitCondition {
    fn check(&self, pair: &CryptoPair, entry_price: f64) -> bool;
}

impl TradingStrategy {
    pub fn new() -> Self {
        TradingStrategy {
            entry_conditions: Vec::new(),
            exit_conditions: Vec::new(),
            stop_loss_percent: 2.0, // 2% stop loss
            take_profit_percent: 5.0, // 5% take profit
        }
    }

    pub fn should_enter(&self, pair: &CryptoPair) -> bool {
        self.entry_conditions.iter().all(|c| c.check(pair))
    }

    pub fn should_exit(&self, pair: &CryptoPair, entry_price: f64) -> bool {
        self.exit_conditions.iter().any(|c| c.check(pair, entry_price))
    }

    pub fn calculate_stop_loss(&self, entry_price: f64) -> f64 {
        entry_price * (1.0 - self.stop_loss_percent / 100.0)
    }

    pub fn calculate_take_profit(&self, entry_price: f64) -> f64 {
        entry_price * (1.0 + self.take_profit_percent / 100.0)
    }
}
EOF

cat > src/models/mod.rs << 'EOF'
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoPair {
    pub symbol: String,
    pub base: String,
    pub quote: String,
    pub price: f64,
    pub volume_24h: f64,
    pub change_24h: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub symbol: String,
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub symbol: String,
    pub side: String, // "buy" or "sell"
    pub amount: f64,
    pub price: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub side: String,
    pub entry_price: f64,
    pub amount: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub entry_time: u64,
}
EOF

# Create utils module stub files
touch src/utils/logger.rs src/utils/config.rs
echo "pub mod logger;" > src/utils/logger.rs
echo "pub mod config;" > src/utils/config.rs

cat > src/utils/mod.rs << 'EOF'
pub mod logger;
pub mod config;

use std::fs;
use std::path::Path;

pub fn ensure_dir(path: &Path) -> Result<(), std::io::Error> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn format_price(price: f64) -> String {
    format!("{:.8}", price)
}

pub fn calculate_percentage_change(old: f64, new: f64) -> f64 {
    ((new - old) / old) * 100.0
}
EOF

# Update main.rs
cat > src/main.rs << 'EOF'
mod api;
mod parser;
mod screener;
mod indicators;
mod strategy;
mod models;
mod utils;

use std::time::Duration;
use std::thread;

fn main() {
    println!("🚀 Crypto Trader Bot Starting...");
    
    // Initialize components
    let screener = screener::Screener::new();
    let strategy = strategy::TradingStrategy::new();
    
    // Main trading loop
    loop {
        println!("📊 Scanning markets...");
        
        // 1. Fetch market data
        // 2. Parse data
        // 3. Screen for opportunities
        // 4. Apply strategy
        // 5. Execute trades
        
        thread::sleep(Duration::from_secs(60)); // Check every minute
    }
}
EOF

# Update Cargo.toml with dependencies
if ! grep -q "^\[dependencies\]" Cargo.toml; then
    cat >> Cargo.toml << 'EOF'

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
chrono = "0.4"
env_logger = "0.11"
log = "0.4"
anyhow = "1.0"
thiserror = "1.0"

[dev-dependencies]
mockito = "1.0"
EOF
fi

# Create workspace file
echo "📋 Creating workspace configuration..."
cat > "$WORKSPACE_FILE" << 'EOF'
{
	"folders": [
		{
			"name": "crypto_trader",
			"path": "/home/crypto/sites/cryptotrader.com"
		}
	],
	"settings": {
		"files.exclude": {
			"**/node_modules": true,
			"**/dist": true,
			"**/.git": true,
			"**/target": true,
			"**/Cargo.lock": false
		},
		"rust-analyzer.checkOnSave": {
			"command": "clippy"
		}
	}
}
EOF

# Create .gitignore
cat > .gitignore << 'EOF'
# Rust
/target/
**/*.rs.bk
*.pdb
Cargo.lock

# IDE
.vscode/
.idea/
*.swp
*.swo

# Logs and data
*.log
data/
logs/

# Config (if contains secrets)
config/secrets.toml
EOF

# Create README
cat > README.md << 'EOF'
# Crypto Trader Bot

Trading bot for cryptocurrency with parser, screener, and API gateway.

## Structure

- `src/api/` - Exchange API integration
- `src/parser/` - Market data parsing
- `src/screener/` - Market scanning and filtering
- `src/indicators/` - Technical indicators (RSI, MACD, etc.)
- `src/strategy/` - Trading strategy and stop-loss logic
- `src/models/` - Data models

## Usage

```bash
cargo build
cargo run
```

## Configuration

Edit `config/config.toml` for trading parameters.
EOF

echo "✅ Setup complete!"
echo "📂 Project location: $PROJECT_DIR"
echo "🔧 Workspace file: $WORKSPACE_FILE"
echo ""
echo "Next steps:"
echo "1. Open workspace: code $WORKSPACE_FILE"
echo "2. Build project: cd $PROJECT_DIR && cargo build"
echo "3. Run: cargo run"
