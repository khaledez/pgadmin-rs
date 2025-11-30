// Query execution routes
// Handles routes for executing SQL queries

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use crate::services::query_service;
use crate::AppState;

#[derive(Deserialize)]
pub struct ExecuteQueryRequest {
    pub query: String,
}

/// Executes a SQL query and returns results
pub async fn execute(
    State(state): State<AppState>,
    Json(payload): Json<ExecuteQueryRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Validate query
    query_service::validate_query(&payload.query)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Execute query
    let result = query_service::execute_query(&state.db_pool, &payload.query)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

/// Gets query history (stored in session)
pub async fn history() -> impl IntoResponse {
    Json(serde_json::json!({
        "queries": []
    }))
}
