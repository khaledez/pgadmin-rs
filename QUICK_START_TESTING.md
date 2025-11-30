# Quick Start: Running Tests Locally

## TL;DR - Run Tests in 30 seconds

```bash
# Option 1: With Docker (recommended)
make test

# Option 2: View what gets tested
make test-integration

# Option 3: With local PostgreSQL
export TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/pgadmin_test
cargo test
```

---

## Option 1: Docker (Recommended)

```bash
# Run all tests with Docker PostgreSQL
make test

# This automatically:
# 1. Starts PostgreSQL in Docker
# 2. Waits for database to be ready
# 3. Runs all tests
# 4. Cleans up containers
```

**Why Docker?**
- No local PostgreSQL installation needed
- Isolated test environment
- Exactly matches CI/CD pipeline
- Automatic cleanup

---

## Option 2: Local PostgreSQL

### Prerequisites
```bash
# Install PostgreSQL (macOS example)
brew install postgresql

# Start the service
brew services start postgresql

# Or manually
postgres -D /usr/local/var/postgres
```

### Run Tests
```bash
# Create test database
./scripts/setup-test-db.sh

# Run tests
TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/pgadmin_test cargo test

# Or as environment variable
export TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/pgadmin_test
cargo test
```

---

## What Gets Tested?

```
✓ Database Connection
  - Can connect to PostgreSQL
  
✓ Schema Operations
  - List all schemas
  - Check for public schema
  
✓ Table Operations
  - Create tables
  - List tables
  - Get column information
  
✓ Data CRUD
  - Insert data
  - Query data (SELECT)
  - Update data
  - Delete data
  
✓ Table Metadata
  - Row counting
  - Table sizing
  - Column introspection
```

---

## Available Make Targets

```bash
make test              # All tests with Docker
make test-integration  # Integration tests only
make test-no-docker    # Tests with local PostgreSQL
make fmt-check         # Check code formatting
make clippy            # Run linter
```

---

## Troubleshooting

### "connection refused"
```bash
# Ensure Docker is running
docker ps

# Check if PostgreSQL container is running
docker-compose ps

# Manually start it
docker-compose up -d postgres
sleep 5
```

### "password authentication failed"
```bash
# Verify TEST_DATABASE_URL
echo $TEST_DATABASE_URL

# Expected:
# postgresql://postgres:postgres@localhost:5432/pgadmin_test

# For Docker, credentials are:
# User: postgres
# Password: postgres
```

### Tests hang or timeout
```bash
# Run with single thread
cargo test -- --test-threads=1

# With timeout
timeout 60 cargo test
```

---

## View Test Output

```bash
# See what tests are running
cargo test -- --nocapture

# See test names without output
cargo test -- --list

# Run specific test
cargo test test_database_connection

# Run tests matching pattern
cargo test test_table
```

---

## CI/CD Pipeline

The same tests run automatically on:
- Every push to `main` or `develop`
- Every pull request to `main`

View status at: https://github.com/khaledez/pgadmin-rs/actions

---

## Quick Reference

| Task | Command |
|------|---------|
| Run all tests | `make test` |
| Run integration tests | `make test-integration` |
| Check code format | `make fmt-check` |
| Run linter | `make clippy` |
| Build release | `cargo build --release` |
| Build Docker image | `docker build -t pgadmin-rs:latest .` |

---

## Next Steps

After running tests successfully:

1. Make code changes
2. Run tests locally: `make test`
3. Check formatting: `make fmt-check`
4. Run linter: `make clippy`
5. Commit and push (CI/CD runs automatically)
6. Check GitHub Actions for results

---

## More Information

- Full testing guide: [TESTING.md](TESTING.md)
- CI/CD details: [CI_CD_SETUP.md](CI_CD_SETUP.md)
- Makefile targets: `make help`
