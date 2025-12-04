// Export routes
// Handles exporting query results and table data in various formats

use crate::services::export_service::{ExportFormat, ExportService};
use crate::services::query_service;
use crate::AppState;
use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Form,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ExportQueryRequest {
    pub query: String,
    #[serde(default)]
    pub format: String,
}

/// Executes a query and exports the results in the specified format
pub async fn export_query(
    State(state): State<AppState>,
    Form(payload): Form<ExportQueryRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let format = ExportFormat::from_str(&payload.format).unwrap_or(ExportFormat::CSV);

    // Validate query
    if let Err(_) = query_service::validate_query(&payload.query) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Execute query
    match query_service::execute_query(&state.db_pool, &payload.query).await {
        Ok(result) => {
            // Export the result
            match ExportService::export(&result, format) {
                Ok(content) => {
                    let mut headers = HeaderMap::new();

                    // Set Content-Type header
                    if let Ok(ct) = format.content_type().parse::<HeaderValue>() {
                        headers.insert("Content-Type", ct);
                    }

                    // Set Content-Disposition header for file download
                    let filename = format!("query_results.{}", format.extension());
                    if let Ok(cd) =
                        format!("attachment; filename=\"{}\"", filename).parse::<HeaderValue>()
                    {
                        headers.insert("Content-Disposition", cd);
                    }

                    Ok((headers, content))
                }
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}
