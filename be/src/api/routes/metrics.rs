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
/// Pool and queue metrics are updated on each scrape to provide current state.
async fn get_metrics(
    metrics: web::Data<Arc<Metrics>>,
    state: web::Data<Arc<crate::AppState>>,
) -> Result<HttpResponse, Error> {
    // Update database connection pool metrics before rendering
    let pool_stats = state.db.pool_stats();
    metrics.update_pool_metrics(&pool_stats);

    // Update job queue metrics
    let (pending_jobs, retry_jobs, dead_letter_jobs) = state.db.get_queue_stats().await?;
    metrics.job_queue_depth.set(pending_jobs);
    metrics.job_retry_queue_depth.set(retry_jobs);
    metrics.job_dead_letter_queue_size.set(dead_letter_jobs);

    let output = metrics
        .render()
        .map_err(|e| Error::Internal(format!("Failed to render metrics: {}", e)))?;

    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(output))
}
