use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::LocalBoxFuture;
use opentelemetry::{
    trace::{FutureExt, Span, SpanKind, Status, TraceContextExt, Tracer},
    Context, KeyValue,
};
use opentelemetry_semantic_conventions::trace::{
    HTTP_REQUEST_METHOD, HTTP_RESPONSE_STATUS_CODE, HTTP_ROUTE, URL_PATH, URL_QUERY,
};
use std::future::{Ready, ready};
use super::correlation_id::CorrelationId;

/// Middleware to create OpenTelemetry spans for HTTP requests
///
/// Creates a span for each request with:
/// - HTTP method, path, route
/// - Request/response sizes
/// - Status code
/// - Correlation ID (if present)
/// - Error details (if request fails)
pub struct TracingMiddleware;

impl<S, B> Transform<S, ServiceRequest> for TracingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = TracingMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TracingMiddlewareService { service }))
    }
}

pub struct TracingMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for TracingMiddlewareService<S>
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
        let tracer = opentelemetry::global::tracer("loupe-api");

        // Extract request metadata
        let method = req.method().as_str().to_string();
        let path = req.path().to_string();
        let query = req.query_string().to_string();

        // Get correlation ID if present
        let correlation_id = req
            .extensions()
            .get::<CorrelationId>()
            .map(|id| id.0.clone());

        // Create span with semantic conventions
        let mut span = tracer
            .span_builder(format!("{} {}", method, path))
            .with_kind(SpanKind::Server)
            .with_attributes(vec![
                KeyValue::new(HTTP_REQUEST_METHOD, method.clone()),
                KeyValue::new(URL_PATH, path.clone()),
            ])
            .start(&tracer);

        // Add query string if present
        if !query.is_empty() {
            span.set_attribute(KeyValue::new(URL_QUERY, query));
        }

        // Add correlation ID if present
        if let Some(ref correlation_id) = correlation_id {
            span.set_attribute(KeyValue::new("correlation_id", correlation_id.clone()));
        }

        // Try to get route pattern for better cardinality
        if let Some(route) = req.match_pattern() {
            span.set_attribute(KeyValue::new(HTTP_ROUTE, route));
        }

        let cx = Context::current_with_span(span);
        let fut = self.service.call(req);

        Box::pin(async move {
            let result = fut.await;

            // Get the current span from the context
            let ctx = Context::current();
            let span = ctx.span();

            match &result {
                Ok(response) => {
                    let status_code = response.status().as_u16();
                    span.set_attribute(KeyValue::new(
                        HTTP_RESPONSE_STATUS_CODE,
                        status_code as i64,
                    ));

                    // Mark span as error if status is 5xx
                    if status_code >= 500 {
                        span.set_status(Status::error(format!("HTTP {}", status_code)));
                    } else {
                        span.set_status(Status::Ok);
                    }
                }
                Err(error) => {
                    span.set_status(Status::error(error.to_string()));
                    span.set_attribute(KeyValue::new("error", true));
                    span.set_attribute(KeyValue::new("error.message", error.to_string()));
                }
            }

            span.end();
            result
        }.with_context(cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, HttpResponse, test, web};

    #[actix_web::test]
    async fn test_tracing_middleware() {
        let app = test::init_service(
            App::new()
                .wrap(TracingMiddleware)
                .route("/test", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_tracing_middleware_with_error() {
        let app = test::init_service(
            App::new()
                .wrap(TracingMiddleware)
                .route(
                    "/error",
                    web::get().to(|| async { HttpResponse::InternalServerError().finish() }),
                ),
        )
        .await;

        let req = test::TestRequest::get().uri("/error").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status().as_u16(), 500);
    }
}
