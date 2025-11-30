# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml ./

# Copy source code
COPY src ./src
COPY static ./static

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/pgadmin-rs .

# Copy static files
COPY --from=builder /app/static ./static

# Create a non-root user
RUN useradd -m -u 1000 appuser && \
    chown -R appuser:appuser /app

USER appuser

# Expose the default port
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info
ENV SERVER_ADDRESS=0.0.0.0:3000

# Run the application
CMD ["./pgadmin-rs"]
