# pgAdmin-rs

> A modern PostgreSQL administration tool built with Rust and minimal JavaScript

## Overview

pgAdmin-rs is a lightweight, fast, and secure web-based PostgreSQL administration tool inspired by phpMyAdmin and Drizzle Studio. Built entirely in Rust with server-side rendering and HTMX for dynamic interactions, it provides a sleek user experience with minimal JavaScript footprint.

## Architecture

### Technology Stack

- **Backend**: Axum (Rust web framework)
- **Database**: SQLx (PostgreSQL client)
- **Templates**: Askama (type-safe templating)
- **Frontend**: HTMX + minimal JavaScript
- **Build System**: Cargo + npm (automatic frontend builds via `build.rs`)
- **Bundler**: esbuild (fast JavaScript bundler)
- **Containerization**: Docker

### Project Structure

```
pgadmin-rs/
├── src/
│   ├── main.rs              # Application entry point
│   ├── config/              # Configuration management
│   ├── routes/              # HTTP route handlers
│   ├── services/            # Business logic
│   ├── models/              # Data models
│   ├── middleware/          # Custom middleware
│   └── templates/           # HTML templates
├── static/                  # CSS, JS, images
├── tests/                   # Integration tests
├── issues/                  # Project planning documents
├── Dockerfile
├── docker-compose.yml
└── Cargo.toml
```

## Quick Start

### Prerequisites

- Rust 1.91+ (latest stable)
- Docker and Docker Compose (for containerized setup)
- PostgreSQL 12+ (for local development)

### Running with Docker Compose

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/pgadmin-rs.git
   cd pgadmin-rs
   ```

2. Copy environment configuration:
   ```bash
   cp .env.example .env
   # Edit .env with your PostgreSQL credentials
   ```

3. Start the application:
   ```bash
   docker-compose up -d
   ```

4. Access the application at `http://localhost:8080`

### Local Development

1. Install dependencies (Node.js 18+ required for frontend builds):
   ```bash
   # npm install and npm build happen automatically via build.rs
   cargo build
   ```

2. Set up environment variables:
   ```bash
   export POSTGRES_HOST=localhost
   export POSTGRES_PORT=5432
   export POSTGRES_USER=postgres
   export POSTGRES_PASSWORD=yourpassword
   export POSTGRES_DB=postgres
   ```

3. Run the application:
   ```bash
   # Automatically rebuilds frontend assets if needed
   cargo run
   ```

4. Visit `http://localhost:3000`

**Note**: Frontend assets (JavaScript) are built automatically during `cargo build` via a build script. See [BUILD_GUIDE.md](BUILD_GUIDE.md) for details.

## Configuration

All configuration is done via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `SERVER_ADDRESS` | Server bind address and port | `0.0.0.0:3000` |
| `POSTGRES_HOST` | PostgreSQL host | `localhost` |
| `POSTGRES_PORT` | PostgreSQL port | `5432` |
| `POSTGRES_USER` | Database user | `postgres` |
| `POSTGRES_PASSWORD` | Database password | - |
| `POSTGRES_DB` | Database name | `postgres` |
| `RATE_LIMIT_REQUESTS_PER_MINUTE` | Max requests per IP per minute | `100` |
| `RUST_LOG` | Logging level | `info` |

## Security

pgAdmin-rs is built with security as a top priority:

- **Rate Limiting**: Per-IP request throttling using token bucket algorithm (configurable, default: 100 req/min)
- **SQL Injection Prevention**: Parameterized queries and input validation
- **XSS Protection**: Template auto-escaping and CSP headers
- **Security Headers**: X-Frame-Options, X-Content-Type-Options, X-XSS-Protection
- **Audit Logging**: Track all security-relevant events
- **Query Validation**: Dangerous operations (DROP, DELETE, etc.) require explicit confirmation
- **Secure Defaults**: HttpOnly cookies, SameSite cookies, secure headers

## Development Progress

This project follows a phased development approach. See the `issues/` directory for detailed planning documents:

### Phase 1: Foundation ✅
- [x] [Issue #01: Architecture and Technology Stack](issues/01-architecture-and-tech-stack.md)
- [x] [Issue #02: Backend Foundation](issues/02-backend-foundation.md)
- [x] [Issue #03: Database Connectivity](issues/03-database-connectivity.md)

### Phase 2: Core Features ✅
- [x] [Issue #04: Security and Authentication (Partial)](issues/04-security-and-authentication.md)
- [x] [Issue #05: Core Features](issues/05-core-features.md)
- [x] [Issue #06: UI/UX Implementation (Mostly Complete)](issues/06-ui-ux-implementation.md)

### Phase 3: Advanced Features & Deployment
- [ ] [Issue #07: Docker Setup](issues/07-docker-setup.md)
- [ ] [Issue #08: Testing and Quality Assurance](issues/08-testing-and-quality.md)
- [ ] Additional features: Data editing, exports, query history

## Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Write tests for your changes
4. Ensure all tests pass (`cargo test`)
5. Format code (`cargo fmt`)
6. Run clippy (`cargo clippy`)
7. Commit your changes (`git commit -m 'Add amazing feature'`)
8. Push to the branch (`git push origin feature/amazing-feature`)
9. Open a Pull Request

## Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --all-features

# Run benchmarks
cargo bench

# Run clippy
cargo clippy --all-targets --all-features

# Format code
cargo fmt
```

## Performance

pgAdmin-rs is designed for performance:

- **Fast startup**: Async/await throughout
- **Connection pooling**: Efficient database connection management
- **Pagination**: Handle large datasets efficiently
- **Minimal JavaScript**: Faster page loads and better performance
- **Compiled binary**: No runtime overhead

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Inspired by phpMyAdmin and pgAdmin
- Built with amazing Rust ecosystem tools
- HTMX for modern UX with minimal JavaScript

## Support

- **Issues**: Report bugs via GitHub Issues
- **Documentation**: See `issues/` directory for detailed documentation
- **Discussions**: Use GitHub Discussions for questions

## Roadmap Future Features

- [ ] Multiple database connections
- [ ] User management
- [ ] Backup/restore functionality
- [ ] Query builder UI
- [ ] Real-time monitoring
- [ ] Dark mode
- [ ] Multi-language support
- [ ] Stored procedure/function editor
- [ ] Visual query explain plans
- [ ] Schema comparison tools

---

Built with ❤️ using Rust
