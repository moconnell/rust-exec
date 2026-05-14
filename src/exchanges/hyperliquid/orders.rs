use hyperliquid_rust_sdk::{
    ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient as HlExchangeClient,
    ExchangeDataStatus, ExchangeResponseStatus,
};
use tracing::{info, warn};

use crate::order_state::{Order, OrderStatus, Side};

pub async fn place_order(
    exchange_client: Option<&HlExchangeClient>,
    order: Order,
) -> anyhow::Result<Order> {
    let exchange_client = exchange_client
        .ok_or_else(|| anyhow::anyhow!("order placement is disabled; set ALLOW_ORDER=true"))?;

    if order.price <= 0.0 {
        anyhow::bail!("order price must be positive");
    }

    if order.size <= 0.0 {
        anyhow::bail!("order size must be positive");
    }

    info!(
        symbol = %order.symbol,
        order_id = order.order_id,
        side = side_as_str(&order.side),
        price = order.price,
        size = order.size,
        "sending order to hyperliquid"
    );

    let response = exchange_client
        .order(
            ClientOrderRequest {
                asset: order.symbol.clone(),
                is_buy: order.side == Side::Buy,
                reduce_only: false,
                limit_px: order.price,
                sz: order.size,
                cloid: None,
                order_type: ClientOrder::Limit(ClientLimit {
                    tif: "Gtc".to_string(),
                }),
            },
            None,
        )
        .await?;

    let submitted_order = order_from_exchange_response(order, response)?;

    info!(
        symbol = %submitted_order.symbol,
        order_id = submitted_order.order_id,
        status = ?submitted_order.status,
        "hyperliquid order response received"
    );

    Ok(submitted_order)
}

fn order_from_exchange_response(
    mut order: Order,
    response: ExchangeResponseStatus,
) -> anyhow::Result<Order> {
    match response {
        ExchangeResponseStatus::Ok(response) => {
            let status = response
                .data
                .and_then(|data| data.statuses.into_iter().next())
                .unwrap_or(ExchangeDataStatus::Success);

            match status {
                ExchangeDataStatus::Success => {
                    order.status = OrderStatus::New;
                }
                ExchangeDataStatus::WaitingForFill | ExchangeDataStatus::WaitingForTrigger => {
                    order.status = OrderStatus::New;
                }
                ExchangeDataStatus::Resting(resting) => {
                    order.order_id = resting.oid;
                    order.status = OrderStatus::New;
                }
                ExchangeDataStatus::Filled(filled) => {
                    order.order_id = filled.oid;
                    order.status = OrderStatus::Filled;

                    if let Ok(avg_px) = filled.avg_px.parse::<f64>() {
                        order.price = avg_px;
                    }

                    if let Ok(total_sz) = filled.total_sz.parse::<f64>() {
                        order.size = total_sz;
                    }
                }
                ExchangeDataStatus::Error(reason) => {
                    warn!(
                        symbol = %order.symbol,
                        order_id = order.order_id,
                        reason,
                        "hyperliquid rejected order"
                    );
                    order.status = OrderStatus::Rejected { reason };
                }
            }

            Ok(order)
        }
        ExchangeResponseStatus::Err(reason) => {
            anyhow::bail!("hyperliquid order request failed: {reason}");
        }
    }
}

fn side_as_str(side: &Side) -> &'static str {
    match side {
        Side::Buy => "buy",
        Side::Sell => "sell",
    }
}
