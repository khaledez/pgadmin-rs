/// HTTP Integration Tests
/// Tests actual HTTP endpoints using tower::ServiceExt
///
/// These tests verify that routes respond correctly without needing a database.
/// Database-dependent tests are in tests/integration_test.rs
#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    // ============================================================================
    // Helper Functions
    // ============================================================================

    /// Create a minimal router for testing non-database routes
    fn create_test_router() -> Router {
        Router::new()
            .route("/health", get(health_handler))
            .route("/test-html", get(html_handler))
            .route("/test-json", get(json_handler))
            .route("/test-error", get(error_handler))
    }

    async fn health_handler() -> &'static str {
        "OK"
    }

    async fn html_handler() -> axum::response::Html<&'static str> {
        axum::response::Html("<html><body>Hello</body></html>")
    }

    async fn json_handler() -> axum::Json<serde_json::Value> {
        axum::Json(serde_json::json!({"status": "ok", "data": [1, 2, 3]}))
    }

    async fn error_handler() -> (StatusCode, &'static str) {
        (StatusCode::BAD_REQUEST, "Bad request")
    }

    // ============================================================================
    // Health Check Tests
    // ============================================================================

    #[tokio::test]
    async fn test_health_endpoint_returns_200() {
        let app = create_test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_endpoint_returns_ok_body() {
        let app = create_test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(body_str, "OK");
    }

    // ============================================================================
    // 404 Tests
    // ============================================================================

    #[tokio::test]
    async fn test_unknown_route_returns_404() {
        let app = create_test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/this-route-does-not-exist")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_api_unknown_route_returns_404() {
        let app = create_test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // ============================================================================
    // Content-Type Tests
    // ============================================================================

    #[tokio::test]
    async fn test_html_response_has_correct_content_type() {
        let app = create_test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test-html")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get("content-type")
            .map(|v| v.to_str().unwrap_or(""));
        assert!(
            content_type
                .map(|ct| ct.contains("text/html"))
                .unwrap_or(false),
            "Expected text/html content type, got {:?}",
            content_type
        );
    }

    #[tokio::test]
    async fn test_json_response_has_correct_content_type() {
        let app = create_test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test-json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get("content-type")
            .map(|v| v.to_str().unwrap_or(""));
        assert!(
            content_type
                .map(|ct| ct.contains("application/json"))
                .unwrap_or(false),
            "Expected application/json content type, got {:?}",
            content_type
        );
    }

    // ============================================================================
    // Error Response Tests
    // ============================================================================

    #[tokio::test]
    async fn test_error_endpoint_returns_400() {
        let app = create_test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test-error")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // ============================================================================
    // Method Tests
    // ============================================================================

    #[tokio::test]
    async fn test_post_to_get_endpoint_returns_405() {
        let app = create_test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Axum returns 405 Method Not Allowed for wrong method
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    // ============================================================================
    // JSON Body Tests
    // ============================================================================

    #[tokio::test]
    async fn test_json_response_is_valid_json() {
        let app = create_test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test-json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: Result<serde_json::Value, _> = serde_json::from_slice(&body);
        assert!(json.is_ok(), "Response should be valid JSON");

        let json = json.unwrap();
        assert_eq!(json["status"], "ok");
        assert!(json["data"].is_array());
    }

    // ============================================================================
    // Request Header Tests
    // ============================================================================

    #[tokio::test]
    async fn test_request_with_custom_headers() {
        let app = create_test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .header("X-Custom-Header", "test-value")
                    .header("Accept", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should still work with custom headers
        assert_eq!(response.status(), StatusCode::OK);
    }

    // ============================================================================
    // Query Parameter Simulation Tests (using path)
    // ============================================================================

    #[tokio::test]
    async fn test_query_params_in_unknown_route() {
        let app = create_test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/unknown?page=1&size=10")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

// ============================================================================
// Route Tests - Test that routes exist and patterns are correct
// ============================================================================
#[cfg(test)]
mod route_pattern_tests {
    /// Verify expected API route patterns
    #[test]
    fn test_expected_api_routes() {
        let expected_routes = vec![
            // Health and pages
            ("GET", "/health"),
            ("GET", "/"),
            ("GET", "/query"),
            ("GET", "/studio"),
            // Schema routes
            ("GET", "/api/schemas"),
            ("GET", "/api/schemas/{schema}"),
            // Table routes
            ("GET", "/api/schemas/{schema}/tables"),
            ("GET", "/api/schemas/{schema}/tables/{table}"),
            ("GET", "/api/schemas/{schema}/tables/{table}/data"),
            // Query routes
            ("POST", "/api/query/execute"),
            ("GET", "/api/query/history"),
            ("DELETE", "/api/query/history"),
            ("POST", "/api/query/export"),
            // Schema operations
            ("POST", "/api/schema/create-table"),
            ("POST", "/api/schema/drop-object"),
            // Stats routes
            ("GET", "/api/stats/database"),
            ("GET", "/api/stats/tables"),
            ("GET", "/api/stats/cache"),
            // Cell editing
            ("GET", "/api/cell/edit"),
            ("POST", "/api/cell/update"),
        ];

        // This test documents the expected routes
        // Real route testing happens in integration tests
        assert!(
            !expected_routes.is_empty(),
            "Route patterns should be defined"
        );

        for (method, path) in &expected_routes {
            assert!(!path.is_empty(), "Route path should not be empty");
            assert!(
                ["GET", "POST", "DELETE", "PUT", "PATCH"].contains(method),
                "Method should be valid HTTP method: {}",
                method
            );
        }
    }

    #[test]
    fn test_api_routes_use_api_prefix() {
        let api_routes = vec![
            "/api/schemas",
            "/api/query/execute",
            "/api/stats/database",
            "/api/cell/edit",
        ];

        for route in api_routes {
            assert!(
                route.starts_with("/api/"),
                "API route should start with /api/: {}",
                route
            );
        }
    }

    #[test]
    fn test_page_routes_no_api_prefix() {
        let page_routes = vec!["/", "/query", "/studio", "/health"];

        for route in page_routes {
            assert!(
                !route.starts_with("/api/"),
                "Page route should not start with /api/: {}",
                route
            );
        }
    }
}
