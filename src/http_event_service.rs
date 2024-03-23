use async_graphql::Result;
use axum::{debug_handler, extract::State, http::StatusCode, Json};
use bson::{DateTime, Uuid};
use log::info;
use mongodb::Collection;
use serde::{Deserialize, Serialize};

use crate::{
    invoice::{InvoiceCreatedDTO, InvoiceDTO},
    order::{Order, OrderDTO, OrderStatus, RejectionReason},
};

/// Data to send to Dapr in order to describe a subscription.
#[derive(Serialize)]
pub struct Pubsub {
    #[serde(rename(serialize = "pubsubName"))]
    pub pubsubname: String,
    pub topic: String,
    pub route: String,
}

/// Reponse data to send to Dapr when receiving an event.
#[derive(Serialize)]
pub struct TopicEventResponse {
    pub status: u8,
}

/// Default status is `0` -> Ok, according to Dapr specs.
impl Default for TopicEventResponse {
    fn default() -> Self {
        Self { status: 0 }
    }
}

/// Relevant part of Dapr event wrapped in a CloudEnvelope.
#[derive(Deserialize, Debug)]
pub struct Event<T> {
    pub topic: String,
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct DiscountValidationSucceededEventData {
    /// Order for which the discount validation succeeded.
    order: OrderEventData,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderEventData {
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
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderItemEventData {
    /// OrderItem UUID.
    pub id: Uuid,
    /// Timestamp when OrderItem was created.
    pub created_at: DateTime,
    /// UUID of product variant associated with OrderItem.
    pub product_variant_id: Uuid,
    /// UUID of product variant version associated with OrderItem.
    pub product_variant_version_id: Uuid,
    /// UUID of tax rate version associated with OrderItem.
    pub tax_rate_version_id: Uuid,
    /// UUID of shopping cart item associated with OrderItem.
    pub shopping_cart_item_id: Uuid,
    /// Specifies the quantity of the OrderItem.
    pub count: u64,
    /// Total cost of product item, which can also be refunded.
    pub compensatable_amount: u64,
    /// UUID of shipment method of order item.
    pub shipment_method_id: Uuid,
    /// UUIDs of discounts applied to order item.
    pub discount_ids: Vec<Uuid>,
}

/// Service state containing database connections.
#[derive(Clone)]
pub struct HttpEventServiceState {
    pub order_collection: Collection<Order>,
}

/// HTTP endpoint to list topic subsciptions.
pub async fn list_topic_subscriptions() -> Result<Json<Vec<Pubsub>>, StatusCode> {
    let pubsub_order = Pubsub {
        pubsubname: "pubsub".to_string(),
        topic: "discount/order/validation-succeeded".to_string(),
        route: "/on-discount-validation-succeded".to_string(),
    };
    Ok(Json(vec![pubsub_order]))
}

/// HTTP endpoint to receive discount order validation succeeded events.
#[debug_handler(state = HttpEventServiceState)]
pub async fn on_discount_order_validation_succeeded_event(
    State(state): State<HttpEventServiceState>,
    Json(event): Json<Event<DiscountValidationSucceededEventData>>,
) -> Result<Json<TopicEventResponse>, StatusCode> {
    info!("{:?}", event);

    match event.topic.as_str() {
        "discount/order/validation-succeeded" => {
            let order = Order::from(event.data.order.clone());
            let order_dto = OrderDTO::from(event.data.order);
            let invoice_dto = InvoiceDTO::from(order.invoice.clone());
            let invoice_created_dto = InvoiceCreatedDTO::from((order_dto, invoice_dto));
            insert_order_in_mongodb(&state.order_collection, order).await?;
            send_invoice_created_event(invoice_created_dto).await?
        }
        _ => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
    Ok(Json(TopicEventResponse::default()))
}

/// Sends an `invoice/invoice/created` created event the order context with the invoice.
async fn send_invoice_created_event(
    invoice_created_dto: InvoiceCreatedDTO,
) -> Result<(), StatusCode> {
    let client = reqwest::Client::new();
    match client
        .post("http://localhost:3500/v1.0/publish/invoice/invoice/created")
        .json(&invoice_created_dto)
        .send()
        .await
    {
        Ok(_) => Ok(()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Inserts order in MongoDB.
pub async fn insert_order_in_mongodb(
    collection: &Collection<Order>,
    order: Order,
) -> Result<(), StatusCode> {
    match collection.insert_one(order, None).await {
        Ok(_) => Ok(()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
