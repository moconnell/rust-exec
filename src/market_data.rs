use std::clone;
use std::collections::HashMap;
use std::sync::Arc;

use tokio::{sync::watch::Sender, time::Instant};

use crate::{config::Config, exchanges::ExchangeClient};

#[derive(clone::Clone)]
pub struct MarketSnapshot {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub mid: f64,
    pub received_at: Instant,
}

#[derive(clone::Clone)]
pub struct MarketState {
    snapshots: HashMap<String, MarketSnapshot>,
}

impl MarketState {
    pub fn new() -> Self {
        Self {
            snapshots: HashMap::new(),
        }
    }

    pub fn update(&mut self, snapshot: MarketSnapshot) {
        self.snapshots.insert(snapshot.symbol.clone(), snapshot);
    }

    pub fn get(&self, symbol: &str) -> Option<&MarketSnapshot> {
        self.snapshots.get(symbol)
    }
}

pub async fn run<E>(config: Arc<Config>, exchange: Arc<E>, market_tx: Sender<MarketState>)
where
    E: ExchangeClient + ?Sized,
{
    if let Err(err) = exchange
        .subscribe_market_data(config.symbols.clone(), market_tx)
        .await
    {
        eprintln!("market data stream stopped: {err:?}");
    }
}
