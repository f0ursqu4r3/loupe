use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry, TextEncoder,
};
use std::sync::Arc;

/// Application metrics registry
#[derive(Clone)]
pub struct Metrics {
    pub registry: Arc<Registry>,
    pub http_requests_total: IntCounterVec,
    pub http_request_duration_seconds: HistogramVec,
    pub http_requests_in_flight: IntCounterVec,
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

        // Register all metrics
        registry.register(Box::new(http_requests_total.clone()))?;
        registry.register(Box::new(http_request_duration_seconds.clone()))?;
        registry.register(Box::new(http_requests_in_flight.clone()))?;

        Ok(Self {
            registry: Arc::new(registry),
            http_requests_total,
            http_request_duration_seconds,
            http_requests_in_flight,
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
