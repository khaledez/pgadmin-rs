# Issue #01: Architecture and Technology Stack

## Overview
Define the technical architecture and select the technology stack for pgAdmin-rs, a PostgreSQL administration tool built in Rust with minimal JavaScript.

## Goals
- Choose optimal Rust web framework for the backend
- Select minimal JavaScript approach for sleek UX
- Define the application architecture
- Plan for Docker containerization

## Proposed Technology Stack

### Backend
- **Web Framework**: Axum (async, ergonomic, built on tokio)
  - Fast and type-safe routing
  - Excellent middleware support
  - Minimal boilerplate
  - Strong ecosystem compatibility

- **Database Client**: SQLx
  - Compile-time checked queries
  - Async/await support
  - Connection pooling
  - PostgreSQL-specific features
  - Migration support

- **Templating**: Askama or Tera
  - Type-safe templates (Askama)
  - Server-side rendering
  - Minimal runtime overhead

- **Serialization**: serde + serde_json
  - Industry standard
  - JSON API responses

### Frontend (Minimal JavaScript)
- **HTMX**: For dynamic interactions without heavy JavaScript
  - AJAX requests with HTML responses
  - Partial page updates
  - WebSocket support for real-time features
  - Progressive enhancement

- **Alpine.js** (optional): For lightweight client-side interactions
  - ~15KB minified
  - Reactive and declarative
  - Complements HTMX well

- **CSS Framework**: Tailwind CSS or custom CSS
  - Utility-first approach
  - Small bundle size when purged
  - Or plain CSS for maximum control

### Container & Deployment
- **Docker**: Multi-stage builds for minimal image size
- **Environment Variables**: For PostgreSQL credentials
  - `POSTGRES_HOST`
  - `POSTGRES_PORT`
  - `POSTGRES_USER`
  - `POSTGRES_PASSWORD`
  - `POSTGRES_DB`

## Architecture

```
┌─────────────────────────────────────────────┐
│              Docker Container               │
│                                             │
│  ┌───────────────────────────────────────┐ │
│  │         Axum Web Server              │ │
│  │                                       │ │
│  │  ┌─────────────┐  ┌────────────────┐ │ │
│  │  │   Routes    │  │   Middleware   │ │ │
│  │  │  /query     │  │  - Auth        │ │ │
│  │  │  /tables    │  │  - CORS        │ │ │
│  │  │  /schema    │  │  - Logging     │ │ │
│  │  └─────────────┘  └────────────────┘ │ │
│  │                                       │ │
│  │  ┌─────────────────────────────────┐ │ │
│  │  │    Business Logic Layer         │ │ │
│  │  │  - Query executor               │ │ │
│  │  │  - Schema inspector             │ │ │
│  │  │  - Table manager                │ │ │
│  │  └─────────────────────────────────┘ │ │
│  │                                       │ │
│  │  ┌─────────────────────────────────┐ │ │
│  │  │    Database Layer (SQLx)        │ │ │
│  │  │  - Connection pool              │ │ │
│  │  │  - Query builder                │ │ │
│  │  └─────────────────────────────────┘ │ │
│  └───────────────────────────────────────┘ │
│                   │                         │
└───────────────────┼─────────────────────────┘
                    │
                    ▼
          ┌──────────────────┐
          │   PostgreSQL     │
          │   Database       │
          └──────────────────┘
```

## Project Structure
```
pgadmin-rs/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── config/
│   │   └── mod.rs              # Configuration management
│   ├── routes/
│   │   ├── mod.rs
│   │   ├── query.rs            # Query execution routes
│   │   ├── tables.rs           # Table management routes
│   │   ├── schema.rs           # Schema routes
│   │   └── auth.rs             # Authentication routes
│   ├── handlers/
│   │   └── ...                 # Request handlers
│   ├── services/
│   │   ├── db_service.rs       # Database operations
│   │   ├── query_service.rs    # Query execution
│   │   └── schema_service.rs   # Schema inspection
│   ├── models/
│   │   └── ...                 # Data models
│   ├── templates/
│   │   └── ...                 # HTML templates
│   └── middleware/
│       └── ...                 # Custom middleware
├── static/
│   ├── css/
│   ├── js/
│   └── images/
├── migrations/
│   └── ...                     # Database migrations (if needed)
├── Dockerfile
├── docker-compose.yml
├── Cargo.toml
└── README.md
```

## Tasks
- [x] Initialize Rust project with Cargo
- [x] Add dependencies to Cargo.toml
- [x] Set up basic Axum server
- [x] Configure project structure
- [x] Set up development Docker environment
- [x] Create basic HTML template structure
- [x] Set up HTMX integration

## Dependencies (Cargo.toml preview)
```toml
[dependencies]
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
askama = "0.12"
tower = "0.4"
tower-http = { version = "0.5", features = ["fs", "trace"] }
tracing = "0.1"
tracing-subscriber = "0.3"
dotenv = "0.15"
```

## Security Considerations
- Input validation on all user inputs
- SQL injection prevention through parameterized queries
- Rate limiting on query execution
- Session management for authentication
- CORS configuration
- Secure headers (CSP, X-Frame-Options, etc.)

## Acceptance Criteria
- [x] Technology stack documented and approved
- [x] Architecture diagram finalized
- [x] Project structure created
- [x] All team members understand the technical decisions

## Implementation Notes

**Completed:** 2024-11-30

### Actual Implementation
- **Dependencies**: Used latest stable versions (Axum 0.8, SQLx 0.8, Askama 0.14)
- **Templating**: Chose Askama for type-safe templates with manual IntoResponse implementation
- **CSS**: Implemented custom CSS with CSS variables for maintainability
- **HTMX**: Integrated for dynamic health status updates (30-second polling)
- **Docker**: Multi-stage Dockerfile for optimal image size
- **Configuration**: Environment-based config with sensible defaults

### Project Files Created
- `src/main.rs`: Axum server with routing and middleware
- `src/config/mod.rs`: Configuration management
- `src/routes/mod.rs`: Route handlers with template rendering
- `templates/index.html`: Homepage with HTMX integration
- `static/css/style.css`: Responsive CSS styling
- `Dockerfile`: Multi-stage build for production
- `docker-compose.yml`: Development environment with PostgreSQL
- `.env.example`: Environment variable template
- `.gitignore`: Standard Rust project ignores

### Verified
- ✅ Project builds successfully with `cargo check`
- ✅ All dependencies at latest stable versions
- ✅ Docker configuration ready for deployment
- ✅ HTMX integration working for dynamic updates
- ✅ Clean project structure following best practices
