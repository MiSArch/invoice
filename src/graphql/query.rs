use std::any::type_name;

use async_graphql::{Context, Error, Object, Result};

use bson::Uuid;
use mongodb::{bson::doc, Collection, Database};
use serde::Deserialize;

use super::model::{invoice::Invoice, order::Order};

/// Describes GraphQL invoice queries.
pub struct Query;

#[Object]
impl Query {
    /// Entity resolver for order of specific UUID.
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

    /// Entity resolver for invoice of specific UUID.
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

    /// Query for invoice of specific UUID.
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

/// Shared function to query an invoice by an order UUID.
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

/// Shared function to query an object: `T` from a MongoDB collection of object: `T`.
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
