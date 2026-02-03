/// Route and Error Handling Tests
/// Tests for request validation, error responses, and edge cases
#[cfg(test)]
mod tests {
    use crate::models::{ColumnInfo, Pagination, QueryResult, Schema, TableInfo};
    use crate::services::export_service::{ExportFormat, ExportService};
    use crate::services::query_service;
    use serde_json::json;

    // ============================================================================
    // Request Body Validation Tests
    // ============================================================================

    #[test]
    fn test_query_request_body_structure() {
        // Valid query request
        let body = json!({
            "query": "SELECT * FROM users"
        });

        assert!(body.is_object());
        assert!(body["query"].is_string());
        assert!(!body["query"].as_str().unwrap().is_empty());
    }

    #[test]
    fn test_export_request_body_structure() {
        let body = json!({
            "query": "SELECT * FROM users",
            "format": "csv"
        });

        assert!(body["query"].is_string());
        assert!(body["format"].is_string());

        // Valid formats
        let valid_formats = ["csv", "json", "sql"];
        let format = body["format"].as_str().unwrap();
        assert!(
            valid_formats.contains(&format),
            "Format should be one of: csv, json, sql"
        );
    }

    #[test]
    fn test_create_table_request_body_structure() {
        let body = json!({
            "table_name": "new_table",
            "schema": "public",
            "columns": [
                {
                    "name": "id",
                    "data_type": "INTEGER",
                    "nullable": false
                },
                {
                    "name": "name",
                    "data_type": "VARCHAR(255)",
                    "nullable": true,
                    "default": "'unknown'"
                }
            ]
        });

        assert!(body["table_name"].is_string());
        assert!(body["schema"].is_string());
        assert!(body["columns"].is_array());
        assert!(!body["columns"].as_array().unwrap().is_empty());

        // Verify column structure
        let first_col = &body["columns"][0];
        assert!(first_col["name"].is_string());
        assert!(first_col["data_type"].is_string());
        assert!(first_col["nullable"].is_boolean());
    }

    #[test]
    fn test_drop_object_request_body_structure() {
        let body = json!({
            "object_name": "old_table",
            "schema": "public",
            "object_type": "TABLE",
            "cascade": false
        });

        assert!(body["object_name"].is_string());
        assert!(body["schema"].is_string());
        assert!(body["object_type"].is_string());
        assert!(body["cascade"].is_boolean());

        // Valid object types
        let valid_types = ["TABLE", "VIEW", "INDEX", "SEQUENCE", "FUNCTION"];
        let obj_type = body["object_type"].as_str().unwrap();
        assert!(
            valid_types.contains(&obj_type),
            "Object type should be one of: TABLE, VIEW, INDEX, SEQUENCE, FUNCTION"
        );
    }

    // ============================================================================
    // Query Validation Error Tests
    // ============================================================================

    #[test]
    fn test_empty_query_validation() {
        // Empty queries pass validation but will fail at execution
        assert!(query_service::validate_query("").is_ok());
    }

    #[test]
    fn test_dangerous_query_returns_error() {
        let dangerous_queries = vec![
            "DROP TABLE users",
            "DELETE FROM users",
            "TRUNCATE TABLE users",
            "INSERT INTO users VALUES (1)",
            "UPDATE users SET admin=true",
        ];

        for query in dangerous_queries {
            let result = query_service::validate_query(query);
            assert!(
                result.is_err(),
                "Query '{}' should return an error",
                query
            );

            let error_msg = result.unwrap_err();
            assert!(
                !error_msg.is_empty(),
                "Error message should not be empty for dangerous query"
            );
        }
    }

    #[test]
    fn test_injection_attack_returns_descriptive_error() {
        let result = query_service::validate_query("SELECT 1; DROP TABLE users;");
        assert!(result.is_err());

        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("Multi-statement") || error_msg.contains("dangerous"),
            "Error should mention multi-statement or dangerous: {}",
            error_msg
        );
    }

    // ============================================================================
    // Model Creation Tests
    // ============================================================================

    #[test]
    fn test_query_result_creation() {
        let result = QueryResult {
            columns: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![json!(1), json!("Alice")],
                vec![json!(2), json!("Bob")],
            ],
            row_count: 2,
            affected_rows: None,
            execution_time_ms: Some(50),
        };

        assert_eq!(result.columns.len(), 2);
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.row_count, 2);
        assert!(result.affected_rows.is_none());
        assert_eq!(result.execution_time_ms, Some(50));
    }

    #[test]
    fn test_query_result_empty() {
        let result = QueryResult {
            columns: vec!["id".to_string()],
            rows: vec![],
            row_count: 0,
            affected_rows: None,
            execution_time_ms: Some(10),
        };

        assert!(!result.columns.is_empty()); // Columns exist even if no rows
        assert!(result.rows.is_empty());
        assert_eq!(result.row_count, 0);
    }

    #[test]
    fn test_query_result_with_nulls() {
        let result = QueryResult {
            columns: vec!["id".to_string(), "email".to_string()],
            rows: vec![
                vec![json!(1), json!(null)],
                vec![json!(2), json!("test@example.com")],
            ],
            row_count: 2,
            affected_rows: None,
            execution_time_ms: Some(25),
        };

        assert!(result.rows[0][1].is_null());
        assert!(!result.rows[1][1].is_null());
    }

    #[test]
    fn test_schema_creation() {
        let schema = Schema {
            name: "public".to_string(),
            owner: Some("postgres".to_string()),
        };

        assert_eq!(schema.name, "public");
        assert_eq!(schema.owner, Some("postgres".to_string()));
    }

    #[test]
    fn test_table_info_creation() {
        let table = TableInfo {
            schema: "public".to_string(),
            name: "users".to_string(),
            table_type: "BASE TABLE".to_string(),
            row_count: Some(1000),
            size: Some(81920),
        };

        assert_eq!(table.schema, "public");
        assert_eq!(table.name, "users");
        assert_eq!(table.table_type, "BASE TABLE");
        assert_eq!(table.row_count, Some(1000));
        assert_eq!(table.size, Some(81920));
    }

    #[test]
    fn test_column_info_creation() {
        let col = ColumnInfo {
            name: "id".to_string(),
            data_type: "integer".to_string(),
            is_nullable: false,
            is_pk: true,
            default: Some("nextval('users_id_seq')".to_string()),
        };

        assert_eq!(col.name, "id");
        assert!(!col.is_nullable);
        assert!(col.is_pk);
        assert!(col.default.is_some());
    }

    #[test]
    fn test_pagination_creation() {
        let pagination = Pagination {
            page: 1,
            page_size: 100,
            total_rows: 1500,
            total_pages: 15,
        };

        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.page_size, 100);
        assert_eq!(pagination.total_rows, 1500);
        assert_eq!(pagination.total_pages, 15);
    }

    #[test]
    fn test_pagination_edge_cases() {
        // First page
        let first_page = Pagination {
            page: 1,
            page_size: 100,
            total_rows: 50,
            total_pages: 1,
        };
        assert_eq!(first_page.page, 1);
        assert!((first_page.total_rows as u32) < first_page.page_size);

        // Empty result
        let empty = Pagination {
            page: 1,
            page_size: 100,
            total_rows: 0,
            total_pages: 0,
        };
        assert_eq!(empty.total_rows, 0);
    }

    // ============================================================================
    // Export Service Tests
    // ============================================================================

    #[test]
    fn test_export_csv_format() {
        let result = QueryResult {
            columns: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![json!(1), json!("Alice")],
                vec![json!(2), json!("Bob")],
            ],
            row_count: 2,
            affected_rows: None,
            execution_time_ms: Some(10),
        };

        let csv = ExportService::export(&result, ExportFormat::Csv).unwrap();
        assert!(csv.contains("id,name"));
        assert!(csv.contains("1,Alice"));
        assert!(csv.contains("2,Bob"));
    }

    #[test]
    fn test_export_csv_escaping() {
        let result = QueryResult {
            columns: vec!["data".to_string()],
            rows: vec![
                vec![json!("hello, world")],   // Contains comma
                vec![json!("say \"hi\"")],     // Contains quotes
                vec![json!("line1\nline2")],   // Contains newline
            ],
            row_count: 3,
            affected_rows: None,
            execution_time_ms: None,
        };

        let csv = ExportService::export(&result, ExportFormat::Csv).unwrap();
        // Values with special chars should be quoted
        assert!(csv.contains("\"hello, world\""));
        assert!(csv.contains("\"say \"\"hi\"\"\""));
    }

    #[test]
    fn test_export_json_format() {
        let result = QueryResult {
            columns: vec!["id".to_string(), "name".to_string()],
            rows: vec![vec![json!(1), json!("Alice")]],
            row_count: 1,
            affected_rows: None,
            execution_time_ms: Some(10),
        };

        let json_str = ExportService::export(&result, ExportFormat::Json).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert!(parsed["columns"].is_array());
        assert!(parsed["data"].is_array()); // Note: JSON export uses "data" not "rows"
        assert!(parsed["row_count"].is_number());
    }

    #[test]
    fn test_export_sql_format() {
        let result = QueryResult {
            columns: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![json!(1), json!("Alice")],
                vec![json!(2), json!(null)],
            ],
            row_count: 2,
            affected_rows: None,
            execution_time_ms: None,
        };

        let sql = ExportService::export(&result, ExportFormat::Sql).unwrap();
        assert!(sql.contains("INSERT INTO"));
        assert!(sql.contains("'Alice'"));
        assert!(sql.contains("NULL")); // NULL value handling
    }

    // ============================================================================
    // Error Response Format Tests
    // ============================================================================

    #[test]
    fn test_error_response_structure() {
        // Simulate error response structure
        let error = json!({
            "error": "Query validation failed",
            "details": "Dangerous operation detected: DROP"
        });

        assert!(error["error"].is_string());
        assert!(!error["error"].as_str().unwrap().is_empty());
    }

    #[test]
    fn test_validation_error_messages_are_descriptive() {
        let test_cases = vec![
            ("DROP TABLE users", "DROP"),
            ("DELETE FROM users", "DELETE"),
            ("SELECT 1; DROP TABLE x;", "Multi-statement"),
        ];

        for (query, expected_keyword) in test_cases {
            let result = query_service::validate_query(query);
            if let Err(msg) = result {
                assert!(
                    msg.to_uppercase().contains(&expected_keyword.to_uppercase()),
                    "Error for '{}' should mention '{}', got: {}",
                    query,
                    expected_keyword,
                    msg
                );
            }
        }
    }

    // ============================================================================
    // Path Parameter Tests
    // ============================================================================

    #[test]
    fn test_valid_schema_names() {
        use crate::services::schema_ops_service::SchemaOpsService;

        let valid_names = vec!["public", "user_data", "schema_v2", "_private", "ABC123"];

        for name in valid_names {
            assert!(
                SchemaOpsService::validate_identifier(name).is_ok(),
                "Schema name '{}' should be valid",
                name
            );
        }
    }

    #[test]
    fn test_invalid_schema_names_rejected() {
        use crate::services::schema_ops_service::SchemaOpsService;

        let too_long = "a".repeat(64);
        let invalid_names = vec![
            "",              // empty
            "user-data",     // hyphen
            "user.data",     // dot
            "user data",     // space
            "123abc",        // starts with digit
            too_long.as_str(), // too long
        ];

        for name in invalid_names {
            assert!(
                SchemaOpsService::validate_identifier(name).is_err(),
                "Schema name '{}' should be invalid",
                name
            );
        }
    }

    // ============================================================================
    // Pagination Parameter Tests
    // ============================================================================

    #[test]
    fn test_pagination_defaults() {
        // Default values when not specified
        let default_page = 1;
        let default_page_size = 100;

        assert_eq!(default_page, 1);
        assert_eq!(default_page_size, 100);
    }

    #[test]
    fn test_pagination_calculation() {
        // Test total_pages calculation
        let test_cases = vec![
            (0, 100, 0),     // No rows
            (50, 100, 1),    // Less than one page
            (100, 100, 1),   // Exactly one page
            (101, 100, 2),   // Just over one page
            (1000, 100, 10), // Multiple pages
        ];

        for (total_rows, page_size, expected_pages) in test_cases {
            let calculated_pages = (total_rows as f64 / page_size as f64).ceil() as u32;
            let calculated_pages = if total_rows == 0 { 0 } else { calculated_pages };
            assert_eq!(
                calculated_pages, expected_pages,
                "total_rows={}, page_size={} should give {} pages",
                total_rows, page_size, expected_pages
            );
        }
    }
}
