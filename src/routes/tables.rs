// Table management routes
// Handles routes for viewing and managing database tables

use crate::models::{ColumnInfo, Pagination, TableDataParams};
use crate::services::schema_service;
use crate::AppState;
use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "components/tables-list.html")]
pub struct TablesListTemplate {
    pub tables: Vec<crate::models::TableInfo>,
}

#[derive(Template)]
#[template(path = "components/table-display.html")]
pub struct TableDisplayTemplate {
    pub table: crate::models::TableInfo,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Template)]
#[template(path = "components/table-data.html")]
pub struct TableDataTemplate {
    pub schema: String,
    pub table: String,
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub pagination: Pagination,
}

/// Lists all tables in a schema (returns HTML)
pub async fn list_tables(
    Path(schema): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let tables = schema_service::list_tables(&state.db_pool, &schema)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let template = TablesListTemplate { tables };
    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Gets details about a specific table (returns HTML)
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

    let template = TableDisplayTemplate {
        table: table_info,
        columns,
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Browses table data with pagination (returns HTML)
pub async fn browse_data(
    Path((schema, table)): Path<(String, String)>,
    Query(params): Query<TableDataParams>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(100);

    let (rows, total_rows) =
        schema_service::get_table_data(&state.db_pool, &schema, &table, page, page_size)
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

    // Convert rows to JSON values
    let json_rows: Vec<Vec<serde_json::Value>> = rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|cell| match cell {
                    Some(s) => serde_json::Value::String(s.clone()),
                    None => serde_json::Value::Null,
                })
                .collect()
        })
        .collect();

    let template = TableDataTemplate {
        schema,
        table,
        columns,
        rows: json_rows,
        pagination,
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
