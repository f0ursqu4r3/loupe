use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

/// Initialize tracing with the standard configuration.
///
/// Uses the RUST_LOG environment variable for filtering, defaulting to
/// "info,sqlx=warn" if not set.
pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,sqlx=warn".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
