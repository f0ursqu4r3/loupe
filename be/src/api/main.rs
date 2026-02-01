mod app_middleware;
mod permissions;
mod routes;

use actix_cors::Cors;
use actix_governor::GovernorConfigBuilder;
use actix_web::{web, App, HttpServer};
use app_middleware::{CorrelationIdMiddleware, MetricsMiddleware, RequestLogger, SecurityHeaders, SentryContextMiddleware, TracingMiddleware};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use loupe::models::OrgRole;
use loupe::{load_env, init_tracing, CacheManager, Config, Database, JwtManager, Metrics};
use std::sync::Arc;

pub struct AppState {
    pub db: Database,
    pub jwt: JwtManager,
    pub cache: CacheManager,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    load_env();

    // Load and validate configuration
    let config = Config::from_env();
    if let Err(e) = config.validate() {
        eprintln!("Configuration error: {}", e);
        std::process::exit(1);
    }

    // Initialize logging
    init_tracing(&config.observability);

    // Initialize OpenTelemetry tracing
    let tracer_provider = loupe::tracing::init_tracer()
        .expect("Failed to initialize OpenTelemetry");

    // Register provider globally (must be done before creating subscriber)
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());

    if let Some(ref endpoint) = config.observability.otel_endpoint {
        tracing::info!(
            endpoint = %endpoint,
            "OpenTelemetry distributed tracing initialized"
        );
    }

    // Initialize Sentry for error tracking
    let _guard = init_sentry(&config);

    tracing::info!("Connecting to database...");
    let db = Database::connect(&config.database.url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Running migrations...");
    db.run_migrations()
        .await
        .expect("Failed to run migrations");

    // Seed default admin if configured
    if let Some(ref admin_config) = config.admin {
        if let Err(e) = seed_default_admin(&db, admin_config).await {
            tracing::warn!("Failed to seed default admin: {}", e);
        }
    } else {
        tracing::debug!("Admin user seeding not configured (ADMIN_USERNAME/ADMIN_PASSWORD not set)");
    }

    // Initialize cache manager
    let cache = CacheManager::new().await
        .unwrap_or_else(|e| {
            if config.cache.enabled {
                tracing::error!("Failed to initialize cache manager: {}. Application will continue without caching.", e);
                panic!("Cache manager initialization failed. Please ensure Redis is running or set CACHE_ENABLED=false in environment.");
            } else {
                panic!("Cache initialization failed unexpectedly even though caching is disabled: {}", e);
            }
        });

    let jwt = JwtManager::new(config.jwt.secret.clone(), config.jwt.expiration_hours as i64);
    let state = Arc::new(AppState { db, jwt, cache });

    // Initialize metrics
    let metrics = Arc::new(Metrics::new().expect("Failed to create metrics registry"));

    tracing::info!("Starting Loupe API server at http://{}:{}", config.api.host, config.api.port);
    tracing::info!("Rate limiting: 100 requests/minute per IP globally");
    tracing::info!("Metrics endpoint: http://{}:{}/metrics", config.api.host, config.api.port);

    let server = HttpServer::new(move || {
        // CORS Configuration
        //
        // Security: CORS (Cross-Origin Resource Sharing) controls which frontend origins
        // can access this API from browsers.
        //
        // **Production Mode** (CORS_ALLOWED_ORIGINS set):
        //   - Only allows requests from explicitly whitelisted origins
        //   - Origins must be comma-separated: "https://app.example.com,https://admin.example.com"
        //   - Restricts methods to: GET, POST, PUT, DELETE, PATCH, OPTIONS
        //   - Restricts headers to: Authorization, Accept, Content-Type
        //   - Prevents unauthorized frontends from accessing the API
        //
        // **Development Mode** (CORS_ALLOWED_ORIGINS not set):
        //   - Allows any origin (for local development convenience)
        //   - Should NEVER be used in production
        //   - Logs a warning when this mode is active
        //
        // Example production configuration:
        //   CORS_ALLOWED_ORIGINS="https://loupe.example.com,https://loupe-staging.example.com"
        let cors = if let Some(ref allowed_origins) = config.api.cors_allowed_origins {
            // Production mode: strict origin validation
            tracing::info!("CORS: Allowing specific origins: {:?}", allowed_origins);

            let mut cors = Cors::default();
            for origin in allowed_origins {
                cors = cors.allowed_origin(origin.as_str());
            }

            cors.allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"])
                .allowed_headers(vec![
                    actix_web::http::header::AUTHORIZATION,
                    actix_web::http::header::ACCEPT,
                    actix_web::http::header::CONTENT_TYPE,
                ])
                .max_age(3600)
        } else {
            // Development mode: permissive for local testing
            tracing::warn!("CORS: CORS_ALLOWED_ORIGINS not set - allowing all origins (development mode only!)");

            Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .max_age(3600)
        };

        // Global API rate limiter: 100 requests per minute per IP
        // This prevents API abuse and brute force attacks
        let governor_conf = GovernorConfigBuilder::default()
            .requests_per_second(2)  // ~120 per minute
            .burst_size(20)          // Allow bursts of 20 requests
            .finish()
            .unwrap();

        App::new()
            .wrap(cors)
            .wrap(sentry_actix::Sentry::new())  // Error tracking with Sentry
            .wrap(SecurityHeaders)  // Add security headers
            .wrap(CorrelationIdMiddleware)  // Generate/extract correlation ID
            .wrap(TracingMiddleware)  // OpenTelemetry distributed tracing
            .wrap(SentryContextMiddleware)  // Enrich Sentry events with context
            .wrap(RequestLogger)  // Structured request logging with correlation IDs
            .wrap(MetricsMiddleware::new(metrics.clone()))  // Collect Prometheus metrics
            .wrap(actix_governor::Governor::new(&governor_conf))  // Apply rate limiting
            .app_data(web::Data::new(state.clone()))
            .app_data(web::Data::new(metrics.clone()))  // Make metrics available to routes
            .configure(routes::configure)
    })
    .bind((config.api.host.as_str(), config.api.port))?
    .run();

    // Run the server and handle shutdown
    let result = server.await;

    // Gracefully shutdown OpenTelemetry tracer provider
    tracing::info!("Shutting down OpenTelemetry tracer provider");
    loupe::tracing::shutdown_tracer_provider(tracer_provider);

    result
}

/// Initialize Sentry for error tracking
///
/// Configures Sentry with:
/// - Environment (dev/staging/prod from APP_ENV)
/// - Release version (from Cargo.toml)
/// - Sample rate for errors and traces
/// - Integration with tracing for breadcrumbs
///
/// Requires SENTRY_DSN environment variable to be set.
/// If not set, Sentry will be disabled (no-op).
fn init_sentry(config: &Config) -> sentry::ClientInitGuard {
    let sentry_dsn = config.observability.sentry_dsn.as_ref();

    if sentry_dsn.is_none() {
        tracing::info!("Sentry DSN not configured - error tracking disabled");
        return sentry::init(sentry::ClientOptions::default());
    }

    let environment = config.app_env.clone();

    let release = format!("loupe@{}", env!("CARGO_PKG_VERSION"));

    // Configure sample rates based on environment
    let (error_sample_rate, traces_sample_rate) = match environment.as_str() {
        "production" | "prod" => (1.0, 0.1), // 100% errors, 10% traces
        "staging" => (1.0, 0.5),              // 100% errors, 50% traces
        _ => (1.0, 1.0),                      // 100% everything in dev/local
    };

    let guard = sentry::init((
        sentry_dsn.cloned(),
        sentry::ClientOptions {
            release: Some(release.into()),
            environment: Some(environment.clone().into()),
            sample_rate: error_sample_rate,
            traces_sample_rate,
            attach_stacktrace: true,
            send_default_pii: false, // Don't send PII by default
            before_send: Some(Arc::new(|event| {
                // Correlation ID and user context will be added by middleware
                // We could add additional filtering here if needed
                Some(event)
            })),
            ..Default::default()
        },
    ));

    tracing::info!(
        environment = %environment,
        release = %format!("loupe@{}", env!("CARGO_PKG_VERSION")),
        "Sentry error tracking initialized"
    );

    guard
}

/// Seeds a default admin user from configuration.
/// Skips if the user already exists.
async fn seed_default_admin(db: &Database, admin_config: &loupe::AdminConfig) -> Result<(), loupe::Error> {
    // Check if user already exists
    if db.get_user_by_email(&admin_config.email).await?.is_some() {
        tracing::debug!("Default admin user already exists");
        return Ok(());
    }

    tracing::info!("Creating default admin user: {}", admin_config.email);

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(admin_config.password.as_bytes(), &salt)
        .map_err(|e| loupe::Error::Internal(format!("Failed to hash password: {}", e)))?
        .to_string();

    // Create organization for admin
    let org = db.create_organization("Default Organization").await?;

    // Create admin user
    db.create_user(org.id, &admin_config.email, &password_hash, "Admin", OrgRole::Admin)
        .await?;

    tracing::info!("Default admin user created successfully");
    Ok(())
}
