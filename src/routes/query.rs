// Query execution routes
// Handles routes for executing SQL queries

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    Form,
    Json,
};
use serde::Deserialize;
use askama::Template;
use crate::services::query_service;
use crate::AppState;

#[derive(Deserialize)]
pub struct ExecuteQueryRequest {
    pub query: String,
}

#[derive(Template)]
#[template(path = "components/query-results.html")]
pub struct QueryResultsTemplate {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
    pub execution_time_ms: Option<u128>,
    pub error: Option<String>,
}

/// Executes a SQL query and returns results as HTML
pub async fn execute(
    State(state): State<AppState>,
    Form(payload): Form<ExecuteQueryRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Validate query
    if let Err(e) = query_service::validate_query(&payload.query) {
        let template = QueryResultsTemplate {
            columns: vec![],
            rows: vec![],
            row_count: 0,
            execution_time_ms: None,
            error: Some(e),
        };
        return match template.render() {
            Ok(html) => Ok(Html(html)),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        };
    }

    // Execute query
    match query_service::execute_query(&state.db_pool, &payload.query).await {
        Ok(result) => {
            let template = QueryResultsTemplate {
                columns: result.columns,
                rows: result.rows,
                row_count: result.row_count,
                execution_time_ms: result.execution_time_ms,
                error: None,
            };
            match template.render() {
                Ok(html) => Ok(Html(html)),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(e) => {
            let template = QueryResultsTemplate {
                columns: vec![],
                rows: vec![],
                row_count: 0,
                execution_time_ms: None,
                error: Some(e.to_string()),
            };
            match template.render() {
                Ok(html) => Ok(Html(html)),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

/// Gets query history (stored in session)
pub async fn history() -> impl IntoResponse {
    Json(serde_json::json!({
        "queries": []
    }))
}
