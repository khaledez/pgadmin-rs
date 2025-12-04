/// Unit tests for data models
/// Tests serialization, deserialization, and model behavior
#[cfg(test)]
mod model_tests {
    use crate::models::*;
    use serde_json::json;

    #[test]
    fn test_query_result_creation() {
        let result = QueryResult {
            columns: vec!["id".to_string(), "name".to_string()],
            rows: vec![vec![json!(1), json!("Alice")]],
            row_count: 1,
            affected_rows: None,
            execution_time_ms: Some(100),
        };

        assert_eq!(result.columns.len(), 2);
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.row_count, 1);
        assert_eq!(result.execution_time_ms, Some(100));
    }

    #[test]
    fn test_query_result_empty() {
        let result = QueryResult {
            columns: vec![],
            rows: vec![],
            row_count: 0,
            affected_rows: None,
            execution_time_ms: Some(50),
        };

        assert!(result.columns.is_empty());
        assert!(result.rows.is_empty());
        assert_eq!(result.row_count, 0);
    }

    #[test]
    fn test_query_result_with_affected_rows() {
        let result = QueryResult {
            columns: vec![],
            rows: vec![],
            row_count: 0,
            affected_rows: Some(5),
            execution_time_ms: Some(10),
        };

        assert_eq!(result.affected_rows, Some(5));
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
            row_count: Some(100),
            size: Some(8192),
        };

        assert_eq!(table.schema, "public");
        assert_eq!(table.name, "users");
        assert_eq!(table.row_count, Some(100));
        assert_eq!(table.size, Some(8192));
    }

    #[test]
    fn test_column_info_creation() {
        let column = ColumnInfo {
            name: "id".to_string(),
            data_type: "integer".to_string(),
            is_nullable: false,
            is_pk: true,
            default: None,
        };

        assert_eq!(column.name, "id");
        assert_eq!(column.data_type, "integer");
        assert!(!column.is_nullable);
        assert!(column.is_pk);
        assert!(column.default.is_none());
    }

    #[test]
    fn test_column_info_nullable() {
        let column = ColumnInfo {
            name: "email".to_string(),
            data_type: "character varying".to_string(),
            is_nullable: true,
            is_pk: false,
            default: Some("NULL".to_string()),
        };

        assert!(column.is_nullable);
        assert!(!column.is_pk);
        assert_eq!(column.default, Some("NULL".to_string()));
    }

    #[test]
    fn test_pagination_creation() {
        let pagination = Pagination {
            page: 1,
            page_size: 100,
            total_rows: 250,
            total_pages: 3,
        };

        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.page_size, 100);
        assert_eq!(pagination.total_rows, 250);
        assert_eq!(pagination.total_pages, 3);
    }

    #[test]
    fn test_pagination_last_page() {
        let pagination = Pagination {
            page: 3,
            page_size: 100,
            total_rows: 250,
            total_pages: 3,
        };

        assert_eq!(pagination.page, 3);
        assert_eq!(pagination.total_pages, 3);
    }

    #[test]
    fn test_query_result_multiple_rows() {
        let result = QueryResult {
            columns: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![json!(1), json!("Alice")],
                vec![json!(2), json!("Bob")],
                vec![json!(3), json!("Charlie")],
            ],
            row_count: 3,
            affected_rows: None,
            execution_time_ms: Some(150),
        };

        assert_eq!(result.row_count, 3);
        assert_eq!(result.rows.len(), 3);

        // Verify each row
        assert_eq!(result.rows[0][0], json!(1));
        assert_eq!(result.rows[1][0], json!(2));
        assert_eq!(result.rows[2][0], json!(3));
    }

    #[test]
    fn test_query_result_with_null_values() {
        let result = QueryResult {
            columns: vec!["id".to_string(), "name".to_string()],
            rows: vec![vec![json!(1), json!(null)], vec![json!(null), json!("Bob")]],
            row_count: 2,
            affected_rows: None,
            execution_time_ms: Some(100),
        };

        assert_eq!(result.row_count, 2);
        assert_eq!(result.rows[0][1], json!(null));
        assert_eq!(result.rows[1][0], json!(null));
    }

    #[test]
    fn test_column_info_with_default() {
        let column = ColumnInfo {
            name: "created_at".to_string(),
            data_type: "timestamp without time zone".to_string(),
            is_nullable: false,
            is_pk: false,
            default: Some("CURRENT_TIMESTAMP".to_string()),
        };

        assert!(!column.is_nullable);
        assert_eq!(column.default, Some("CURRENT_TIMESTAMP".to_string()));
    }

    #[test]
    fn test_table_info_without_size() {
        let table = TableInfo {
            schema: "public".to_string(),
            name: "test_table".to_string(),
            table_type: "BASE TABLE".to_string(),
            row_count: Some(0),
            size: None,
        };

        assert_eq!(table.row_count, Some(0));
        assert!(table.size.is_none());
    }
}
