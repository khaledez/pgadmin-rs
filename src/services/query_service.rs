// Query service module
// Handles SQL query execution and result processing

use sqlx::{Column, Pool, Postgres, Row};
use serde_json::json;
use crate::models::QueryResult;
use std::time::Instant;

/// Executes a SQL query and returns the results
pub async fn execute_query(
    pool: &Pool<Postgres>,
    query: &str,
) -> Result<QueryResult, Box<dyn std::error::Error>> {
    // Basic validation
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err("Query cannot be empty".into());
    }

    let start = Instant::now();
    
    let rows = sqlx::query(trimmed)
        .fetch_all(pool)
        .await?;

    let execution_time_ms = start.elapsed().as_millis();

    let columns = if let Some(first_row) = rows.first() {
        first_row.columns()
            .iter()
            .map(|col| col.name().to_string())
            .collect()
    } else {
        Vec::new()
    };

    let row_count = rows.len();

    // Convert SQL rows to JSON values
    let rows_data: Vec<Vec<serde_json::Value>> = rows.iter()
        .map(|row| {
            columns.iter()
                .enumerate()
                .map(|(i, _)| {
                    row.try_get::<String, _>(i)
                        .map(|v| json!(v))
                        .or_else(|_| row.try_get::<i32, _>(i).map(|v| json!(v)))
                        .or_else(|_| row.try_get::<i64, _>(i).map(|v| json!(v)))
                        .or_else(|_| row.try_get::<f64, _>(i).map(|v| json!(v)))
                        .or_else(|_| row.try_get::<bool, _>(i).map(|v| json!(v)))
                        .or_else(|_| row.try_get::<sqlx::types::Uuid, _>(i).map(|v| json!(v.to_string())))
                        .unwrap_or(json!(null))
                })
                .collect()
        })
        .collect();

    Ok(QueryResult {
        columns,
        rows: rows_data,
        row_count,
        affected_rows: None,
        execution_time_ms: Some(execution_time_ms),
    })
}



/// Validates a SQL query for dangerous patterns
pub fn validate_query(query: &str) -> Result<(), String> {
    let trimmed = query.trim().to_uppercase();
    
    // Check for dangerous keywords in non-SELECT queries
    if !trimmed.starts_with("SELECT") && !trimmed.starts_with("WITH") {
        if trimmed.contains("DROP") || trimmed.contains("DELETE") {
            return Err("Dangerous operation detected. Please confirm explicitly.".to_string());
        }
    }
    
    Ok(())
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_read_only_query() {
        assert!(is_read_only_query("SELECT * FROM users"));
        assert!(is_read_only_query("WITH cte AS (SELECT 1) SELECT * FROM cte"));
        assert!(!is_read_only_query("INSERT INTO users VALUES (1, 'test')"));
        assert!(!is_read_only_query("UPDATE users SET name = 'test'"));
        assert!(!is_read_only_query("DELETE FROM users"));
    }

    #[test]
    fn test_validate_query() {
        assert!(validate_query("SELECT * FROM users").is_ok());
        assert!(validate_query("DELETE FROM users").is_err());
        assert!(validate_query("DROP TABLE users").is_err());
    }

    #[test]
    fn test_format_value_for_sql() {
        assert_eq!(format_value_for_sql(&json!(null)), "NULL");
        assert_eq!(format_value_for_sql(&json!(true)), "true");
        assert_eq!(format_value_for_sql(&json!(42)), "42");
        assert_eq!(format_value_for_sql(&json!("test")), "'test'");
        assert_eq!(format_value_for_sql(&json!("it's")), "'it''s'");
    }
}
