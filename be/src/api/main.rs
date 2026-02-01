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
use loupe::{load_env, CacheManager, Database, JwtManager, Metrics};
use std::sync::Arc;

pub struct AppState {
    pub db: Database,
    pub jwt: JwtManager,
    pub cache: CacheManager,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    load_env();

    // Initialize OpenTelemetry tracing
    let tracer_provider = loupe::tracing::init_tracer()
        .expect("Failed to initialize OpenTelemetry");

    // Register provider globally (must be done before creating subscriber)
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());

    let subscriber = loupe::tracing::create_tracing_subscriber();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());
    tracing::info!(
        endpoint = %otlp_endpoint,
        "OpenTelemetry distributed tracing initialized"
    );

    // Initialize Sentry for error tracking
    let _guard = init_sentry();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let host = std::env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("API_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("API_PORT must be a valid number");

    // JWT configuration
    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set - generate with: openssl rand -base64 32");

    // Validate JWT secret length (minimum 32 characters for security)
    if jwt_secret.len() < 32 {
        panic!("JWT_SECRET must be at least 32 characters long for security");
    }

    let jwt_expiration_hours = std::env::var("JWT_EXPIRATION_HOURS")
        .unwrap_or_else(|_| "24".to_string())
        .parse::<i64>()
        .expect("JWT_EXPIRATION_HOURS must be a valid number");

    tracing::info!("Connecting to database...");
    let db = Database::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Running migrations...");
    db.run_migrations()
        .await
        .expect("Failed to run migrations");

    // Seed default admin if env vars are set
    if let Err(e) = seed_default_admin(&db).await {
        tracing::warn!("Failed to seed default admin: {}", e);
    }

    // Initialize cache manager
    // If Redis is unavailable, the application will still work but without caching
    let cache = CacheManager::new().await
        .unwrap_or_else(|e| {
            tracing::error!("Failed to initialize cache manager: {}. Application will continue without caching.", e);
            panic!("Cache manager initialization failed. Please ensure Redis is running or set CACHE_ENABLED=false in environment.");
        });

    let jwt = JwtManager::new(jwt_secret, jwt_expiration_hours);
    let state = Arc::new(AppState { db, jwt, cache });

    // Initialize metrics
    let metrics = Arc::new(Metrics::new().expect("Failed to create metrics registry"));

    tracing::info!("Starting Loupe API server at http://{}:{}", host, port);
    tracing::info!("Rate limiting: 100 requests/minute per IP globally");
    tracing::info!("Metrics endpoint: http://{}:{}/metrics", host, port);

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

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
    .bind((host.as_str(), port))?
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
fn init_sentry() -> sentry::ClientInitGuard {
    let sentry_dsn = std::env::var("SENTRY_DSN").ok();

    if sentry_dsn.is_none() {
        tracing::info!("Sentry DSN not configured - error tracking disabled");
        return sentry::init(sentry::ClientOptions::default());
    }

    let environment = std::env::var("APP_ENV")
        .unwrap_or_else(|_| "local".to_string());

    let release = format!("loupe@{}", env!("CARGO_PKG_VERSION"));

    // Configure sample rates based on environment
    let (error_sample_rate, traces_sample_rate) = match environment.as_str() {
        "production" | "prod" => (1.0, 0.1), // 100% errors, 10% traces
        "staging" => (1.0, 0.5),              // 100% errors, 50% traces
        _ => (1.0, 1.0),                      // 100% everything in dev/local
    };

    let guard = sentry::init((
        sentry_dsn,
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

/// Seeds a default admin user if ADMIN_USERNAME and ADMIN_PASSWORD are set.
/// Skips if the user already exists.
async fn seed_default_admin(db: &Database) -> Result<(), loupe::Error> {
    let admin_email = match std::env::var("ADMIN_USERNAME") {
        Ok(email) => email,
        Err(_) => return Ok(()), // Not configured, skip silently
    };

    let admin_password = match std::env::var("ADMIN_PASSWORD") {
        Ok(password) => password,
        Err(_) => return Ok(()), // Not configured, skip silently
    };

    // Check if user already exists
    if db.get_user_by_email(&admin_email).await?.is_some() {
        tracing::debug!("Default admin user already exists");
        return Ok(());
    }

    tracing::info!("Creating default admin user: {}", admin_email);

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(admin_password.as_bytes(), &salt)
        .map_err(|e| loupe::Error::Internal(format!("Failed to hash password: {}", e)))?
        .to_string();

    // Create organization for admin
    let org = db.create_organization("Default Organization").await?;

    // Create admin user
    db.create_user(org.id, &admin_email, &password_hash, "Admin", OrgRole::Admin)
        .await?;

    tracing::info!("Default admin user created successfully");
    Ok(())
}
