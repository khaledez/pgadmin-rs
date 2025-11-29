# Issue #02: Backend Foundation

## Overview
Set up the core backend infrastructure using Axum, including routing, middleware, configuration management, and logging.

## Goals
- Create a functional Axum web server
- Implement configuration management from environment variables
- Set up logging and error handling
- Create basic middleware stack
- Establish routing structure

## Tasks

### 1. Project Initialization
- [ ] Initialize new Rust project: `cargo init`
- [ ] Configure Cargo.toml with required dependencies
- [ ] Set up workspace structure
- [ ] Create .env.example file with required environment variables

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
- [ ] Server starts successfully
- [ ] Health check endpoint returns 200
- [ ] Configuration loads from environment variables
- [ ] Missing required config causes startup failure
- [ ] Static files are served correctly
- [ ] Middleware chain executes in correct order
- [ ] Errors are handled gracefully

## Security Checklist
- [ ] No sensitive data in logs (passwords, tokens)
- [ ] Security headers configured
- [ ] Rate limiting in place
- [ ] Input validation framework ready
- [ ] Error messages don't leak implementation details

## Performance Considerations
- Async/await throughout
- Connection pooling configured
- Static file caching headers
- Efficient middleware ordering

## Documentation
- [ ] Document environment variables in .env.example
- [ ] Add inline code comments for complex logic
- [ ] Document middleware execution order
- [ ] Create basic API documentation

## Acceptance Criteria
- [ ] Server starts and listens on configured port
- [ ] Environment-based configuration works
- [ ] All middleware functions correctly
- [ ] Logging outputs structured logs
- [ ] Health check endpoint operational
- [ ] Error handling returns appropriate responses
- [ ] Static files served correctly
- [ ] Basic templates render
