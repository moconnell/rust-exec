pub mod hyperliquid;

use tokio::sync::watch::Sender;

use crate::{
    market_data::{MarketSnapshot, MarketState},
    order_state::Order,
    wallet::WalletHolding,
};

#[async_trait::async_trait]
pub trait ExchangeClient: Send + Sync {
    async fn get_market_data(&self, symbol: &str) -> anyhow::Result<MarketSnapshot>;

    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        market_tx: Sender<MarketState>,
    ) -> anyhow::Result<()>;

    async fn get_wallet_holdings(&self) -> anyhow::Result<Vec<WalletHolding>>;

    async fn place_order(&self, order: Order) -> anyhow::Result<Order>;
}
