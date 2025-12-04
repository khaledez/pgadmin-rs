/// Rate Limiting Middleware
///
/// Implements per-IP rate limiting to prevent abuse and DoS attacks.
/// Uses a token bucket algorithm to limit the number of requests per minute.
use axum::{extract::ConnectInfo, http::StatusCode, middleware::Next, response::IntoResponse};
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

type LimiterMap = Arc<
    parking_lot::RwLock<
        std::collections::HashMap<String, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>,
    >,
>;

/// Configuration for rate limiting
pub struct RateLimitConfig {
    /// Requests allowed per minute per IP
    pub requests_per_minute: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 100,
        }
    }
}

/// Rate limiter that tracks requests per IP address
///
/// Uses the `governor` crate for efficient rate limiting with a token bucket algorithm.
pub struct RateLimitState {
    limiters: LimiterMap,
    config: RateLimitConfig,
}

impl RateLimitState {
    /// Create a new rate limit state with the given configuration
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            limiters: Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
            config,
        }
    }

    /// Get or create a rate limiter for the given IP address
    fn get_or_create_limiter(
        &self,
        ip: &str,
    ) -> Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>> {
        let mut limiters = self.limiters.write();

        if let Some(limiter) = limiters.get(ip) {
            Arc::clone(limiter)
        } else {
            let quota =
                Quota::per_minute(NonZeroU32::new(self.config.requests_per_minute).unwrap());
            let limiter = Arc::new(RateLimiter::direct(quota));
            limiters.insert(ip.to_string(), Arc::clone(&limiter));
            limiter
        }
    }

    /// Check if a request from the given IP should be allowed
    pub fn check_limit(&self, ip: &str) -> bool {
        let limiter = self.get_or_create_limiter(ip);
        limiter.check().is_ok()
    }
}

/// Rate limiting middleware that checks requests against per-IP limits
///
/// Extracts the client IP address and checks if the rate limit for that IP
/// has been exceeded. If the limit is exceeded, returns 429 Too Many Requests.
///
/// # Example
///
/// ```ignore
/// let rate_limit_state = RateLimitState::new(RateLimitConfig::default());
/// let app = Router::new()
///     .route("/api/query", post(handler))
///     .layer(middleware::from_fn_with_state(
///         rate_limit_state,
///         rate_limit_middleware,
///     ))
/// ```
pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    state: axum::extract::State<Arc<RateLimitState>>,
    req: axum::extract::Request,
    next: Next,
) -> impl IntoResponse {
    let ip = addr.ip().to_string();

    // Check rate limit
    if !state.check_limit(&ip) {
        return axum::response::Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .body(axum::body::Body::from("Rate limit exceeded"))
            .unwrap()
            .into_response();
    }

    // Request within limits, proceed normally
    next.run(req).await.into_response()
}

/// Endpoint-specific rate limiting configuration
/// Different endpoints may have different rate limits
#[allow(dead_code)]
pub struct EndpointRateLimits {
    /// Query execution: lower limit due to resource usage
    pub query_execute: u32,
    /// Table browsing: moderate limit
    pub table_browse: u32,
    /// Schema operations: lower limit due to modification
    pub schema_operations: u32,
    /// General API: standard limit
    pub general: u32,
}

impl Default for EndpointRateLimits {
    fn default() -> Self {
        Self {
            query_execute: 20,     // 20 queries per minute
            table_browse: 100,     // 100 table browses per minute
            schema_operations: 10, // 10 schema operations per minute
            general: 100,          // 100 general requests per minute
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_creation() {
        let config = RateLimitConfig {
            requests_per_minute: 60,
        };
        let state = RateLimitState::new(config);
        assert!(state.check_limit("127.0.0.1"));
    }

    #[test]
    fn test_rate_limit_exceeded() {
        let config = RateLimitConfig {
            requests_per_minute: 2,
        };
        let state = RateLimitState::new(config);
        let ip = "192.168.1.1";

        // First two requests should succeed
        assert!(state.check_limit(ip));
        assert!(state.check_limit(ip));

        // Third request should fail (quota exhausted)
        assert!(!state.check_limit(ip));
    }

    #[test]
    fn test_different_ips_separate_limits() {
        let config = RateLimitConfig {
            requests_per_minute: 2,
        };
        let state = RateLimitState::new(config);

        // IP1 uses up its quota
        assert!(state.check_limit("192.168.1.1"));
        assert!(state.check_limit("192.168.1.1"));
        assert!(!state.check_limit("192.168.1.1"));

        // IP2 should have its own quota available
        assert!(state.check_limit("192.168.1.2"));
        assert!(state.check_limit("192.168.1.2"));
        assert!(!state.check_limit("192.168.1.2"));
    }

    #[test]
    fn test_default_limits() {
        let limits = EndpointRateLimits::default();
        assert_eq!(limits.query_execute, 20);
        assert_eq!(limits.table_browse, 100);
        assert_eq!(limits.schema_operations, 10);
        assert_eq!(limits.general, 100);
    }
}
