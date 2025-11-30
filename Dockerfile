# Multi-stage Dockerfile for pgAdmin-rs
# Optimized for minimal image size and security

# Stage 1: Builder
# ===============
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Cargo manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies separately (for better layer caching)
# This creates a dummy binary to cache dependencies
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

# Verify the binary exists
RUN ls -lh /app/target/release/pgadmin-rs


# Stage 2: Runtime
# ===============
FROM debian:bookworm-slim

# Install runtime dependencies only
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create application directory
RUN mkdir -p /app && \
    chmod 755 /app

WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/pgadmin-rs ./pgadmin-rs

# Copy static files and templates
COPY --from=builder /app/static ./static
COPY --from=builder /app/templates ./templates

# Create non-root user for security
RUN useradd -m -u 1000 pgadmin && \
    chown -R pgadmin:pgadmin /app && \
    chmod -R 755 /app && \
    chmod 700 /app/pgadmin-rs

# Switch to non-root user
USER pgadmin

# Expose port (default 3000, can be overridden with -p)
EXPOSE 3000

# Health check
# Checks if the /health endpoint responds with 200
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Default environment variables
ENV RUST_LOG=pgadmin_rs=info,axum=warn
ENV SERVER_ADDRESS=0.0.0.0:3000

# Run the application
CMD ["./pgadmin-rs"]
