mod config;
mod routes;
mod handlers;
mod services;
mod models;
mod middleware;

use axum::{
    routing::{get, post, delete},
    Router,
    extract::DefaultBodyLimit,
    middleware as axum_middleware,
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
    pub audit_logger: Arc<services::audit_service::AuditLogger>,
    pub query_history: Arc<services::query_history::QueryHistory>,
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
    tracing::info!("Connecting to PostgreSQL at {}:{}/{}", 
        config.postgres_host, config.postgres_port, config.postgres_db);

    // Create database pool
    let db_pool = match services::db_service::create_pool(&config).await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("\n❌ Failed to create database pool");
            eprintln!("Error: {}", e);
            eprintln!("\nConnection details:");
            eprintln!("  Host: {}", config.postgres_host);
            eprintln!("  Port: {}", config.postgres_port);
            eprintln!("  User: {}", config.postgres_user);
            eprintln!("  Database: {}", config.postgres_db);
            eprintln!("\nPlease check:");
            eprintln!("  1. PostgreSQL is running");
            eprintln!("  2. Host/port are correct in .env");
            eprintln!("  3. Username and password are correct");
            eprintln!("  4. Database exists (or use 'postgres' as default)");
            std::process::exit(1);
        }
    };

    // Test database connection
    if let Err(e) = services::db_service::test_connection(&db_pool).await {
        eprintln!("\n❌ Failed to connect to database");
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    tracing::info!("Connected to PostgreSQL database");

    // Create audit logger (stores last 1000 events)
    let audit_logger = Arc::new(services::audit_service::AuditLogger::new(1000));
    tracing::info!("Audit logging system initialized");

    // Create query history manager (stores last 500 queries)
    let query_history = Arc::new(services::query_history::QueryHistory::new(500));
    tracing::info!("Query history system initialized");

    let state = AppState {
        db_pool: Arc::new(db_pool),
        audit_logger: audit_logger.clone(),
        query_history: query_history.clone(),
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
        .route("/api/schemas/{schema}", get(routes::schema::schema_details))
        
        // Table routes
        .route("/api/schemas/{schema}/tables", get(routes::tables::list_tables))
        .route("/api/schemas/{schema}/tables/{table}", get(routes::tables::table_details))
        .route("/api/schemas/{schema}/tables/{table}/data", get(routes::tables::browse_data))
        
        // Query routes
        .route("/api/query/execute", post(routes::query::execute))
        .route("/api/query/history", get(routes::query::history))
        .route("/api/query/history", delete(routes::query::clear_history))
        .route("/api/query/history/stats", get(routes::query::history_stats))
        .route("/api/query/export", post(routes::export::export_query))
        
        // Schema operations routes
        .route("/api/schema/create-table", post(routes::schema_ops::create_table))
        .route("/api/schema/drop-object", post(routes::schema_ops::drop_object))
        .route("/api/schema/create-index", post(routes::schema_ops::create_index))
        .route("/api/schema/{schema}/tables", get(routes::schema_ops::list_tables))
        .route("/api/schema/{schema}/tables/{table}/columns", get(routes::schema_ops::get_table_columns))
        
        // Statistics routes
        .route("/api/stats/database", get(routes::stats::database_stats))
        .route("/api/stats/tables", get(routes::stats::table_stats))
        .route("/api/stats/indexes", get(routes::stats::index_stats))
        .route("/api/stats/cache", get(routes::stats::cache_stats))
        .route("/api/stats/overview", get(routes::stats::overview))
        
        .nest_service("/static", ServeDir::new("static"))
        // Apply middleware layers in order (executed bottom-to-top)
        .layer(axum_middleware::from_fn(middleware::security_headers::security_headers))
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
