use bson::Uuid;
use serde::Serialize;

use crate::graphql::model::invoice::Invoice;

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
