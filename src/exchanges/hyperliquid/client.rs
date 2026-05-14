use std::sync::Arc;

use ethers::{signers::LocalWallet, types::H160};
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient as HlExchangeClient, InfoClient};
use tokio::sync::watch::Sender;

use crate::{
    config::Config,
    exchanges::ExchangeClient,
    market_data::{MarketSnapshot, MarketState},
    order_state::Order,
    wallet::WalletHolding,
};

use super::market_data;

pub struct HyperliquidClient {
    base_url: BaseUrl,
    info_client: InfoClient,
    exchange_client: Option<HlExchangeClient>,
    wallet_address: H160,
}

impl HyperliquidClient {
    pub async fn new(config: &Arc<Config>) -> anyhow::Result<Self> {
        let base_url = if config.use_mainnet {
            BaseUrl::Mainnet
        } else {
            BaseUrl::Testnet
        };

        let exchange_client = if config.allow_order {
            let wallet = config.private_key.as_ref().ok_or_else(|| {
                anyhow::anyhow!("PRIVATE_KEY is required when ALLOW_ORDER is true")
            })?;
            let wallet = wallet.parse::<LocalWallet>()?;
            let vault_address = None; // TODO: Add support for vault address if needed
            Some(HlExchangeClient::new(None, wallet, Some(base_url), None, vault_address).await?)
        } else {
            None
        };

        Ok(Self {
            base_url,
            info_client: InfoClient::new(None, Some(base_url)).await?,
            exchange_client,
            wallet_address: config.wallet_address.parse()?,
        })
    }
}

#[async_trait::async_trait]
impl ExchangeClient for HyperliquidClient {
    async fn get_market_data(&self, symbol: &str) -> anyhow::Result<MarketSnapshot> {
        let l2_snapshot = self.info_client.l2_snapshot(symbol.to_string()).await?;
        market_data::snapshot_from_l2_snapshot(&l2_snapshot)
    }

    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        market_tx: Sender<MarketState>,
    ) -> anyhow::Result<()> {
        market_data::subscribe(self.base_url, symbols, market_tx).await
    }

    async fn get_wallet_holdings(&self) -> anyhow::Result<Vec<WalletHolding>> {
        let address = self.wallet_address;
        let user_state = self.info_client.user_state(address).await?;

        Ok(user_state
            .asset_positions
            .into_iter()
            .map(|asset_position| {
                let position = asset_position.position;

                WalletHolding {
                    coin: position.coin,
                    total: position.szi,
                    entry_px: position.entry_px,
                    leverage: Some(position.leverage.value.to_string()),
                    unrealized_pnl: Some(position.unrealized_pnl),
                    realised_pnl: None,
                    funding_unlocked: Some(position.cum_funding.since_open),
                    collateral: Some(position.margin_used),
                }
            })
            .collect())
    }

    async fn place_order(&self, order: Order) -> anyhow::Result<Order> {
        Ok(order)
    }
}
