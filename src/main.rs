mod config;
mod routes;

use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    // Build the application with routes
    let app = Router::new()
        .route("/", get(routes::index))
        .route("/health", get(routes::health_check))
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http());

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
