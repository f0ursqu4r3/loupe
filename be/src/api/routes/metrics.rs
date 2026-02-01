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
///
/// Pool metrics are updated on each scrape to provide current connection pool state.
async fn get_metrics(
    metrics: web::Data<Arc<Metrics>>,
    state: web::Data<Arc<crate::AppState>>,
) -> Result<HttpResponse, Error> {
    // Update database connection pool metrics before rendering
    let pool_stats = state.db.pool_stats();
    metrics.update_pool_metrics(&pool_stats);

    let output = metrics
        .render()
        .map_err(|e| Error::Internal(format!("Failed to render metrics: {}", e)))?;

    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(output))
}
