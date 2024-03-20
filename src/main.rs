use std::{env, fs::File, io::Write};

use async_graphql::{
    extensions::Logger, http::GraphiQLSource, EmptyMutation, EmptySubscription, SDLExportOptions,
    Schema,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use authentication::AuthorizedUserHeader;
use axum::{
    extract::State,
    http::HeaderMap,
    response::{self, IntoResponse},
    routing::{get, post},
    Router, Server,
};
use clap::{arg, command, Parser};
use order::Order;
use simple_logger::SimpleLogger;

use log::info;
use mongodb::{options::ClientOptions, Client, Database};

mod invoice;

mod query;
use query::Query;
mod authentication;

mod http_event_service;
use http_event_service::{
    list_topic_subscriptions, on_discount_order_validation_succeeded_event, HttpEventServiceState,
};

mod order;

/// Builds the GraphiQL frontend.
async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/").finish())
}

/// Establishes database connection and returns the client.
async fn db_connection() -> Client {
    let uri = match env::var_os("MONGODB_URI") {
        Some(uri) => uri.into_string().unwrap(),
        None => panic!("$MONGODB_URI is not set."),
    };

    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse(uri).await.unwrap();

    // Manually set an option.
    client_options.app_name = Some("Invoice".to_string());

    // Get a handle to the deployment.
    Client::with_options(client_options).unwrap()
}

/// Returns Router that establishes connection to Dapr.
///
/// Adds endpoints to define pub/sub interaction with Dapr.
async fn build_dapr_router(db_client: Database) -> Router {
    let order_collection: mongodb::Collection<Order> = db_client.collection::<Order>("orders");

    // Define routes.
    let app = Router::new()
        .route("/dapr/subscribe", get(list_topic_subscriptions))
        .route(
            "/on-discount-validation-succeded",
            post(on_discount_order_validation_succeeded_event),
        )
        .with_state(HttpEventServiceState { order_collection });
    app
}

/// Command line argument to toggle schema generation instead of service execution.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Generates GraphQL schema in `./schemas/invoice.graphql`.
    #[arg(long)]
    generate_schema: bool,
}

/// Activates logger and parses argument for optional schema generation. Otherwise starts gRPC and GraphQL server.
#[tokio::main]
async fn main() -> std::io::Result<()> {
    SimpleLogger::new().init().unwrap();

    let args = Args::parse();
    if args.generate_schema {
        let schema = Schema::build(Query, EmptyMutation, EmptySubscription).finish();
        let mut file = File::create("./schemas/invoice.graphql")?;
        let sdl_export_options = SDLExportOptions::new().federation();
        let schema_sdl = schema.sdl_with_options(sdl_export_options);
        file.write_all(schema_sdl.as_bytes())?;
        info!("GraphQL schema: ./schemas/invoice.graphql was successfully generated!");
    } else {
        start_service().await;
    }
    Ok(())
}

/// Describes the handler for GraphQL requests.
///
/// Parses the "Authenticate-User" header and writes it in the context data of the specfic request.
/// Then executes the GraphQL schema with the request.
async fn graphql_handler(
    State(schema): State<Schema<Query, EmptyMutation, EmptySubscription>>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();
    if let Ok(authenticate_user_header) = AuthorizedUserHeader::try_from(&headers) {
        req = req.data(authenticate_user_header);
    }
    schema.execute(req).await.into()
}

/// Starts invoice service on port 8000.
async fn start_service() {
    let client = db_connection().await;
    let db_client: Database = client.database("invoice-database");

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .extension(Logger)
        .data(db_client.clone())
        .enable_federation()
        .finish();

    let graphiql = Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .with_state(schema);
    let dapr_router = build_dapr_router(db_client).await;
    let app = Router::new().merge(graphiql).merge(dapr_router);

    info!("GraphiQL IDE: http://0.0.0.0:8080");
    Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
