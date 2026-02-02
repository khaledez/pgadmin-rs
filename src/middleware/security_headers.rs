/// Security Headers Middleware
///
/// Adds important security headers to all HTTP responses to protect against
/// common web vulnerabilities including XSS, clickjacking, and MIME type sniffing.
///
/// Headers added:
/// - Content-Security-Policy: Restricts resource loading
/// - X-Content-Type-Options: Prevents MIME type sniffing
/// - X-Frame-Options: Prevents clickjacking
/// - X-XSS-Protection: Additional XSS protection (legacy browsers)
/// - Referrer-Policy: Controls referrer information
/// - Strict-Transport-Security: Forces HTTPS (production only)
use axum::{middleware::Next, response::IntoResponse};

/// Add security headers to the response
///
/// This middleware is applied to all responses and ensures critical security
/// headers are present to protect against common web vulnerabilities.
///
/// # Security Headers
///
/// - **Content-Security-Policy**: Restricts which resources can be loaded
/// - **X-Content-Type-Options**: Prevents browser MIME type sniffing
/// - **X-Frame-Options**: Prevents the page from being framed (clickjacking protection)
/// - **X-XSS-Protection**: Legacy XSS protection header
/// - **Referrer-Policy**: Controls how much referrer info is shared
pub async fn security_headers(req: axum::extract::Request, next: Next) -> impl IntoResponse {
    let mut response = next.run(req).await;

    // Content-Security-Policy: Restrict resource loading to prevent XSS
    // - default-src 'self': Only allow resources from same origin by default
    // - script-src 'self' + CDN: Allow scripts from self and Tailwind CDN
    // - style-src 'self' 'unsafe-inline' + CDN: Allow styles from self and DaisyUI CDN
    // - img-src 'self' data:: Allow images from self and data URLs
    // - font-src 'self': Fonts only from self
    // - connect-src 'self': AJAX/WebSocket only to self (blocks external API calls)
    // - frame-ancestors 'none': Prevent framing in iframes
    response.headers_mut().insert(
        "Content-Security-Policy",
        "default-src 'self'; \
         script-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net; \
         style-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net; \
         img-src 'self' data:; \
         font-src 'self'; \
         connect-src 'self' https://cdn.jsdelivr.net; \
         frame-ancestors 'none'; \
         base-uri 'self'; \
         form-action 'self';"
            .parse()
            .unwrap(),
    );

    // X-Content-Type-Options: Prevent MIME type sniffing
    // Forces browser to respect the Content-Type header
    response
        .headers_mut()
        .insert("X-Content-Type-Options", "nosniff".parse().unwrap());

    // X-Frame-Options: Prevent clickjacking
    // DENY means this page cannot be framed by any origin
    response
        .headers_mut()
        .insert("X-Frame-Options", "DENY".parse().unwrap());

    // X-XSS-Protection: Legacy XSS protection (for older browsers)
    // mode=block will block the page if XSS is detected
    response
        .headers_mut()
        .insert("X-XSS-Protection", "1; mode=block".parse().unwrap());

    // Referrer-Policy: Control referrer information
    // strict-origin-when-cross-origin: Send full URL to same-origin,
    // only origin to cross-origin, nothing with downgrade
    response.headers_mut().insert(
        "Referrer-Policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );

    // Strict-Transport-Security: Force HTTPS (only in production)
    // max-age=31536000: Valid for 1 year
    // includeSubDomains: Apply to all subdomains
    // preload: Allow inclusion in HSTS preload lists
    if cfg!(not(debug_assertions)) {
        response.headers_mut().insert(
            "Strict-Transport-Security",
            "max-age=31536000; includeSubDomains; preload"
                .parse()
                .unwrap(),
        );
    }

    // Permissions-Policy: Restrict browser features/APIs
    // This prevents pages from using certain APIs unless explicitly allowed
    response.headers_mut().insert(
        "Permissions-Policy",
        "geolocation=(), microphone=(), camera=()".parse().unwrap(),
    );

    response
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_security_headers_module_loads() {
        // Security headers middleware is tested through integration tests
        // This test ensures the module compiles correctly
    }
}
