use hyperliquid_rust_exec_poc::config::Config;
use hyperliquid_rust_exec_poc::exchanges;
use hyperliquid_rust_exec_poc::execution;
use hyperliquid_rust_exec_poc::market_data::{self, MarketState};
use hyperliquid_rust_exec_poc::order_state;

use std::sync::Arc;

use tokio::sync::{mpsc, watch};

use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Arc::new(Config::from_env()?);
    init_tracing(&config);

    info!(
        symbols = ?config.symbols,
        allow_order = config.allow_order,
        use_mainnet = config.use_mainnet,
        "application starting"
    );

    let exchange = Arc::new(exchanges::hyperliquid::HyperliquidClient::new(&config).await?);
    let (market_tx, market_rx) = watch::channel(MarketState::new());
    let (order_tx, order_rx) = mpsc::channel(1024);

    tokio::spawn(market_data::run(
        Arc::clone(&config),
        Arc::clone(&exchange),
        market_tx,
    ));
    tokio::spawn(order_state::run(
        Arc::clone(&config),
        market_rx.clone(),
        order_tx,
    ));

    execution::run_loop(config, exchange, market_rx, order_rx).await?;

    Ok(())
}

fn init_tracing(config: &Config) {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.log_level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    fmt().with_env_filter(filter).json().init();
}
