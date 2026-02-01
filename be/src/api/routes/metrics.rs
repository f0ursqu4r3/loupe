use actix_web::{HttpResponse, web};
use loupe::{Error, Metrics};
use std::sync::Arc;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/metrics", web::get().to(get_metrics));
}

/// GET /metrics - Prometheus metrics endpoint
///
/// Returns metrics in Prometheus text format for scraping by monitoring systems.
/// This endpoint is typically not authenticated to allow Prometheus to scrape it.
async fn get_metrics(metrics: web::Data<Arc<Metrics>>) -> Result<HttpResponse, Error> {
    let output = metrics
        .render()
        .map_err(|e| Error::Internal(format!("Failed to render metrics: {}", e)))?;

    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(output))
}
