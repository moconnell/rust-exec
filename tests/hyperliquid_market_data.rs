use std::{sync::Arc, time::Duration};

use hyperliquid_rust_exec_poc::{
    config::Config,
    exchanges::{ExchangeClient, hyperliquid::HyperliquidClient},
    market_data::MarketState,
};
use tokio::{sync::watch, time::timeout};

fn test_config(symbols: Vec<String>) -> Arc<Config> {
    Arc::new(Config {
        wallet_address: "0000000000000000000000000000000000000000".to_string(),
        private_key: None,
        symbols,
        allow_order: false,
        use_mainnet: false,
        max_order_usd: 0.0,
        max_position_usd: 0.0,
        quote_asset: "USDC".to_string(),
        log_level: "info".to_string(),
    })
}

#[tokio::test]
#[ignore = "connects to Hyperliquid testnet"]
async fn retrieves_l2_snapshot_price() -> anyhow::Result<()> {
    let symbol = std::env::var("HL_TEST_SYMBOL").unwrap_or_else(|_| "ETH".to_string());
    let config = test_config(vec![symbol.clone()]);
    let client = HyperliquidClient::new(&config).await?;

    let snapshot = client.get_market_data(&symbol).await?;

    assert_eq!(snapshot.symbol, symbol);
    assert!(
        snapshot.bid > 0.0,
        "expected positive bid, got {}",
        snapshot.bid
    );
    assert!(
        snapshot.ask > 0.0,
        "expected positive ask, got {}",
        snapshot.ask
    );
    assert!(
        snapshot.mid > 0.0,
        "expected positive mid, got {}",
        snapshot.mid
    );

    Ok(())
}

#[tokio::test]
#[ignore = "connects to Hyperliquid testnet websocket"]
async fn streams_l2_book_price_into_market_state() -> anyhow::Result<()> {
    let symbol = std::env::var("HL_TEST_SYMBOL").unwrap_or_else(|_| "ETH".to_string());
    let config = test_config(vec![symbol.clone()]);
    let client = Arc::new(HyperliquidClient::new(&config).await?);
    let (market_tx, mut market_rx) = watch::channel(MarketState::new());

    let stream_client = Arc::clone(&client);
    let symbols = config.symbols.clone();
    let stream_task = tokio::spawn(async move {
        stream_client
            .subscribe_market_data(symbols, market_tx)
            .await
    });

    timeout(Duration::from_secs(15), market_rx.changed()).await??;

    let state = market_rx.borrow().clone();
    let snapshot = state
        .get(&symbol)
        .ok_or_else(|| anyhow::anyhow!("missing streamed snapshot for {symbol}"))?;

    assert!(
        snapshot.bid > 0.0,
        "expected positive bid, got {}",
        snapshot.bid
    );
    assert!(
        snapshot.ask > 0.0,
        "expected positive ask, got {}",
        snapshot.ask
    );
    assert!(
        snapshot.mid > 0.0,
        "expected positive mid, got {}",
        snapshot.mid
    );

    stream_task.abort();

    Ok(())
}
