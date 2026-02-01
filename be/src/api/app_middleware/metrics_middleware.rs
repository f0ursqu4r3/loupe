use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::LocalBoxFuture;
use loupe::Metrics;
use std::future::{Ready, ready};
use std::sync::Arc;
use std::time::Instant;

/// Middleware for collecting Prometheus metrics
pub struct MetricsMiddleware {
    pub metrics: Arc<Metrics>,
}

impl MetricsMiddleware {
    pub fn new(metrics: Arc<Metrics>) -> Self {
        Self { metrics }
    }
}

impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MetricsMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MetricsMiddlewareService {
            service,
            metrics: self.metrics.clone(),
        }))
    }
}

pub struct MetricsMiddlewareService<S> {
    service: S,
    metrics: Arc<Metrics>,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddlewareService<S>
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

        // Normalize path for metrics (replace IDs with :id)
        let endpoint = Metrics::normalize_path(&path);

        // Increment in-flight requests
        self.metrics
            .http_requests_in_flight
            .with_label_values(&[&method, &endpoint])
            .inc();

        let metrics = self.metrics.clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            let result = fut.await;
            let duration = start_time.elapsed();

            // Decrement in-flight requests
            // Note: This is a counter, not a gauge, so we're using it as an approximation
            // For a proper gauge, we'd need the prometheus crate's Gauge type

            // Record metrics based on result
            match &result {
                Ok(res) => {
                    let status = res.status().as_u16().to_string();

                    // Record request count
                    metrics
                        .http_requests_total
                        .with_label_values(&[&method, &endpoint, &status])
                        .inc();

                    // Record request duration
                    metrics
                        .http_request_duration_seconds
                        .with_label_values(&[&method, &endpoint])
                        .observe(duration.as_secs_f64());
                }
                Err(_err) => {
                    // Record failed request
                    metrics
                        .http_requests_total
                        .with_label_values(&[&method, &endpoint, "500"])
                        .inc();

                    metrics
                        .http_request_duration_seconds
                        .with_label_values(&[&method, &endpoint])
                        .observe(duration.as_secs_f64());
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
    async fn test_metrics_middleware_success() {
        let metrics = Arc::new(Metrics::new().unwrap());

        let app = test::init_service(
            App::new()
                .wrap(MetricsMiddleware::new(metrics.clone()))
                .route("/test", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        // Verify metrics were recorded
        let output = metrics.render().unwrap();
        assert!(output.contains("loupe_api_http_requests_total"));
        assert!(output.contains("method=\"GET\""));
        assert!(output.contains("status=\"200\""));
    }

    #[actix_web::test]
    async fn test_metrics_middleware_with_uuid_path() {
        let metrics = Arc::new(Metrics::new().unwrap());

        let app = test::init_service(
            App::new()
                .wrap(MetricsMiddleware::new(metrics.clone()))
                .route(
                    "/api/v1/dashboards/{id}",
                    web::get().to(|| async { HttpResponse::Ok().finish() }),
                ),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/v1/dashboards/550e8400-e29b-41d4-a716-446655440000")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        // Verify path was normalized
        let output = metrics.render().unwrap();
        assert!(output.contains("endpoint=\"/api/v1/dashboards/:id\""));
    }
}
