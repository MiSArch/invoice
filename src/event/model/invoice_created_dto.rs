use serde::Serialize;

use super::{super::http_event_service::OrderEventData, invoice_dto::InvoiceDTO};

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
