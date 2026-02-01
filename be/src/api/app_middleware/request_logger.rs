use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::LocalBoxFuture;
use std::future::{Ready, ready};
use std::time::Instant;
use super::correlation_id::CorrelationId;

/// Middleware for structured request logging with correlation IDs and request duration
pub struct RequestLogger;

impl<S, B> Transform<S, ServiceRequest> for RequestLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestLoggerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestLoggerMiddleware { service }))
    }
}

pub struct RequestLoggerMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestLoggerMiddleware<S>
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
        let start_time = Instant::now();
        let method = req.method().to_string();
        let path = req.path().to_string();
        let query = req.query_string().to_string();

        // Extract correlation ID from request extensions
        let correlation_id = req
            .extensions()
            .get::<CorrelationId>()
            .map(|c| c.0.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let fut = self.service.call(req);

        Box::pin(async move {
            let result = fut.await;
            let duration = start_time.elapsed();
            let duration_ms = duration.as_millis();

            match &result {
                Ok(res) => {
                    let status = res.status().as_u16();

                    // Structured logging with all relevant fields
                    if status >= 500 {
                        tracing::error!(
                            request_id = %correlation_id,
                            method = %method,
                            path = %path,
                            query = %query,
                            status = status,
                            duration_ms = duration_ms,
                            "Request completed with server error"
                        );
                    } else if status >= 400 {
                        tracing::warn!(
                            request_id = %correlation_id,
                            method = %method,
                            path = %path,
                            query = %query,
                            status = status,
                            duration_ms = duration_ms,
                            "Request completed with client error"
                        );
                    } else {
                        tracing::info!(
                            request_id = %correlation_id,
                            method = %method,
                            path = %path,
                            query = %query,
                            status = status,
                            duration_ms = duration_ms,
                            "Request completed"
                        );
                    }
                }
                Err(err) => {
                    tracing::error!(
                        request_id = %correlation_id,
                        method = %method,
                        path = %path,
                        query = %query,
                        error = %err,
                        duration_ms = duration_ms,
                        "Request failed with error"
                    );
                }
            }

            result
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, HttpResponse, test, web};

    #[actix_web::test]
    async fn test_request_logger_success() {
        let app = test::init_service(
            App::new()
                .wrap(RequestLogger)
                .route("/test", web::get().to(|| async { HttpResponse::Ok().body("success") })),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_request_logger_error() {
        let app = test::init_service(
            App::new()
                .wrap(RequestLogger)
                .route("/error", web::get().to(|| async {
                    HttpResponse::InternalServerError().body("error")
                })),
        )
        .await;

        let req = test::TestRequest::get().uri("/error").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status().as_u16(), 500);
    }
}
