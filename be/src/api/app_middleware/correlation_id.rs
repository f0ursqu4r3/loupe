use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    http::header::{HeaderName, HeaderValue},
};
use futures_util::future::LocalBoxFuture;
use std::future::{Ready, ready};
use uuid::Uuid;

/// Header name for correlation ID
const REQUEST_ID_HEADER: &str = "x-request-id";

/// Correlation ID for distributed tracing and request tracking
#[derive(Clone, Debug)]
pub struct CorrelationId(pub String);

/// Middleware to add correlation IDs to all requests
pub struct CorrelationIdMiddleware;

impl<S, B> Transform<S, ServiceRequest> for CorrelationIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CorrelationIdMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CorrelationIdMiddlewareService { service }))
    }
}

pub struct CorrelationIdMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CorrelationIdMiddlewareService<S>
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
        // Extract correlation ID from request header or generate a new one
        let correlation_id = req
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        // Store correlation ID in request extensions
        req.extensions_mut()
            .insert(CorrelationId(correlation_id.clone()));

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            // Add correlation ID to response headers
            if let Ok(header_value) = HeaderValue::from_str(&correlation_id) {
                res.headers_mut().insert(
                    HeaderName::from_static(REQUEST_ID_HEADER),
                    header_value,
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
    async fn test_generates_correlation_id() {
        let app = test::init_service(
            App::new()
                .wrap(CorrelationIdMiddleware)
                .route("/test", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        // Should have generated a correlation ID
        assert!(resp.headers().contains_key(REQUEST_ID_HEADER));
        let id = resp.headers().get(REQUEST_ID_HEADER).unwrap().to_str().unwrap();

        // Should be a valid UUID format
        assert!(Uuid::parse_str(id).is_ok());
    }

    #[actix_web::test]
    async fn test_preserves_existing_correlation_id() {
        let app = test::init_service(
            App::new()
                .wrap(CorrelationIdMiddleware)
                .route("/test", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let existing_id = "test-correlation-id-123";
        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header((REQUEST_ID_HEADER, existing_id))
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should preserve the existing correlation ID
        let id = resp.headers().get(REQUEST_ID_HEADER).unwrap().to_str().unwrap();
        assert_eq!(id, existing_id);
    }

    #[actix_web::test]
    async fn test_correlation_id_in_extensions() {
        let app = test::init_service(
            App::new()
                .wrap(CorrelationIdMiddleware)
                .route("/test", web::get().to(|req: actix_web::HttpRequest| async move {
                    // Extract correlation ID from extensions
                    let correlation_id = req.extensions().get::<CorrelationId>().unwrap().0.clone();
                    HttpResponse::Ok().body(correlation_id)
                })),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        let body_id = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body_id).unwrap();

        // Should be a valid UUID
        assert!(Uuid::parse_str(body_str).is_ok());
    }
}
