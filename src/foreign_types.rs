use async_graphql::SimpleObject;
use bson::{doc, Bson, Uuid};
use serde::{Deserialize, Serialize};

use crate::http_event_service::{UserAddressEventData, UserEventData, VendorAddressEventData};

/// Foreign type of a user.
#[derive(Debug, Serialize, Deserialize, SimpleObject, Clone)]
#[graphql(unresolvable = "id")]
pub struct User {
    pub _id: Uuid,
    #[graphql(skip)]
    pub first_name: String,
    #[graphql(skip)]
    pub last_name: String,
    #[graphql(skip)]
    pub addresses: Vec<UserAddress>,
}

impl From<UserEventData> for User {
    fn from(value: UserEventData) -> Self {
        Self {
            _id: value.id,
            first_name: value.first_name,
            last_name: value.last_name,
            addresses: vec![],
        }
    }
}

/// Foreign type of a user address.
#[derive(Debug, Serialize, Deserialize, SimpleObject, Clone)]
#[graphql(unresolvable = "id")]
pub struct UserAddress {
    pub _id: Uuid,
    #[graphql(skip)]
    pub street1: String,
    #[graphql(skip)]
    pub street2: String,
    #[graphql(skip)]
    pub city: String,
    #[graphql(skip)]
    pub postal_code: String,
    #[graphql(skip)]
    pub country: String,
    #[graphql(skip)]
    pub company_name: String,
    #[graphql(skip)]
    pub user_id: Uuid,
}

impl From<UserAddressEventData> for UserAddress {
    fn from(value: UserAddressEventData) -> Self {
        Self {
            _id: value.id,
            street1: value.street1,
            street2: value.street2,
            city: value.city,
            postal_code: value.postal_code,
            country: value.country,
            company_name: value.company_name,
            user_id: value.user_id,
        }
    }
}

impl From<UserAddress> for Bson {
    fn from(value: UserAddress) -> Self {
        Bson::Document(doc!(
            "_id": value._id,
            "street1": value.street1,
            "street2": value.street2,
            "city": value.city,
            "postal_code": value.postal_code,
            "country": value.country,
            "company_name": value.company_name,
            "user_id": value.user_id
        ))
    }
}

/// Foreign type of a vendor address.
#[derive(Debug, Serialize, Deserialize, SimpleObject, Clone)]
#[graphql(unresolvable = "id")]
pub struct VendorAddress {
    pub _id: Uuid,
    #[graphql(skip)]
    pub street1: String,
    #[graphql(skip)]
    pub street2: String,
    #[graphql(skip)]
    pub city: String,
    #[graphql(skip)]
    pub postal_code: String,
    #[graphql(skip)]
    pub country: String,
    #[graphql(skip)]
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
