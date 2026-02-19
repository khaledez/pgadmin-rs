use axum::{extract::State, http::StatusCode, middleware::Next, response::IntoResponse};
use axum_extra::extract::CookieJar;
use sha2::{Digest, Sha256};
use std::sync::Arc;

const SESSION_COOKIE_NAME: &str = "pgadmin_session";

#[derive(Clone)]
pub struct AuthState {
    /// The expected session token (hash of the password). None means auth is disabled.
    pub session_token: Option<String>,
}

/// Derive a deterministic session token from the password.
pub fn derive_session_token(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"pgadmin-rs-session:");
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

/// Middleware that checks for a valid session cookie when password protection is enabled.
/// Allows through `/login`, `/health`, and `/static` paths without auth.
pub async fn auth_middleware(
    State(auth): State<Arc<AuthState>>,
    jar: CookieJar,
    req: axum::extract::Request,
    next: Next,
) -> impl IntoResponse {
    let expected_token = match &auth.session_token {
        Some(token) => token,
        None => return next.run(req).await.into_response(),
    };

    let path = req.uri().path().to_string();

    // Allow unauthenticated access to login page, login action, health check, and static files
    if path == "/login"
        || path == "/health"
        || path.starts_with("/static")
    {
        return next.run(req).await.into_response();
    }

    // Check session cookie
    if let Some(cookie) = jar.get(SESSION_COOKIE_NAME) {
        if cookie.value() == expected_token {
            return next.run(req).await.into_response();
        }
    }

    // Not authenticated — redirect browser requests, return 401 for API calls
    if path.starts_with("/api/") {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    axum::response::Redirect::to("/login").into_response()
}
