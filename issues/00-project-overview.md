# Project Overview: pgAdmin-rs

## Vision

Build a modern, secure, and performant PostgreSQL administration tool using Rust with minimal JavaScript on the frontend, designed to run in Docker containers.

## Core Principles

1. **Security First**: Built-in protection against SQL injection, XSS, CSRF, and other vulnerabilities
2. **Simplicity**: Clean, maintainable code following the philosophy of minimal cognitive load
3. **Performance**: Fast, async operations with efficient resource usage
4. **Minimal JavaScript**: Leverage HTMX for dynamic UX with server-side rendering
5. **Container Native**: Designed for Docker deployment from day one

## Technology Choices

### Backend
- **Rust**: Type safety, memory safety, and performance
- **Axum**: Modern, ergonomic web framework built on tokio
- **SQLx**: Compile-time checked SQL queries
- **Askama**: Type-safe templating engine

### Frontend
- **HTMX**: Dynamic interactions without heavy JavaScript
- **Vanilla CSS**: Custom styling with CSS variables
- **Minimal JS**: Only where absolutely necessary (<50KB total)

### Infrastructure
- **Docker**: Containerized deployment
- **PostgreSQL**: Target database (12+)

## Project Phases

### Phase 1: Foundation (Issues #01-#03)
Establish the core architecture, backend infrastructure, and database connectivity.

**Key Deliverables:**
- Working Axum web server
- Configuration management
- Database connection pooling
- Basic routing and middleware
- Health checks

**Estimated Time**: 2-3 weeks

### Phase 2: Security (Issue #04)
Implement comprehensive security measures before building features.

**Key Deliverables:**
- Authentication system
- Session management
- CSRF protection
- Rate limiting
- Input validation
- Audit logging

**Estimated Time**: 1-2 weeks

### Phase 3: Core Features (Issues #05-#06)
Build the main application features and user interface.

**Key Deliverables:**
- Database browser
- SQL query editor
- Table data viewer/editor
- Data export
- Query history
- Statistics dashboard
- Responsive UI with HTMX

**Estimated Time**: 4-6 weeks

### Phase 4: Deployment & Quality (Issues #07-#08)
Prepare for production deployment with Docker and comprehensive testing.

**Key Deliverables:**
- Optimized Docker images
- Docker Compose configurations
- Comprehensive test suite
- CI/CD pipeline
- Documentation

**Estimated Time**: 2-3 weeks

## Success Metrics

### Performance
- Docker image size: <100MB
- Server startup time: <5 seconds
- Query response time: <100ms for simple queries
- Page load time: <2 seconds

### Code Quality
- Test coverage: >75%
- Security vulnerabilities: 0 critical
- Clippy warnings: 0
- Documentation coverage: 100% for public APIs

### Security
- All OWASP Top 10 vulnerabilities addressed
- Parameterized queries: 100%
- Security headers: Configured
- Audit logging: Comprehensive

### User Experience
- Minimal JavaScript footprint: <50KB
- Responsive design: Mobile-friendly
- Accessibility: WCAG AA compliant
- Browser support: Latest 2 versions of major browsers

## Development Workflow

1. **Plan** (Issues): Read and understand the issue
2. **Design**: Plan implementation approach
3. **Implement**: Write code following best practices
4. **Test**: Write and run tests
5. **Review**: Code review and quality checks
6. **Document**: Update documentation
7. **Iterate**: Address feedback and improve

## Code Philosophy (Per CLAUDE.md)

### Readability Over Cleverness
- Use intermediate variables with meaningful names
- Prefer early returns over nested ifs
- Keep functions/modules deep (simple interface, complex functionality)
- Avoid shallow abstractions

### Examples

**Bad (Hard for human brains):**
```rust
if val > limit && (cond1 || cond2) && (cond3 && !cond4) {
    // Complex logic
}
```

**Good (Easy to understand):**
```rust
let is_valid = val > limit;
let is_allowed = cond1 || cond2;
let is_secure = cond3 && !cond4;

if is_valid && is_allowed && is_secure {
    // Complex logic
}
```

### Comments
- **WHY** comments: Explain motivation and reasoning
- **WHAT** comments: Only for high-level overviews
- Avoid comments that duplicate code

### Simplicity
- Use minimal subset of language features
- Prefer composition over deep inheritance
- A little duplication > unnecessary dependencies
- Avoid unnecessary abstraction layers

## Architecture Decisions

### Why Axum?
- Type-safe routing
- Excellent async support
- Minimal boilerplate
- Strong ecosystem

### Why HTMX?
- Eliminates need for heavy frontend framework
- Server-side rendering for better security
- Progressive enhancement
- Minimal JavaScript footprint

### Why SQLx?
- Compile-time query checking
- Prevents SQL errors at runtime
- Excellent PostgreSQL support
- Connection pooling built-in

### Why Docker?
- Consistent deployment
- Easy configuration via environment variables
- Isolation and security
- Simple scaling

## Risk Management

### Technical Risks
- **Risk**: SQLx compile-time checks slow down builds
  - **Mitigation**: Use offline mode for CI/CD

- **Risk**: HTMX limitations for complex interactions
  - **Mitigation**: Minimal Alpine.js where needed

- **Risk**: Docker image size
  - **Mitigation**: Multi-stage builds, minimal base image

### Security Risks
- **Risk**: SQL injection via dynamic queries
  - **Mitigation**: Strict input validation, parameterized queries only

- **Risk**: Session hijacking
  - **Mitigation**: HttpOnly cookies, secure flags, CSRF protection

- **Risk**: DoS attacks
  - **Mitigation**: Rate limiting, query timeouts, connection limits

## Future Expansion

### Version 2.0 Features
- Multiple database connections
- User management with roles
- Query builder UI
- Visual explain plans
- Schema comparison

### Version 3.0 Features
- Real-time monitoring
- Backup/restore automation
- Stored procedure editor
- Multi-language support
- Dark mode

## Getting Started

1. Read all issues in order (#01-#08)
2. Set up development environment
3. Start with Issue #01 (Architecture)
4. Follow each issue sequentially
5. Test thoroughly at each phase
6. Document as you build

## Resources

- [Axum Documentation](https://docs.rs/axum)
- [SQLx Documentation](https://docs.rs/sqlx)
- [HTMX Documentation](https://htmx.org/docs/)
- [Askama Documentation](https://docs.rs/askama)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)

## Contact & Support

- **Issues**: Track progress in issues/ directory
- **Questions**: Document in code comments and READMEs
- **Decisions**: Record in issue files

---

**Remember**: The goal is to build something that's not just functional, but maintainable, secure, and pleasant to work with. Code for human brains, not machines.
