// Schema operations routes
// Handles DDL operations like CREATE/DROP tables, views, indexes

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use crate::services::schema_ops_service::{
    SchemaOpsService, CreateTableRequest, DropObjectRequest, CreateIndexRequest,
};
use crate::AppState;

/// Create a new table
pub async fn create_table(
    State(state): State<AppState>,
    Json(payload): Json<CreateTableRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    SchemaOpsService::create_table(&state.db_pool, &payload)
        .await
        .map(|msg| Json(serde_json::json!({ "message": msg })))
        .map_err(|_| StatusCode::BAD_REQUEST)
}

/// Drop a table, view, or other object
pub async fn drop_object(
    State(state): State<AppState>,
    Json(payload): Json<DropObjectRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    SchemaOpsService::drop_object(&state.db_pool, &payload)
        .await
        .map(|msg| Json(serde_json::json!({ "message": msg })))
        .map_err(|_| StatusCode::BAD_REQUEST)
}

/// Create an index
pub async fn create_index(
    State(state): State<AppState>,
    Json(payload): Json<CreateIndexRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    SchemaOpsService::create_index(&state.db_pool, &payload)
        .await
        .map(|msg| Json(serde_json::json!({ "message": msg })))
        .map_err(|_| StatusCode::BAD_REQUEST)
}

/// List tables in a schema
pub async fn list_tables(
    State(state): State<AppState>,
    axum::extract::Path(schema): axum::extract::Path<String>,
) -> Result<Json<Vec<crate::services::schema_ops_service::TableInfo>>, StatusCode> {
    SchemaOpsService::list_tables(&state.db_pool, &schema)
        .await
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

/// Get table column definitions
pub async fn get_table_columns(
    State(state): State<AppState>,
    axum::extract::Path((schema, table)): axum::extract::Path<(String, String)>,
) -> Result<Json<Vec<crate::services::schema_ops_service::ColumnDef>>, StatusCode> {
    SchemaOpsService::get_table_columns(&state.db_pool, &schema, &table)
        .await
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}
