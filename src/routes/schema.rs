// Schema routes
// Handles routes for database schema inspection

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use askama::Template;
use crate::services::schema_service;
use crate::AppState;

#[derive(Template)]
#[template(path = "components/schema-list.html")]
pub struct SchemaListTemplate {
    pub schemas: Vec<crate::models::Schema>,
}

/// Lists all schemas in the current database (returns HTML)
pub async fn list_schemas(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let schemas = schema_service::list_schemas(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let template = SchemaListTemplate { schemas };
    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
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
