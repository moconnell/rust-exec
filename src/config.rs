use std::clone;

#[derive(clone::Clone)]
pub struct Config {
    pub hl_base_url: String,
    pub wallet_address: String,
    pub private_key: Option<String>,

    pub symbols: Vec<String>,
    pub allow_order: bool,
    pub use_mainnet: bool,

    pub max_order_usd: f64,
    pub max_position_usd: f64,
    pub quote_asset: String,

    pub log_level: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let hl_base_url = std::env::var("HL_BASE_URL")?;
        let wallet_address = std::env::var("WALLET_ADDRESS")?;
        let private_key = std::env::var("PRIVATE_KEY").ok();

        let symbols = std::env::var("SYMBOLS")?
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let allow_order = std::env::var("ALLOW_ORDER")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let use_mainnet = std::env::var("USE_MAINNET")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let max_order_usd = std::env::var("MAX_ORDER_USD")?.parse()?;
        let max_position_usd = std::env::var("MAX_POSITION_USD")?.parse()?;
        let quote_asset = std::env::var("QUOTE_ASSET").unwrap_or_else(|_| "USD".to_string());

        let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        Ok(Self {
            hl_base_url,
            wallet_address,
            private_key,
            symbols,
            allow_order,
            use_mainnet,
            max_order_usd,
            max_position_usd,
            quote_asset,
            log_level,
        })
    }
}
