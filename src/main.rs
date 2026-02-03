mod config;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;

#[cfg(test)]
mod security_tests;

#[cfg(test)]
mod routes_tests;

#[cfg(test)]
mod http_tests;

use axum::{
    extract::DefaultBodyLimit,
    middleware as axum_middleware,
    routing::{delete, get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
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
    tracing::info!(
        "Connecting to PostgreSQL at {}:{}/{}",
        config.postgres_host,
        config.postgres_port,
        config.postgres_db
    );

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

    // Create rate limiter
    let rate_limit_config = middleware::rate_limit::RateLimitConfig {
        requests_per_minute: config.rate_limit_requests_per_minute,
    };
    let rate_limit_state = Arc::new(middleware::rate_limit::RateLimitState::new(
        rate_limit_config,
    ));
    tracing::info!(
        "Rate limiting enabled: {} requests per minute per IP",
        config.rate_limit_requests_per_minute
    );

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
        .route("/health", get(routes::health_check))
        // Database routes
        .route("/api/databases", get(routes::database::list_databases))
        .route(
            "/api/databases/json",
            get(routes::database::list_databases_json),
        )
        .route(
            "/api/databases/{db_name}",
            get(routes::database::get_database),
        )
        .route(
            "/api/databases/create",
            post(routes::database::create_database),
        )
        .route("/api/databases/drop", post(routes::database::drop_database))
        // Schema routes
        .route("/api/schemas", get(routes::schema::list_schemas))
        .route("/api/schemas/{schema}", get(routes::schema::schema_details))
        // Table routes
        .route(
            "/api/schemas/{schema}/tables",
            get(routes::tables::list_tables),
        )
        .route(
            "/api/schemas/{schema}/tables/{table}",
            get(routes::tables::table_details),
        )
        .route(
            "/api/schemas/{schema}/tables/{table}/data",
            get(routes::tables::browse_data),
        )
        // Query routes
        .route("/api/query/execute", post(routes::query::execute))
        .route("/api/query/history", get(routes::query::history))
        .route("/api/query/history", delete(routes::query::clear_history))
        .route(
            "/api/query/history/stats",
            get(routes::query::history_stats),
        )
        .route("/api/query/export", post(routes::export::export_query))
        // Schema operations routes
        .route(
            "/api/schema/create-table",
            post(routes::schema_ops::create_table),
        )
        .route(
            "/api/schema/drop-object",
            post(routes::schema_ops::drop_object),
        )
        .route(
            "/api/schema/create-index",
            post(routes::schema_ops::create_index),
        )
        .route(
            "/api/schema/{schema}/tables",
            get(routes::schema_ops::list_tables),
        )
        .route(
            "/api/schema/{schema}/tables/{table}/columns",
            get(routes::schema_ops::get_table_columns),
        )
        // Statistics routes
        .route("/api/stats/database", get(routes::stats::database_stats))
        .route("/api/stats/tables", get(routes::stats::table_stats))
        .route("/api/stats/indexes", get(routes::stats::index_stats))
        .route("/api/stats/cache", get(routes::stats::cache_stats))
        .route(
            "/api/stats/overview",
            get(routes::stats::dashboard_metrics_widget),
        )
        .route(
            "/api/stats/table-stats-widget",
            get(routes::stats::table_stats_widget),
        )
        .route(
            "/api/stats/cache-stats-widget",
            get(routes::stats::cache_stats_widget),
        )
        // Query widget routes
        .route(
            "/api/query/recent-widget",
            get(routes::query::recent_queries_widget),
        )
        // Studio routes
        .route("/studio", get(routes::studio::studio_index))
        .route("/studio/{schema}", get(routes::studio::studio_schema))
        .route(
            "/studio/{schema}/{table}",
            get(routes::studio::studio_table),
        )
        .route(
            "/studio/{schema}/{table}/structure",
            get(routes::studio::studio_table_structure_page),
        )
        .route(
            "/api/studio/table/{schema}/{table}",
            get(routes::studio::studio_table_data),
        )
        .route(
            "/api/studio/structure/{schema}/{table}",
            get(routes::studio::studio_table_structure),
        )
        .route(
            "/api/studio/table/{schema}/{table}/indexes",
            get(routes::studio::studio_table_indexes),
        )
        // Cell editing routes
        .route("/api/cell/edit", get(routes::cell::get_cell_edit))
        .route("/api/cell/update", post(routes::cell::update_cell))
        .route(
            "/api/table/{schema}/{table}/row",
            post(routes::cell::add_row),
        )
        .route(
            "/api/table/{schema}/{table}/row/{pk_value}",
            delete(routes::cell::delete_row),
        )
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
        // Apply middleware layers in order (executed bottom-to-top)
        .layer(
            ServiceBuilder::new()
                .layer(axum_middleware::from_fn(
                    middleware::security_headers::security_headers,
                ))
                .layer(axum_middleware::from_fn_with_state(
                    rate_limit_state,
                    middleware::rate_limit::rate_limit_middleware,
                ))
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(DefaultBodyLimit::max(10 * 1024 * 1024)), // 10MB max body
        );

    // Parse the server address
    let addr: SocketAddr = match config.server_address.parse() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!(
                "Error: Invalid server address '{}': {}",
                config.server_address, e
            );
            std::process::exit(1);
        }
    };

    // Start the server
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                eprintln!("Error: Address {} is already in use", addr);
                eprintln!("Another process is already listening on this port.");
                eprintln!(
                    "Please stop the other process or use a different port in your configuration."
                );
            } else {
                eprintln!("Error: Failed to bind to address {}: {}", addr, e);
            }
            std::process::exit(1);
        }
    };

    tracing::info!("Server listening on {}", addr);

    // Serve with ConnectInfo to extract client IP for rate limiting
    if let Err(e) = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}
