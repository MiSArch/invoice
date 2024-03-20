use std::any::type_name;

use crate::{authentication::authenticate_user, order::Order};
use async_graphql::{Context, Error, Object, Result};

use bson::Uuid;
use mongodb::{bson::doc, Collection, Database};
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
        let collection: Collection<Order> = db_client.collection::<Order>("orders");
        let order = query_object(&collection, id).await?;
        authenticate_user(&ctx, order.user_id)?;
        Ok(order)
    }
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
