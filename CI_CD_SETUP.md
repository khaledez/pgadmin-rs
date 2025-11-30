# CI/CD Setup and Integration Tests

This document describes the continuous integration and testing infrastructure for pgAdmin-rs.

## Overview

The project now has a complete CI/CD pipeline with:
- ✅ GitHub Actions workflows
- ✅ Integration tests with PostgreSQL
- ✅ Code quality checks (formatting, linting)
- ✅ Automated builds
- ✅ Docker image validation

## GitHub Actions Workflow

**File**: `.github/workflows/ci.yml`

The workflow runs on:
- Every push to `main` or `develop` branches
- Every pull request to `main` branch

### Jobs

#### 1. **Test** (`test`)
- Runs all tests with a PostgreSQL service
- Single-threaded test execution for reliability
- Test database: `pgadmin_test`
- Duration: ~2-3 minutes

```bash
# Local equivalent
make test
```

#### 2. **Format Check** (`fmt`)
- Validates code formatting with `cargo fmt`
- Fails if code doesn't match Rust style standards

```bash
# Local equivalent
make fmt-check
```

#### 3. **Linting** (`clippy`)
- Runs Clippy with all warnings as errors (`-D warnings`)
- Checks all targets and features
- Identifies potential bugs and style issues

```bash
# Local equivalent
make clippy
```

#### 4. **Build** (`build`)
- Builds release binary
- Validates compilation with optimizations
- Caches build artifacts for faster runs

```bash
# Local equivalent
cargo build --release
```

#### 5. **Docker Build** (`docker`)
- Validates Docker image builds successfully
- Depends on all previous jobs
- Uses GitHub Actions cache for faster builds

```bash
# Local equivalent
docker build -t pgadmin-rs:latest .
```

## Integration Tests

**File**: `tests/integration_test.rs`

### Test Coverage

- ✅ Database connection
- ✅ Schema listing
- ✅ Table creation and operations
- ✅ Column introspection
- ✅ Data CRUD operations
- ✅ Row counting
- ✅ Table sizing

### Test Utilities

**File**: `tests/common/mod.rs`

Provides helpers for:
- Creating test database pools
- Seeding test data
- Cleaning up test tables

### Running Tests Locally

```bash
# All tests with Docker (recommended)
make test

# Integration tests only
make test-integration

# With local PostgreSQL (manual setup required)
make test-no-docker
```

### Test Database

The test database is created automatically:
- Via Docker Compose initialization script (`scripts/init-db.sh`)
- Via GitHub Actions PostgreSQL service
- Manually via `scripts/setup-test-db.sh`

**Database details**:
- Name: `pgadmin_test`
- User: `postgres` (default)
- Password: `postgres` (default)
- URL: `postgresql://postgres:postgres@localhost:5432/pgadmin_test`

## Setting Up Tests Locally

### Option 1: Docker Compose (Recommended)

```bash
# Start PostgreSQL in Docker
docker-compose up -d postgres

# Wait for it to be ready
sleep 5

# Run tests
TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/pgadmin_test cargo test

# Clean up
docker-compose down
```

### Option 2: Using Make

```bash
# All tests with Docker
make test

# Integration tests only
make test-integration

# Manual PostgreSQL setup required
make test-no-docker
```

### Option 3: Manual PostgreSQL Setup

```bash
# Create test database
./scripts/setup-test-db.sh

# Run tests
export TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/pgadmin_test
cargo test
```

## Writing Integration Tests

### Basic Template

```rust
#[tokio::test]
async fn test_my_feature() {
    let pool = create_test_pool().await;
    let _ = cleanup_test_data(&pool).await;
    let _ = seed_test_data(&pool).await;
    
    // Arrange
    // Act
    // Assert
    
    // Clean up
    let _ = cleanup_test_data(&pool).await;
}
```

### Using Test Utilities

```rust
use common::{create_test_pool, cleanup_test_data, seed_test_data};

// Create a connection pool
let pool = create_test_pool().await;

// Seed test data (creates users table with sample data)
let _ = seed_test_data(&pool).await;

// Your test code here
// ...

// Clean up
let _ = cleanup_test_data(&pool).await;
```

### Adding New Test Tables

Modify `tests/common/mod.rs` to add more seeding:

```rust
pub async fn seed_test_data(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Existing users table code...
    
    // Add new table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS posts (
            id SERIAL PRIMARY KEY,
            title VARCHAR(200),
            user_id INTEGER REFERENCES users(id)
        )"
    )
    .execute(pool)
    .await?;
    
    Ok(())
}
```

## Code Quality Checks

### Local Development

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
cargo make fmt-check

# Run linter
cargo clippy --all-targets --all-features -- -D warnings
make clippy

# Run tests
cargo test
make test

# Combined quality check
make fmt-check && make clippy && make test
```

## CI/CD Workflow

```
Push to main/develop
    ↓
GitHub Actions triggered
    ├─→ Run tests (PostgreSQL service)
    ├─→ Check formatting
    ├─→ Run clippy linter
    ├─→ Build release binary
    └─→ Validate Docker build
    ↓
All jobs pass → ✅ Ready to merge
Any job fails  → ❌ Fix and push again
```

## Cache Strategy

The workflow uses GitHub Actions caching for:
- `~/.cargo/registry` - Dependency crates
- `~/.cargo/git` - Git dependencies
- `target/` - Build artifacts
- Docker layers - BuildKit cache

This reduces test runtime from ~5 min to ~2-3 min on average.

## Troubleshooting

### Tests fail with "connection refused"

**In GitHub Actions**:
- PostgreSQL service is automatically started
- Tests wait for service health check

**Locally**:
- Ensure PostgreSQL is running: `docker-compose up -d postgres`
- Check TEST_DATABASE_URL is correct
- Wait a few seconds for the database to be ready

### "password authentication failed"

- Check PostgreSQL credentials match TEST_DATABASE_URL
- Default: `postgres:postgres`

### Clippy fails with "unused code"

The rate limiting middleware is currently unused but available for future integration:
- File: `src/middleware/rate_limit.rs`
- Can be suppressed or removed if not needed

### Build takes too long

- GitHub Actions caches are per-branch
- First run on a new branch will be slower
- Subsequent runs will use cache

## Best Practices

1. **Run tests locally before pushing**
   ```bash
   make test
   ```

2. **Use meaningful commit messages** for better CI/CD tracking

3. **Keep tests isolated** - Each test should be independent

4. **Clean up test data** - Always cleanup after tests

5. **Run clippy locally** before committing
   ```bash
   make clippy
   ```

6. **Check formatting** before pushing
   ```bash
   make fmt-check
   ```

## Monitoring and Maintenance

### View CI/CD Status

1. Go to: https://github.com/khaledez/pgadmin-rs/actions
2. Select the latest workflow run
3. View job logs for details

### Disable Workflow Temporarily

Edit `.github/workflows/ci.yml` and change `on:` section:
```yaml
on:
  workflow_dispatch:  # Manual trigger only
```

### View Test Logs

In GitHub Actions:
1. Select failing job
2. Expand the test step
3. View output and errors

## Future Enhancements

- [ ] Code coverage reporting (Codecov)
- [ ] Performance benchmarks
- [ ] Security scanning (cargo audit)
- [ ] Database migration tests
- [ ] End-to-end browser tests
- [ ] Deployment to staging environment

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [SQLx Testing](https://github.com/launchbadge/sqlx)
- [Tokio Async Testing](https://tokio.rs/)
