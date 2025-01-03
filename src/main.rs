mod bigquery;
mod models;
mod routes;
mod sync;

use axum::Router;
use bigquery::{BigQueryClient, BigQueryConfig};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::{env, net::SocketAddr, time::Duration};
use sync::SyncService;
use tokio::time;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Database connection
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&db_pool).await?;

    // Initialize BigQuery client
    let bq_config = BigQueryConfig {
        project_id: env::var("BIGQUERY_PROJECT_ID").expect("BIGQUERY_PROJECT_ID must be set"),
        credentials_path: env::var("BIGQUERY_CREDENTIALS_PATH")
            .expect("BIGQUERY_CREDENTIALS_PATH must be set"),
    };
    let bq_client = BigQueryClient::new(bq_config).await?;

    // Initialize sync service
    let sync_service = SyncService::new(db_pool.clone(), bq_client);

    // Perform initial sync
    if let Err(e) = sync_service.initial_sync().await {
        error!("Initial sync failed: {}", e);
    }

    // Start background sync task
    let sync_db = db_pool.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60)); // Sync every minute
        loop {
            interval.tick().await;

            // Get the latest block we've processed
            let latest_block =
                match sqlx::query!("SELECT MAX(block_timestamp) as last_block FROM proposals")
                    .fetch_one(&sync_db)
                    .await
                {
                    Ok(result) => result.last_block.unwrap_or(0),
                    Err(e) => {
                        error!("Failed to get latest block: {}", e);
                        continue;
                    }
                };

            if let Err(e) = sync_service.incremental_sync(latest_block).await {
                error!("Incremental sync failed: {}", e);
            }
        }
    });

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .merge(routes::create_router(db_pool))
        .layer(cors);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
