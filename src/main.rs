mod config;
mod routes;
mod handlers;
mod services;
mod models;
mod middleware;

use axum::{
    routing::{get, post},
    Router,
    extract::DefaultBodyLimit,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{
    services::ServeDir,
    trace::TraceLayer,
    cors::CorsLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<sqlx::Pool<sqlx::Postgres>>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing for logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pgadmin_rs=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = config::Config::from_env();

    tracing::info!("Starting pgAdmin-rs server on {}", config.server_address);

    // Create database pool
    let db_pool = services::db_service::create_pool(&config)
        .await
        .expect("Failed to create database pool");

    // Test database connection
    services::db_service::test_connection(&db_pool)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Connected to PostgreSQL database");

    let state = AppState {
        db_pool: Arc::new(db_pool),
    };

    // Build the application with routes
    let app = Router::new()
        // Web pages
        .route("/", get(routes::index))
        .route("/query", get(routes::page_query))
        .route("/browser", get(routes::page_browser))
        .route("/health", get(routes::health_check))
        
        // Schema routes
        .route("/api/schemas", get(routes::schema::list_schemas))
        .route("/api/schemas/:schema", get(routes::schema::schema_details))
        
        // Table routes
        .route("/api/schemas/:schema/tables", get(routes::tables::list_tables))
        .route("/api/schemas/:schema/tables/:table", get(routes::tables::table_details))
        .route("/api/schemas/:schema/tables/:table/data", get(routes::tables::browse_data))
        
        // Query routes
        .route("/api/query/execute", post(routes::query::execute))
        .route("/api/query/history", get(routes::query::history))
        
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB max body
        .with_state(state);

    // Parse the server address
    let addr: SocketAddr = config.server_address.parse()
        .expect("Invalid server address");

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
