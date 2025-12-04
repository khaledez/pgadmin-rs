// Statistics routes
// Provides database performance and usage statistics

use crate::services::stats_service::StatsService;
use crate::AppState;
use askama::Template;
use axum::{extract::State, http::StatusCode, response::Html, Json};
use serde_json::json;

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
    let cache_stats = StatsService::cache_stats(&state.db_pool).await.ok();

    let heap_ratio = cache_stats
        .as_ref()
        .map(|s| format!("{:.2}%", StatsService::cache_hit_ratio(s)))
        .unwrap_or_else(|| "N/A".to_string());

    Ok(Json(json!({
        "database": db_stats,
        "top_tables": table_stats.iter().take(10).collect::<Vec<_>>(),
        "cache_hit_ratio": heap_ratio,
        "total_tables": table_stats.len(),
    })))
}

#[derive(Template)]
#[template(path = "components/dashboard-metrics.html")]
struct DashboardMetricsTemplate {
    database: crate::services::stats_service::DatabaseStats,
    total_tables: usize,
    cache_hit_ratio: String,
}

/// Dashboard metrics widget - returns HTML
pub async fn dashboard_metrics_widget(
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let db_stats = StatsService::database_stats(&state.db_pool, "postgres")
        .await
        .map_err(|e| {
            tracing::error!("Failed to get database stats: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let table_stats = StatsService::table_stats(&state.db_pool)
        .await
        .unwrap_or_default();

    let cache_stats = StatsService::cache_stats(&state.db_pool).await.ok();

    let heap_ratio = cache_stats
        .as_ref()
        .map(|s| format!("{:.2}%", StatsService::cache_hit_ratio(s)))
        .unwrap_or_else(|| "N/A".to_string());

    let template = DashboardMetricsTemplate {
        database: db_stats,
        total_tables: table_stats.len(),
        cache_hit_ratio: heap_ratio,
    };

    template.render().map(Html).map_err(|e| {
        tracing::error!("Failed to render dashboard metrics template: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(Template)]
#[template(path = "components/table-stats.html")]
struct TableStatsTemplate {
    tables: Vec<crate::services::stats_service::TableStats>,
}

/// Table stats widget - returns HTML
pub async fn table_stats_widget(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    let tables = StatsService::table_stats(&state.db_pool)
        .await
        .unwrap_or_default();

    let template = TableStatsTemplate {
        tables: tables.into_iter().take(10).collect(),
    };

    template
        .render()
        .map(Html)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Template)]
#[template(path = "components/cache-stats.html")]
struct CacheStatsTemplate {
    cache_hit_ratio: String,
    index_hit_ratio: String,
    heap_blks_read: i64,
    heap_blks_hit: i64,
    cache_class: String,
    index_class: String,
}

fn get_performance_class(ratio_str: &str) -> String {
    let ratio = ratio_str
        .trim_end_matches('%')
        .parse::<f64>()
        .unwrap_or(0.0);
    if ratio >= 90.0 {
        "good".to_string()
    } else if ratio >= 70.0 {
        "warning".to_string()
    } else {
        "bad".to_string()
    }
}

/// Cache stats widget - returns HTML
pub async fn cache_stats_widget(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    let stats = StatsService::cache_stats(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let heap_ratio = StatsService::cache_hit_ratio(&stats);
    let idx_ratio = StatsService::index_hit_ratio(&stats);

    let cache_ratio_str = format!("{:.2}%", heap_ratio);
    let index_ratio_str = format!("{:.2}%", idx_ratio);

    let template = CacheStatsTemplate {
        cache_hit_ratio: cache_ratio_str.clone(),
        index_hit_ratio: index_ratio_str.clone(),
        heap_blks_read: stats.heap_blks_read,
        heap_blks_hit: stats.heap_blks_hit,
        cache_class: get_performance_class(&cache_ratio_str),
        index_class: get_performance_class(&index_ratio_str),
    };

    template
        .render()
        .map(Html)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
