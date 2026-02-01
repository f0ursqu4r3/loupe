use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, fmt};

/// Load environment files based on APP_ENV.
///
/// First loads the base `.env` file, then loads the environment-specific
/// file (e.g., `.env.local`, `.env.dev`, `.env.prod`) based on the APP_ENV
/// environment variable. Defaults to "local" if APP_ENV is not set.
///
/// Later values override earlier ones, so environment-specific settings
/// take precedence over base settings.
pub fn load_env() {
    // Load base .env file first
    dotenvy::dotenv().ok();

    // Then load environment-specific .env file
    let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "local".to_string());
    let env_file = format!(".env.{}", app_env);
    dotenvy::from_filename(&env_file).ok();
}

/// Initialize tracing with structured logging.
///
/// Uses the RUST_LOG environment variable for filtering, defaulting to
/// "info,sqlx=warn" if not set.
///
/// Supports two output formats via LOG_FORMAT environment variable:
/// - "json": Structured JSON logging (recommended for production)
/// - "text": Human-readable text format (default for development)
pub fn init_tracing() {
    let env_filter = tracing_subscriber::EnvFilter::new(
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info,sqlx=warn".to_string()),
    );

    let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "text".to_string());

    match log_format.as_str() {
        "json" => {
            // JSON format for structured logging (production)
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer()
                    .json()
                    .with_current_span(true)
                    .with_span_list(true)
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_thread_names(false)
                    .with_file(true)
                    .with_line_number(true))
                .init();
        }
        _ => {
            // Text format for human-readable logs (development)
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer()
                    .with_target(true)
                    .with_thread_ids(false)
                    .with_file(true)
                    .with_line_number(true))
                .init();
        }
    }
}
