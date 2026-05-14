use crate::{
    config::Config, exchanges::ExchangeClient, market_data::MarketState, order_state::OrderState,
};
use std::sync::Arc;
use tokio::sync::watch::Receiver;

pub async fn run_loop(
    config: Arc<Config>,
    exchange: Arc<dyn ExchangeClient>,
    market_rx: Receiver<MarketState>,
    order_rx: Receiver<OrderState>,
) -> anyhow::Result<()> {
    // , order_state: OrderState
    loop {
        // TODO: Check for new proposed orders from order_state, validate with risk::validate_order, and execute if valid
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
