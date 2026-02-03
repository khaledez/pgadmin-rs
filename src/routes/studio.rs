use askama::Template;
use axum::extract::{Path, Query, State};
use serde::Deserialize;

use crate::{
    models::ColumnInfo,
    routes::HtmlTemplate,
    services::{cell_service, schema_service},
    AppState,
};

#[derive(Template)]
#[template(path = "studio.html")]
pub struct StudioTemplate {
    pub schema_name: Option<String>,
    pub table_name: Option<String>,
    pub active_table: Option<String>,
    pub tables: Vec<crate::models::TableInfo>,
    pub views: Vec<crate::models::TableInfo>,
    pub active_view: String,
}

/// A row with its PK value for editing
pub struct EditableRow {
    pub pk_value: Option<String>,
    pub cells: Vec<serde_json::Value>,
}

#[derive(Template)]
#[template(path = "components/studio-data.html")]
pub struct StudioDataTemplate {
    pub schema: String,
    pub table: String,
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<EditableRow>,
    pub pagination: crate::models::Pagination,
    pub pk_column: Option<String>,
}

#[derive(Template)]
#[template(path = "components/studio-structure.html")]
pub struct StudioStructureTemplate {
    pub schema: String,
    pub table: String,
    pub columns: Vec<ColumnInfo>,
    pub row_count: i64,
}

#[derive(Template)]
#[template(path = "components/studio-indexes.html")]
pub struct StudioIndexesTemplate {
    pub indexes: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

/// GET /studio - Studio main page (default schema)
pub async fn studio_index(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    // Get tables from public schema by default
    let schema_name = "public".to_string();
    let all_tables = schema_service::list_tables(&state.db_pool, &schema_name)
        .await
        .unwrap_or_default();
    let (tables, views) = split_tables_and_views(all_tables);

    HtmlTemplate(StudioTemplate {
        schema_name: Some(schema_name),
        table_name: None,
        active_table: None,
        tables,
        views,
        active_view: "data".to_string(),
    })
}

/// GET /studio/:schema - Studio for a specific schema
pub async fn studio_schema(
    State(state): State<AppState>,
    Path(schema): Path<String>,
) -> impl axum::response::IntoResponse {
    let all_tables = schema_service::list_tables(&state.db_pool, &schema)
        .await
        .unwrap_or_default();
    let (tables, views) = split_tables_and_views(all_tables);

    HtmlTemplate(StudioTemplate {
        schema_name: Some(schema),
        table_name: None,
        active_table: None,
        tables,
        views,
        active_view: "data".to_string(),
    })
}

/// GET /studio/:schema/:table - Studio with a table selected
pub async fn studio_table(
    State(state): State<AppState>,
    Path((schema, table)): Path<(String, String)>,
) -> impl axum::response::IntoResponse {
    let all_tables = schema_service::list_tables(&state.db_pool, &schema)
        .await
        .unwrap_or_default();
    let (tables, views) = split_tables_and_views(all_tables);

    HtmlTemplate(StudioTemplate {
        schema_name: Some(schema),
        table_name: Some(table.clone()),
        active_table: Some(table),
        tables,
        views,
        active_view: "data".to_string(),
    })
}

/// GET /studio/:schema/:table/structure - Studio with structure selected
pub async fn studio_table_structure_page(
    State(state): State<AppState>,
    Path((schema, table)): Path<(String, String)>,
) -> impl axum::response::IntoResponse {
    let all_tables = schema_service::list_tables(&state.db_pool, &schema)
        .await
        .unwrap_or_default();
    let (tables, views) = split_tables_and_views(all_tables);

    HtmlTemplate(StudioTemplate {
        schema_name: Some(schema),
        table_name: Some(table.clone()),
        active_table: Some(table),
        tables,
        views,
        active_view: "structure".to_string(),
    })
}

fn split_tables_and_views(
    all_tables: Vec<crate::models::TableInfo>,
) -> (Vec<crate::models::TableInfo>, Vec<crate::models::TableInfo>) {
    let (views, tables): (Vec<_>, Vec<_>) =
        all_tables.into_iter().partition(|table| table.table_type == "VIEW");
    (tables, views)
}

/// GET /api/studio/table/:schema/:table - Get table data for studio (HTMX fragment)
pub async fn studio_table_data(
    State(state): State<AppState>,
    Path((schema, table)): Path<(String, String)>,
    Query(pagination): Query<PaginationQuery>,
) -> impl axum::response::IntoResponse {
    let page = pagination.page.unwrap_or(1);
    let page_size = pagination.page_size.unwrap_or(100);

    // Get columns
    let columns = schema_service::get_table_columns(&state.db_pool, &schema, &table)
        .await
        .unwrap_or_default();

    // Get primary key column for editing
    let pk_column = cell_service::get_primary_key_column(&state.db_pool, &schema, &table)
        .await
        .ok()
        .flatten();

    // Find PK column index
    let pk_idx = pk_column
        .as_ref()
        .and_then(|pk| columns.iter().position(|c| &c.name == pk));

    // Get data with pagination
    let (raw_rows, total_rows) =
        schema_service::get_table_data(&state.db_pool, &schema, &table, page, page_size)
            .await
            .unwrap_or_default();

    // Convert to EditableRow with PK values
    let rows: Vec<EditableRow> = raw_rows
        .into_iter()
        .map(|row| {
            // Extract PK value if we have a PK column
            let pk_value = pk_idx.and_then(|i| row.get(i).cloned()).flatten();

            // Convert cells to JSON values
            let cells: Vec<serde_json::Value> = row
                .into_iter()
                .map(|cell| match cell {
                    Some(s) => serde_json::Value::String(s),
                    None => serde_json::Value::Null,
                })
                .collect();

            EditableRow { pk_value, cells }
        })
        .collect();

    let total_pages = if total_rows > 0 {
        ((total_rows as f64) / (page_size as f64)).ceil() as u32
    } else {
        1
    };

    HtmlTemplate(StudioDataTemplate {
        schema,
        table,
        columns,
        rows,
        pagination: crate::models::Pagination {
            page,
            page_size,
            total_rows,
            total_pages,
        },
        pk_column,
    })
}

/// GET /api/studio/structure/:schema/:table - Get table structure for studio (HTMX fragment)
pub async fn studio_table_structure(
    State(state): State<AppState>,
    Path((schema, table)): Path<(String, String)>,
) -> impl axum::response::IntoResponse {
    let columns = schema_service::get_table_columns(&state.db_pool, &schema, &table)
        .await
        .unwrap_or_default();

    let row_count = schema_service::get_table_row_count(&state.db_pool, &schema, &table)
        .await
        .unwrap_or(0);

    HtmlTemplate(StudioStructureTemplate {
        schema,
        table,
        columns,
        row_count,
    })
}

/// GET /api/studio/table/:schema/:table/indexes - Get table indexes for studio (HTMX fragment)
pub async fn studio_table_indexes(
    State(state): State<AppState>,
    Path((schema, table)): Path<(String, String)>,
) -> impl axum::response::IntoResponse {
    let indexes = schema_service::get_table_indexes(&state.db_pool, &schema, &table)
        .await
        .unwrap_or_default();

    HtmlTemplate(StudioIndexesTemplate { indexes })
}
