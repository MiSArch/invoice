use std::any::type_name;

use crate::{
    foreign_types::{User, UserAddress, VendorAddress},
    invoice::Invoice,
    order::Order,
};
use async_graphql::{Context, Error, Object, Result};

use bson::Uuid;
use mongodb::{bson::doc, options::FindOneOptions, Collection, Database};
use serde::Deserialize;

/// Describes GraphQL invoice queries.
pub struct Query;

#[Object]
impl Query {
    /// Entity resolver for order of specific id.
    #[graphql(entity)]
    async fn order_entity_resolver<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of order to retrieve.")] id: Uuid,
    ) -> Result<Order> {
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<Invoice> = db_client.collection::<Invoice>("invoices");
        let invoice = query_invoice_by_order_id(&collection, id).await?;
        let order = Order { _id: id, invoice };
        Ok(order)
    }

    /// Entity resolver for invoice of specific id.
    #[graphql(entity)]
    async fn invoice_entity_resolver<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of invoice to retrieve.")] id: Uuid,
    ) -> Result<Invoice> {
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<Invoice> = db_client.collection::<Invoice>("invoices");
        let invoice = query_object(&collection, id).await?;
        Ok(invoice)
    }

    /// Query for invoice of specific id.
    async fn invoice<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of invoice to retrieve.")] id: Uuid,
    ) -> Result<Invoice> {
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<Invoice> = db_client.collection::<Invoice>("invoices");
        let invoice = query_object(&collection, id).await?;
        Ok(invoice)
    }
}

/// Shared function to query an address from a MongoDB collection of users.
/// Returns User which only contains the queried address.
pub async fn query_user_address_user(
    collection: &mongodb::Collection<User>,
    address_id: Uuid,
) -> Result<User> {
    let find_options = FindOneOptions::builder()
        .projection(Some(doc! {
            "addresses.$": 1,
            "_id": 1
        }))
        .build();
    let message = format!("Address of UUID: `{}` not found.", address_id);
    match collection
        .find_one(
            doc! {"addresses": {
                "$elemMatch": {
                    "_id": address_id
                }
            }},
            Some(find_options),
        )
        .await
    {
        Ok(maybe_user) => maybe_user.ok_or(Error::new(message.clone())),
        Err(e) => Err(e.into()),
    }
}

/// Projects result of user address query, which is of type User, to the contained UserAddress.
pub fn project_user_to_user_address(user: User) -> Result<UserAddress> {
    let message = format!("Projection failed, address could not be extracted from user.");
    user.addresses
        .iter()
        .next()
        .cloned()
        .ok_or(Error::new(message.clone()))
}

/// Shared function to query the current vendor address.
pub async fn query_vendor_address(collection: &Collection<VendorAddress>) -> Result<VendorAddress> {
    collection
        .find_one(None, None)
        .await?
        .ok_or(Error::new("Vendor address is not set locally."))
}

/// Shared function to query an invoice by an order id.
pub async fn query_invoice_by_order_id(
    collection: &Collection<Invoice>,
    order_id: Uuid,
) -> Result<Invoice> {
    let message = format!("Invoice with order_id UUID: `{}` not found.", order_id);
    collection
        .find_one(doc! {"order_id": order_id }, None)
        .await?
        .ok_or(Error::new(message))
}

/// Shared function to query an object: T from a MongoDB collection of object: T.
///
/// * `connection` - MongoDB database connection.
/// * `id` - UUID of object.
pub async fn query_object<T: for<'a> Deserialize<'a> + Unpin + Send + Sync>(
    collection: &Collection<T>,
    id: Uuid,
) -> Result<T> {
    match collection.find_one(doc! {"_id": id }, None).await {
        Ok(maybe_object) => match maybe_object {
            Some(object) => Ok(object),
            None => {
                let message = format!("{} with UUID: `{}` not found.", type_name::<T>(), id);
                Err(Error::new(message))
            }
        },
        Err(_) => {
            let message = format!("{} with UUID: `{}` not found.", type_name::<T>(), id);
            Err(Error::new(message))
        }
    }
}
