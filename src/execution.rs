use crate::{
    config::Config,
    exchanges::ExchangeClient,
    market_data::MarketState,
    order_state::{Order, Side},
    risk::validate_order,
};
use std::sync::Arc;
use tokio::sync::{mpsc, watch::Receiver};
use tracing::{error, info, warn};

pub async fn run_loop(
    config: Arc<Config>,
    exchange: Arc<dyn ExchangeClient>,
    market_rx: Receiver<MarketState>,
    mut order_rx: mpsc::Receiver<Order>,
) -> anyhow::Result<()> {
    info!("execution loop starting");

    while let Some(order) = order_rx.recv().await {
        let side = side_as_str(&order.side);
        let market = market_rx.borrow().get(&order.symbol).cloned();
        let Some(market) = market else {
            warn!(
                symbol = %order.symbol,
                order_id = order.order_id,
                side,
                price = order.price,
                size = order.size,
                "rejecting order because market data is unavailable"
            );
            continue;
        };

        if let Err(reject) = validate_order(&order, &market, &config) {
            warn!(
                symbol = %order.symbol,
                order_id = order.order_id,
                side,
                price = order.price,
                size = order.size,
                bid = market.bid,
                ask = market.ask,
                mid = market.mid,
                reject_reason = ?reject,
                "order rejected by risk"
            );
            continue;
        }

        info!(
            symbol = %order.symbol,
            order_id = order.order_id,
            side,
            price = order.price,
            size = order.size,
            bid = market.bid,
            ask = market.ask,
            mid = market.mid,
            "submitting order"
        );

        match exchange.place_order(order.clone()).await {
            Ok(submitted_order) => {
                info!(
                    symbol = %submitted_order.symbol,
                    order_id = submitted_order.order_id,
                    side = side_as_str(&submitted_order.side),
                    price = submitted_order.price,
                    size = submitted_order.size,
                    "order submitted"
                );
            }
            Err(err) => {
                error!(
                    symbol = %order.symbol,
                    order_id = order.order_id,
                    side,
                    price = order.price,
                    size = order.size,
                    error = ?err,
                    "order submission failed"
                );
            }
        }
    }

    info!("order sender dropped; execution loop stopping");

    Ok(())
}

fn side_as_str(side: &Side) -> &'static str {
    match side {
        Side::Buy => "buy",
        Side::Sell => "sell",
    }
}
