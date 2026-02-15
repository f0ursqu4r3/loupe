use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounter, IntCounterVec, IntGauge, Opts, Registry,
    TextEncoder,
};
use std::sync::Arc;

/// Application metrics registry
#[derive(Clone)]
pub struct Metrics {
    pub registry: Arc<Registry>,

    // HTTP metrics
    pub http_requests_total: IntCounterVec,
    pub http_request_duration_seconds: HistogramVec,
    pub http_requests_in_flight: IntCounterVec,

    // Database connection pool metrics
    pub db_pool_connections_active: IntGauge,
    pub db_pool_connections_idle: IntGauge,
    pub db_pool_connections_max: IntGauge,
    pub db_pool_acquire_duration_seconds: HistogramVec,
    pub db_pool_acquire_timeout_total: IntCounter,

    // Query execution metrics
    pub query_executions_total: IntCounterVec,
    pub query_execution_duration_seconds: HistogramVec,
    pub query_rows_returned: HistogramVec,
    pub query_timeouts_total: IntCounter,
    pub queries_in_flight: IntGauge,

    // Job processing metrics
    pub jobs_claimed_total: IntCounterVec,
    pub job_processing_duration_seconds: HistogramVec,
    pub job_queue_depth: IntGauge,
    pub job_retry_queue_depth: IntGauge,
    pub job_dead_letter_queue_size: IntGauge,

    // Cache metrics
    pub cache_requests_total: IntCounterVec,
    pub cache_hit_rate: prometheus::Gauge,
}

impl Metrics {
    /// Create a new metrics registry with default metrics
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();

        // HTTP request counter: tracks total requests by method, path, and status
        let http_requests_total = IntCounterVec::new(
            Opts::new("http_requests_total", "Total number of HTTP requests")
                .namespace("loupe")
                .subsystem("api"),
            &["method", "endpoint", "status"],
        )?;

        // HTTP request duration: tracks request latency as histogram
        let http_request_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request latency in seconds",
            )
            .namespace("loupe")
            .subsystem("api")
            // Buckets: 1ms, 5ms, 10ms, 25ms, 50ms, 100ms, 250ms, 500ms, 1s, 2.5s, 5s, 10s
            .buckets(vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ]),
            &["method", "endpoint"],
        )?;

        // In-flight requests gauge (using counter with inc/dec)
        let http_requests_in_flight = IntCounterVec::new(
            Opts::new(
                "http_requests_in_flight",
                "Number of HTTP requests currently being processed",
            )
            .namespace("loupe")
            .subsystem("api"),
            &["method", "endpoint"],
        )?;

        // Database connection pool metrics
        let db_pool_connections_active = IntGauge::new(
            "loupe_db_pool_connections_active",
            "Number of active database connections currently in use",
        )?;

        let db_pool_connections_idle = IntGauge::new(
            "loupe_db_pool_connections_idle",
            "Number of idle database connections in the pool",
        )?;

        let db_pool_connections_max = IntGauge::new(
            "loupe_db_pool_connections_max",
            "Maximum number of database connections allowed",
        )?;

        let db_pool_acquire_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "loupe_db_pool_acquire_duration_seconds",
                "Time taken to acquire a database connection from the pool",
            )
            // Buckets: 1ms, 5ms, 10ms, 25ms, 50ms, 100ms, 250ms, 500ms, 1s, 5s
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 5.0]),
            &["operation"],
        )?;

        let db_pool_acquire_timeout_total = IntCounter::new(
            "loupe_db_pool_acquire_timeout_total",
            "Total number of database connection acquisition timeouts",
        )?;

        // Query execution metrics
        let query_executions_total = IntCounterVec::new(
            Opts::new("loupe_query_executions_total", "Total number of query executions")
                .namespace("loupe")
                .subsystem("runner"),
            &["status"], // completed, failed, timeout, cancelled
        )?;

        let query_execution_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "loupe_query_execution_duration_seconds",
                "Query execution duration in seconds",
            )
            .namespace("loupe")
            .subsystem("runner")
            // Buckets: 100ms, 500ms, 1s, 2s, 5s, 10s, 30s, 60s, 120s, 300s
            .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0]),
            &["query_id"],
        )?;

        let query_rows_returned = HistogramVec::new(
            HistogramOpts::new(
                "loupe_query_rows_returned",
                "Number of rows returned by queries",
            )
            .namespace("loupe")
            .subsystem("runner")
            // Buckets: 10, 100, 1K, 10K, 100K
            .buckets(vec![10.0, 100.0, 1000.0, 10000.0, 100000.0]),
            &["query_id"],
        )?;

        let query_timeouts_total = IntCounter::new(
            "loupe_query_timeouts_total",
            "Total number of query timeouts",
        )?;

        let queries_in_flight = IntGauge::new(
            "loupe_queries_in_flight",
            "Number of queries currently executing",
        )?;

        // Job processing metrics
        let jobs_claimed_total = IntCounterVec::new(
            Opts::new("loupe_jobs_claimed_total", "Total number of jobs claimed from queue")
                .namespace("loupe")
                .subsystem("runner"),
            &["type"], // "new" or "retry"
        )?;

        let job_processing_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "loupe_job_processing_duration_seconds",
                "Job processing duration (from claim to completion) in seconds",
            )
            .namespace("loupe")
            .subsystem("runner")
            // Buckets: 100ms, 500ms, 1s, 5s, 10s, 30s, 60s, 120s, 300s, 600s
            .buckets(vec![0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0, 600.0]),
            &["status"], // completed, failed, timeout
        )?;

        let job_queue_depth = IntGauge::new(
            "loupe_job_queue_depth",
            "Number of jobs waiting in queue (status=queued)",
        )?;

        let job_retry_queue_depth = IntGauge::new(
            "loupe_job_retry_queue_depth",
            "Number of jobs waiting for retry (status=failed with next_retry_at)",
        )?;

        let job_dead_letter_queue_size = IntGauge::new(
            "loupe_job_dead_letter_queue_size",
            "Number of permanently failed jobs in dead letter queue",
        )?;

        // Cache metrics
        let cache_requests_total = IntCounterVec::new(
            Opts::new("loupe_cache_requests_total", "Total number of cache requests")
                .namespace("loupe")
                .subsystem("cache"),
            &["result"], // "hit" or "miss"
        )?;

        let cache_hit_rate = prometheus::Gauge::with_opts(
            prometheus::Opts::new("loupe_cache_hit_rate", "Cache hit rate percentage (0-100)")
                .namespace("loupe")
                .subsystem("cache"),
        )?;

        // Register all metrics
        registry.register(Box::new(http_requests_total.clone()))?;
        registry.register(Box::new(http_request_duration_seconds.clone()))?;
        registry.register(Box::new(http_requests_in_flight.clone()))?;
        registry.register(Box::new(db_pool_connections_active.clone()))?;
        registry.register(Box::new(db_pool_connections_idle.clone()))?;
        registry.register(Box::new(db_pool_connections_max.clone()))?;
        registry.register(Box::new(db_pool_acquire_duration_seconds.clone()))?;
        registry.register(Box::new(db_pool_acquire_timeout_total.clone()))?;
        registry.register(Box::new(query_executions_total.clone()))?;
        registry.register(Box::new(query_execution_duration_seconds.clone()))?;
        registry.register(Box::new(query_rows_returned.clone()))?;
        registry.register(Box::new(query_timeouts_total.clone()))?;
        registry.register(Box::new(queries_in_flight.clone()))?;
        registry.register(Box::new(jobs_claimed_total.clone()))?;
        registry.register(Box::new(job_processing_duration_seconds.clone()))?;
        registry.register(Box::new(job_queue_depth.clone()))?;
        registry.register(Box::new(job_retry_queue_depth.clone()))?;
        registry.register(Box::new(job_dead_letter_queue_size.clone()))?;
        registry.register(Box::new(cache_requests_total.clone()))?;
        registry.register(Box::new(cache_hit_rate.clone()))?;

        Ok(Self {
            registry: Arc::new(registry),
            http_requests_total,
            http_request_duration_seconds,
            http_requests_in_flight,
            db_pool_connections_active,
            db_pool_connections_idle,
            db_pool_connections_max,
            db_pool_acquire_duration_seconds,
            db_pool_acquire_timeout_total,
            query_executions_total,
            query_execution_duration_seconds,
            query_rows_returned,
            query_timeouts_total,
            queries_in_flight,
            jobs_claimed_total,
            job_processing_duration_seconds,
            job_queue_depth,
            job_retry_queue_depth,
            job_dead_letter_queue_size,
            cache_requests_total,
            cache_hit_rate,
        })
    }

    /// Render metrics in Prometheus text format
    pub fn render(&self) -> Result<String, prometheus::Error> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        String::from_utf8(buffer).map_err(|e| {
            prometheus::Error::Msg(format!("Failed to encode metrics as UTF-8: {}", e))
        })
    }

    /// Update database connection pool metrics
    ///
    /// Should be called periodically to update the pool statistics.
    /// Can be called from middleware or a background task.
    pub fn update_pool_metrics(&self, stats: &crate::db::PoolStats) {
        self.db_pool_connections_active.set(stats.connections_active as i64);
        self.db_pool_connections_idle.set(stats.connections_idle as i64);
        self.db_pool_connections_max.set(stats.connections_max as i64);
    }

    /// Normalize endpoint path for metrics (remove IDs and params)
    /// Examples:
    /// - /api/v1/dashboards/123 -> /api/v1/dashboards/:id
    /// - /api/v1/queries?page=1 -> /api/v1/queries
    pub fn normalize_path(path: &str) -> String {
        // Remove query parameters
        let path = path.split('?').next().unwrap_or(path);

        // Split into segments
        let segments: Vec<&str> = path.split('/').collect();

        // Rebuild path, replacing IDs (UUIDs or numeric) with :id
        let normalized: Vec<String> = segments
            .iter()
            .map(|segment| {
                // Skip empty segments
                if segment.is_empty() {
                    return segment.to_string();
                }

                // Check if segment looks like a UUID (contains hyphens and is 36 chars)
                // or if it's all numeric (like "123", "456")
                if (segment.len() == 36 && segment.contains('-'))
                    || segment.chars().all(|c| c.is_ascii_digit())
                {
                    ":id".to_string()
                } else {
                    segment.to_string()
                }
            })
            .collect();

        normalized.join("/")
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics registry")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new().unwrap();
        // Record a metric to ensure registry is working
        metrics
            .http_requests_total
            .with_label_values(&["GET", "/test", "200"])
            .inc();
        assert!(!metrics.registry.gather().is_empty());
    }

    #[test]
    fn test_path_normalization() {
        assert_eq!(
            Metrics::normalize_path("/api/v1/dashboards/550e8400-e29b-41d4-a716-446655440000"),
            "/api/v1/dashboards/:id"
        );

        assert_eq!(
            Metrics::normalize_path("/api/v1/queries?page=1&limit=10"),
            "/api/v1/queries"
        );

        assert_eq!(
            Metrics::normalize_path("/api/v1/health"),
            "/api/v1/health"
        );

        assert_eq!(
            Metrics::normalize_path("/api/v1/dashboards/123/tiles/456"),
            "/api/v1/dashboards/:id/tiles/:id"
        );
    }

    #[test]
    fn test_metrics_render() {
        let metrics = Metrics::new().unwrap();

        // Record some test metrics
        metrics
            .http_requests_total
            .with_label_values(&["GET", "/api/v1/health", "200"])
            .inc();

        let output = metrics.render().unwrap();
        assert!(output.contains("loupe_api_http_requests_total"));
        assert!(output.contains("method=\"GET\""));
    }

    #[test]
    fn test_request_duration_buckets() {
        let metrics = Metrics::new().unwrap();

        // Observe some durations
        metrics
            .http_request_duration_seconds
            .with_label_values(&["GET", "/api/v1/health"])
            .observe(0.05); // 50ms

        let output = metrics.render().unwrap();
        assert!(output.contains("loupe_api_http_request_duration_seconds"));
    }
}
