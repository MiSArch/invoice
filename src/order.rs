use async_graphql::{Enum, SimpleObject};
use bson::{DateTime, Uuid};
use serde::{Deserialize, Serialize};

use crate::{
    http_event_service::{OrderEventData, OrderItemEventData},
    invoice::{Invoice, InvoiceDTO},
};

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

/// DTO of an order of a user.
///
/// Includes invoice created by this service.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderDTO {
    /// Order UUID.
    pub id: Uuid,
    /// UUID of user connected with Order.
    pub user_id: Uuid,
    /// Timestamp when Order was created.
    pub created_at: DateTime,
    /// The status of the Order.
    pub order_status: OrderStatus,
    /// Timestamp of Order placement. `None` until Order is placed.
    pub placed_at: Option<DateTime>,
    /// The rejection reason if status of the Order is `OrderStatus::Rejected`.
    pub rejection_reason: Option<RejectionReason>,
    /// OrderItems associated with the order.
    pub order_items: Vec<OrderItemEventData>,
    /// Total compensatable amount of order.
    pub compensatable_order_amount: u64,
    /// UUID of payment information that the order should be processed with.
    pub payment_information_id: Uuid,
    /// Invoice of order.
    pub invoice: InvoiceDTO,
}

impl From<(OrderEventData, Invoice)> for OrderDTO {
    fn from((order_event_data, invoice): (OrderEventData, Invoice)) -> Self {
        let invoice_dto = InvoiceDTO::from(invoice);
        Self {
            id: order_event_data.id,
            user_id: order_event_data.user_id,
            created_at: order_event_data.created_at,
            order_status: order_event_data.order_status,
            placed_at: order_event_data.placed_at,
            rejection_reason: order_event_data.rejection_reason,
            order_items: order_event_data.order_items,
            compensatable_order_amount: order_event_data.compensatable_order_amount,
            payment_information_id: order_event_data.payment_information_id,
            invoice: invoice_dto,
        }
    }
}
