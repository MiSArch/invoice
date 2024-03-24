use async_graphql::SimpleObject;
use bson::Uuid;
use serde::{Deserialize, Serialize};

use crate::http_event_service::OrderEventData;

/// Invoice of an order.
#[derive(Debug, Serialize, Deserialize, SimpleObject, Clone)]
pub struct Invoice {
    pub order_id: Uuid,
    pub content: String,
}

impl From<OrderEventData> for Invoice {
    fn from(value: OrderEventData) -> Self {
        let mut content = String::new();
        content.push_str(&format!("Invoice for Order {}\n", value.id));
        content.push_str(&format!("User UUID: {}\n", value.user_id));
        let created_at_string = value.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
        content.push_str(&format!("Created at: {}\n", created_at_string));
        let placed_at_string = value.placed_at.format("%Y-%m-%d %H:%M:%S").to_string();
        content.push_str(&format!("Placed at: {}\n", placed_at_string));
        content.push_str(&format!("Order Status: {:?}\n", value.order_status));
        if let Some(reason) = &value.rejection_reason {
            content.push_str(&format!("Rejection Reason: {:?}\n", reason));
        }
        build_order_item_invoice_content(&mut content, &value);
        content.push_str(&format!(
            "Total Compensatable Amount: {}\n",
            value.compensatable_order_amount
        ));
        content.push_str(&format!(
            "Payment Information UUID: {}\n",
            value.payment_information_id
        ));
        Invoice {
            order_id: value.id,
            content: content,
        }
    }
}

/// Builds the part of the invoice content which describes the order items.
fn build_order_item_invoice_content(content: &mut String, value: &OrderEventData) {
    content.push_str("\nOrder Items:\n");
    for item in &value.order_items {
        content.push_str(&format!("Item UUID: {}\n", item.id));
        content.push_str(&format!(
            "Product Variant UUID: {}\n",
            item.product_variant_id
        ));
        content.push_str(&format!("Count: {}\n", item.count));
        content.push_str(&format!(
            "Compensatable Amount: {}\n",
            item.compensatable_amount
        ));
        content.push_str("\n");
    }
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
