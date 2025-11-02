#![allow(dead_code)]

pub mod simple_quote;
pub mod btc_strategy;
pub mod stop_loss;
pub mod adaptive_channel;
pub mod market_making;
pub mod hft;
pub mod channel_split;

pub use simple_quote::{QuoteConfig, QuotePlan, ReferenceMeta, SimpleQuoteStrategy};
pub use btc_strategy::{BtcTradingStrategy, BtcStrategyConfig};
pub use adaptive_channel::{AdaptiveChannelStrategy, StrategyVariant};
pub use market_making::{MarketMakingStrategy, MarketMakingSignal};
pub use hft::{HFTStrategy, HFTSignal};
pub use channel_split::{ChannelSplitStrategy, ChannelSplitSignal, OrderPart};
