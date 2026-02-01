use crate::AppState;
use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health_check))
        .route("/health/live", web::get().to(liveness_check))
        .route("/health/ready", web::get().to(readiness_check));
}

/// Overall health status
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Individual component health check result
#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ComponentHealth {
    pub fn healthy() -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: None,
            latency_ms: None,
            details: None,
        }
    }

    pub fn healthy_with_latency(latency_ms: u64) -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: None,
            latency_ms: Some(latency_ms),
            details: None,
        }
    }

    pub fn unhealthy(message: String) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            message: Some(message),
            latency_ms: None,
            details: None,
        }
    }
}

/// Detailed health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub timestamp: String,
    pub version: String,
    pub checks: HashMap<String, ComponentHealth>,
}

impl HealthResponse {
    pub fn new(checks: HashMap<String, ComponentHealth>) -> Self {
        // Determine overall status from component statuses
        let overall_status = if checks.values().all(|c| c.status == HealthStatus::Healthy) {
            HealthStatus::Healthy
        } else if checks.values().any(|c| c.status == HealthStatus::Unhealthy) {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Degraded
        };

        Self {
            status: overall_status,
            timestamp: Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            checks,
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.status == HealthStatus::Healthy
    }
}

/// GET /health - Detailed health check with all components
///
/// Returns comprehensive health information including:
/// - Database connectivity
/// - Migration status
/// - Memory usage
/// - Overall service status
async fn health_check(state: web::Data<Arc<AppState>>) -> HttpResponse {
    let mut checks = HashMap::new();

    // Database connectivity check
    let db_start = std::time::Instant::now();
    let db_health = match state.db.health_check().await {
        Ok(_) => {
            let latency = db_start.elapsed().as_millis() as u64;
            ComponentHealth::healthy_with_latency(latency)
        }
        Err(e) => ComponentHealth::unhealthy(format!("Database error: {}", e)),
    };
    checks.insert("database".to_string(), db_health);

    // Migrations status check
    let migrations_health = match state.db.check_migrations_applied().await {
        Ok(true) => ComponentHealth::healthy(),
        Ok(false) => ComponentHealth::unhealthy("Pending migrations".to_string()),
        Err(e) => ComponentHealth::unhealthy(format!("Migration check failed: {}", e)),
    };
    checks.insert("migrations".to_string(), migrations_health);

    // Memory usage (basic check)
    #[cfg(target_os = "linux")]
    {
        if let Ok(usage) = get_memory_usage() {
            let memory_health = if usage.percent < 90.0 {
                ComponentHealth {
                    status: HealthStatus::Healthy,
                    message: None,
                    latency_ms: None,
                    details: Some(serde_json::json!({
                        "used_mb": usage.used_mb,
                        "percent": usage.percent
                    })),
                }
            } else {
                ComponentHealth::unhealthy(format!("High memory usage: {:.1}%", usage.percent))
            };
            checks.insert("memory".to_string(), memory_health);
        }
    }

    let response = HealthResponse::new(checks);

    if response.is_healthy() {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::ServiceUnavailable().json(response)
    }
}

/// GET /health/live - Liveness probe
///
/// Indicates whether the application is running.
/// Returns 200 if the process is alive, regardless of dependencies.
/// Used by Kubernetes to restart unhealthy pods.
async fn liveness_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "alive",
        "timestamp": Utc::now().to_rfc3339()
    }))
}

/// GET /health/ready - Readiness probe
///
/// Indicates whether the application is ready to serve traffic.
/// Checks critical dependencies (database, migrations).
/// Returns 200 if ready, 503 if not ready.
/// Used by Kubernetes to route traffic only to ready pods.
async fn readiness_check(state: web::Data<Arc<AppState>>) -> HttpResponse {
    let mut checks = HashMap::new();

    // Database connectivity is critical for readiness
    let db_health = match state.db.health_check().await {
        Ok(_) => ComponentHealth::healthy(),
        Err(e) => ComponentHealth::unhealthy(format!("Database error: {}", e)),
    };
    checks.insert("database".to_string(), db_health);

    // Migrations must be applied for readiness
    let migrations_health = match state.db.check_migrations_applied().await {
        Ok(true) => ComponentHealth::healthy(),
        Ok(false) => ComponentHealth::unhealthy("Pending migrations".to_string()),
        Err(e) => ComponentHealth::unhealthy(format!("Migration check failed: {}", e)),
    };
    checks.insert("migrations".to_string(), migrations_health);

    let response = HealthResponse::new(checks);

    if response.is_healthy() {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::ServiceUnavailable().json(response)
    }
}

#[cfg(target_os = "linux")]
struct MemoryUsage {
    used_mb: f64,
    percent: f64,
}

#[cfg(target_os = "linux")]
fn get_memory_usage() -> Result<MemoryUsage, std::io::Error> {
    use std::fs;

    let status = fs::read_to_string("/proc/self/status")?;
    let mut vm_rss = 0;

    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            if let Some(value) = line.split_whitespace().nth(1) {
                vm_rss = value.parse::<u64>().unwrap_or(0);
                break;
            }
        }
    }

    // Convert KB to MB
    let used_mb = vm_rss as f64 / 1024.0;

    // Get total system memory for percentage
    let meminfo = fs::read_to_string("/proc/meminfo")?;
    let mut mem_total = 0;

    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            if let Some(value) = line.split_whitespace().nth(1) {
                mem_total = value.parse::<u64>().unwrap_or(0);
                break;
            }
        }
    }

    let percent = if mem_total > 0 {
        (vm_rss as f64 / mem_total as f64) * 100.0
    } else {
        0.0
    };

    Ok(MemoryUsage { used_mb, percent })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_health_creation() {
        let healthy = ComponentHealth::healthy();
        assert_eq!(healthy.status, HealthStatus::Healthy);
        assert!(healthy.message.is_none());

        let unhealthy = ComponentHealth::unhealthy("Test error".to_string());
        assert_eq!(unhealthy.status, HealthStatus::Unhealthy);
        assert_eq!(unhealthy.message, Some("Test error".to_string()));
    }

    #[test]
    fn test_health_response_overall_status() {
        let mut checks = HashMap::new();
        checks.insert("db".to_string(), ComponentHealth::healthy());
        checks.insert("cache".to_string(), ComponentHealth::healthy());

        let response = HealthResponse::new(checks);
        assert_eq!(response.status, HealthStatus::Healthy);
        assert!(response.is_healthy());
    }

    #[test]
    fn test_health_response_unhealthy_if_any_component_fails() {
        let mut checks = HashMap::new();
        checks.insert("db".to_string(), ComponentHealth::healthy());
        checks.insert(
            "cache".to_string(),
            ComponentHealth::unhealthy("Connection failed".to_string()),
        );

        let response = HealthResponse::new(checks);
        assert_eq!(response.status, HealthStatus::Unhealthy);
        assert!(!response.is_healthy());
    }
}
