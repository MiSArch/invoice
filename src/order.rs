use async_graphql::{Enum, SimpleObject};
use bson::Uuid;
use serde::{Deserialize, Serialize};

use crate::{http_event_service::OrderEventData, invoice::Invoice};

/// Foreign type of an order.
#[derive(Debug, Serialize, Deserialize, SimpleObject, Clone)]
#[graphql(unresolvable)]
pub struct Order {
    /// UUID of the order.
    pub _id: Uuid,
    /// UUID of user connected with Order.
    #[graphql(skip)]
    pub user_id: Uuid,
    /// Invoice of the order.
    pub invoice: Invoice,
}

impl From<OrderEventData> for Order {
    fn from(value: OrderEventData) -> Self {
        let _id = value.id;
        let user_id = value.user_id;
        let invoice = Invoice::from(value);
        Order {
            _id,
            user_id,
            invoice,
        }
    }
}

/// Describes if Order is placed, or yet pending. An Order can be rejected during its lifetime.
#[derive(Debug, Enum, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Placed,
    Rejected,
}

/// Describes the reason why an Order was rejected, in case of rejection: `OrderStatus::Rejected`.
#[derive(Debug, Enum, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RejectionReason {
    InvalidOrderData,
    InventoryReservationFailed,
}
