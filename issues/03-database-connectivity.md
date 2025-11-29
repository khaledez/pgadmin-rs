# Issue #03: Database Connectivity and Connection Management

## Overview
Implement robust PostgreSQL database connectivity using SQLx with connection pooling, health checks, and error handling.

## Goals
- Establish secure database connections
- Implement connection pooling
- Create database service layer
- Handle connection failures gracefully
- Implement connection health monitoring

## Tasks

### 1. SQLx Setup and Configuration

**Connection pool configuration:**
```rust
// src/db/pool.rs
use sqlx::postgres::{PgPoolOptions, PgPool};

pub async fn create_pool(config: &Config) -> Result<PgPool, sqlx::Error> {
    let database_url = format!(
        "postgresql://{}:{}@{}:{}/{}",
        config.postgres_user,
        config.postgres_password,
        config.postgres_host,
        config.postgres_port,
        config.postgres_db
    );

    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&database_url)
        .await
}
```

**Configuration parameters to expose:**
- `MAX_DB_CONNECTIONS` (default: 20)
- `MIN_DB_CONNECTIONS` (default: 2)
- `DB_ACQUIRE_TIMEOUT` (default: 5s)
- `DB_IDLE_TIMEOUT` (default: 300s)
- `DB_MAX_LIFETIME` (default: 1800s)

### 2. Database Service Layer

Create a service layer to abstract database operations:

```rust
// src/services/db_service.rs
pub struct DatabaseService {
    pool: PgPool,
}

impl DatabaseService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Core operations
    pub async fn execute_query(&self, query: &str) -> Result<QueryResult> {
        // Execute raw SQL query with safety checks
    }

    pub async fn list_databases(&self) -> Result<Vec<Database>> {
        // Get all databases user has access to
    }

    pub async fn list_schemas(&self, database: &str) -> Result<Vec<Schema>> {
        // Get all schemas in a database
    }

    pub async fn list_tables(&self, schema: &str) -> Result<Vec<Table>> {
        // Get all tables in a schema
    }

    pub async fn get_table_info(&self, schema: &str, table: &str) -> Result<TableInfo> {
        // Get detailed table information
    }

    pub async fn test_connection(&self) -> Result<bool> {
        // Test database connectivity
    }
}
```

### 3. Database Introspection Queries

**List databases:**
```sql
SELECT datname, pg_database_size(datname) as size
FROM pg_database
WHERE datistemplate = false
ORDER BY datname;
```

**List schemas:**
```sql
SELECT schema_name
FROM information_schema.schemata
WHERE schema_name NOT IN ('pg_catalog', 'information_schema')
ORDER BY schema_name;
```

**List tables:**
```sql
SELECT
    table_name,
    table_type,
    pg_size_pretty(pg_total_relation_size(quote_ident(table_schema) || '.' || quote_ident(table_name))) as size
FROM information_schema.tables
WHERE table_schema = $1
ORDER BY table_name;
```

**Get table columns:**
```sql
SELECT
    column_name,
    data_type,
    character_maximum_length,
    is_nullable,
    column_default
FROM information_schema.columns
WHERE table_schema = $1 AND table_name = $2
ORDER BY ordinal_position;
```

**Get table indexes:**
```sql
SELECT
    indexname,
    indexdef
FROM pg_indexes
WHERE schemaname = $1 AND tablename = $2;
```

**Get foreign keys:**
```sql
SELECT
    tc.constraint_name,
    kcu.column_name,
    ccu.table_name AS foreign_table_name,
    ccu.column_name AS foreign_column_name
FROM information_schema.table_constraints AS tc
JOIN information_schema.key_column_usage AS kcu
  ON tc.constraint_name = kcu.constraint_name
JOIN information_schema.constraint_column_usage AS ccu
  ON ccu.constraint_name = tc.constraint_name
WHERE tc.constraint_type = 'FOREIGN KEY'
  AND tc.table_schema = $1
  AND tc.table_name = $2;
```

### 4. Data Models

Create Rust models for database objects:

```rust
// src/models/database.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct Database {
    pub name: String,
    pub size: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub table_type: String,
    pub size: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub max_length: Option<i32>,
    pub nullable: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub schema: String,
    pub columns: Vec<Column>,
    pub indexes: Vec<Index>,
    pub foreign_keys: Vec<ForeignKey>,
    pub row_count: i64,
}
```

### 5. Query Execution Safety

Implement safe query execution with safeguards:

**Dangerous operations to handle carefully:**
- DROP statements
- TRUNCATE statements
- DELETE without WHERE
- UPDATE without WHERE

**Safety measures:**
```rust
// src/services/query_validator.rs
pub struct QueryValidator;

impl QueryValidator {
    pub fn is_dangerous(query: &str) -> bool {
        let query_upper = query.to_uppercase();

        // Check for dangerous patterns
        if query_upper.contains("DROP") {
            return true;
        }

        // DELETE/UPDATE without WHERE
        if (query_upper.contains("DELETE") || query_upper.contains("UPDATE"))
            && !query_upper.contains("WHERE") {
            return true;
        }

        false
    }

    pub fn requires_confirmation(query: &str) -> bool {
        // List operations that need user confirmation
    }
}
```

### 6. Connection Health Monitoring

Implement connection health checks:

```rust
// src/services/health_service.rs
pub struct HealthService {
    pool: PgPool,
}

impl HealthService {
    pub async fn check_db_health(&self) -> HealthStatus {
        match sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
        {
            Ok(_) => HealthStatus::Healthy,
            Err(e) => HealthStatus::Unhealthy(e.to_string()),
        }
    }

    pub async fn get_pool_stats(&self) -> PoolStats {
        PoolStats {
            active_connections: self.pool.size(),
            idle_connections: self.pool.num_idle(),
        }
    }
}
```

### 7. Error Handling

Database-specific error handling:

```rust
// src/error.rs (extend existing)
pub enum DatabaseError {
    ConnectionFailed(String),
    QueryFailed(String),
    PoolExhausted,
    Timeout,
    InvalidQuery(String),
    PermissionDenied,
    TableNotFound(String),
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::PoolTimedOut => DatabaseError::PoolExhausted,
            sqlx::Error::RowNotFound => DatabaseError::TableNotFound("Row not found".into()),
            _ => DatabaseError::QueryFailed(err.to_string()),
        }
    }
}
```

### 8. Connection Retry Logic

Implement retry logic for transient failures:

```rust
pub async fn with_retry<F, T>(operation: F, max_retries: u32) -> Result<T>
where
    F: Fn() -> Future<Output = Result<T>>,
{
    let mut attempts = 0;
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempts < max_retries && is_retryable(&e) => {
                attempts += 1;
                sleep(Duration::from_millis(100 * 2_u64.pow(attempts))).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### 9. Query Result Pagination

Implement pagination for large result sets:

```rust
pub struct PaginationParams {
    pub page: u32,
    pub page_size: u32,
}

pub async fn paginated_query(
    query: &str,
    params: PaginationParams,
) -> Result<PaginatedResult> {
    let offset = (params.page - 1) * params.page_size;
    let limit = params.page_size;

    // Add LIMIT and OFFSET to query
    // Execute and return results with metadata
}
```

## File Structure
```
src/
├── db/
│   ├── mod.rs
│   └── pool.rs
├── services/
│   ├── mod.rs
│   ├── db_service.rs
│   ├── query_validator.rs
│   └── health_service.rs
├── models/
│   ├── mod.rs
│   ├── database.rs
│   ├── schema.rs
│   └── table.rs
```

## Testing Requirements
- [ ] Connection pool created successfully
- [ ] Connection pool handles exhaustion gracefully
- [ ] Database introspection queries work
- [ ] Query validator detects dangerous operations
- [ ] Health checks work correctly
- [ ] Connection retry logic functions
- [ ] Pagination works for large datasets
- [ ] All database errors handled appropriately

## Security Checklist
- [ ] No SQL injection vulnerabilities (use parameterized queries)
- [ ] Connection strings don't leak in logs
- [ ] Dangerous queries require confirmation
- [ ] User permissions respected
- [ ] Query timeout limits enforced
- [ ] Connection pool limits prevent DoS

## Performance Considerations
- Connection pooling properly configured
- Prepared statements for repeated queries
- Lazy loading for large result sets
- Pagination for table browsing
- Index on frequently queried columns
- Query timeout to prevent runaway queries

## Acceptance Criteria
- [ ] Database connection established via environment variables
- [ ] Connection pool configured and working
- [ ] All introspection queries implemented
- [ ] Query execution with safety checks
- [ ] Health monitoring functional
- [ ] Error handling comprehensive
- [ ] Tests pass
- [ ] Documentation complete
