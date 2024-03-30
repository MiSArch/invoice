use async_graphql::Result;
use axum::{debug_handler, extract::State, http::StatusCode, Json};
use bson::{doc, Uuid};
use log::info;
use mongodb::{options::UpdateOptions, Collection};
use serde::{Deserialize, Serialize};

use crate::{
    foreign_types::{User, UserAddress, VendorAddress},
    invoice::{Invoice, InvoiceCreatedDTO, InvoiceDTO},
    order::{OrderStatus, RejectionReason},
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

#[derive(Deserialize, Debug)]
pub struct VendorAddressEventData {
    /// Vendor address UUID.
    pub id: Uuid,
    /// First vendor address street field.
    pub street1: String,
    /// First vendor address street field.
    pub street2: String,
    /// Vendor city.
    pub city: String,
    /// Vendor postal code.
    pub postal_code: String,
    /// Country which vendor is located in.
    pub country: String,
    /// Name of vendor.
    pub company_name: String,
}

#[derive(Deserialize, Debug)]
pub struct UserEventData {
    /// User UUID.
    pub id: Uuid,
    /// First name of user.
    pub first_name: String,
    /// Last name of user.
    pub last_name: String,
}

// TODO: Optionals!
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserAddressEventData {
    /// Vendor address UUID.
    pub id: Uuid,
    /// First vendor address street field.
    pub street1: String,
    /// First vendor address street field.
    pub street2: String,
    /// Vendor city.
    pub city: String,
    /// Vendor postal code.
    pub postal_code: String,
    /// Country which vendor is located in.
    pub country: String,
    /// Name of vendor.
    pub company_name: String,
    /// User UUID.
    pub user_id: Uuid,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserAddressArchivedEventData {
    /// Vendor address UUID.
    pub id: Uuid,
    /// User UUID.
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct DiscountValidationSucceededEventData {
    /// Order for which the discount validation succeeded.
    order: OrderEventData,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderEventData {
    /// Order UUID.
    pub id: Uuid,
    /// UUID of user connected with Order.
    pub user_id: Uuid,
    /// Timestamp when Order was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// The status of the Order.
    pub order_status: OrderStatus,
    /// Timestamp of Order placement. `None` until Order is placed.
    pub placed_at: chrono::DateTime<chrono::Utc>,
    /// The rejection reason if status of the Order is `OrderStatus::Rejected`.
    pub rejection_reason: Option<RejectionReason>,
    /// OrderItems associated with the order.
    pub order_items: Vec<OrderItemEventData>,
    /// UUID of address to where the order should be shipped to.
    pub shipment_address_id: Uuid,
    /// UUID of address of invoice.
    pub invoice_address_id: Uuid,
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
    pub created_at: chrono::DateTime<chrono::Utc>,
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
    pub invoice_collection: Collection<Invoice>,
    pub vendor_address_collection: Collection<VendorAddress>,
    pub user_collection: Collection<User>,
}

/// HTTP endpoint to list topic subsciptions.
pub async fn list_topic_subscriptions() -> Result<Json<Vec<Pubsub>>, StatusCode> {
    let pubsub_order = Pubsub {
        pubsubname: "pubsub".to_string(),
        topic: "discount/order/validation-succeeded".to_string(),
        route: "/on-discount-validation-succeded".to_string(),
    };
    let pubsub_vendor_address = Pubsub {
        pubsubname: "pubsub".to_string(),
        topic: "address/vendor-address/created".to_string(),
        route: "/on-vendor-address-creation-event".to_string(),
    };
    let pubsub_user = Pubsub {
        pubsubname: "pubsub".to_string(),
        topic: "user/user/created".to_string(),
        route: "/on-id-creation-event".to_string(),
    };
    let pubsub_user_address = Pubsub {
        pubsubname: "pubsub".to_string(),
        topic: "address/user-address/created".to_string(),
        route: "/on-user-address-creation-event".to_string(),
    };
    let pubsub_user_address_archived = Pubsub {
        pubsubname: "pubsub".to_string(),
        topic: "address/user-address/archived".to_string(),
        route: "/on-user-address-archived-event".to_string(),
    };
    Ok(Json(vec![
        pubsub_order,
        pubsub_vendor_address,
        pubsub_user,
        pubsub_user_address,
        pubsub_user_address_archived,
    ]))
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
            let invoice = Invoice::new(event.data.order.clone(), &state)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let invoice_dto = InvoiceDTO::from(invoice.clone());
            let invoice_created_dto = InvoiceCreatedDTO::from((event.data.order, invoice_dto));
            insert_invoice_in_mongodb(&state.invoice_collection, invoice).await?;
            send_invoice_created_event(invoice_created_dto).await?
        }
        _ => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
    Ok(Json(TopicEventResponse::default()))
}

/// HTTP endpoint to receive vendor address creation events.
#[debug_handler(state = HttpEventServiceState)]
pub async fn on_vendor_address_created_event(
    State(state): State<HttpEventServiceState>,
    Json(event): Json<Event<VendorAddressEventData>>,
) -> Result<Json<TopicEventResponse>, StatusCode> {
    info!("{:?}", event);

    match event.topic.as_str() {
        "address/vendor-address/created" => {
            let vendor_address = VendorAddress::from(event.data);
            create_or_update_vendor_address_in_mongodb(
                &state.vendor_address_collection,
                vendor_address,
            )
            .await?
        }
        _ => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
    Ok(Json(TopicEventResponse::default()))
}

/// HTTP endpoint to receive user Address creation events.
#[debug_handler(state = HttpEventServiceState)]
pub async fn on_user_address_creation_event(
    State(state): State<HttpEventServiceState>,
    Json(event): Json<Event<UserAddressEventData>>,
) -> Result<Json<TopicEventResponse>, StatusCode> {
    info!("{:?}", event);

    match event.topic.as_str() {
        "address/user-address/created" => {
            let user_address = UserAddress::from(event.data);
            insert_user_address_in_mongodb(&state.user_collection, user_address).await?
        }
        _ => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
    Ok(Json(TopicEventResponse::default()))
}

/// HTTP endpoint to receive user Address archive events.
#[debug_handler(state = HttpEventServiceState)]
pub async fn on_user_address_archived_event(
    State(state): State<HttpEventServiceState>,
    Json(event): Json<Event<UserAddressArchivedEventData>>,
) -> Result<Json<TopicEventResponse>, StatusCode> {
    info!("{:?}", event);

    match event.topic.as_str() {
        "address/user-address/archived" => {
            remove_user_address_in_mongodb(&state.user_collection, event.data).await?
        }
        _ => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
    Ok(Json(TopicEventResponse::default()))
}

/// HTTP endpoint to receive user creation events.
#[debug_handler(state = HttpEventServiceState)]
pub async fn on_user_created_event(
    State(state): State<HttpEventServiceState>,
    Json(event): Json<Event<UserEventData>>,
) -> Result<Json<TopicEventResponse>, StatusCode> {
    info!("{:?}", event);

    match event.topic.as_str() {
        "user/user/created" => {
            create_user_in_mongodb(event.data, &state.user_collection).await?
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

/// Inserts invoice in MongoDB.
pub async fn insert_invoice_in_mongodb(
    collection: &Collection<Invoice>,
    invoice: Invoice,
) -> Result<(), StatusCode> {
    match collection.insert_one(invoice, None).await {
        Ok(_) => Ok(()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Create or update VendorAddress in MongoDB.
pub async fn create_or_update_vendor_address_in_mongodb(
    collection: &Collection<VendorAddress>,
    vendor_address: VendorAddress,
) -> Result<(), StatusCode> {
    let update_options = UpdateOptions::builder().upsert(true).build();
    match collection
        .update_one(
            doc! {"_id": vendor_address._id },
            doc! {"$set": {"_id": vendor_address._id}},
            update_options,
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Inserts user Address in MongoDB.
pub async fn insert_user_address_in_mongodb(
    collection: &Collection<User>,
    user_address: UserAddress,
) -> Result<(), StatusCode> {
    match collection
        .update_one(
            doc! {"_id": user_address.user_id },
            doc! {"$push": {"user_addresses": user_address }},
            None,
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Remove user Address in MongoDB.
pub async fn remove_user_address_in_mongodb(
    collection: &Collection<User>,
    user_address_event_data: UserAddressArchivedEventData,
) -> Result<(), StatusCode> {
    match collection
        .update_one(
            doc! {"_id": user_address_event_data.user_id },
            doc! {"$pull": {"user_addresses._id": user_address_event_data.id }},
            None,
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Create User in MongoDB.
async fn create_user_in_mongodb(
    user_event_data: UserEventData,
    collection: &Collection<User>,
) -> Result<(), StatusCode> {
    let user = User::from(user_event_data);
    match collection.insert_one(user, None).await {
        Ok(_) => Ok(()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
