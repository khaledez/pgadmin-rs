/// Security validation tests
/// Tests for SQL injection, XSS, and other security concerns
#[cfg(test)]
mod tests {
    use crate::services::query_service;
    use crate::services::schema_ops_service::SchemaOpsService;

    // ============================================================================
    // PRIORITY 1: Multi-Statement SQL Injection Prevention (CRITICAL!)
    // These tests verify that dangerous SQL injection patterns are blocked.
    // ============================================================================

    #[test]
    fn test_multi_statement_drop_injection() {
        // CRITICAL: This is the main vulnerability - SELECT followed by DROP
        // An attacker could execute: "SELECT 1; DROP TABLE users;"
        let result = query_service::validate_query("SELECT 1; DROP TABLE users;");
        assert!(
            result.is_err(),
            "SECURITY VULNERABILITY: 'SELECT 1; DROP TABLE users;' should be rejected!"
        );
    }

    #[test]
    fn test_multi_statement_delete_injection() {
        let result = query_service::validate_query("SELECT * FROM users; DELETE FROM users;");
        assert!(
            result.is_err(),
            "SECURITY VULNERABILITY: 'SELECT; DELETE' should be rejected!"
        );
    }

    #[test]
    fn test_multi_statement_truncate_injection() {
        let result = query_service::validate_query("SELECT 1; TRUNCATE TABLE users;");
        assert!(
            result.is_err(),
            "SECURITY VULNERABILITY: 'SELECT; TRUNCATE' should be rejected!"
        );
    }

    #[test]
    fn test_multi_statement_insert_injection() {
        let result =
            query_service::validate_query("SELECT 1; INSERT INTO admins VALUES ('hacker');");
        assert!(
            result.is_err(),
            "SECURITY VULNERABILITY: 'SELECT; INSERT' should be rejected!"
        );
    }

    #[test]
    fn test_multi_statement_update_injection() {
        let result =
            query_service::validate_query("SELECT id FROM users; UPDATE users SET admin=true;");
        assert!(
            result.is_err(),
            "SECURITY VULNERABILITY: 'SELECT; UPDATE' should be rejected!"
        );
    }

    #[test]
    fn test_multi_statement_alter_injection() {
        let result =
            query_service::validate_query("SELECT 1; ALTER TABLE users DROP COLUMN password;");
        assert!(
            result.is_err(),
            "SECURITY VULNERABILITY: 'SELECT; ALTER' should be rejected!"
        );
    }

    #[test]
    fn test_multi_statement_create_injection() {
        let result = query_service::validate_query("SELECT 1; CREATE TABLE backdoor (data TEXT);");
        assert!(
            result.is_err(),
            "SECURITY VULNERABILITY: 'SELECT; CREATE' should be rejected!"
        );
    }

    #[test]
    fn test_multi_statement_grant_injection() {
        let result = query_service::validate_query("SELECT 1; GRANT ALL ON users TO public;");
        assert!(
            result.is_err(),
            "SECURITY VULNERABILITY: 'SELECT; GRANT' should be rejected!"
        );
    }

    // ============================================================================
    // Standalone Dangerous Operations (should be blocked)
    // ============================================================================

    #[test]
    fn test_standalone_drop_blocked() {
        assert!(query_service::validate_query("DROP TABLE users").is_err());
        assert!(query_service::validate_query("drop table users cascade").is_err());
    }

    #[test]
    fn test_standalone_delete_blocked() {
        assert!(query_service::validate_query("DELETE FROM users").is_err());
        assert!(query_service::validate_query("delete from users where 1=1").is_err());
    }

    #[test]
    fn test_standalone_truncate_blocked() {
        let result = query_service::validate_query("TRUNCATE TABLE users");
        assert!(result.is_err(), "TRUNCATE should be blocked");
    }

    #[test]
    fn test_standalone_insert_blocked() {
        let result = query_service::validate_query("INSERT INTO users VALUES (1, 'test')");
        assert!(result.is_err(), "INSERT should be blocked");
    }

    #[test]
    fn test_standalone_update_blocked() {
        let result = query_service::validate_query("UPDATE users SET name = 'hacked'");
        assert!(result.is_err(), "UPDATE should be blocked");
    }

    #[test]
    fn test_standalone_alter_blocked() {
        let result = query_service::validate_query("ALTER TABLE users ADD COLUMN backdoor TEXT");
        assert!(result.is_err(), "ALTER should be blocked");
    }

    #[test]
    fn test_standalone_create_blocked() {
        let result = query_service::validate_query("CREATE TABLE backdoor (id INT)");
        assert!(result.is_err(), "CREATE should be blocked");
    }

    // ============================================================================
    // Safe Operations (should be allowed)
    // ============================================================================

    #[test]
    fn test_select_allowed() {
        assert!(query_service::validate_query("SELECT * FROM users").is_ok());
        assert!(query_service::validate_query("SELECT 1").is_ok());
        assert!(query_service::validate_query("select count(*) from orders").is_ok());
    }

    #[test]
    fn test_with_cte_allowed() {
        assert!(query_service::validate_query("WITH cte AS (SELECT 1) SELECT * FROM cte").is_ok());
    }

    #[test]
    fn test_explain_allowed() {
        let result = query_service::validate_query("EXPLAIN SELECT * FROM users");
        assert!(result.is_ok(), "EXPLAIN should be allowed");
    }

    #[test]
    fn test_multiple_selects_allowed() {
        // Multiple SELECT statements should be safe
        assert!(query_service::validate_query("SELECT 1; SELECT 2;").is_ok());
    }

    #[test]
    fn test_trailing_semicolon_allowed() {
        assert!(query_service::validate_query("SELECT * FROM users;").is_ok());
    }

    // ============================================================================
    // PRIORITY 2: Identifier Validation (Path Injection Prevention)
    // ============================================================================

    #[test]
    fn test_identifier_path_traversal_blocked() {
        // Path traversal attempts in identifiers should be rejected
        assert!(SchemaOpsService::validate_identifier("../etc/passwd").is_err());
        assert!(SchemaOpsService::validate_identifier("..\\windows").is_err());
        assert!(SchemaOpsService::validate_identifier("foo/../bar").is_err());
    }

    #[test]
    fn test_identifier_sql_injection_blocked() {
        // SQL injection in identifiers should be rejected
        assert!(SchemaOpsService::validate_identifier("users; DROP TABLE--").is_err());
        assert!(SchemaOpsService::validate_identifier("users'--").is_err());
        assert!(SchemaOpsService::validate_identifier("users\"--").is_err());
    }

    #[test]
    fn test_identifier_special_chars_blocked() {
        assert!(SchemaOpsService::validate_identifier("user-name").is_err());
        assert!(SchemaOpsService::validate_identifier("user.name").is_err());
        assert!(SchemaOpsService::validate_identifier("user name").is_err());
        assert!(SchemaOpsService::validate_identifier("user;name").is_err());
    }

    #[test]
    fn test_identifier_valid_names_allowed() {
        assert!(SchemaOpsService::validate_identifier("users").is_ok());
        assert!(SchemaOpsService::validate_identifier("user_data").is_ok());
        assert!(SchemaOpsService::validate_identifier("Users123").is_ok());
        assert!(SchemaOpsService::validate_identifier("_private").is_ok());
    }

    #[test]
    fn test_identifier_empty_blocked() {
        assert!(SchemaOpsService::validate_identifier("").is_err());
    }

    #[test]
    fn test_identifier_too_long_blocked() {
        let long_name = "a".repeat(64);
        assert!(SchemaOpsService::validate_identifier(&long_name).is_err());
    }

    #[test]
    fn test_identifier_starts_with_digit_blocked() {
        assert!(SchemaOpsService::validate_identifier("123users").is_err());
        assert!(SchemaOpsService::validate_identifier("1table").is_err());
    }

    // ============================================================================
    // PRIORITY 3: XSS Prevention Tests (Template Escaping)
    // These test that malicious HTML/JS is properly escaped by Askama templates.
    // ============================================================================

    #[test]
    fn test_xss_script_tag_escaped() {
        use crate::models::{ColumnInfo, Pagination};
        use crate::routes::tables::TableDataTemplate;
        use askama::Template;

        // Create a template with XSS payload in data
        let xss_payload = "<script>alert('xss')</script>";
        let template = TableDataTemplate {
            schema: "public".to_string(),
            table: "test".to_string(),
            columns: vec![ColumnInfo {
                name: "data".to_string(),
                data_type: "text".to_string(),
                is_nullable: true,
                is_pk: false,
                default: None,
            }],
            rows: vec![vec![serde_json::Value::String(xss_payload.to_string())]],
            pagination: Pagination {
                page: 1,
                page_size: 100,
                total_rows: 1,
                total_pages: 1,
            },
        };

        let html = template.render().expect("Template should render");

        // The script tag should be HTML-escaped, not raw
        // Askama escapes < as &#60; (numeric entity)
        assert!(
            !html.contains("<script>"),
            "XSS VULNERABILITY: Raw <script> tag found in output! Check template escaping."
        );
        // Check for any form of HTML escaping (Askama uses numeric entities like &#60;)
        assert!(
            html.contains("&lt;script&gt;")
                || html.contains("&#60;script&#62;")
                || html.contains("&#x3c;script")
                || !html.contains("script"),
            "Script tag should be HTML-escaped (expected &#60;script&#62;)"
        );
    }

    #[test]
    fn test_xss_event_handler_escaped() {
        use crate::models::{ColumnInfo, Pagination};
        use crate::routes::tables::TableDataTemplate;
        use askama::Template;

        let xss_payload = "<img src=x onerror=\"alert('xss')\">";
        let template = TableDataTemplate {
            schema: "public".to_string(),
            table: "test".to_string(),
            columns: vec![ColumnInfo {
                name: "data".to_string(),
                data_type: "text".to_string(),
                is_nullable: true,
                is_pk: false,
                default: None,
            }],
            rows: vec![vec![serde_json::Value::String(xss_payload.to_string())]],
            pagination: Pagination {
                page: 1,
                page_size: 100,
                total_rows: 1,
                total_pages: 1,
            },
        };

        let html = template.render().expect("Template should render");

        // The < character should be escaped, which neutralizes any HTML/JS injection
        // Even if "onerror" appears in escaped form, the < being escaped prevents execution
        assert!(
            !html.contains("<img"),
            "XSS VULNERABILITY: Raw <img tag found in output!"
        );
        // Verify the angle bracket is escaped (&#60; is the numeric entity for <)
        assert!(
            html.contains("&#60;img") || html.contains("&lt;img"),
            "The <img tag should be HTML-escaped"
        );
    }

    #[test]
    fn test_xss_in_column_name_escaped() {
        use crate::models::{ColumnInfo, Pagination};
        use crate::routes::tables::TableDataTemplate;
        use askama::Template;

        // XSS in column name (could come from malicious table structure)
        let template = TableDataTemplate {
            schema: "public".to_string(),
            table: "test".to_string(),
            columns: vec![ColumnInfo {
                name: "<script>alert('xss')</script>".to_string(),
                data_type: "text".to_string(),
                is_nullable: true,
                is_pk: false,
                default: None,
            }],
            rows: vec![],
            pagination: Pagination {
                page: 1,
                page_size: 100,
                total_rows: 0,
                total_pages: 0,
            },
        };

        let html = template.render().expect("Template should render");

        assert!(
            !html.contains("<script>"),
            "XSS VULNERABILITY: Raw <script> in column name!"
        );
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[test]
    fn test_empty_query_allowed() {
        assert!(query_service::validate_query("").is_ok());
        assert!(query_service::validate_query("   ").is_ok());
    }

    #[test]
    fn test_case_insensitive_detection() {
        // DROP in various cases
        assert!(query_service::validate_query("drop table users").is_err());
        assert!(query_service::validate_query("DROP TABLE users").is_err());
        assert!(query_service::validate_query("DrOp TaBlE users").is_err());

        // DELETE in various cases
        assert!(query_service::validate_query("delete from users").is_err());
        assert!(query_service::validate_query("DELETE FROM users").is_err());
    }
}
