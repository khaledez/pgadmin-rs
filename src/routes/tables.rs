// Table management routes
// Handles routes for viewing and managing database tables

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use crate::models::{TableDataParams, Pagination};
use crate::services::schema_service;
use crate::AppState;

/// Lists all tables in a schema
pub async fn list_tables(
    Path(schema): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let tables = schema_service::list_tables(&state.db_pool, &schema)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(tables))
}

/// Gets details about a specific table
pub async fn table_details(
    Path((schema, table)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let table_info = schema_service::get_table_info(&state.db_pool, &schema, &table)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let columns = schema_service::get_table_columns(&state.db_pool, &schema, &table)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "table": table_info,
        "columns": columns
    })))
}

/// Browses table data with pagination
pub async fn browse_data(
    Path((schema, table)): Path<(String, String)>,
    Query(params): Query<TableDataParams>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(100);

    let (rows, total_rows) = schema_service::get_table_data(
        &state.db_pool,
        &schema,
        &table,
        page,
        page_size,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let columns = schema_service::get_table_columns(&state.db_pool, &schema, &table)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_pages = (total_rows as f64 / page_size as f64).ceil() as u32;

    let pagination = Pagination {
        page,
        page_size,
        total_rows,
        total_pages,
    };

    Ok(Json(serde_json::json!({
        "columns": columns,
        "rows": rows,
        "pagination": pagination
    })))
}
