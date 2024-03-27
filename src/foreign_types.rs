use async_graphql::SimpleObject;
use bson::Uuid;
use serde::{Deserialize, Serialize};

use crate::http_event_service::EventData;
#[derive(Debug, Serialize, Deserialize, SimpleObject, Clone)]
pub struct VendorAddress {
    pub _id: Uuid,
}

impl From<EventData> for VendorAddress {
    fn from(value: EventData) -> Self {
        Self { _id: value.id }
    }
}
