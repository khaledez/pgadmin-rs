/// Statistics Service
///
/// Provides database statistics including:
/// - Database size
/// - Table sizes and row counts
/// - Index information
/// - Cache hit ratios
/// - Slow queries

use sqlx::PgPool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub database_name: String,
    pub database_size: String,
    pub table_count: i64,
    pub index_count: i64,
    pub total_connections: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStats {
    pub schema_name: String,
    pub table_name: String,
    pub row_count: Option<i64>,
    pub table_size: String,
    pub index_size: String,
    pub total_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub schema_name: String,
    pub index_name: String,
    pub table_name: String,
    pub index_size: String,
    pub is_unique: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub heap_blks_read: i64,
    pub heap_blks_hit: i64,
    pub idx_blks_read: i64,
    pub idx_blks_hit: i64,
}

pub struct StatsService;

impl StatsService {
    /// Get overall database statistics
    pub async fn database_stats(pool: &PgPool, _db_name: &str) -> Result<DatabaseStats, String> {
        let query = r#"
            SELECT 
                current_database() as database_name,
                pg_size_pretty(pg_database_size(current_database())) as database_size,
                (SELECT count(*) FROM information_schema.tables 
                 WHERE table_schema NOT IN ('pg_catalog', 'information_schema')) as table_count,
                (SELECT count(*) FROM information_schema.tables t
                 JOIN information_schema.statistics s 
                 ON t.table_name = s.table_name 
                 WHERE t.table_schema NOT IN ('pg_catalog', 'information_schema')) as index_count,
                (SELECT count(*) FROM pg_stat_activity) as total_connections
        "#;

        let row = sqlx::query_as::<_, (String, String, i64, i64, i32)>(query)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Failed to get database stats: {}", e))?;

        Ok(DatabaseStats {
            database_name: row.0,
            database_size: row.1,
            table_count: row.2,
            index_count: row.3,
            total_connections: row.4,
        })
    }

    /// Get statistics for all tables
    pub async fn table_stats(pool: &PgPool) -> Result<Vec<TableStats>, String> {
        let query = r#"
            SELECT 
                t.table_schema,
                t.table_name,
                (SELECT n_live_tup FROM pg_stat_user_tables 
                 WHERE schemaname = t.table_schema AND relname = t.table_name) as row_count,
                pg_size_pretty(pg_total_relation_size(to_regclass(t.table_schema||'.'||t.table_name))) as total_size,
                pg_size_pretty(pg_relation_size(to_regclass(t.table_schema||'.'||t.table_name))) as table_size,
                pg_size_pretty(pg_total_relation_size(to_regclass(t.table_schema||'.'||t.table_name)) - 
                               pg_relation_size(to_regclass(t.table_schema||'.'||t.table_name))) as index_size
            FROM information_schema.tables t
            WHERE t.table_schema NOT IN ('pg_catalog', 'information_schema')
            ORDER BY pg_total_relation_size(to_regclass(t.table_schema||'.'||t.table_name)) DESC
            LIMIT 50
        "#;

        let rows = sqlx::query_as::<_, (String, String, Option<i64>, String, String, String)>(query)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Failed to get table stats: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| TableStats {
                schema_name: row.0,
                table_name: row.1,
                row_count: row.2,
                table_size: row.4,
                index_size: row.5,
                total_size: row.3,
            })
            .collect())
    }

    /// Get statistics for all indexes
    pub async fn index_stats(pool: &PgPool) -> Result<Vec<IndexStats>, String> {
        let query = r#"
            SELECT 
                schemaname,
                indexname,
                tablename,
                pg_size_pretty(pg_relation_size(indexrelid)) as index_size,
                idx_blks_hit > 0 as is_unique
            FROM pg_stat_user_indexes
            ORDER BY pg_relation_size(indexrelid) DESC
            LIMIT 50
        "#;

        let rows = sqlx::query_as::<_, (String, String, String, String, bool)>(query)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Failed to get index stats: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| IndexStats {
                schema_name: row.0,
                index_name: row.1,
                table_name: row.2,
                index_size: row.3,
                is_unique: row.4,
            })
            .collect())
    }

    /// Get cache hit ratios
    pub async fn cache_stats(pool: &PgPool) -> Result<CacheStats, String> {
        let query = r#"
            SELECT 
                COALESCE(sum(heap_blks_read), 0)::bigint as heap_blks_read,
                COALESCE(sum(heap_blks_hit), 0)::bigint as heap_blks_hit,
                COALESCE(sum(idx_blks_read), 0)::bigint as idx_blks_read,
                COALESCE(sum(idx_blks_hit), 0)::bigint as idx_blks_hit
            FROM pg_statio_user_tables
        "#;

        let row = sqlx::query_as::<_, (i64, i64, i64, i64)>(query)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Failed to get cache stats: {}", e))?;

        Ok(CacheStats {
            heap_blks_read: row.0,
            heap_blks_hit: row.1,
            idx_blks_read: row.2,
            idx_blks_hit: row.3,
        })
    }

    /// Calculate cache hit ratio as percentage
    pub fn cache_hit_ratio(stats: &CacheStats) -> f64 {
        let total_heap = stats.heap_blks_read + stats.heap_blks_hit;
        if total_heap == 0 {
            return 100.0;
        }
        (stats.heap_blks_hit as f64 / total_heap as f64) * 100.0
    }

    /// Get index hit ratio
    pub fn index_hit_ratio(stats: &CacheStats) -> f64 {
        let total_idx = stats.idx_blks_read + stats.idx_blks_hit;
        if total_idx == 0 {
            return 100.0;
        }
        (stats.idx_blks_hit as f64 / total_idx as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_hit_ratio() {
        let stats = CacheStats {
            heap_blks_read: 100,
            heap_blks_hit: 900,
            idx_blks_read: 50,
            idx_blks_hit: 450,
        };

        assert_eq!(StatsService::cache_hit_ratio(&stats), 90.0);
        assert_eq!(StatsService::index_hit_ratio(&stats), 90.0);
    }

    #[test]
    fn test_cache_hit_ratio_no_reads() {
        let stats = CacheStats {
            heap_blks_read: 0,
            heap_blks_hit: 0,
            idx_blks_read: 0,
            idx_blks_hit: 0,
        };

        assert_eq!(StatsService::cache_hit_ratio(&stats), 100.0);
    }

    #[test]
    fn test_cache_hit_ratio_all_reads() {
        let stats = CacheStats {
            heap_blks_read: 1000,
            heap_blks_hit: 0,
            idx_blks_read: 100,
            idx_blks_hit: 0,
        };

        assert_eq!(StatsService::cache_hit_ratio(&stats), 0.0);
    }
}
