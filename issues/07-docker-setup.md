# Issue #07: Docker Setup and Containerization

## Overview
Create Docker configuration for building and running pgAdmin-rs in a container, with support for both development and production environments.

## Goals
- Create optimized Docker image
- Support environment-based configuration
- Enable easy deployment
- Minimize image size
- Ensure security best practices

## Tasks

### 1. Multi-Stage Dockerfile

**Create efficient multi-stage build:**

```dockerfile
# Dockerfile

# Stage 1: Build stage
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src
COPY templates ./templates
COPY static ./static

# Build the application
RUN cargo build --release

# Stage 2: Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 pgadmin && \
    mkdir -p /app && \
    chown -R pgadmin:pgadmin /app

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/pgadmin-rs .

# Copy static assets
COPY --from=builder /app/templates ./templates
COPY --from=builder /app/static ./static

# Change ownership
RUN chown -R pgadmin:pgadmin /app

# Switch to non-root user
USER pgadmin

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["./pgadmin-rs"]
```

### 2. Docker Compose Configuration

**Development and production compose files:**

```yaml
# docker-compose.yml (Development)
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    container_name: pgadmin-dev-db
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: testdb
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql
    networks:
      - pgadmin-network
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5

  pgadmin-rs:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: pgadmin-rs-app
    ports:
      - "8080:8080"
    environment:
      # Server config
      SERVER_HOST: 0.0.0.0
      SERVER_PORT: 8080

      # PostgreSQL connection
      POSTGRES_HOST: postgres
      POSTGRES_PORT: 5432
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: testdb

      # Security
      SESSION_SECRET: ${SESSION_SECRET:-development-secret-change-in-production}
      APP_PASSWORD: ${APP_PASSWORD:-admin}

      # Logging
      LOG_LEVEL: info
      RUST_LOG: pgadmin_rs=debug,axum=info,sqlx=warn

    depends_on:
      postgres:
        condition: service_healthy
    networks:
      - pgadmin-network
    volumes:
      # Mount source for development hot-reload (optional)
      - ./src:/app/src:ro
      - ./templates:/app/templates:ro
      - ./static:/app/static:ro
    restart: unless-stopped

volumes:
  postgres_data:
    driver: local

networks:
  pgadmin-network:
    driver: bridge
```

**Production compose file:**

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  pgadmin-rs:
    image: pgadmin-rs:latest
    container_name: pgadmin-rs-prod
    ports:
      - "8080:8080"
    environment:
      # Server config
      SERVER_HOST: 0.0.0.0
      SERVER_PORT: 8080

      # PostgreSQL connection (from external database)
      POSTGRES_HOST: ${POSTGRES_HOST}
      POSTGRES_PORT: ${POSTGRES_PORT:-5432}
      POSTGRES_USER: ${POSTGRES_USER}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
      POSTGRES_DB: ${POSTGRES_DB}

      # Security (MUST be set via environment)
      SESSION_SECRET: ${SESSION_SECRET}
      APP_PASSWORD: ${APP_PASSWORD}

      # Connection pool
      MAX_DB_CONNECTIONS: ${MAX_DB_CONNECTIONS:-20}
      MIN_DB_CONNECTIONS: ${MIN_DB_CONNECTIONS:-2}

      # Logging
      LOG_LEVEL: ${LOG_LEVEL:-warn}
      RUST_LOG: ${RUST_LOG:-pgadmin_rs=info,axum=warn}

    restart: always
    read_only: true
    tmpfs:
      - /tmp
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    networks:
      - pgadmin-network

networks:
  pgadmin-network:
    driver: bridge
```

### 3. Environment Configuration

**Create .env.example file:**

```bash
# .env.example

# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# PostgreSQL Connection
POSTGRES_HOST=postgres
POSTGRES_PORT=5432
POSTGRES_USER=postgres
POSTGRES_PASSWORD=changeme
POSTGRES_DB=postgres

# Security
# IMPORTANT: Generate a strong random secret for production!
# Example: openssl rand -hex 32
SESSION_SECRET=your-secret-key-min-32-chars
APP_PASSWORD=your-admin-password

# Connection Pool Settings
MAX_DB_CONNECTIONS=20
MIN_DB_CONNECTIONS=2
DB_ACQUIRE_TIMEOUT=5
DB_IDLE_TIMEOUT=300
DB_MAX_LIFETIME=1800

# Session Configuration
SESSION_TIMEOUT=3600

# Logging
LOG_LEVEL=info
RUST_LOG=pgadmin_rs=debug,axum=info,sqlx=warn

# Performance
WORKERS=4
```

### 4. Dockerignore

**Optimize build context:**

```
# .dockerignore

# Git
.git
.gitignore

# Rust
target/
**/*.rs.bk
Cargo.lock

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# Docker
Dockerfile*
docker-compose*.yml
.dockerignore

# Docs
*.md
docs/

# CI/CD
.github/
.gitlab-ci.yml

# Testing
tests/
coverage/

# Environment
.env
.env.*
!.env.example

# Misc
issues/
tmp/
*.log
```

### 5. Health Check Endpoint

**Implement comprehensive health check:**

```rust
// src/routes/health.rs
use axum::{extract::State, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    database: String,
    version: String,
}

pub async fn health_check(
    State(state): State<AppState>,
) -> Json<HealthResponse> {
    let db_status = match state.db_service.test_connection().await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    Json(HealthResponse {
        status: if db_status == "healthy" { "ok".to_string() } else { "degraded".to_string() },
        database: db_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub async fn readiness_check(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, StatusCode> {
    match state.db_service.test_connection().await {
        Ok(_) => Ok(Json(HealthResponse {
            status: "ready".to_string(),
            database: "connected".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        })),
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

pub async fn liveness_check() -> StatusCode {
    StatusCode::OK
}
```

### 6. Build and Run Scripts

**Makefile for common operations:**

```makefile
# Makefile

.PHONY: help build run dev stop clean logs test

help:
	@echo "Available commands:"
	@echo "  make build       - Build Docker image"
	@echo "  make run         - Run in production mode"
	@echo "  make dev         - Run in development mode"
	@echo "  make stop        - Stop all containers"
	@echo "  make clean       - Remove containers and volumes"
	@echo "  make logs        - View logs"
	@echo "  make test        - Run tests"

build:
	docker build -t pgadmin-rs:latest .

run:
	docker-compose -f docker-compose.prod.yml up -d

dev:
	docker-compose up -d

stop:
	docker-compose down

clean:
	docker-compose down -v
	docker rmi pgadmin-rs:latest || true

logs:
	docker-compose logs -f pgadmin-rs

test:
	cargo test

shell:
	docker exec -it pgadmin-rs-app /bin/bash
```

### 7. Security Hardening

**Docker security best practices:**

```dockerfile
# Security-hardened Dockerfile additions

# Use specific version tags, not 'latest'
FROM rust:1.75-slim as builder

# Scan for vulnerabilities during build
RUN cargo install cargo-audit && \
    cargo audit

# In runtime stage:
FROM debian:bookworm-slim

# Update packages and remove cache
RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl && \
    rm -rf /var/lib/apt/lists/*

# Set secure file permissions
RUN chmod -R 755 /app && \
    chmod 500 /app/pgadmin-rs

# Drop capabilities
# (Added in docker-compose.yml)
```

### 8. Logging Configuration

**Structured logging for containers:**

```rust
// src/logging.rs
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init_logging() {
    let format = if cfg!(debug_assertions) {
        // Pretty format for development
        fmt::format()
            .pretty()
            .with_source_location(true)
            .with_thread_ids(true)
    } else {
        // JSON format for production (better for log aggregation)
        fmt::format()
            .json()
            .with_current_span(true)
            .with_span_list(true)
    };

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer().event_format(format))
        .init();
}
```

### 9. Volume Management

**Persistent data considerations:**

```yaml
# Optional: If implementing persistent query history or sessions
volumes:
  pgadmin_data:
    driver: local

services:
  pgadmin-rs:
    volumes:
      - pgadmin_data:/app/data
```

### 10. CI/CD Integration

**GitHub Actions example:**

```yaml
# .github/workflows/docker.yml
name: Docker Build and Push

on:
  push:
    branches: [ main ]
    tags: [ 'v*' ]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v2

      - name: Log in to Docker Hub
        uses: docker/login-action@v2
        with:
          username: ${{ secrets.DOCKER_USERNAME }}
          password: ${{ secrets.DOCKER_PASSWORD }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v4
        with:
          images: yourusername/pgadmin-rs

      - name: Build and push
        uses: docker/build-push-action@v4
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

## File Structure
```
.
├── Dockerfile
├── .dockerignore
├── docker-compose.yml
├── docker-compose.prod.yml
├── .env.example
├── Makefile
└── scripts/
    ├── docker-entrypoint.sh
    └── init.sql
```

## Testing Requirements
- [ ] Docker image builds successfully
- [ ] Container starts and serves traffic
- [ ] Environment variables passed correctly
- [ ] Health checks pass
- [ ] Database connection works from container
- [ ] Static files served correctly
- [ ] Logs output properly
- [ ] Container restarts automatically on failure

## Security Checklist
- [ ] Non-root user in container
- [ ] Minimal base image used
- [ ] No secrets in Dockerfile
- [ ] Read-only filesystem where possible
- [ ] Capabilities dropped
- [ ] Security scanning in CI/CD
- [ ] Specific version tags used
- [ ] Vulnerability scanning enabled

## Performance Considerations
- Layer caching optimized
- Multi-stage build minimizes image size
- Dependencies cached separately
- Static assets compiled into image

## Documentation Required
- [ ] Docker setup instructions
- [ ] Environment variable documentation
- [ ] Deployment guide
- [ ] Troubleshooting guide
- [ ] Security best practices

## Acceptance Criteria
- [ ] Docker image builds under 100MB
- [ ] Container starts in under 5 seconds
- [ ] All environment variables documented
- [ ] Health checks functional
- [ ] Production-ready configuration provided
- [ ] Security best practices implemented
- [ ] Documentation complete
- [ ] CI/CD pipeline configured
