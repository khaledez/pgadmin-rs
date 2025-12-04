# Docker Setup and Deployment Guide

pgAdmin-rs is designed to run in Docker containers for easy deployment and consistent environments.

## Base Image

This project uses the [LinuxServer.io Alpine base image](https://docs.linuxserver.io/) (`ghcr.io/linuxserver/baseimage-alpine:3.21`), which provides:

- **Minimal footprint**: Alpine Linux keeps the image size small (~20-30MB total)
- **PUID/PGID support**: Proper file permission mapping between container and host
- **Security-focused**: Regular updates and security patches from LinuxServer.io team
- **Battle-tested**: Used by thousands of production deployments

## Quick Start

### Prerequisites
- Docker (version 20.10+)
- Docker Compose (version 1.29+)
- 2GB free disk space (for base image + database)

### Development Environment

1. **Clone and setup:**
```bash
cd pgadmin-rs
cp .env.example .env
```

2. **Start the application:**
```bash
# Using Docker Compose (recommended)
docker-compose up

# Or using Make
make dev
```

3. **Access the application:**
- Open http://localhost:3000 in your browser
- PostgreSQL database: localhost:5432

### Production Deployment

1. **Build the image:**
```bash
make prod-build
# Or manually:
docker build -t pgadmin-rs:latest .
```

2. **Configure environment:**
```bash
# Set required variables before deployment
export POSTGRES_HOST=your-db-host
export POSTGRES_USER=your-db-user
export POSTGRES_PASSWORD=your-db-password
export POSTGRES_DB=your-db-name
```

3. **Deploy with proper user mapping:**
```bash
# Using Docker CLI with PUID/PGID
docker run -d \
  --name pgadmin-rs \
  -p 3000:3000 \
  -e PUID=$(id -u) \
  -e PGID=$(id -g) \
  -e POSTGRES_HOST=your-db-host \
  -e POSTGRES_USER=your-user \
  -e POSTGRES_PASSWORD=your-password \
  -e POSTGRES_DB=your-db \
  pgadmin-rs:latest

# Or using Docker Compose
docker-compose -f docker-compose.prod.yml up -d
```

## Docker Architecture

### Multi-Stage Build

The Dockerfile uses a multi-stage build process for minimal image size:

1. **Builder Stage**: Compiles Rust code (uses rust:1.91-alpine)
2. **Runtime Stage**: Runs application (uses ghcr.io/linuxserver/baseimage-alpine:3.21)

**Result**: Significantly smaller final image (no build artifacts in runtime)

### Image Size

- Expected final size: ~20-30MB (Alpine-based)
- Builder stage: ~500MB (discarded after build)
- Runtime: Alpine base is very lightweight

## Configuration

### Environment Variables

All configuration is done via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `PUID` | `1000` | User ID for file permissions (LinuxServer.io) |
| `PGID` | `1000` | Group ID for file permissions (LinuxServer.io) |
| `SERVER_ADDRESS` | `0.0.0.0:3000` | Server host and port |
| `POSTGRES_HOST` | - | PostgreSQL host (required) |
| `POSTGRES_PORT` | `5432` | PostgreSQL port |
| `POSTGRES_USER` | - | PostgreSQL username (required) |
| `POSTGRES_PASSWORD` | - | PostgreSQL password (required) |
| `POSTGRES_DB` | - | PostgreSQL database (required) |
| `RUST_LOG` | `info` | Logging level |

### PUID and PGID Configuration

The container uses LinuxServer.io base image which supports PUID (Process User ID) and PGID (Process Group ID) for proper file permission mapping between the container and host.

**Why is this important?**
- Prevents permission issues with mounted volumes
- Files created by the container are owned by your user, not root
- Required for secure, non-root operation

**Finding your IDs:**
```bash
id $(whoami)
# Output example: uid=1000(username) gid=1000(username) groups=1000(username)
```

**Using PUID/PGID:**
```bash
# Docker CLI
docker run -e PUID=1000 -e PGID=1000 pgadmin-rs:latest

# Docker Compose
environment:
  - PUID=1000
  - PGID=1000
```

The container will automatically create/map a user with these IDs and run the application as that user.

### Development vs Production

**Development (.env):**
```bash
SERVER_ADDRESS=0.0.0.0:3000
POSTGRES_HOST=postgres
POSTGRES_USER=postgres
POSTGRES_PASSWORD=postgres
POSTGRES_DB=testdb
RUST_LOG=pgadmin_rs=debug,axum=info
```

**Production (.env.prod):**
```bash
SERVER_ADDRESS=0.0.0.0:3000
POSTGRES_HOST=prod-db.example.com
POSTGRES_USER=pgadmin_prod
POSTGRES_PASSWORD=<strong-random-password>
POSTGRES_DB=pgadmin_prod
RUST_LOG=pgadmin_rs=info,axum=warn
```

## Docker Compose Files

### docker-compose.yml (Development)

Includes:
- PostgreSQL service with persistent volume
- Application service
- Network configuration
- Health checks
- Optional volume mounts for code

**Usage:**
```bash
docker-compose up              # Start services
docker-compose up -d           # Start in background
docker-compose down            # Stop services
docker-compose down -v         # Stop and remove volumes
```

### docker-compose.prod.yml (Production)

Features:
- Application only (assumes external PostgreSQL)
- Security hardening:
  - Read-only filesystem
  - No new privileges
  - Dropped capabilities
  - Non-root user
- Resource limits (commented, uncomment as needed)
- Proper logging configuration
- Health checks

**Usage:**
```bash
docker-compose -f docker-compose.prod.yml up -d
```

## Security Best Practices

### Image Security

- ✅ Uses LinuxServer.io Alpine base image (security-focused, minimal)
- ✅ Multi-stage build (no build tools in runtime)
- ✅ Non-root user via PUID/PGID mapping
- ✅ Minimal attack surface (Alpine Linux)
- ✅ Regular dependency updates recommended
- ✅ Latest Rust 1.91 with security patches

### Container Security (Production)

- ✅ `read_only: true` - Read-only filesystem
- ✅ `cap_drop: ALL` - Drop all capabilities
- ✅ `security_opt: no-new-privileges:true` - Prevent privilege escalation
- ✅ `tmpfs: /tmp` - Temporary mount for needed writes

### Network Security

- ✅ HTTPS enforced via security headers
- ✅ Database connections use environment variables (not hardcoded)
- ✅ Health checks verify connectivity
- ✅ CORS properly configured

## Health Checks

### Endpoint

- **Path**: `/health`
- **Method**: GET
- **Response**: 
  ```json
  {
    "status": "ok",
    "database": "healthy",
    "version": "0.1.0"
  }
  ```

### Docker Health Check

```bash
# Manual health check
curl http://localhost:3000/health

# View container health status
docker ps

# View health check details
docker inspect pgadmin-rs-app
```

## Common Tasks

### View Logs

```bash
# Development (follow logs)
make logs
# Or manually:
docker-compose logs -f app

# Production
docker logs pgadmin-rs-prod -f
```

### Execute Commands

```bash
# Open shell in running container
make shell
# Or manually:
docker exec -it pgadmin-rs-app /bin/bash

# Run specific command
docker exec pgadmin-rs-app curl http://localhost:3000/health
```

### Check Image Size

```bash
make size
# Or manually:
docker images pgadmin-rs --human-readable
```

### View Image Layers

```bash
make layers
# Or manually:
docker history pgadmin-rs:latest
```

## Troubleshooting

### Application won't start

1. **Check logs**: `docker-compose logs app`
2. **Verify environment**: `docker exec app env | grep POSTGRES`
3. **Test database connection**: Ensure PostgreSQL is accessible
4. **Check port**: Port 3000 must be available

### Database connection fails

1. **Verify hostname**: Use service name in compose (e.g., `postgres`)
2. **Check credentials**: User/password must match
3. **Wait for database**: Service has healthcheck, should be ready
4. **View database logs**: `docker-compose logs postgres`

### High memory usage

1. **Check process**: `docker stats pgadmin-rs-app`
2. **View logs for errors**: `docker-compose logs app`
3. **Limit resources**: Uncomment resource limits in prod compose
4. **Reduce logging**: Set `RUST_LOG=warn` or `error`

### Image too large

With the Alpine-based image, the size is optimized. Typical sizes:
- Final image: 20-30MB (Alpine-based)
- PostgreSQL: 100-150MB (use postgres:16-alpine for smaller size)
- Combined: ~150MB total

If you need to reduce size further:
- Implement image pruning: `docker system prune`
- Use multi-arch builds for specific platforms
- Enable Docker BuildKit for better layer caching

## Performance Optimization

### Database Connection Pooling

Configured in `src/config/mod.rs`:
- Max connections: 20
- Min connections: 2
- Idle timeout: 300 seconds
- Acquire timeout: 5 seconds

### Static File Serving

- Static files served directly from binary
- CSS and JS included in build
- No additional web server needed

### Caching

- HTTP headers configured for caching
- Browser caches static assets
- Database connection pooling active

## Monitoring

### Container Metrics

```bash
# Real-time metrics
docker stats pgadmin-rs-app

# Historical metrics (Docker events)
docker events --filter type=container
```

### Application Logs

```bash
# View all logs
docker logs pgadmin-rs-app

# Follow logs with tail
docker logs -f pgadmin-rs-app

# Last 100 lines
docker logs --tail 100 pgadmin-rs-app
```

## Updating the Application

### Development

1. Modify code
2. Docker rebuild on next `docker-compose up`
3. Or manually: `docker-compose up --build`

### Production

1. Build new image: `docker build -t pgadmin-rs:latest .`
2. Tag for registry: `docker tag pgadmin-rs:latest registry/pgadmin-rs:v0.1.0`
3. Push to registry: `docker push registry/pgadmin-rs:v0.1.0`
4. Update compose file: Change image tag
5. Redeploy: `docker-compose -f docker-compose.prod.yml up -d`

## Integration with Container Orchestration

### Kubernetes

The Dockerfile is compatible with Kubernetes. Example manifests can be created for:
- Deployment
- Service
- ConfigMap (for environment variables)
- Secret (for database credentials)
- Ingress (for HTTPS/routing)

### Docker Swarm

The service can be deployed to Docker Swarm:
```bash
docker service create \
  --name pgadmin-rs \
  --publish 3000:3000 \
  --env POSTGRES_HOST=postgres-service \
  pgadmin-rs:latest
```

## Resources

- [Dockerfile Reference](https://docs.docker.com/engine/reference/builder/)
- [Docker Compose Reference](https://docs.docker.com/compose/compose-file/)
- [Docker Best Practices](https://docs.docker.com/develop/dev-best-practices/)
- [Rust Docker Guide](https://docs.docker.com/language/rust/)
- [LinuxServer.io Documentation](https://docs.linuxserver.io/)
- [Understanding PUID and PGID](https://docs.linuxserver.io/general/understanding-puid-and-pgid/)
