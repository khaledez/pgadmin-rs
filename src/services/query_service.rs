// Query service module
// Handles SQL query execution and result processing

use crate::models::QueryResult;
use serde_json::json;
use sqlx::{Column, Pool, Postgres, Row};
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

    let rows = sqlx::query(trimmed).fetch_all(pool).await?;

    let execution_time_ms = start.elapsed().as_millis();

    let columns = if let Some(first_row) = rows.first() {
        first_row
            .columns()
            .iter()
            .map(|col| col.name().to_string())
            .collect()
    } else {
        Vec::new()
    };

    let row_count = rows.len();

    // Convert SQL rows to JSON values
    let rows_data: Vec<Vec<serde_json::Value>> = rows
        .iter()
        .map(|row| {
            columns
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    row.try_get::<String, _>(i)
                        .map(|v| json!(v))
                        .or_else(|_| row.try_get::<i32, _>(i).map(|v| json!(v)))
                        .or_else(|_| row.try_get::<i64, _>(i).map(|v| json!(v)))
                        .or_else(|_| row.try_get::<f64, _>(i).map(|v| json!(v)))
                        .or_else(|_| row.try_get::<bool, _>(i).map(|v| json!(v)))
                        .or_else(|_| {
                            row.try_get::<sqlx::types::Uuid, _>(i)
                                .map(|v| json!(v.to_string()))
                        })
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
/// 
/// Security: This function prevents SQL injection attacks by:
/// 1. Blocking dangerous standalone operations (DROP, DELETE, INSERT, UPDATE, etc.)
/// 2. Detecting multi-statement injection attacks (e.g., "SELECT 1; DROP TABLE users;")
/// 3. Only allowing safe read operations (SELECT, WITH, EXPLAIN, SHOW)
pub fn validate_query(query: &str) -> Result<(), String> {
    let trimmed = query.trim().to_uppercase();

    // Empty queries are allowed (will fail at execution)
    if trimmed.is_empty() {
        return Ok(());
    }

    // Dangerous keywords that should always trigger rejection
    let dangerous_keywords = [
        "DROP",
        "DELETE",
        "TRUNCATE",
        "ALTER",
        "CREATE",
        "INSERT",
        "UPDATE",
        "GRANT",
        "REVOKE",
    ];

    // CRITICAL: Check for multi-statement attacks
    // Attackers try: "SELECT 1; DROP TABLE users;"
    if let Some(semicolon_pos) = trimmed.find(';') {
        let after_semicolon = trimmed[semicolon_pos + 1..].trim();
        if !after_semicolon.is_empty() {
            // Check for dangerous keywords after the semicolon
            for keyword in &dangerous_keywords {
                if after_semicolon.contains(keyword) {
                    return Err(format!(
                        "Multi-statement query with dangerous operation detected: {}. This could be a SQL injection attempt.",
                        keyword
                    ));
                }
            }
            // Allow multiple safe statements (SELECT, WITH, EXPLAIN)
            let safe_starts = ["SELECT", "WITH", "EXPLAIN", "SHOW"];
            let starts_safe = safe_starts.iter().any(|s| after_semicolon.starts_with(s));
            if !starts_safe && !after_semicolon.chars().all(|c| c == '-' || c.is_whitespace()) {
                return Err(
                    "Multi-statement queries are not allowed for security reasons.".to_string(),
                );
            }
        }
    }

    // Check if query starts with a safe keyword
    let safe_starts = ["SELECT", "WITH", "EXPLAIN", "SHOW"];
    let starts_safe = safe_starts.iter().any(|s| trimmed.starts_with(s));

    // If query doesn't start with safe keyword, check for dangerous operations
    if !starts_safe {
        for keyword in &dangerous_keywords {
            if trimmed.contains(keyword) {
                return Err(format!(
                    "Dangerous operation detected: {}. Use the schema operations API for DDL commands.",
                    keyword
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Safe Operations - Should Pass
    // ============================================================================

    #[test]
    fn test_select_queries_allowed() {
        assert!(validate_query("SELECT * FROM users").is_ok());
        assert!(validate_query("SELECT 1").is_ok());
        assert!(validate_query("SELECT COUNT(*) FROM orders").is_ok());
        assert!(validate_query("SELECT DISTINCT name FROM users").is_ok());
        assert!(validate_query("select * from users").is_ok()); // lowercase
    }

    #[test]
    fn test_with_cte_allowed() {
        assert!(validate_query("WITH cte AS (SELECT 1) SELECT * FROM cte").is_ok());
        assert!(validate_query("WITH RECURSIVE tree AS (SELECT 1) SELECT * FROM tree").is_ok());
    }

    #[test]
    fn test_explain_allowed() {
        assert!(validate_query("EXPLAIN SELECT * FROM users").is_ok());
        assert!(validate_query("EXPLAIN ANALYZE SELECT * FROM users").is_ok());
    }

    #[test]
    fn test_show_allowed() {
        assert!(validate_query("SHOW search_path").is_ok());
        assert!(validate_query("SHOW ALL").is_ok());
    }

    #[test]
    fn test_empty_and_whitespace() {
        assert!(validate_query("").is_ok());
        assert!(validate_query("   ").is_ok());
        assert!(validate_query("\n\t").is_ok());
    }

    #[test]
    fn test_trailing_semicolon() {
        assert!(validate_query("SELECT 1;").is_ok());
        assert!(validate_query("SELECT * FROM users;").is_ok());
    }

    #[test]
    fn test_multiple_select_statements() {
        assert!(validate_query("SELECT 1; SELECT 2;").is_ok());
    }

    // ============================================================================
    // Dangerous Standalone Operations - Should Fail
    // ============================================================================

    #[test]
    fn test_drop_blocked() {
        assert!(validate_query("DROP TABLE users").is_err());
        assert!(validate_query("drop table users cascade").is_err());
        assert!(validate_query("DROP DATABASE production").is_err());
    }

    #[test]
    fn test_delete_blocked() {
        assert!(validate_query("DELETE FROM users").is_err());
        assert!(validate_query("delete from users where id = 1").is_err());
    }

    #[test]
    fn test_truncate_blocked() {
        assert!(validate_query("TRUNCATE TABLE users").is_err());
        assert!(validate_query("truncate users").is_err());
    }

    #[test]
    fn test_insert_blocked() {
        assert!(validate_query("INSERT INTO users VALUES (1, 'test')").is_err());
    }

    #[test]
    fn test_update_blocked() {
        assert!(validate_query("UPDATE users SET name = 'hacked'").is_err());
    }

    #[test]
    fn test_alter_blocked() {
        assert!(validate_query("ALTER TABLE users ADD COLUMN backdoor TEXT").is_err());
    }

    #[test]
    fn test_create_blocked() {
        assert!(validate_query("CREATE TABLE backdoor (id INT)").is_err());
    }

    #[test]
    fn test_grant_revoke_blocked() {
        assert!(validate_query("GRANT ALL ON users TO public").is_err());
        assert!(validate_query("REVOKE ALL ON users FROM public").is_err());
    }

    // ============================================================================
    // Multi-Statement Injection - Critical Security Tests
    // ============================================================================

    #[test]
    fn test_multi_statement_drop_injection() {
        let result = validate_query("SELECT 1; DROP TABLE users;");
        assert!(result.is_err(), "Multi-statement DROP injection should be blocked");
    }

    #[test]
    fn test_multi_statement_delete_injection() {
        let result = validate_query("SELECT * FROM users; DELETE FROM users;");
        assert!(result.is_err(), "Multi-statement DELETE injection should be blocked");
    }

    #[test]
    fn test_multi_statement_insert_injection() {
        let result = validate_query("SELECT 1; INSERT INTO admins VALUES ('hacker');");
        assert!(result.is_err(), "Multi-statement INSERT injection should be blocked");
    }

    #[test]
    fn test_multi_statement_update_injection() {
        let result = validate_query("SELECT id FROM users; UPDATE users SET admin=true;");
        assert!(result.is_err(), "Multi-statement UPDATE injection should be blocked");
    }

    #[test]
    fn test_multi_statement_truncate_injection() {
        let result = validate_query("SELECT 1; TRUNCATE users;");
        assert!(result.is_err(), "Multi-statement TRUNCATE injection should be blocked");
    }

    #[test]
    fn test_multi_statement_alter_injection() {
        let result = validate_query("SELECT 1; ALTER TABLE users DROP COLUMN password;");
        assert!(result.is_err(), "Multi-statement ALTER injection should be blocked");
    }

    #[test]
    fn test_multi_statement_create_injection() {
        let result = validate_query("SELECT 1; CREATE TABLE backdoor (data TEXT);");
        assert!(result.is_err(), "Multi-statement CREATE injection should be blocked");
    }

    #[test]
    fn test_multi_statement_grant_injection() {
        let result = validate_query("SELECT 1; GRANT ALL TO public;");
        assert!(result.is_err(), "Multi-statement GRANT injection should be blocked");
    }

    // ============================================================================
    // Case Insensitivity
    // ============================================================================

    #[test]
    fn test_case_insensitive_detection() {
        // DROP variants
        assert!(validate_query("drop table users").is_err());
        assert!(validate_query("DROP TABLE users").is_err());
        assert!(validate_query("DrOp TaBlE users").is_err());
        
        // DELETE variants
        assert!(validate_query("delete from users").is_err());
        assert!(validate_query("DELETE FROM users").is_err());
        
        // SELECT variants (should pass)
        assert!(validate_query("select * from users").is_ok());
        assert!(validate_query("SELECT * FROM users").is_ok());
        assert!(validate_query("SeLeCt * from users").is_ok());
    }

    // ============================================================================
    // Complex Queries
    // ============================================================================

    #[test]
    fn test_complex_select_with_subqueries() {
        let query = "SELECT u.*, (SELECT COUNT(*) FROM orders WHERE user_id = u.id) as order_count FROM users u";
        assert!(validate_query(query).is_ok());
    }

    #[test]
    fn test_join_queries() {
        assert!(validate_query("SELECT * FROM users u JOIN orders o ON u.id = o.user_id").is_ok());
        assert!(validate_query("SELECT * FROM users LEFT OUTER JOIN orders ON true").is_ok());
    }

    #[test]
    fn test_union_queries() {
        assert!(validate_query("SELECT 1 UNION SELECT 2").is_ok());
        assert!(validate_query("SELECT id FROM users UNION ALL SELECT id FROM admins").is_ok());
    }

    #[test]
    fn test_window_functions() {
        let query = "SELECT id, ROW_NUMBER() OVER (ORDER BY created_at) FROM users";
        assert!(validate_query(query).is_ok());
    }
}
