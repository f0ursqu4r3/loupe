use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::LocalBoxFuture;
use std::future::{Ready, ready};

/// Middleware to add comprehensive security headers to all HTTP responses
///
/// This middleware implements defense-in-depth security by adding multiple
/// layers of HTTP security headers to protect against common web vulnerabilities.
///
/// # Headers Added
///
/// ## Content Security Policy (CSP)
/// - **Purpose**: Prevents XSS attacks by controlling which resources can be loaded
/// - **Configuration**: Environment-based via `CSP_MODE` (strict/development)
/// - **Strict Mode**: No inline scripts/styles, only same-origin resources
/// - **Dev Mode**: Allows unsafe-inline/eval for development convenience
///
/// ## X-Frame-Options
/// - **Value**: `DENY`
/// - **Purpose**: Prevents clickjacking by blocking iframe embedding
///
/// ## X-Content-Type-Options
/// - **Value**: `nosniff`
/// - **Purpose**: Prevents MIME type sniffing attacks
///
/// ## X-XSS-Protection
/// - **Value**: `1; mode=block`
/// - **Purpose**: Enables browser's built-in XSS filter (legacy, but harmless)
///
/// ## Strict-Transport-Security (HSTS)
/// - **Value**: `max-age=31536000; includeSubDomains; preload`
/// - **Purpose**: Forces HTTPS connections for 1 year
/// - **Enabled**: Only when `ENABLE_HSTS=true` (production HTTPS deployments)
///
/// ## Referrer-Policy
/// - **Value**: `strict-origin-when-cross-origin`
/// - **Purpose**: Controls referrer information sent to other sites
///
/// ## Permissions-Policy
/// - **Value**: Blocks geolocation, microphone, camera
/// - **Purpose**: Prevents unauthorized access to sensitive browser APIs
///
/// ## Cache-Control
/// - **Health Endpoint**: `public, max-age=60` (cacheable)
/// - **All Others**: `no-store, no-cache` (sensitive data, no caching)
///
/// # Environment Variables
///
/// - `CSP_MODE`: "strict" (production) or "development" (default: "strict")
/// - `ENABLE_HSTS`: "true" to enable HSTS (default: "false", only use with HTTPS)
///
/// # Security Considerations
///
/// - HSTS should only be enabled when serving over HTTPS
/// - CSP strict mode may break functionality if frontend uses inline scripts
/// - Adjust CSP directives based on your frontend requirements
pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityHeadersMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersMiddleware { service }))
    }
}

pub struct SecurityHeadersMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();
        let fut = self.service.call(req);

        // Read environment configuration once
        let csp_mode = std::env::var("CSP_MODE").unwrap_or_else(|_| "strict".to_string());
        let enable_hsts = std::env::var("ENABLE_HSTS")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        Box::pin(async move {
            let mut res = fut.await?;

            // Add security headers
            let headers = res.headers_mut();

            // Cache-Control headers
            // Health endpoint can be cached for a short duration
            if path == "/api/v1/health" {
                headers.insert(
                    actix_web::http::header::CACHE_CONTROL,
                    actix_web::http::header::HeaderValue::from_static("public, max-age=60"),
                );
            } else {
                // All other endpoints: no caching (contains user-specific or sensitive data)
                headers.insert(
                    actix_web::http::header::CACHE_CONTROL,
                    actix_web::http::header::HeaderValue::from_static("no-store, no-cache, must-revalidate, private"),
                );
                headers.insert(
                    actix_web::http::header::PRAGMA,
                    actix_web::http::header::HeaderValue::from_static("no-cache"),
                );
            }

            // Prevent clickjacking attacks
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-frame-options"),
                actix_web::http::header::HeaderValue::from_static("DENY"),
            );

            // Prevent MIME type sniffing
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-content-type-options"),
                actix_web::http::header::HeaderValue::from_static("nosniff"),
            );

            // Enable XSS protection (legacy, but doesn't hurt)
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-xss-protection"),
                actix_web::http::header::HeaderValue::from_static("1; mode=block"),
            );

            // Control referrer information
            headers.insert(
                actix_web::http::header::HeaderName::from_static("referrer-policy"),
                actix_web::http::header::HeaderValue::from_static(
                    "strict-origin-when-cross-origin",
                ),
            );

            // Content Security Policy (CSP)
            // Environment-based CSP for security vs development flexibility
            let csp_value = if csp_mode == "development" {
                // Development mode: Allow inline scripts/styles for easier debugging
                // WARNING: Only use in development, never in production
                "default-src 'self'; \
                 script-src 'self' 'unsafe-inline' 'unsafe-eval'; \
                 style-src 'self' 'unsafe-inline'; \
                 img-src 'self' data: https:; \
                 font-src 'self' data:; \
                 connect-src 'self'; \
                 frame-ancestors 'none'; \
                 base-uri 'self'; \
                 form-action 'self';"
            } else {
                // Strict mode (production): Maximum security, no inline scripts/styles
                // This is the recommended production configuration
                "default-src 'self'; \
                 script-src 'self'; \
                 style-src 'self'; \
                 img-src 'self' data: https:; \
                 font-src 'self' data:; \
                 connect-src 'self'; \
                 frame-ancestors 'none'; \
                 base-uri 'self'; \
                 form-action 'self'; \
                 upgrade-insecure-requests;"
            };

            headers.insert(
                actix_web::http::header::HeaderName::from_static("content-security-policy"),
                actix_web::http::header::HeaderValue::from_str(csp_value)
                    .unwrap_or_else(|_| actix_web::http::header::HeaderValue::from_static("default-src 'self'")),
            );

            // Permissions Policy (formerly Feature Policy)
            headers.insert(
                actix_web::http::header::HeaderName::from_static("permissions-policy"),
                actix_web::http::header::HeaderValue::from_static(
                    "geolocation=(), microphone=(), camera=()",
                ),
            );

            // Strict-Transport-Security (HSTS)
            // Only enable when serving over HTTPS (production deployments)
            // CRITICAL: Do NOT enable HSTS on HTTP-only servers or localhost
            if enable_hsts {
                headers.insert(
                    actix_web::http::header::HeaderName::from_static("strict-transport-security"),
                    actix_web::http::header::HeaderValue::from_static(
                        "max-age=31536000; includeSubDomains; preload",
                    ),
                );
            }

            Ok(res)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, HttpResponse, test, web};

    #[actix_web::test]
    async fn test_security_headers() {
        let app = test::init_service(App::new().wrap(SecurityHeaders).route(
            "/test",
            web::get().to(|| async { HttpResponse::Ok().body("test") }),
        ))
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.headers().contains_key("x-frame-options"));
        assert!(resp.headers().contains_key("x-content-type-options"));
        assert!(resp.headers().contains_key("content-security-policy"));
        assert!(resp.headers().contains_key("referrer-policy"));
    }
}
