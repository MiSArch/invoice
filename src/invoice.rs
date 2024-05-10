use async_graphql::{Error, SimpleObject};
use bson::{DateTime, Uuid};
use serde::{Deserialize, Serialize};

use crate::{
    foreign_types::{User, UserAddress, VendorAddress},
    http_event_service::{HttpEventServiceState, OrderEventData},
    query::{
        project_user_to_user_address, query_object, query_user_address_user, query_vendor_address,
    },
};

static INVOICE_TERMS: &str = "This invoice is created according the the companies terms and conditions specified on the website.";

/// Invoice of an order.
#[derive(Debug, Serialize, Deserialize, SimpleObject, Clone)]
pub struct Invoice {
    pub _id: Uuid,
    pub order_id: Uuid,
    pub issued_at: DateTime,
    pub content: String,
    pub user_address: UserAddress,
    pub vendor_address: VendorAddress,
    pub vat_number: String,
}

impl Invoice {
    /// Creates a new invoice from OrderEventData and HttpEventServiceState (containing the database connections).
    pub async fn new(
        order_event_data: OrderEventData,
        state: &HttpEventServiceState,
    ) -> Result<Self, Error> {
        let _id = Uuid::new();
        let (
            issued_at,
            issued_at_string,
            order_item_invoice_overview,
            user_address,
            vendor_address,
            user,
        ) = invoice_attribute_setup(&order_event_data, state).await?;
        let content = format!(
            r#"
# Invoice

### Company information:
{}
{}, {}
{}, {}

VAT number: {}

### Customer information:
ID: {}
Name: {}, {}
Address:
{}
{}, {}
{}, {}

### Invoice ID: {}, issued at: {} 

Terms and conditions: {}

---

Purchased items overview:

{}

---

Total compensatable amount: {}
"#,
            vendor_address.company_name,
            vendor_address.street1,
            vendor_address.street2,
            vendor_address.city,
            vendor_address.country,
            order_event_data.vat_number,
            user._id,
            user.first_name,
            user.last_name,
            user_address.company_name,
            user_address.street1,
            user_address.street2,
            user_address.city,
            user_address.country,
            _id,
            issued_at_string,
            INVOICE_TERMS,
            order_item_invoice_overview,
            order_event_data.compensatable_order_amount
        );
        let invoice = Invoice {
            _id,
            order_id: order_event_data.id,
            issued_at,
            content: content,
            user_address,
            vendor_address,
            vat_number: order_event_data.vat_number,
        };
        Ok(invoice)
    }
}

/// Sets up all the attributes from OrderEventData and HttpEventServiceState (containing the database connections) that are required for invoice creation.
async fn invoice_attribute_setup(
    order_event_data: &OrderEventData,
    state: &HttpEventServiceState,
) -> Result<(DateTime, String, String, UserAddress, VendorAddress, User), Error> {
    let issued_at = DateTime::now();
    let issued_at_string = issued_at
        .to_chrono()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let order_item_invoice_overview = build_order_item_invoice_content(order_event_data);
    let user_address_user =
        query_user_address_user(&state.user_collection, order_event_data.invoice_address_id)
            .await?;
    let user_address = project_user_to_user_address(user_address_user)?;
    let vendor_address = query_vendor_address(&state.vendor_address_collection).await?;
    let user = query_object(&state.user_collection, order_event_data.user_id).await?;
    Ok((
        issued_at,
        issued_at_string,
        order_item_invoice_overview,
        user_address,
        vendor_address,
        user,
    ))
}

/// Builds the part of the invoice content which describes the order items as a markdown table.
fn build_order_item_invoice_content(value: &OrderEventData) -> String {
    let mut content = String::new();
    content.push_str("| Item UUID | Product variant UUID | count | Compensatable amount |\n");
    content.push_str("| --- | --- | --- | --- |\n");
    for item in &value.order_items {
        content.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            item.id, item.product_variant_id, item.count, item.compensatable_amount
        ));
    }
    content
}

/// DTO of an invoice for an order.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceDTO {
    pub order_id: Uuid,
    pub issued_at: chrono::DateTime<chrono::Utc>,
    pub content: String,
}

impl From<Invoice> for InvoiceDTO {
    fn from(value: Invoice) -> Self {
        InvoiceDTO {
            order_id: value.order_id,
            issued_at: value.issued_at.to_chrono(),
            content: value.content,
        }
    }
}

/// DTO which describes the event context on invoice creation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceCreatedDTO {
    pub order: OrderEventData,
    pub invoice: InvoiceDTO,
}

impl From<(OrderEventData, InvoiceDTO)> for InvoiceCreatedDTO {
    fn from((order, invoice): (OrderEventData, InvoiceDTO)) -> Self {
        Self { order, invoice }
    }
}
