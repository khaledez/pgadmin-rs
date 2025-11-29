# Issue #08: Testing and Quality Assurance

## Overview
Implement comprehensive testing strategy including unit tests, integration tests, security tests, and quality assurance measures.

## Goals
- Achieve high test coverage
- Ensure code quality
- Validate security measures
- Automate testing in CI/CD
- Maintain code standards

## Testing Strategy

### Test Pyramid
```
         ╱╲
        ╱E2E╲         Small number (critical user flows)
       ╱──────╲
      ╱Integr.╲       Medium number (API endpoints, DB)
     ╱──────────╲
    ╱Unit Tests ╲     Large number (business logic)
   ╱──────────────╲
```

## Tasks

### 1. Unit Testing Setup

**Configure test environment:**

```toml
# Cargo.toml
[dev-dependencies]
tokio-test = "0.4"
mockall = "0.12"
assert_matches = "1.5"
pretty_assertions = "1.4"
```

**Example unit tests:**

```rust
// src/services/query_validator.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_dangerous_drop_statement() {
        let query = "DROP TABLE users;";
        assert!(QueryValidator::is_dangerous(query));
    }

    #[test]
    fn test_is_dangerous_delete_without_where() {
        let query = "DELETE FROM users;";
        assert!(QueryValidator::is_dangerous(query));
    }

    #[test]
    fn test_is_safe_select_query() {
        let query = "SELECT * FROM users WHERE id = 1;";
        assert!(!QueryValidator::is_dangerous(query));
    }

    #[test]
    fn test_is_dangerous_case_insensitive() {
        let query = "drop table users;";
        assert!(QueryValidator::is_dangerous(query));
    }

    #[test]
    fn test_requires_confirmation_update_without_where() {
        let query = "UPDATE users SET active = false;";
        assert!(QueryValidator::requires_confirmation(query));
    }
}
```

**Test coverage goals:**
- Core business logic: 90%+
- Validation functions: 100%
- Error handling: 80%+
- Overall coverage: 75%+

### 2. Integration Testing

**Database integration tests:**

```rust
// tests/integration_test.rs
use sqlx::PgPool;
use pgadmin_rs::services::DatabaseService;

#[sqlx::test]
async fn test_list_tables(pool: PgPool) -> sqlx::Result<()> {
    // Create test table
    sqlx::query("CREATE TABLE test_table (id INTEGER PRIMARY KEY)")
        .execute(&pool)
        .await?;

    let db_service = DatabaseService::new(pool);
    let tables = db_service.list_tables("public").await?;

    assert!(tables.iter().any(|t| t.name == "test_table"));

    Ok(())
}

#[sqlx::test]
async fn test_get_table_info(pool: PgPool) -> sqlx::Result<()> {
    sqlx::query(
        "CREATE TABLE users (
            id SERIAL PRIMARY KEY,
            username VARCHAR(50) NOT NULL,
            email VARCHAR(100)
        )"
    )
    .execute(&pool)
    .await?;

    let db_service = DatabaseService::new(pool);
    let info = db_service.get_table_info("public", "users").await?;

    assert_eq!(info.columns.len(), 3);
    assert!(info.columns.iter().any(|c| c.name == "username"));

    Ok(())
}

#[sqlx::test]
async fn test_execute_safe_query(pool: PgPool) -> sqlx::Result<()> {
    let db_service = DatabaseService::new(pool);
    let result = db_service.execute_query("SELECT 1 as num").await?;

    assert_eq!(result.row_count, 1);

    Ok(())
}
```

**Setup test database:**

```bash
# scripts/setup-test-db.sh
#!/bin/bash

export TEST_DATABASE_URL="postgresql://postgres:postgres@localhost:5432/pgadmin_test"

# Create test database
psql -U postgres -c "DROP DATABASE IF EXISTS pgadmin_test;"
psql -U postgres -c "CREATE DATABASE pgadmin_test;"

# Run migrations if needed
sqlx migrate run --database-url $TEST_DATABASE_URL
```

### 3. API/Route Testing

**Test HTTP endpoints:**

```rust
// tests/api_test.rs
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use pgadmin_rs::create_app;

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_app().await;

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
async fn test_unauthenticated_access_redirects() {
    let app = create_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/query")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn test_query_execution() {
    let app = create_app().await;

    // Login first
    let login_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("password=admin"))
                .unwrap(),
        )
        .await
        .unwrap();

    // Extract session cookie
    let cookie = login_response
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap();

    // Execute query
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/query/execute")
                .header("cookie", cookie)
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("query=SELECT 1"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

### 4. Security Testing

**SQL injection tests:**

```rust
#[tokio::test]
async fn test_sql_injection_prevention() {
    let pool = get_test_pool().await;
    let db_service = DatabaseService::new(pool);

    // Attempt SQL injection
    let malicious_input = "'; DROP TABLE users; --";

    let result = db_service
        .get_table_data("public", malicious_input, 1, 100, None, None)
        .await;

    // Should fail safely without executing DROP
    assert!(result.is_err());
}

#[test]
fn test_identifier_validation() {
    // Valid identifiers
    assert!(InputValidator::validate_identifier("users").is_ok());
    assert!(InputValidator::validate_identifier("user_data").is_ok());

    // Invalid identifiers (SQL injection attempts)
    assert!(InputValidator::validate_identifier("users; DROP TABLE").is_err());
    assert!(InputValidator::validate_identifier("../etc/passwd").is_err());
    assert!(InputValidator::validate_identifier("users--").is_err());
}
```

**XSS protection tests:**

```rust
#[test]
fn test_template_escaping() {
    let template = QueryResultTemplate {
        rows: vec![vec![
            serde_json::Value::String("<script>alert('xss')</script>".to_string())
        ]],
        columns: vec!["data".to_string()],
        row_count: 1,
        duration: Duration::from_secs(1),
    };

    let rendered = template.render().unwrap();

    // Should not contain unescaped script tags
    assert!(!rendered.contains("<script>alert('xss')</script>"));
    assert!(rendered.contains("&lt;script&gt;"));
}
```

**CSRF protection tests:**

```rust
#[tokio::test]
async fn test_csrf_protection() {
    let app = create_app().await;

    // Attempt POST without CSRF token
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/query/execute")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("query=SELECT 1"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
```

**Authentication tests:**

```rust
#[tokio::test]
async fn test_login_with_valid_password() {
    let app = create_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("password=correct_password"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert!(response.headers().contains_key("set-cookie"));
}

#[tokio::test]
async fn test_login_with_invalid_password() {
    let app = create_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("password=wrong_password"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
```

### 5. Performance Testing

**Load testing with criterion:**

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["async_tokio"] }

[[bench]]
name = "query_benchmark"
harness = false
```

```rust
// benches/query_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pgadmin_rs::services::QueryValidator;

fn benchmark_query_validation(c: &mut Criterion) {
    c.bench_function("validate_select_query", |b| {
        b.iter(|| {
            QueryValidator::is_dangerous(black_box("SELECT * FROM users WHERE id = 1"))
        })
    });

    c.bench_function("validate_complex_query", |b| {
        b.iter(|| {
            QueryValidator::is_dangerous(black_box(
                "SELECT u.*, o.* FROM users u JOIN orders o ON u.id = o.user_id WHERE u.active = true"
            ))
        })
    });
}

criterion_group!(benches, benchmark_query_validation);
criterion_main!(benches);
```

### 6. Code Quality Tools

**Clippy configuration:**

```toml
# Cargo.toml
[lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
```

**Run quality checks:**

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Check for security vulnerabilities
cargo audit

# Check for unused dependencies
cargo machete
```

### 7. Continuous Integration

**GitHub Actions workflow:**

```yaml
# .github/workflows/test.yml
name: Test

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always
  DATABASE_URL: postgresql://postgres:postgres@localhost:5432/pgadmin_test

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: pgadmin_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v3

      - uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true
          components: rustfmt, clippy

      - uses: Swatinem/rust-cache@v2

      - name: Run tests
        run: cargo test --all-features --verbose

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Security audit
        run: |
          cargo install cargo-audit
          cargo audit

  coverage:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: pgadmin_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v3

      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        run: cargo tarpaulin --verbose --all-features --workspace --timeout 120 --out Xml

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
```

### 8. Test Data Fixtures

**Create reusable test data:**

```rust
// tests/fixtures/mod.rs
use sqlx::PgPool;

pub struct TestDatabase {
    pub pool: PgPool,
}

impl TestDatabase {
    pub async fn new() -> Self {
        let pool = PgPool::connect(&std::env::var("TEST_DATABASE_URL").unwrap())
            .await
            .unwrap();

        Self { pool }
    }

    pub async fn seed_data(&self) -> sqlx::Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                username VARCHAR(50) NOT NULL,
                email VARCHAR(100)
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "INSERT INTO users (username, email) VALUES
             ('alice', 'alice@example.com'),
             ('bob', 'bob@example.com'),
             ('charlie', 'charlie@example.com')"
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn cleanup(&self) -> sqlx::Result<()> {
        sqlx::query("DROP TABLE IF EXISTS users CASCADE")
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
```

### 9. Documentation Tests

**Test code examples in documentation:**

```rust
/// Validates a PostgreSQL identifier
///
/// # Examples
///
/// ```
/// use pgadmin_rs::validation::InputValidator;
///
/// assert!(InputValidator::validate_identifier("users").is_ok());
/// assert!(InputValidator::validate_identifier("user_data").is_ok());
/// assert!(InputValidator::validate_identifier("users; DROP").is_err());
/// ```
pub fn validate_identifier(name: &str) -> Result<()> {
    // Implementation
}
```

### 10. End-to-End Testing (Optional)

**Browser automation with headless Chrome:**

```toml
[dev-dependencies]
fantoccini = "0.19"
```

```rust
// tests/e2e_test.rs
use fantoccini::{Client, ClientBuilder};

#[tokio::test]
async fn test_login_flow() {
    let client = ClientBuilder::native()
        .connect("http://localhost:4444")
        .await
        .expect("failed to connect to WebDriver");

    client.goto("http://localhost:8080/login").await.unwrap();

    let password_input = client.find(Locator::Css("input[name='password']")).await.unwrap();
    password_input.send_keys("admin").await.unwrap();

    let submit_button = client.find(Locator::Css("button[type='submit']")).await.unwrap();
    submit_button.click().await.unwrap();

    // Should redirect to dashboard
    let url = client.current_url().await.unwrap();
    assert!(url.path().contains("/dashboard"));

    client.close().await.unwrap();
}
```

## File Structure
```
tests/
├── integration_test.rs
├── api_test.rs
├── security_test.rs
└── fixtures/
    └── mod.rs

benches/
└── query_benchmark.rs

scripts/
├── setup-test-db.sh
└── run-tests.sh
```

## Testing Checklist
- [ ] Unit tests for all business logic
- [ ] Integration tests for database operations
- [ ] API endpoint tests
- [ ] Security tests (SQL injection, XSS, CSRF)
- [ ] Authentication tests
- [ ] Performance benchmarks
- [ ] Code coverage >75%
- [ ] All tests pass in CI/CD
- [ ] Documentation tests
- [ ] Error handling tests

## Quality Metrics
- **Code Coverage**: >75% overall, >90% for critical paths
- **Test Execution Time**: <2 minutes for full suite
- **Performance**: Query validation <1ms
- **Security**: 0 critical vulnerabilities

## Acceptance Criteria
- [ ] Comprehensive test suite implemented
- [ ] CI/CD pipeline runs all tests
- [ ] Code coverage meets targets
- [ ] Security tests validate protections
- [ ] Performance benchmarks established
- [ ] Documentation complete
- [ ] All quality checks automated
