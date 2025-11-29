# Issue #04: Security and Authentication

## Overview
Implement comprehensive security measures including authentication, authorization, session management, and protection against common web vulnerabilities.

## Goals
- Secure the application with authentication
- Implement session management
- Protect against common web vulnerabilities
- Ensure secure communication
- Implement audit logging

## Security Principles
1. **Defense in depth**: Multiple layers of security
2. **Least privilege**: Minimal access by default
3. **Fail securely**: Errors should not expose sensitive information
4. **Security by default**: Secure settings out of the box
5. **Input validation**: Validate all user inputs

## Tasks

### 1. Authentication System

**Simple session-based authentication:**

```rust
// src/auth/session.rs
use tower_sessions::{Session, MemoryStore, SessionManagerLayer};

pub struct AuthService {
    session_secret: String,
}

impl AuthService {
    pub async fn login(&self, session: &Session, password: &str) -> Result<()> {
        if self.verify_password(password).await? {
            session.insert("authenticated", true).await?;
            session.insert("login_time", Utc::now()).await?;
            Ok(())
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }

    pub async fn logout(&self, session: &Session) -> Result<()> {
        session.delete().await?;
        Ok(())
    }

    pub async fn is_authenticated(&self, session: &Session) -> bool {
        session.get::<bool>("authenticated")
            .await
            .unwrap_or(false)
            .unwrap_or(false)
    }
}
```

**Environment variables:**
- `APP_PASSWORD` - Simple password protection for the entire app
- `SESSION_SECRET` - Secret for signing session cookies
- `SESSION_TIMEOUT` - Session timeout in seconds (default: 3600)

### 2. Authentication Middleware

```rust
// src/middleware/auth.rs
pub async fn require_auth(
    session: Session,
    req: Request<Body>,
    next: Next,
) -> Result<Response> {
    let authenticated = session
        .get::<bool>("authenticated")
        .await
        .unwrap_or(false)
        .unwrap_or(false);

    if !authenticated {
        // Redirect to login page
        return Ok(Redirect::to("/login").into_response());
    }

    // Check session expiry
    if let Some(login_time) = session.get::<DateTime<Utc>>("login_time").await? {
        let session_age = Utc::now() - login_time;
        if session_age.num_seconds() > config.session_timeout {
            session.delete().await?;
            return Ok(Redirect::to("/login").into_response());
        }
    }

    Ok(next.run(req).await)
}
```

### 3. SQL Injection Prevention

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

### 4. Cross-Site Scripting (XSS) Prevention

**Template auto-escaping:**
```html
<!-- Askama templates auto-escape by default -->
<div>{{ user_input }}</div>  <!-- Automatically escaped -->

<!-- For raw HTML (use sparingly, only for trusted content) -->
<div>{{ user_input|safe }}</div>  <!-- NOT escaped - dangerous! -->
```

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

### 5. CSRF Protection

**Token-based CSRF protection:**
```rust
// src/middleware/csrf.rs
pub struct CsrfMiddleware {
    secret: String,
}

impl CsrfMiddleware {
    pub fn generate_token(&self, session_id: &str) -> String {
        // Generate HMAC-based token
        let mut mac = Hmac::<Sha256>::new_from_slice(self.secret.as_bytes()).unwrap();
        mac.update(session_id.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    pub fn verify_token(&self, session_id: &str, token: &str) -> bool {
        let expected = self.generate_token(session_id);
        constant_time_eq(expected.as_bytes(), token.as_bytes())
    }
}
```

**Template usage:**
```html
<form method="POST" action="/query/execute">
    <input type="hidden" name="csrf_token" value="{{ csrf_token }}">
    <!-- form fields -->
</form>
```

### 6. Rate Limiting

**Implement rate limiting to prevent abuse:**

```rust
// src/middleware/rate_limit.rs
use governor::{Quota, RateLimiter};

pub struct RateLimitLayer {
    limiter: RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>,
}

impl RateLimitLayer {
    pub fn new() -> Self {
        // 100 requests per minute per IP
        let quota = Quota::per_minute(nonzero!(100u32));
        Self {
            limiter: RateLimiter::keyed(quota),
        }
    }

    pub async fn check(&self, ip: &str) -> Result<()> {
        match self.limiter.check_key(&ip.to_string()) {
            Ok(_) => Ok(()),
            Err(_) => Err(RateLimitError::TooManyRequests),
        }
    }
}
```

**Different limits for different endpoints:**
- Query execution: 20/minute
- Table browsing: 100/minute
- Schema operations: 50/minute

### 7. Secure Session Management

**Session configuration:**
```rust
pub fn create_session_layer(config: &Config) -> SessionManagerLayer<MemoryStore> {
    let session_store = MemoryStore::default();

    SessionManagerLayer::new(session_store)
        .with_secure(true)  // HTTPS only
        .with_http_only(true)  // Not accessible via JavaScript
        .with_same_site(SameSite::Strict)
        .with_name("pgadmin_session")
        .with_max_age(Some(Duration::from_secs(config.session_timeout)))
}
```

**Session security:**
- [ ] Regenerate session ID after login
- [ ] Clear old sessions on logout
- [ ] Implement session timeout
- [ ] Use secure cookie flags (HttpOnly, Secure, SameSite)
- [ ] Store minimal data in sessions

### 8. Audit Logging

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
- Login attempts (success/failure)
- Query executions
- Schema modifications
- Table data changes
- Failed authorization attempts
- Rate limit violations

### 9. Secure Configuration

**Configuration security checklist:**
```rust
pub fn validate_security_config(config: &Config) -> Result<()> {
    // Check session secret is strong enough
    if config.session_secret.len() < 32 {
        return Err(ConfigError::WeakSessionSecret);
    }

    // Warn if running without password
    if config.app_password.is_none() {
        warn!("Running without authentication - NOT recommended for production");
    }

    // Check if TLS is configured (in production)
    if config.is_production() && !config.tls_enabled {
        return Err(ConfigError::TlsRequired);
    }

    Ok(())
}
```

### 10. Password Security

**If implementing user management in future:**
```rust
use argon2::{Argon2, PasswordHash, PasswordVerifier, PasswordHasher};

pub struct PasswordService;

impl PasswordService {
    pub fn hash_password(password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

        Ok(password_hash)
    }

    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
```

## File Structure
```
src/
├── auth/
│   ├── mod.rs
│   ├── session.rs
│   └── password.rs
├── middleware/
│   ├── auth.rs
│   ├── csrf.rs
│   ├── rate_limit.rs
│   └── security_headers.rs
├── audit/
│   ├── mod.rs
│   └── logger.rs
└── validation/
    ├── mod.rs
    └── input.rs
```

## Testing Requirements
- [ ] Authentication flow works correctly
- [ ] Unauthenticated users cannot access protected routes
- [ ] Session timeout enforced
- [ ] CSRF protection blocks invalid tokens
- [ ] Rate limiting triggers correctly
- [ ] SQL injection attempts blocked
- [ ] XSS attempts neutralized
- [ ] Security headers present in responses
- [ ] Audit log captures events correctly

## Security Audit Checklist
- [ ] No hardcoded secrets in code
- [ ] All user inputs validated
- [ ] Parameterized queries used exclusively
- [ ] Template auto-escaping enabled
- [ ] HTTPS enforced in production
- [ ] Security headers configured
- [ ] CSRF protection implemented
- [ ] Rate limiting active
- [ ] Session management secure
- [ ] Audit logging comprehensive
- [ ] Error messages don't leak sensitive info
- [ ] Dependencies scanned for vulnerabilities

## Compliance Considerations
- GDPR: Audit logs, data privacy
- SOC 2: Access controls, audit logging
- PCI DSS: Strong authentication, encryption

## Acceptance Criteria
- [ ] Authentication system functional
- [ ] All endpoints protected appropriately
- [ ] Security middleware active
- [ ] Input validation comprehensive
- [ ] Audit logging working
- [ ] No security vulnerabilities in static analysis
- [ ] Security tests pass
- [ ] Documentation complete
