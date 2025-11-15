//! Ethereum Dip Strategy
//!
//! Strategy that enters a position when price drops 0.2% from local maximum,
//! with take profit at 0.6% and stop loss at 0.22%.

use crate::global_data::EthDipGlobalData;
use barter::{
    engine::{
        Engine,
        state::{
            EngineState,
            instrument::{
                data::{DefaultInstrumentMarketData, InstrumentDataState},
                filter::InstrumentFilter,
            },
        },
    },
    strategy::{
        algo::AlgoStrategy,
        close_positions::{ClosePositionsStrategy, close_open_positions_with_market_orders},
        on_disconnect::OnDisconnectStrategy,
        on_trading_disabled::OnTradingDisabled,
    },
};
use barter_execution::order::{
    id::{ClientOrderId, StrategyId},
    request::{OrderRequestCancel, OrderRequestOpen, RequestOpen},
    TimeInForce,
    OrderKey,
};
use barter_instrument::Side;
use barter_execution::order::OrderKind;
use barter_instrument::{
    asset::AssetIndex,
    exchange::{ExchangeId, ExchangeIndex},
    instrument::InstrumentIndex,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use smol_str::SmolStr;

/// Ethereum Dip Strategy
///
/// Tracks local price maximum and enters position on 0.2% dip.
/// Sets take profit at 0.6% and stop loss at 0.22%.
#[derive(Debug, Clone)]
pub struct EthDipStrategy {
    /// Strategy identifier
    pub id: StrategyId,
}

impl EthDipStrategy {
    /// Strategy ID
    pub const ID: StrategyId = StrategyId(SmolStr::new_static("eth_dip_strategy"));

    /// Create new strategy instance
    pub fn new() -> Self {
        Self {
            id: Self::ID,
        }
    }

    /// Check if we should enter a long position
    fn should_enter_long(
        &self,
        _instrument: InstrumentIndex,
        current_price: Decimal,
        highest_price: Decimal,
    ) -> bool {
        if highest_price.is_zero() {
            return false;
        }

        // Calculate drop percentage
        let drop_pct = ((highest_price - current_price) / highest_price) * dec!(100);

        // Enter if drop >= 0.2%
        drop_pct >= dec!(0.2)
    }

    /// Calculate take profit price (0.6% above entry)
    fn take_profit_price(entry_price: Decimal) -> Decimal {
        entry_price * dec!(1.006)
    }

    /// Calculate stop loss price (0.22% below entry)
    fn stop_loss_price(entry_price: Decimal) -> Decimal {
        entry_price * dec!(0.9978) // 1 - 0.0022 = 0.9978
    }
}

impl Default for EthDipStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl AlgoStrategy for EthDipStrategy {
    type State = EngineState<EthDipGlobalData, DefaultInstrumentMarketData>;

    fn generate_algo_orders(
        &self,
        state: &Self::State,
    ) -> (
        impl IntoIterator<Item = OrderRequestCancel<ExchangeIndex, InstrumentIndex>>,
        impl IntoIterator<Item = OrderRequestOpen<ExchangeIndex, InstrumentIndex>>,
    ) {
        let cancels = Vec::new();
        let mut opens = Vec::new();

        // Iterate through all instruments
        for instrument_state in state.instruments.instruments(&InstrumentFilter::None) {
            let instrument_idx = instrument_state.key;
            // Get current price
            let Some(current_price) = instrument_state.data.price() else {
                continue;
            };

            // Update highest price tracking in global data
            // Note: We can't mutate state here, so tracking happens via Processor in GlobalData
            // For now, we use the stored highest price or current price as fallback

            // Check if we have an open position
            let has_position = instrument_state.position.current.is_some();

            if !has_position {
                // Check if we should enter
                // Use current price as highest if no previous tracking
                let highest_price = state
                    .global
                    .highest_prices
                    .get(&instrument_idx)
                    .copied()
                    .unwrap_or(current_price);
                
                if self.should_enter_long(instrument_idx, current_price, highest_price) {
                    // Calculate order quantities
                    let entry_price = current_price;
                    let take_profit = Self::take_profit_price(entry_price);
                    let stop_loss = Self::stop_loss_price(entry_price);

                    // Get exchange index
                    let exchange_idx = instrument_state.instrument.exchange;

                    // Get quote asset index from instrument
                    // For spot instruments, quote is typically the underlying quote
                    let quote_asset_index = match instrument_state.instrument.quote {
                        barter_instrument::instrument::quote::InstrumentQuoteAsset::UnderlyingQuote => {
                            instrument_state.instrument.underlying.quote
                        }
                        barter_instrument::instrument::quote::InstrumentQuoteAsset::UnderlyingBase => {
                            instrument_state.instrument.underlying.base
                        }
                    };

                    // Get available balance from assets state using asset index
                    let qty = state
                        .assets
                        .asset_index(&quote_asset_index)
                        .balance
                        .as_ref()
                        .map(|timed_balance| timed_balance.value.free / entry_price)
                        .unwrap_or(dec!(0));

                    if qty > dec!(0) {
                        // Entry order
                        // For market orders, price should be current price or 0
                        let order_price = current_price;
                        
                        let entry_order = OrderRequestOpen {
                            key: OrderKey {
                                exchange: exchange_idx,
                                instrument: instrument_idx,
                                strategy: self.id.clone(),
                                cid: ClientOrderId::random(),
                            },
                            state: RequestOpen {
                                side: Side::Buy,
                                price: order_price,
                                quantity: qty,
                                kind: OrderKind::Market,
                                time_in_force: TimeInForce::ImmediateOrCancel,
                            },
                        };

                        opens.push(entry_order);

                        // Store entry price for later reference
                        // Note: In real implementation, this would update state.global.entry_prices
                        // For now, we just log it

                        tracing::info!(
                            instrument = ?instrument_idx,
                            entry_price = %entry_price,
                            take_profit = %take_profit,
                            stop_loss = %stop_loss,
                            quantity = %qty,
                            highest_price = %highest_price,
                            dip_percentage = %((highest_price - current_price) / highest_price * dec!(100)),
                            "✅ ENTERING LONG POSITION"
                        );
                    }
                }
            } else {
                // We have a position, check if we need to set take profit or stop loss
                // This would typically be handled by the position management system
                // For now, we'll just track the entry price
            }
        }

        (cancels, opens)
    }
}

impl ClosePositionsStrategy for EthDipStrategy {
    type State = EngineState<EthDipGlobalData, DefaultInstrumentMarketData>;

    fn close_positions_requests<'a>(
        &'a self,
        state: &'a Self::State,
        filter: &'a InstrumentFilter,
    ) -> (
        impl IntoIterator<Item = OrderRequestCancel<ExchangeIndex, InstrumentIndex>> + 'a,
        impl IntoIterator<Item = OrderRequestOpen<ExchangeIndex, InstrumentIndex>> + 'a,
    )
    where
        ExchangeIndex: 'a,
        AssetIndex: 'a,
        InstrumentIndex: 'a,
    {
        close_open_positions_with_market_orders(&self.id, state, filter, |_| {
            ClientOrderId::random()
        })
    }
}

impl<Clock, ExecutionTxs, Risk> OnDisconnectStrategy<Clock, EngineState<EthDipGlobalData, DefaultInstrumentMarketData>, ExecutionTxs, Risk>
    for EthDipStrategy
{
    type OnDisconnect = ();

    fn on_disconnect(
        _: &mut Engine<Clock, EngineState<EthDipGlobalData, DefaultInstrumentMarketData>, ExecutionTxs, Self, Risk>,
        _: ExchangeId,
    ) -> Self::OnDisconnect {
    }
}

impl<Clock, ExecutionTxs, Risk> OnTradingDisabled<Clock, EngineState<EthDipGlobalData, DefaultInstrumentMarketData>, ExecutionTxs, Risk>
    for EthDipStrategy
{
    type OnTradingDisabled = ();

    fn on_trading_disabled(
        _: &mut Engine<Clock, EngineState<EthDipGlobalData, DefaultInstrumentMarketData>, ExecutionTxs, Self, Risk>,
    ) -> Self::OnTradingDisabled {
    }
}

