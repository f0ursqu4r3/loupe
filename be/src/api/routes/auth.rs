use crate::AppState;
use actix_web::{web, HttpResponse};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use loupe::models::{CreateUserRequest, LoginRequest, OrgRole, UserResponse};
use loupe::Error;
use std::sync::Arc;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login)),
    );
}

async fn register(
    state: web::Data<Arc<AppState>>,
    req: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, Error> {
    // Check if user already exists
    if state.db.get_user_by_email(&req.email).await?.is_some() {
        return Err(Error::Conflict("Email already registered".to_string()));
    }

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)?
        .to_string();

    // Create organization for new user (simple model for v1)
    let org = state.db.create_organization(&format!("{}'s Org", req.name)).await?;

    // Create user
    let user = state
        .db
        .create_user(org.id, &req.email, &password_hash, &req.name, OrgRole::Admin)
        .await?;

    Ok(HttpResponse::Created().json(UserResponse::from(user)))
}

async fn login(
    state: web::Data<Arc<AppState>>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, Error> {
    let user = state
        .db
        .get_user_by_email(&req.email)
        .await?
        .ok_or_else(|| Error::Unauthorized("Invalid credentials".to_string()))?;

    // Verify password
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| Error::Internal("Invalid password hash".to_string()))?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| Error::Unauthorized("Invalid credentials".to_string()))?;

    // For v1, we'll return user info. In production, you'd return a JWT token.
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user": UserResponse::from(user),
        "token": "placeholder_token_for_v1"
    })))
}
