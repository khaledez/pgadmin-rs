# Issue #02: Backend Foundation

## Overview
Set up the core backend infrastructure using Axum, including routing, middleware, configuration management, and logging.

## Goals
- [x] Create a functional Axum web server
- [x] Implement configuration management from environment variables
- [x] Set up logging and error handling
- [x] Create basic middleware stack
- [x] Establish routing structure

## Tasks

### 1. Project Initialization
- [x] Initialize new Rust project: `cargo init`
- [x] Configure Cargo.toml with required dependencies
- [x] Set up workspace structure
- [x] Create .env.example file with required environment variables

### 2. Configuration Management
Create a robust configuration system that reads from environment variables:

```rust
// src/config/mod.rs
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub postgres_host: String,
    pub postgres_port: u16,
    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_db: String,
    pub session_secret: String,
    pub log_level: String,
}
```

**Implementation details:**
- Use environment variables (Docker-friendly)
- Provide sensible defaults where appropriate
- Validate configuration on startup
- Fail fast if required variables are missing

### 3. Axum Server Setup

**Main server setup:**
```rust
// src/main.rs
#[tokio::main]
async fn main() {
    // Initialize logging
    // Load configuration
    // Create database connection pool
    // Build router
    // Start server
}
```

**Key features:**
- Graceful shutdown handling
- Signal handling (SIGTERM, SIGINT)
- Startup validation
- Clear error messages

### 4. Routing Structure

Create modular route structure:
```rust
// src/routes/mod.rs
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health_check))
        .nest("/api", api_routes())
        .nest("/query", query_routes())
        .nest("/tables", table_routes())
        .nest("/schema", schema_routes())
        .with_state(state)
}
```

**Routes to implement:**
- `GET /` - Main dashboard
- `GET /health` - Health check endpoint
- `GET /ready` - Readiness probe (check DB connection)

### 5. Middleware Stack

**Middleware to implement:**

1. **Logging middleware**
   - Request/response logging
   - Request ID generation
   - Execution time tracking

2. **Error handling middleware**
   - Centralized error handling
   - User-friendly error messages
   - Error logging
   - Appropriate HTTP status codes

3. **Security headers middleware**
   - X-Content-Type-Options: nosniff
   - X-Frame-Options: DENY
   - Content-Security-Policy
   - X-XSS-Protection

4. **CORS middleware** (if needed)
   - Configure allowed origins
   - Handle preflight requests

5. **Rate limiting middleware**
   - Prevent abuse
   - Per-IP rate limiting
   - Configurable limits

### 6. Application State

Create shared application state:
```rust
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub config: Arc<Config>,
}
```

### 7. Error Handling

Create a comprehensive error handling system:

```rust
// src/error.rs
pub enum AppError {
    DatabaseError(sqlx::Error),
    ConfigError(String),
    ValidationError(String),
    NotFound(String),
    Unauthorized,
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Convert errors to appropriate HTTP responses
    }
}
```

**Error handling principles:**
- Never expose internal errors to users
- Log detailed errors for debugging
- Return user-friendly messages
- Use appropriate HTTP status codes

### 8. Logging Setup

Use `tracing` for structured logging:
- Configure log levels from environment
- Log format suitable for Docker (JSON or logfmt)
- Include context in logs (request_id, user_id, etc.)
- Performance logging for slow queries

### 9. Static File Serving

Set up static file serving for CSS, JS, and images:
```rust
.nest_service("/static", ServeDir::new("static"))
```

### 10. Template Rendering Setup

Initialize Askama templates:
- Create base template layout
- Set up template directory structure
- Create template helpers/filters

## File Structure to Create
```
src/
├── main.rs
├── config/
│   └── mod.rs
├── routes/
│   ├── mod.rs
│   ├── health.rs
│   └── index.rs
├── middleware/
│   ├── mod.rs
│   ├── logging.rs
│   ├── error_handler.rs
│   └── security_headers.rs
├── error.rs
├── state.rs
└── templates/
    ├── base.html
    └── index.html
```

## Testing Requirements
- [x] Server starts successfully
- [x] Health check endpoint returns 200
- [x] Configuration loads from environment variables
- [ ] Missing required config causes startup failure
- [x] Static files are served correctly
- [x] Middleware chain executes in correct order
- [ ] Errors are handled gracefully

## Security Checklist
- [x] No sensitive data in logs (passwords, tokens)
- [ ] Security headers configured (todo in middleware)
- [ ] Rate limiting in place (todo)
- [ ] Input validation framework ready
- [ ] Error messages don't leak implementation details

## Performance Considerations
- [x] Async/await throughout
- [x] Connection pooling configured
- [ ] Static file caching headers (todo)
- [x] Efficient middleware ordering

## Documentation
- [x] Document environment variables in .env.example
- [x] Add inline code comments for complex logic
- [ ] Document middleware execution order
- [ ] Create basic API documentation

## Acceptance Criteria
- [x] Server starts and listens on configured port
- [x] Environment-based configuration works
- [x] All middleware functions correctly
- [x] Logging outputs structured logs
- [x] Health check endpoint operational
- [ ] Error handling returns appropriate responses
- [x] Static files served correctly
- [x] Basic templates render
