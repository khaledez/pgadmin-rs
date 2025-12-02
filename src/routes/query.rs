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
use std::time::Instant;
use crate::services::query_service;
use crate::services::query_history::HistoryEntry;
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
    let start = Instant::now();
    let query = payload.query.clone();

    // Validate query
    if let Err(e) = query_service::validate_query(&query) {
        let duration = start.elapsed().as_millis() as u64;
        let entry = HistoryEntry::failed(query, duration, e.clone());
        let history = state.query_history.clone();
        // Record failed validation asynchronously
        tokio::spawn(async move {
            history.add(entry).await;
        });

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
    match query_service::execute_query(&state.db_pool, &query).await {
        Ok(result) => {
            let duration = start.elapsed().as_millis() as u64;
            let row_count = Some(result.row_count as i64);
            let entry = HistoryEntry::new(query, duration, row_count);
            let history = state.query_history.clone();
            // Record successful query asynchronously
            tokio::spawn(async move {
                history.add(entry).await;
            });

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
            let duration = start.elapsed().as_millis() as u64;
            let error_msg = e.to_string();
            let entry = HistoryEntry::failed(query, duration, error_msg.clone());
            let history = state.query_history.clone();
            // Record failed query asynchronously
            tokio::spawn(async move {
                history.add(entry).await;
            });

            let template = QueryResultsTemplate {
                columns: vec![],
                rows: vec![],
                row_count: 0,
                execution_time_ms: None,
                error: Some(error_msg),
            };
            match template.render() {
                Ok(html) => Ok(Html(html)),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

/// Gets recent query history
pub async fn history(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let entries = state.query_history.get_recent(20).await;
    Json(entries)
}

/// Clears all query history
pub async fn clear_history(
    State(state): State<AppState>,
) -> impl IntoResponse {
    state.query_history.clear().await;
    Json(serde_json::json!({
        "status": "success",
        "message": "Query history cleared"
    }))
}

/// Gets query history statistics
pub async fn history_stats(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let stats = state.query_history.stats().await;
    Json(stats)
}

#[derive(Template)]
#[template(path = "components/recent-queries.html")]
pub struct RecentQueriesTemplate {
    pub queries: Vec<HistoryEntry>,
}

/// Recent queries widget - returns HTML
pub async fn recent_queries_widget(
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let queries = state.query_history.get_recent(5).await;

    let template = RecentQueriesTemplate { queries };

    template.render()
        .map(Html)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
