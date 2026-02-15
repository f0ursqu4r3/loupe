use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, fmt};

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    /// Database configuration
    pub database: DatabaseConfig,

    /// API server configuration
    pub api: ApiConfig,

    /// JWT authentication configuration
    pub jwt: JwtConfig,

    /// Cache configuration (Redis)
    pub cache: CacheConfig,

    /// Observability configuration
    pub observability: ObservabilityConfig,

    /// Admin user seeding configuration
    pub admin: Option<AdminConfig>,

    /// Application environment (local, dev, staging, prod)
    pub app_env: String,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub cors_allowed_origins: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: u64,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub enabled: bool,
    pub redis_url: String,
    pub default_ttl_secs: u64,
}

#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    pub log_level: String,
    pub log_format: LogFormat,
    pub otel_endpoint: Option<String>,
    pub sentry_dsn: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AdminConfig {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogFormat {
    Json,
    Text,
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// # Panics
    ///
    /// Panics if required configuration is missing or invalid
    pub fn from_env() -> Self {
        Self {
            database: DatabaseConfig::from_env(),
            api: ApiConfig::from_env(),
            jwt: JwtConfig::from_env(),
            cache: CacheConfig::from_env(),
            observability: ObservabilityConfig::from_env(),
            admin: AdminConfig::from_env_optional(),
            app_env: std::env::var("APP_ENV").unwrap_or_else(|_| "local".to_string()),
        }
    }

    /// Validate the configuration
    ///
    /// Returns Err with descriptive messages if configuration is invalid
    pub fn validate(&self) -> Result<(), String> {
        // Validate database URL format
        if !self.database.url.contains("://") {
            return Err(format!(
                "DATABASE_URL must be a valid connection string (e.g., postgres://user:pass@host:port/db), got: {}",
                self.database.url
            ));
        }

        // Validate port range
        if self.api.port == 0 {
            return Err("API_PORT must be greater than 0".to_string());
        }

        // Validate JWT secret length (should be at least 32 chars for security)
        if self.jwt.secret.len() < 32 {
            return Err(format!(
                "JWT_SECRET must be at least 32 characters long for security. Current length: {}. Generate with: openssl rand -base64 32",
                self.jwt.secret.len()
            ));
        }

        // Validate JWT expiration
        if self.jwt.expiration_hours == 0 || self.jwt.expiration_hours > 720 {
            return Err(format!(
                "JWT_EXPIRATION_HOURS must be between 1 and 720 (30 days), got: {}",
                self.jwt.expiration_hours
            ));
        }

        // Validate cache TTL
        if self.cache.enabled && self.cache.default_ttl_secs == 0 {
            return Err("CACHE_DEFAULT_TTL must be greater than 0 when caching is enabled".to_string());
        }

        Ok(())
    }
}

impl DatabaseConfig {
    fn from_env() -> Self {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        Self { url }
    }
}

impl ApiConfig {
    fn from_env() -> Self {
        let host = std::env::var("API_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let port = std::env::var("API_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .expect("API_PORT must be a valid port number (0-65535)");

        let cors_allowed_origins = std::env::var("CORS_ALLOWED_ORIGINS")
            .ok()
            .map(|origins| {
                origins
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            });

        Self {
            host,
            port,
            cors_allowed_origins,
        }
    }
}

impl JwtConfig {
    fn from_env() -> Self {
        let secret = std::env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set - generate with: openssl rand -base64 32");

        let expiration_hours = std::env::var("JWT_EXPIRATION_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse::<u64>()
            .expect("JWT_EXPIRATION_HOURS must be a valid number");

        Self {
            secret,
            expiration_hours,
        }
    }
}

impl CacheConfig {
    fn from_env() -> Self {
        let enabled = std::env::var("CACHE_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let default_ttl_secs = std::env::var("CACHE_DEFAULT_TTL")
            .unwrap_or_else(|_| "300".to_string())
            .parse::<u64>()
            .unwrap_or(300);

        Self {
            enabled,
            redis_url,
            default_ttl_secs,
        }
    }
}

impl ObservabilityConfig {
    pub fn from_env() -> Self {
        let log_level = std::env::var("RUST_LOG")
            .unwrap_or_else(|_| "info,sqlx=warn".to_string());

        let log_format = match std::env::var("LOG_FORMAT")
            .unwrap_or_else(|_| "text".to_string())
            .to_lowercase()
            .as_str()
        {
            "json" => LogFormat::Json,
            _ => LogFormat::Text,
        };

        let otel_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();
        let sentry_dsn = std::env::var("SENTRY_DSN").ok();

        Self {
            log_level,
            log_format,
            otel_endpoint,
            sentry_dsn,
        }
    }
}

impl AdminConfig {
    fn from_env_optional() -> Option<Self> {
        let email = std::env::var("ADMIN_USERNAME").ok()?;
        let password = std::env::var("ADMIN_PASSWORD").ok()?;

        // Only return Some if both are set
        if email.is_empty() || password.is_empty() {
            return None;
        }

        Some(Self { email, password })
    }
}

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

/// Initialize tracing with structured logging based on configuration
pub fn init_tracing(config: &ObservabilityConfig) {
    let env_filter = tracing_subscriber::EnvFilter::new(&config.log_level);

    match config.log_format {
        LogFormat::Json => {
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
        LogFormat::Text => {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_format_parsing() {
        // Test case insensitivity
        assert_eq!(
            match "JSON".to_lowercase().as_str() {
                "json" => LogFormat::Json,
                _ => LogFormat::Text,
            },
            LogFormat::Json
        );

        assert_eq!(
            match "text".to_lowercase().as_str() {
                "json" => LogFormat::Json,
                _ => LogFormat::Text,
            },
            LogFormat::Text
        );
    }
}
