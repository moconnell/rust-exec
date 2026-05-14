use std::clone;
use std::sync::Arc;

use crate::config::Config;

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

#[derive(clone::Clone)]
pub enum OrderStatus {
    None,
    New,
    PartiallyFilled { filled_size: f64 },
    Filled,
    Canceled,
    Rejected { reason: String },
}

#[derive(clone::Clone, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(clone::Clone)]
pub struct OrderState {
    // For simplicity, we just track the latest proposed order. In a real implementation, this would likely be a queue or more complex structure.
    pub proposed_order: Option<Order>,
}

impl OrderState {
    pub fn new() -> Self {
        Self {
            proposed_order: None,
        }
    }
}

pub async fn run(config: Arc<Config>, order_tx: tokio::sync::watch::Sender<OrderState>) {
    // TODO: Connect to Hyperliquid WebSocket to receive order updates and update order_state.proposed_order accordingly
}
