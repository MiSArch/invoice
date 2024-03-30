use async_graphql::SimpleObject;
use bson::Uuid;
use serde::{Deserialize, Serialize};

use crate::http_event_service::{UserEventData, VendorAddressEventData};
#[derive(Debug, Serialize, Deserialize, SimpleObject, Clone)]
pub struct VendorAddress {
    pub _id: Uuid,
    pub street1: String,
    pub street2: String,
    pub city: String,
    pub postal_code: String,
    pub country: String,
    pub company_name: String,
}

impl From<VendorAddressEventData> for VendorAddress {
    fn from(value: VendorAddressEventData) -> Self {
        Self {
            _id: value.id,
            street1: value.street1,
            street2: value.street2,
            city: value.city,
            postal_code: value.postal_code,
            country: value.country,
            company_name: value.company_name,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, SimpleObject, Clone)]
pub struct User {
    pub _id: Uuid,
    pub first_name: String,
    pub last_name: String,
}

impl From<UserEventData> for User {
    fn from(value: UserEventData) -> Self {
        Self {
            _id: value.id,
            first_name: value.first_name,
            last_name: value.last_name,
        }
    }
}
