use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletHolding {
    pub coin: String,
    pub total: String,
    pub entry_px: Option<String>,
    pub leverage: Option<String>,
    pub unrealized_pnl: Option<String>,
    pub realised_pnl: Option<String>,
    pub funding_unlocked: Option<String>,
    pub collateral: Option<String>,
}
