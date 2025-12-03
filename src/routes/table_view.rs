// Table view routes
// Handles rendering table structure and data views

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use askama::Template;
use crate::services::schema_service;
use crate::AppState;

#[derive(Template)]
#[template(path = "table-view.html")]
pub struct TableViewTemplate {
    pub schema_name: String,
    pub table_name: String,
    pub columns: Vec<crate::models::ColumnInfo>,
    pub row_count: i64,
}

#[derive(Template)]
#[template(path = "table-view-content.html")]
pub struct TableViewContentTemplate {
    pub schema_name: String,
    pub table_name: String,
    pub columns: Vec<crate::models::ColumnInfo>,
    pub row_count: i64,
}

/// Renders the full table view page (for direct navigation)
pub async fn table_view(
    Path((schema_name, table_name)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let columns = schema_service::get_table_columns(&state.db_pool, &schema_name, &table_name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row_count = schema_service::get_table_row_count(&state.db_pool, &schema_name, &table_name)
        .await
        .unwrap_or(0);

    let template = TableViewTemplate {
        schema_name,
        table_name,
        columns,
        row_count,
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Renders only the table view content (for HTMX partial loading)
pub async fn table_view_content(
    Path((schema_name, table_name)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let columns = schema_service::get_table_columns(&state.db_pool, &schema_name, &table_name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row_count = schema_service::get_table_row_count(&state.db_pool, &schema_name, &table_name)
        .await
        .unwrap_or(0);

    let template = TableViewContentTemplate {
        schema_name,
        table_name,
        columns,
        row_count,
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Template)]
#[template(path = "components/table-indexes.html")]
pub struct TableIndexesTemplate {
    pub indexes: Vec<serde_json::Value>,
}

/// Gets indexes for a table (HTML)
pub async fn table_indexes(
    Path((schema_name, table_name)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let indexes = schema_service::get_table_indexes(&state.db_pool, &schema_name, &table_name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let template = TableIndexesTemplate { indexes };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
