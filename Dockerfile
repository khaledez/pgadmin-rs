# Multi-stage Dockerfile for pgAdmin-rs
# Optimized for minimal image size and security

# Stage 1: Builder
# ===============
FROM rust:1.91-alpine AS builder

# Install build dependencies (including Node.js for frontend assets)
RUN apk add --no-cache \
    pkgconfig \
    openssl-dev \
    musl-dev \
    nodejs \
    npm

WORKDIR /app

# Copy Cargo manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies separately (for better layer caching)
# This creates a dummy binary to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    SKIP_NPM_BUILD=1 cargo build --release && \
    rm -rf src

# Copy npm package files for frontend build
COPY package.json package-lock.json build.js ./
COPY static ./static

# Install npm dependencies and build frontend assets
RUN npm ci && \
    npm run build

# Copy actual source code
COPY src ./src
COPY templates ./templates

# Build the application (skip npm build since we already built assets)
RUN SKIP_NPM_BUILD=1 cargo build --release

# Stage 2: Runtime
# ===============
FROM ghcr.io/linuxserver/baseimage-alpine:3.23

# Install runtime dependencies only
RUN apk add --no-cache \
    ca-certificates \
    libssl3 \
    curl

# Create application directory
RUN mkdir -p /app && \
    chmod 755 /app

WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/pgadmin-rs ./pgadmin-rs

# Copy static files and templates
COPY --from=builder /app/static ./static
COPY --from=builder /app/templates ./templates

# Set proper permissions for PUID/PGID compatibility
# The base image will handle user mapping via PUID/PGID environment variables
RUN chmod -R 755 /app && \
    chmod +x /app/pgadmin-rs

# Expose port (default 3000, can be overridden with -p)
EXPOSE 3000

# Health check
# Checks if the /health endpoint responds with 200
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Default environment variables
ENV PUID=1000 \
    PGID=1000 \
    RUST_LOG=pgadmin_rs=info,axum=warn \
    SERVER_ADDRESS=0.0.0.0:3000

# Run the application
CMD ["./pgadmin-rs"]
