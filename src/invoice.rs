use async_graphql::SimpleObject;
use bson::{DateTime, Uuid};
use serde::{Deserialize, Serialize};

use crate::{
    foreign_types::{User, VendorAddress},
    http_event_service::OrderEventData,
};

static INVOICE_TERMS: &str = "This invoice is created according the the companies terms and conditions specified on the website.";

/// Invoice of an order.
#[derive(Debug, Serialize, Deserialize, SimpleObject, Clone)]
pub struct Invoice {
    pub order_id: Uuid,
    pub issued_at: DateTime,
    pub content: String,
}

impl From<(OrderEventData, VendorAddress, User)> for Invoice {
    fn from(
        (order_event_data, vendor_address, user): (OrderEventData, VendorAddress, User),
    ) -> Self {
        let issued_at = DateTime::now();
        let issued_at_string = issued_at
            .to_chrono()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let order_item_invoice_overview = build_order_item_invoice_content(&order_event_data);
        let content = format!(
            r#"
# Invoice

### Company information:
{}
{}, {}
{}, {}

### Customer information:
ID: {}
Name: {}, {}

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
            user._id,
            user.first_name,
            user.last_name,
            order_event_data.id,
            issued_at_string,
            INVOICE_TERMS,
            order_item_invoice_overview,
            order_event_data.compensatable_order_amount
        );
        Invoice {
            order_id: order_event_data.id,
            issued_at,
            content: content,
        }
    }
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
    pub content: String,
}

impl From<Invoice> for InvoiceDTO {
    fn from(value: Invoice) -> Self {
        InvoiceDTO {
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
