//! Global data for Ethereum Dip Strategy
//!
//! Stores strategy state that needs to persist across iterations.

use barter::engine::Processor;
use barter_execution::AccountEvent;
use barter_data::event::MarketEvent;
use barter_instrument::instrument::InstrumentIndex;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Global data for Ethereum Dip Strategy
///
/// Tracks highest prices and entry prices per instrument.
#[derive(Debug, Clone, Default)]
pub struct EthDipGlobalData {
    /// Track highest price per instrument
    pub highest_prices: HashMap<InstrumentIndex, Decimal>,
    /// Track entry prices per instrument
    pub entry_prices: HashMap<InstrumentIndex, Decimal>,
}

impl<ExchangeKey, AssetKey, InstrumentKey>
    Processor<&AccountEvent<ExchangeKey, AssetKey, InstrumentKey>> for EthDipGlobalData
{
    type Audit = ();
    fn process(&mut self, _: &AccountEvent<ExchangeKey, AssetKey, InstrumentKey>) -> Self::Audit {}
}

impl<InstrumentKey, Kind> Processor<&MarketEvent<InstrumentKey, Kind>> for EthDipGlobalData {
    type Audit = ();
    fn process(&mut self, _: &MarketEvent<InstrumentKey, Kind>) -> Self::Audit {}
}

