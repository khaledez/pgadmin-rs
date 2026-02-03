use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{routes::HtmlTemplate, services::cell_service, AppState};

#[derive(Template)]
#[template(path = "components/cell-edit.html")]
pub struct CellEditTemplate {
    pub schema: String,
    pub table: String,
    pub column: String,
    pub pk_column: String,
    pub pk_value: String,
    pub value: Option<String>,
    pub data_type: String,
}

#[derive(Template)]
#[template(path = "components/cell-display.html")]
pub struct CellDisplayTemplate {
    pub schema: String,
    pub table: String,
    pub column: String,
    pub pk_column: String,
    pub pk_value: String,
    pub value: Option<String>,
    pub data_type: String,
}

#[derive(Deserialize)]
pub struct CellEditQuery {
    pub schema: String,
    pub table: String,
    pub column: String,
    pub pk_column: String,
    pub pk_value: String,
    pub data_type: Option<String>,
}

#[derive(Deserialize)]
pub struct CellUpdateRequest {
    pub schema: String,
    pub table: String,
    pub column: String,
    pub pk_column: String,
    pub pk_value: String,
    pub value: Option<String>,
    pub data_type: Option<String>,
}

#[derive(Serialize)]
pub struct CellUpdateResponse {
    pub success: bool,
    pub message: String,
}

/// GET /api/cell/edit - Get the edit form for a cell
pub async fn get_cell_edit(
    State(state): State<AppState>,
    Query(params): Query<CellEditQuery>,
) -> impl IntoResponse {
    // Get current value
    let value = cell_service::get_cell_value(
        &state.db_pool,
        &params.schema,
        &params.table,
        &params.pk_column,
        &params.pk_value,
        &params.column,
    )
    .await
    .ok()
    .flatten();

    HtmlTemplate(CellEditTemplate {
        schema: params.schema,
        table: params.table,
        column: params.column,
        pk_column: params.pk_column,
        pk_value: params.pk_value,
        value,
        data_type: params.data_type.unwrap_or_else(|| "text".to_string()),
    })
}

/// POST /api/cell/update - Update a cell value
pub async fn update_cell(
    State(state): State<AppState>,
    Json(request): Json<CellUpdateRequest>,
) -> Response {
    let result = cell_service::update_cell(
        &state.db_pool,
        &request.schema,
        &request.table,
        &request.pk_column,
        &request.pk_value,
        &request.column,
        request.value.as_deref(),
    )
    .await;

    match result {
        Ok(()) => {
            // Return the display template with updated value
            HtmlTemplate(CellDisplayTemplate {
                schema: request.schema,
                table: request.table,
                column: request.column,
                pk_column: request.pk_column,
                pk_value: request.pk_value,
                value: request.value,
                data_type: request.data_type.unwrap_or_else(|| "text".to_string()),
            })
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CellUpdateResponse {
                success: false,
                message: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// POST /api/table/:schema/:table/row - Add a new row
pub async fn add_row(
    State(state): State<AppState>,
    Path((schema, table)): Path<(String, String)>,
) -> impl IntoResponse {
    match cell_service::insert_row(&state.db_pool, &schema, &table).await {
        Ok(pk_value) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "pk_value": pk_value,
                "message": "Row added successfully"
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "message": e.to_string()
            })),
        ),
    }
}

/// DELETE /api/table/:schema/:table/row/:pk_value - Delete a row
pub async fn delete_row(
    State(state): State<AppState>,
    Path((schema, table, pk_value)): Path<(String, String, String)>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    // Get pk_column from query params or try to detect it
    let pk_column = match params.get("pk_column") {
        Some(col) => col.clone(),
        None => match cell_service::get_primary_key_column(&state.db_pool, &schema, &table).await {
            Ok(Some(col)) => col,
            _ => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "success": false,
                        "message": "Could not determine primary key column"
                    })),
                )
            }
        },
    };

    match cell_service::delete_row(&state.db_pool, &schema, &table, &pk_column, &pk_value).await {
        Ok(rows) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "rows_affected": rows,
                "message": format!("Deleted {} row(s)", rows)
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "message": e.to_string()
            })),
        ),
    }
}
