use crate::AppState;
use actix_web::{web, HttpResponse};
use serde_json::json;
use std::sync::Arc;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health_check));
}

/// Health check endpoint
/// Returns 200 OK if the service is healthy, 503 Service Unavailable if not
async fn health_check(state: web::Data<Arc<AppState>>) -> HttpResponse {
    // Check database connectivity
    match state.db.health_check().await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "status": "healthy",
            "service": "loupe-api",
            "database": "connected"
        })),
        Err(e) => {
            tracing::error!("Health check failed: {:?}", e);
            HttpResponse::ServiceUnavailable().json(json!({
                "status": "unhealthy",
                "service": "loupe-api",
                "database": "disconnected"
            }))
        }
    }
}
