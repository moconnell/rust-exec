use ethers::types::H160;
use hyperliquid_rust_sdk::InfoClient;

use crate::wallet::WalletHolding;

pub async fn get_wallet_holdings(
    info_client: &InfoClient,
    wallet_address: H160,
) -> anyhow::Result<Vec<WalletHolding>> {
    let user_state = info_client.user_state(wallet_address).await?;

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
