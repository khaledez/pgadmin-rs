# Testing Guide

This document describes how to run and write tests for pgAdmin-rs.

## Prerequisites

- Rust 1.70+
- PostgreSQL 14+ (for integration tests)
- Docker (optional, for containerized testing)

## Running Tests

### Using Docker Compose (Recommended)

The easiest way to run tests is using Docker Compose, which automatically sets up PostgreSQL:

```bash
# Start PostgreSQL in Docker
docker-compose up -d postgres

# Run all tests
cargo test --all-features

# Run specific test
cargo test test_database_connection

# Run tests with output
cargo test -- --nocapture

# Stop PostgreSQL
docker-compose down
```

### Manual Setup (Local PostgreSQL)

If you have PostgreSQL installed locally:

```bash
# Create test database
./scripts/setup-test-db.sh

# Run tests
TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/pgadmin_test cargo test

# Or set it as an environment variable
export TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/pgadmin_test
cargo test
```

## Test Structure

```
tests/
├── integration_test.rs    # Integration tests with database
├── common/
│   └── mod.rs             # Common test utilities
└── fixtures/              # Test data and helpers
```

## Test Categories

### Integration Tests (`tests/integration_test.rs`)

Test database operations and API endpoints:

- Database connection
- Schema listing
- Table creation and listing
- Column introspection
- Data queries (SELECT, INSERT, UPDATE, DELETE)
- Row counting
- Table sizing

**Run integration tests:**
```bash
cargo test --test integration_test
```

## Writing Tests

### Basic Test Template

```rust
#[tokio::test]
async fn test_something() {
    let pool = create_test_pool().await;
    let _ = cleanup_test_data(&pool).await;
    let _ = seed_test_data(&pool).await;
    
    // Your test code here
    
    // Clean up
    let _ = cleanup_test_data(&pool).await;
}
```

### Using Test Utilities

```rust
use common::{create_test_pool, cleanup_test_data, seed_test_data};

// Create a connection pool for testing
let pool = create_test_pool().await;

// Seed test data (creates users table with sample data)
seed_test_data(&pool).await?;

// Clean up after tests
cleanup_test_data(&pool).await?;
```

## Continuous Integration

Tests automatically run on:
- Every push to `main` or `develop` branches
- Every pull request to `main` branch

The CI pipeline runs:
- Cargo test (all features)
- Rustfmt (code formatting)
- Clippy (linting)
- Docker build validation

See `.github/workflows/ci.yml` for the full pipeline.

## Test Coverage

Current test coverage focuses on:
- ✅ Database connectivity
- ✅ Schema operations
- ✅ Table operations
- ✅ Data CRUD operations
- ⚠️ API endpoints (in progress)
- ⚠️ Security validation (in progress)
- ⚠️ Error handling (in progress)

## Performance Testing

For performance testing, use `cargo bench`:

```bash
# Install benchmark dependencies
cargo add --dev criterion

# Run benchmarks
cargo bench
```

## Debugging Tests

### Run a single test with output:
```bash
cargo test test_database_connection -- --nocapture
```

### Run tests without parallelization:
```bash
cargo test -- --test-threads=1
```

### Run with RUST_BACKTRACE:
```bash
RUST_BACKTRACE=1 cargo test
```

### Check test compilation:
```bash
cargo test --no-run
```

## Common Issues

### "password authentication failed"
- Ensure PostgreSQL is running
- Check TEST_DATABASE_URL is correct
- For Docker: `docker-compose up -d postgres`

### "connection refused"
- PostgreSQL not listening on expected host/port
- Default: localhost:5432
- Check `.env` or TEST_DATABASE_URL

### Test hangs
- Set timeout: `cargo test -- --test-threads=1 --timeout 30`
- Check database connection

## Best Practices

1. **Isolation**: Each test should be independent
2. **Cleanup**: Always clean up test data after tests
3. **Naming**: Use descriptive test names that start with `test_`
4. **Documentation**: Add comments for complex test logic
5. **Speed**: Keep tests fast; mock external dependencies if needed
6. **Clarity**: Arrange-Act-Assert pattern
   ```rust
   // Arrange
   let pool = create_test_pool().await;
   
   // Act
   let result = some_operation(&pool).await;
   
   // Assert
   assert!(result.is_ok());
   ```

## References

- [Rust Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [SQLx Testing](https://github.com/launchbadge/sqlx/blob/main/sqlx-core/README.md#compile-time-verification)
- [Tokio Testing](https://tokio.rs/tokio/tutorial/select#the-select-macro)
