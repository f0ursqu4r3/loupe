use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::LocalBoxFuture;
use std::future::{Ready, ready};
use super::correlation_id::CorrelationId;

/// Middleware to enrich Sentry events with correlation IDs and user context
pub struct SentryContextMiddleware;

impl<S, B> Transform<S, ServiceRequest> for SentryContextMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SentryContextMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SentryContextMiddlewareService { service }))
    }
}

pub struct SentryContextMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SentryContextMiddlewareService<S>
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
        // Configure Sentry scope with request context
        sentry::configure_scope(|scope| {
            // Add correlation ID
            if let Some(correlation_id) = req.extensions().get::<CorrelationId>() {
                scope.set_tag("request_id", &correlation_id.0);
                scope.set_extra("correlation_id", correlation_id.0.clone().into());
            }

            // Add request metadata
            scope.set_tag("http.method", req.method().as_str());
            scope.set_tag("http.route", req.path());
        });

        let fut = self.service.call(req);

        Box::pin(async move {
            let result = fut.await;
            result
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, HttpResponse, test, web};

    #[actix_web::test]
    async fn test_sentry_context_middleware() {
        let app = test::init_service(
            App::new()
                .wrap(SentryContextMiddleware)
                .route("/test", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }
}
