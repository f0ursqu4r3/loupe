use opentelemetry::{
    trace::TraceError,
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    runtime,
    trace::{RandomIdGenerator, Sampler, TracerProvider},
    Resource,
};
use tracing::Subscriber;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

/// Initialize OpenTelemetry tracer provider with OTLP exporter
///
/// # Configuration (via environment variables)
///
/// - `OTEL_EXPORTER_OTLP_ENDPOINT`: OTLP endpoint (default: http://localhost:4317)
/// - `APP_ENV`: Environment name (local, staging, production)
/// - `OTEL_SERVICE_NAME`: Service name override (default: loupe-api)
///
/// # Examples
///
/// ```bash
/// # Export to Jaeger
/// OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317
///
/// # Export to Honeycomb
/// OTEL_EXPORTER_OTLP_ENDPOINT=https://api.honeycomb.io
/// OTEL_EXPORTER_OTLP_HEADERS=x-honeycomb-team=YOUR_API_KEY
/// ```
pub fn init_tracer() -> Result<TracerProvider, TraceError> {
    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let environment = std::env::var("APP_ENV").unwrap_or_else(|_| "local".to_string());

    let service_name = std::env::var("OTEL_SERVICE_NAME")
        .unwrap_or_else(|_| "loupe-api".to_string());

    // Determine sampling rate based on environment
    let sampler = match environment.as_str() {
        "production" | "prod" => Sampler::TraceIdRatioBased(0.1), // 10% in production
        "staging" => Sampler::TraceIdRatioBased(0.5),             // 50% in staging
        _ => Sampler::AlwaysOn,                                   // 100% in dev/local
    };

    // Create resource with service metadata
    let resource = Resource::new(vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            service_name.clone(),
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_VERSION,
            env!("CARGO_PKG_VERSION"),
        ),
        KeyValue::new(
            "deployment.environment",
            environment.clone(),
        ),
    ]);

    // Configure OTLP exporter
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint)
        .build()?;

    // Create tracer provider
    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_sampler(sampler)
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource)
        .build();

    Ok(provider)
}

/// Create a tracing subscriber with OpenTelemetry layer
///
/// This combines:
/// - OpenTelemetry layer for distributed tracing
/// - JSON formatting for structured logs
/// - Environment-based filtering
///
/// Note: Must call opentelemetry::global::set_tracer_provider() before this
pub fn create_tracing_subscriber() -> impl Subscriber + Send + Sync {
    // OpenTelemetry layer - uses global tracer provider
    let telemetry_layer = tracing_opentelemetry::layer();

    // JSON formatting layer
    let formatting_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true);

    // Environment filter (e.g., RUST_LOG=info,sqlx=warn)
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info,sqlx=warn"))
        .unwrap();

    Registry::default()
        .with(filter_layer)
        .with(telemetry_layer)
        .with(formatting_layer)
}

/// Gracefully shutdown tracing, ensuring all spans are exported
pub fn shutdown_tracer_provider(provider: TracerProvider) {
    if let Err(e) = provider.shutdown() {
        eprintln!("Error shutting down tracer provider: {:?}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracer_initialization() {
        // Should not panic even if OTLP endpoint is unreachable
        let result = init_tracer();
        assert!(result.is_ok());
    }

    #[test]
    fn test_subscriber_creation() {
        let provider = init_tracer().unwrap();
        opentelemetry::global::set_tracer_provider(provider.clone());
        let _subscriber = create_tracing_subscriber();
        // If we get here, subscriber was created successfully
        shutdown_tracer_provider(provider);
    }
}
