use hyperliquid_rust_sdk::{
    BaseUrl, InfoClient, L2BookData, L2SnapshotResponse, Message, Subscription,
};
use tokio::{
    sync::{mpsc::unbounded_channel, watch::Sender},
    time::Instant,
};
use tracing::{debug, info};

use crate::market_data::{MarketSnapshot, MarketState};

pub fn snapshot_from_l2_snapshot(snapshot: &L2SnapshotResponse) -> anyhow::Result<MarketSnapshot> {
    let bid_px = snapshot
        .levels
        .first()
        .and_then(|levels| levels.first())
        .map(|level| level.px.as_str());
    let ask_px = snapshot
        .levels
        .get(1)
        .and_then(|levels| levels.first())
        .map(|level| level.px.as_str());

    snapshot_from_prices(&snapshot.coin, bid_px, ask_px)
}

pub fn snapshot_from_l2_data(data: &L2BookData) -> anyhow::Result<MarketSnapshot> {
    let bid_px = data
        .levels
        .first()
        .and_then(|levels| levels.first())
        .map(|level| level.px.as_str());
    let ask_px = data
        .levels
        .get(1)
        .and_then(|levels| levels.first())
        .map(|level| level.px.as_str());

    snapshot_from_prices(&data.coin, bid_px, ask_px)
}

pub async fn subscribe(
    base_url: BaseUrl,
    symbols: Vec<String>,
    market_tx: Sender<MarketState>,
) -> anyhow::Result<()> {
    info!(
        symbol_count = symbols.len(),
        "opening hyperliquid market data stream"
    );

    let mut info_client = InfoClient::new(None, Some(base_url)).await?;
    let (sender, mut receiver) = unbounded_channel();

    for symbol in symbols {
        info!(
            symbol = %symbol,
            "subscribing to hyperliquid l2 book"
        );

        info_client
            .subscribe(Subscription::L2Book { coin: symbol }, sender.clone())
            .await?;
    }

    drop(sender);

    let mut market_state = market_tx.borrow().clone();

    while let Some(message) = receiver.recv().await {
        let Message::L2Book(l2_book) = message else {
            continue;
        };

        market_state.update(snapshot_from_l2_data(&l2_book.data)?);

        debug!(
            symbol = %l2_book.data.coin,
            "received hyperliquid l2 book update"
        );

        if market_tx.send(market_state.clone()).is_err() {
            info!("market data receiver dropped");
            break;
        }
    }

    info!("hyperliquid market data stream closed");

    Ok(())
}

fn snapshot_from_prices(
    symbol: &str,
    bid_px: Option<&str>,
    ask_px: Option<&str>,
) -> anyhow::Result<MarketSnapshot> {
    let bid = bid_px
        .map(|px| px.parse::<f64>())
        .transpose()?
        .unwrap_or(0.0);
    let ask = ask_px
        .map(|px| px.parse::<f64>())
        .transpose()?
        .unwrap_or(0.0);

    Ok(MarketSnapshot {
        symbol: symbol.to_string(),
        bid,
        ask,
        mid: if bid > 0.0 && ask > 0.0 {
            (bid + ask) / 2.0
        } else {
            0.0
        },
        received_at: Instant::now(),
    })
}
