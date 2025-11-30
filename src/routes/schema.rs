// Schema routes
// Handles routes for database schema inspection

use axum::{
    http::StatusCode,
    response::IntoResponse,
};

/// Placeholder for schema listing endpoint
/// Will be implemented in future iterations with:
/// - Schema enumeration
/// - Schema details
/// - Object counts per schema
pub async fn list() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Schema listing endpoint - Coming soon")
}

/// Placeholder for schema details endpoint
pub async fn details() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Schema details endpoint - Coming soon")
}
