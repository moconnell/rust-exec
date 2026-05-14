use hyperliquid_rust_exec_poc::config::Config;
use hyperliquid_rust_exec_poc::exchanges;
use hyperliquid_rust_exec_poc::execution;
use hyperliquid_rust_exec_poc::market_data::{self, MarketState};
use hyperliquid_rust_exec_poc::order_state::{self, OrderState};
// use crate::risk::validate_order

use std::sync::Arc;

use tokio::sync::watch::channel;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Arc::new(Config::from_env()?);
    // telemetry::init(&config)?;

    let exchange = Arc::new(exchanges::hyperliquid::HyperliquidClient::new(&config).await?);
    let (market_tx, market_rx) = channel(MarketState::new());
    let (order_tx, order_rx) = channel(OrderState::new());

    tokio::spawn(market_data::run(
        Arc::clone(&config),
        Arc::clone(&exchange),
        market_tx,
    ));
    tokio::spawn(order_state::run(Arc::clone(&config), order_tx));

    execution::run_loop(config, exchange, market_rx, order_rx).await?;

    Ok(())
}
