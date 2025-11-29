# pgAdmin-rs

> A modern PostgreSQL administration tool built with Rust and minimal JavaScript

## Overview

pgAdmin-rs is a lightweight, fast, and secure web-based PostgreSQL administration tool inspired by phpMyAdmin. Built entirely in Rust with server-side rendering and HTMX for dynamic interactions, it provides a sleek user experience with minimal JavaScript footprint.

## Features

- **Database Browsing**: Hierarchical navigation through databases, schemas, tables, and views
- **SQL Query Editor**: Execute queries with syntax highlighting and results display
- **Table Data Management**: View, edit, insert, and delete table data with inline editing
- **Schema Operations**: Create, modify, and drop database objects
- **Data Export**: Export query results and table data to CSV, JSON, or SQL formats
- **Query History**: Track and replay executed queries
- **Statistics Dashboard**: View database size, table statistics, and performance metrics
- **Security First**: Built-in SQL injection prevention, CSRF protection, and authentication
- **Docker Ready**: Easy deployment in containerized environments

## Architecture

### Technology Stack

- **Backend**: Axum (Rust web framework)
- **Database**: SQLx (PostgreSQL client)
- **Templates**: Askama (type-safe templating)
- **Frontend**: HTMX + minimal JavaScript
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

- Rust 1.75+
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

1. Install dependencies:
   ```bash
   cargo build
   ```

2. Set up environment variables:
   ```bash
   export POSTGRES_HOST=localhost
   export POSTGRES_PORT=5432
   export POSTGRES_USER=postgres
   export POSTGRES_PASSWORD=yourpassword
   export POSTGRES_DB=postgres
   export SESSION_SECRET=your-secret-key-min-32-chars
   export APP_PASSWORD=admin
   ```

3. Run the application:
   ```bash
   cargo run
   ```

4. Visit `http://localhost:8080`

## Configuration

All configuration is done via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `SERVER_HOST` | Server bind address | `0.0.0.0` |
| `SERVER_PORT` | Server port | `8080` |
| `POSTGRES_HOST` | PostgreSQL host | `localhost` |
| `POSTGRES_PORT` | PostgreSQL port | `5432` |
| `POSTGRES_USER` | Database user | `postgres` |
| `POSTGRES_PASSWORD` | Database password | - |
| `POSTGRES_DB` | Database name | `postgres` |
| `SESSION_SECRET` | Secret for session cookies | - |
| `APP_PASSWORD` | Application password | - |
| `MAX_DB_CONNECTIONS` | Max connection pool size | `20` |
| `SESSION_TIMEOUT` | Session timeout (seconds) | `3600` |
| `LOG_LEVEL` | Logging level | `info` |

## Security

pgAdmin-rs is built with security as a top priority:

- **Authentication**: Simple password protection for the entire application
- **SQL Injection Prevention**: Parameterized queries and input validation
- **XSS Protection**: Template auto-escaping and CSP headers
- **CSRF Protection**: Token-based protection for state-changing operations
- **Rate Limiting**: Prevent abuse and DoS attacks
- **Audit Logging**: Track all security-relevant events
- **Secure Defaults**: Security headers, HttpOnly cookies, SameSite cookies

## Development Roadmap

This project follows a phased development approach. See the `issues/` directory for detailed planning documents:

### Phase 1: Foundation
- [x] [Issue #01: Architecture and Technology Stack](issues/01-architecture-and-tech-stack.md)
- [ ] [Issue #02: Backend Foundation](issues/02-backend-foundation.md)
- [ ] [Issue #03: Database Connectivity](issues/03-database-connectivity.md)

### Phase 2: Security
- [ ] [Issue #04: Security and Authentication](issues/04-security-and-authentication.md)

### Phase 3: Core Features
- [ ] [Issue #05: Core Features](issues/05-core-features.md)
- [ ] [Issue #06: UI/UX Implementation](issues/06-ui-ux-implementation.md)

### Phase 4: Deployment
- [ ] [Issue #07: Docker Setup](issues/07-docker-setup.md)
- [ ] [Issue #08: Testing and Quality Assurance](issues/08-testing-and-quality.md)

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
