// Schema routes
// Handles routes for database schema inspection

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use crate::services::schema_service;
use crate::AppState;

/// Lists all schemas in the current database
pub async fn list_schemas(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let schemas = schema_service::list_schemas(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(schemas))
}

/// Gets details about a specific schema
pub async fn schema_details(
    Path(schema_name): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let tables = schema_service::list_tables(&state.db_pool, &schema_name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "name": schema_name,
        "tables": tables
    })))
}
