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
