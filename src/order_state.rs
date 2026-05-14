use std::clone;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::mpsc;
use tokio::sync::watch::Receiver;
use tracing::{debug, info, warn};

use crate::{config::Config, market_data::MarketState, weights::SymbolWeights};

static NEXT_ORDER_ID: AtomicU64 = AtomicU64::new(1);

#[derive(clone::Clone)]
pub struct Position {
    pub side: Side,
    pub symbol: String,
    pub size: f64,
    pub entry_px: f64,
    pub mark_px: f64,
    pub notional_usd: f64,
}

#[derive(clone::Clone)]
pub struct Order {
    pub status: OrderStatus,
    pub symbol: String,
    pub order_id: u64,
    pub side: Side,
    pub price: f64,
    pub size: f64,
}

impl Order {
    pub fn notional_usd(&self, market: &crate::market_data::MarketSnapshot) -> f64 {
        assert_eq!(self.symbol, market.symbol, "Market data symbol mismatch");
        let px = if self.side == Side::Buy {
            market.ask
        } else {
            market.bid
        };
        self.size * px
    }
}

#[derive(Debug, clone::Clone)]
pub enum OrderStatus {
    None,
    New,
    PartiallyFilled { filled_size: f64 },
    Filled,
    Canceled,
    Rejected { reason: String },
}

#[derive(Debug, clone::Clone, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

pub async fn run(
    config: Arc<Config>,
    market_rx: Receiver<MarketState>,
    order_tx: mpsc::Sender<Order>,
) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));

    loop {
        interval.tick().await;

        let weights = match crate::weights::get_target_weights(&config).await {
            Ok(weights) => weights,
            Err(err) => {
                warn!(
                    error = ?err,
                    "failed to retrieve target weights"
                );
                continue;
            }
        };

        debug!(
            weights = ?weights,
            "retrieved target weights"
        );

        let market_state = market_rx.borrow().clone();
        let orders = generate_orders(&config, &market_state, &weights);

        if orders.is_empty() {
            warn!("no orders generated from target weights");
            continue;
        }

        for order in orders {
            info!(
                symbol = %order.symbol,
                order_id = order.order_id,
                side = side_as_str(&order.side),
                price = order.price,
                size = order.size,
                "publishing proposed order"
            );

            if order_tx.send(order).await.is_err() {
                info!("order receiver dropped; stopping order producer");
                return;
            }
        }
    }
}

fn generate_orders(
    config: &Config,
    market_state: &MarketState,
    weights: &SymbolWeights,
) -> Vec<Order> {
    weights
        .iter()
        .filter_map(|(symbol, weight)| {
            if *weight <= 0.0 {
                return None;
            }

            let market = match market_state.get(symbol) {
                Some(market) => market,
                None => {
                    warn!(
                        symbol = %symbol,
                        weight,
                        "skipping order because market snapshot is unavailable"
                    );
                    return None;
                }
            };

            let price = market.ask;
            if price <= 0.0 {
                warn!(
                    symbol = %symbol,
                    weight,
                    price,
                    "skipping order because market ask is unavailable"
                );
                return None;
            }

            let notional_usd = config.max_order_usd * weight;
            let size = notional_usd / price;

            Some(Order {
                status: OrderStatus::New,
                symbol: symbol.clone(),
                order_id: NEXT_ORDER_ID.fetch_add(1, Ordering::Relaxed),
                side: Side::Buy,
                price,
                size,
            })
        })
        .collect()
}

fn side_as_str(side: &Side) -> &'static str {
    match side {
        Side::Buy => "buy",
        Side::Sell => "sell",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tokio::time::Instant;

    use super::*;
    use crate::market_data::{MarketSnapshot, MarketState};

    fn config(symbols: Vec<&str>) -> Config {
        Config {
            wallet_address: "0000000000000000000000000000000000000000".to_string(),
            private_key: None,
            symbols: symbols.into_iter().map(str::to_string).collect(),
            allow_order: false,
            use_mainnet: false,
            max_order_usd: 100.0,
            max_position_usd: 500.0,
            quote_asset: "USDC".to_string(),
            log_level: "info".to_string(),
        }
    }

    fn market_snapshot(symbol: &str, bid: f64, ask: f64) -> MarketSnapshot {
        MarketSnapshot {
            symbol: symbol.to_string(),
            bid,
            ask,
            mid: (bid + ask) / 2.0,
            received_at: Instant::now(),
        }
    }

    #[test]
    fn generate_orders_sizes_orders_from_weights_and_market_asks() {
        let config = config(vec!["ETH", "BTC"]);
        let mut market_state = MarketState::new();
        market_state.update(market_snapshot("ETH", 99.0, 100.0));
        market_state.update(market_snapshot("BTC", 199.0, 200.0));

        let weights = HashMap::from([("ETH".to_string(), 0.25), ("BTC".to_string(), 0.75)]);

        let mut orders = generate_orders(&config, &market_state, &weights);
        orders.sort_by(|left, right| left.symbol.cmp(&right.symbol));

        assert_eq!(orders.len(), 2);

        let btc = &orders[0];
        assert_eq!(btc.symbol, "BTC");
        assert_eq!(btc.side, Side::Buy);
        assert_eq!(btc.price, 200.0);
        assert_eq!(btc.size, 0.375);

        let eth = &orders[1];
        assert_eq!(eth.symbol, "ETH");
        assert_eq!(eth.side, Side::Buy);
        assert_eq!(eth.price, 100.0);
        assert_eq!(eth.size, 0.25);
    }

    #[test]
    fn generate_orders_skips_symbols_without_market_data() {
        let config = config(vec!["ETH", "BTC"]);
        let mut market_state = MarketState::new();
        market_state.update(market_snapshot("ETH", 99.0, 100.0));

        let weights = HashMap::from([("ETH".to_string(), 0.4), ("BTC".to_string(), 0.6)]);

        let orders = generate_orders(&config, &market_state, &weights);

        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].symbol, "ETH");
        assert_eq!(orders[0].size, 0.4);
    }

    #[test]
    fn generate_orders_skips_zero_or_negative_weights() {
        let config = config(vec!["ETH", "BTC"]);
        let mut market_state = MarketState::new();
        market_state.update(market_snapshot("ETH", 99.0, 100.0));
        market_state.update(market_snapshot("BTC", 199.0, 200.0));

        let weights = HashMap::from([("ETH".to_string(), 0.0), ("BTC".to_string(), -0.25)]);

        let orders = generate_orders(&config, &market_state, &weights);

        assert!(orders.is_empty());
    }
}
