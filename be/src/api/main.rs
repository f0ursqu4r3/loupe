mod routes;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use loupe::models::OrgRole;
use loupe::{init_tracing, load_env, Database};
use std::sync::Arc;

pub struct AppState {
    pub db: Database,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    load_env();
    init_tracing();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let host = std::env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("API_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("API_PORT must be a valid number");

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

    let state = Arc::new(AppState { db });

    tracing::info!("Starting Loupe API server at http://{}:{}", host, port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(state.clone()))
            .configure(routes::configure)
    })
    .bind((host.as_str(), port))?
    .run()
    .await
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
