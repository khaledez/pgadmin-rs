pub mod rate_limit;
/// Middleware module
///
/// Contains custom middleware for the application including:
/// - Security headers (XSS, clickjacking, MIME sniffing prevention)
/// - Rate limiting (per-IP request throttling)
/// - Request logging and tracing
pub mod security_headers;
