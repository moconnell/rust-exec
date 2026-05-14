use std::time::Duration;

use crate::{config::Config, market_data::MarketSnapshot, order_state::Order};

pub fn validate_order(
    order: &Order,
    market: &MarketSnapshot,
    // account: &AccountState,
    config: &Config,
) -> Result<(), RiskReject> {
    if order.notional_usd(market) > config.max_order_usd {
        return Err(RiskReject::OrderTooLarge);
    }

    if market.received_at.elapsed() > Duration::from_secs(5) {
        return Err(RiskReject::StaleMarketData);
    }

    Ok(())
}

#[derive(Debug)]
pub enum RiskReject {
    OrderTooLarge,
    StaleMarketData,
}
