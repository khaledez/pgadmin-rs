# Issue #04: Security and Authentication

## Overview
Implement comprehensive security measures including protection against common web vulnerabilities, secure communication, and audit logging. Authentication/authorization is out of scope for this project and should be managed externally.

## Goals
- [x] Protect against common web vulnerabilities (partial - XSS done, others placeholder)
- [ ] Ensure secure communication
- [ ] Implement audit logging
- [x] Secure database query execution

## Security Principles
1. **Defense in depth**: Multiple layers of security
2. **Least privilege**: Minimal access by default
3. **Fail securely**: Errors should not expose sensitive information
4. **Security by default**: Secure settings out of the box
5. **Input validation**: Validate all user inputs

## Tasks

> **Note**: Authentication and authorization are out of scope for this project. The application is intended to be used in secure environments where access control is managed externally.

### 1. SQL Injection Prevention

**Use parameterized queries exclusively:**

```rust
// GOOD - Parameterized query
pub async fn get_table_data(
    &self,
    schema: &str,
    table: &str,
) -> Result<Vec<Row>> {
    // Use identifier quoting for table/schema names
    let query = format!(
        "SELECT * FROM {}.{} LIMIT 1000",
        Self::quote_identifier(schema),
        Self::quote_identifier(table)
    );

    sqlx::query(&query)
        .fetch_all(&self.pool)
        .await
}

fn quote_identifier(name: &str) -> String {
    // Escape and quote identifiers
    format!("\"{}\"", name.replace("\"", "\"\""))
}

// For user SQL queries, use a safe execution context
pub async fn execute_user_query(&self, query: &str) -> Result<QueryResult> {
    // Validate query before execution
    self.validator.validate(query)?;

    // Execute in read-only transaction if query is SELECT
    if self.is_read_only_query(query) {
        self.execute_readonly(query).await
    } else {
        // Require explicit confirmation for write operations
        Err(QueryError::WriteOperationRequiresConfirmation)
    }
}
```

**Input validation:**
```rust
// src/validation/mod.rs
pub struct InputValidator;

impl InputValidator {
    pub fn validate_identifier(name: &str) -> Result<()> {
        // Check for valid PostgreSQL identifier
        let regex = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();

        if !regex.is_match(name) {
            return Err(ValidationError::InvalidIdentifier(name.to_string()));
        }

        // Check length
        if name.len() > 63 {
            return Err(ValidationError::IdentifierTooLong);
        }

        Ok(())
    }

    pub fn sanitize_sql_query(query: &str) -> String {
        // Remove comments
        // Validate syntax
        // Check for dangerous patterns
    }
}
```

- [ ] Input validation for identifiers
- [ ] SQL query validation before execution
- [ ] Result set size limits
- [ ] Query timeout handling

### 2. Cross-Site Scripting (XSS) Prevention

**Template auto-escaping:**
```html
<!-- Askama templates auto-escape by default -->
<div>{{ user_input }}</div>  <!-- Automatically escaped -->

<!-- For raw HTML (use sparingly, only for trusted content) -->
<div>{{ user_input|safe }}</div>  <!-- NOT escaped - dangerous! -->
```

- [x] Askama template engine with auto-escaping enabled (index.html)
- [x] Review all templates to ensure no unsafe markup

**Content Security Policy:**
```rust
// src/middleware/security_headers.rs
pub fn security_headers_middleware() -> impl Fn(Response) -> Response {
    move |mut response| {
        response.headers_mut().insert(
            "Content-Security-Policy",
            "default-src 'self'; \
             script-src 'self' https://unpkg.com; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data:; \
             font-src 'self'; \
             connect-src 'self'; \
             frame-ancestors 'none';"
            .parse().unwrap()
        );

        response.headers_mut().insert(
            "X-Content-Type-Options",
            "nosniff".parse().unwrap()
        );

        response.headers_mut().insert(
            "X-Frame-Options",
            "DENY".parse().unwrap()
        );

        response.headers_mut().insert(
            "X-XSS-Protection",
            "1; mode=block".parse().unwrap()
        );

        response.headers_mut().insert(
            "Referrer-Policy",
            "strict-origin-when-cross-origin".parse().unwrap()
        );

        response
    }
}
```

- [ ] Implement security headers middleware
- [ ] Add CSP header
- [ ] Add X-Content-Type-Options
- [ ] Add X-Frame-Options
- [ ] Add Referrer-Policy

### 3. Rate Limiting ✅

**Status: IMPLEMENTED**

Rate limiting has been implemented using the `governor` crate with a token bucket algorithm:

```rust
// src/middleware/rate_limit.rs - IMPLEMENTED
use governor::{Quota, RateLimiter};

pub struct RateLimitState {
    limiters: LimiterMap,
    config: RateLimitConfig,
}

impl RateLimitState {
    pub fn new(config: RateLimitConfig) -> Self {
        // Configurable requests per minute per IP
        Self {
            limiters: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            config,
        }
    }

    pub fn check_limit(&self, ip: &str) -> bool {
        // Returns true if request allowed, false if limit exceeded
    }
}
```

**Configuration:**
- Environment variable: `RATE_LIMIT_REQUESTS_PER_MINUTE`
- Default: 100 requests per minute per IP
- Development: 1000 (high limit for testing)
- Production: 100 (balanced protection)
- Strict: 30 (aggressive rate limiting)

**Features:**
- ✅ Per-IP tracking with separate token buckets
- ✅ Configurable via environment variables
- ✅ Returns 429 Too Many Requests when limit exceeded
- ✅ Integrated into main middleware stack

**Endpoint-specific limits (prepared for future use):**
- Query execution: 20/minute
- Table browsing: 100/minute
- Schema operations: 10/minute
- General API: 100/minute

- [x] Implement rate limiting middleware
- [x] Configure global rate limit
- [ ] Configure endpoint-specific rate limits (future enhancement)

### 4. Audit Logging

**Log security-relevant events:**

```rust
// src/audit/logger.rs
pub struct AuditLogger {
    pool: PgPool,
}

pub struct AuditEvent {
    pub user_id: Option<String>,
    pub ip_address: String,
    pub action: String,
    pub resource: String,
    pub success: bool,
    pub details: Option<String>,
}

impl AuditLogger {
    pub async fn log(&self, event: AuditEvent) -> Result<()> {
        sqlx::query!(
            "INSERT INTO audit_log (user_id, ip_address, action, resource, success, details, timestamp)
             VALUES ($1, $2, $3, $4, $5, $6, NOW())",
            event.user_id,
            event.ip_address,
            event.action,
            event.resource,
            event.success,
            event.details
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
```

**Events to log:**
- Query executions
- Schema modifications
- Table data changes
- Rate limit violations
- SQL errors

- [ ] Audit logger implementation
- [ ] Audit log table schema
- [ ] Log query executions
- [ ] Log schema modifications

### 5. Secure Configuration

**Configuration security checklist:**
```rust
pub fn validate_security_config(config: &Config) -> Result<()> {
    // Check if TLS is configured (in production)
    if config.is_production() && !config.tls_enabled {
        return Err(ConfigError::TlsRequired);
    }

    // Ensure database credentials are not logged
    // Ensure sensitive data is not exposed in error messages

    Ok(())
}
```

- [ ] Validate security configuration on startup
- [ ] No hardcoded secrets in code
- [ ] Sensitive data not logged

## File Structure
```
src/
├── middleware/
│   ├── security_headers.rs
│   └── rate_limit.rs
├── audit/
│   ├── mod.rs
│   └── logger.rs
└── validation/
    ├── mod.rs
    └── input.rs
```

## Testing Requirements
- [ ] SQL injection attempts blocked
- [ ] XSS attempts neutralized
- [ ] Security headers present in responses
- [ ] Audit log captures events correctly
- [ ] Rate limiting triggers correctly
- [ ] Input validation works correctly

## Security Audit Checklist
- [ ] No hardcoded secrets in code
- [ ] All user inputs validated
- [ ] Parameterized queries used exclusively
- [ ] Template auto-escaping enabled
- [ ] HTTPS enforced in production
- [ ] Security headers configured
- [ ] Rate limiting active
- [ ] Audit logging comprehensive
- [ ] Error messages don't leak sensitive info
- [ ] Dependencies scanned for vulnerabilities

## Compliance Considerations
- GDPR: Audit logs, data privacy
- SOC 2: Access controls, audit logging
- PCI DSS: Strong encryption, secure communication

## Acceptance Criteria
- [ ] Security middleware active
- [ ] Input validation comprehensive
- [ ] Audit logging working
- [ ] No security vulnerabilities in static analysis
- [ ] Security tests pass
- [ ] Documentation complete
