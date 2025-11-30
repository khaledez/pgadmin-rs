// Statistics routes
// Provides database performance and usage statistics

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde_json::json;
use crate::services::stats_service::StatsService;
use crate::AppState;

/// Get overall database statistics
pub async fn database_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    StatsService::database_stats(&state.db_pool, "postgres")
        .await
        .map(|stats| Json(json!(stats)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Get statistics for all tables
pub async fn table_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    StatsService::table_stats(&state.db_pool)
        .await
        .map(|tables| Json(json!(tables)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Get statistics for all indexes
pub async fn index_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    StatsService::index_stats(&state.db_pool)
        .await
        .map(|indexes| Json(json!(indexes)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Get cache hit statistics
pub async fn cache_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    StatsService::cache_stats(&state.db_pool)
        .await
        .map(|stats| {
            let heap_ratio = StatsService::cache_hit_ratio(&stats);
            let idx_ratio = StatsService::index_hit_ratio(&stats);
            
            Json(json!({
                "heap_blks_read": stats.heap_blks_read,
                "heap_blks_hit": stats.heap_blks_hit,
                "idx_blks_read": stats.idx_blks_read,
                "idx_blks_hit": stats.idx_blks_hit,
                "cache_hit_ratio": format!("{:.2}%", heap_ratio),
                "index_hit_ratio": format!("{:.2}%", idx_ratio),
            }))
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Get comprehensive database overview
pub async fn overview(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let db_stats = StatsService::database_stats(&state.db_pool, "postgres")
        .await
        .ok();
    let table_stats = StatsService::table_stats(&state.db_pool)
        .await
        .unwrap_or_default();
    let cache_stats = StatsService::cache_stats(&state.db_pool)
        .await
        .ok();

    let heap_ratio = cache_stats.as_ref()
        .map(|s| format!("{:.2}%", StatsService::cache_hit_ratio(s)))
        .unwrap_or_else(|| "N/A".to_string());

    Ok(Json(json!({
        "database": db_stats,
        "top_tables": table_stats.iter().take(10).collect::<Vec<_>>(),
        "cache_hit_ratio": heap_ratio,
        "total_tables": table_stats.len(),
    })))
}
