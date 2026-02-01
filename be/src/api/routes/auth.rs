use crate::AppState;
use actix_web::{HttpRequest, HttpResponse, web};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use loupe::Error;
use loupe::models::{CreateUserRequest, LoginRequest, OrgRole, UserResponse};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/refresh", web::post().to(refresh_token))
            .route("/me", web::get().to(me)),
    );
}

/// Extract token from Authorization header
pub fn extract_token(req: &HttpRequest) -> Result<String, Error> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| Error::Unauthorized("Missing authorization header".to_string()))?
        .to_str()
        .map_err(|_| Error::Unauthorized("Invalid authorization header".to_string()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(Error::Unauthorized(
            "Invalid authorization format. Use: Bearer <token>".to_string(),
        ));
    }

    Ok(auth_header[7..].to_string())
}

/// Get current user and org from request using JWT
pub fn get_auth_context(state: &AppState, req: &HttpRequest) -> Result<(Uuid, Uuid), Error> {
    let token = extract_token(req)?;
    let claims = state.jwt.validate_token(&token)?;
    let user_id = claims.user_id()?;
    let org_id = claims.org_id()?;
    Ok((user_id, org_id))
}

async fn register(
    state: web::Data<Arc<AppState>>,
    req: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, Error> {
    // Validate request using validator crate
    req.validate()
        .map_err(|e| Error::BadRequest(format!("Validation failed: {}", e)))?;

    // Check if user already exists
    if state.db.get_user_by_email(&req.email).await?.is_some() {
        return Err(Error::Conflict("Email already registered".to_string()));
    }

    // Hash password using Argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)?
        .to_string();

    // Create organization for new user (simple model for v1)
    let org = state
        .db
        .create_organization(&format!("{}'s Org", req.name))
        .await?;

    // Create user
    let user = state
        .db
        .create_user(
            org.id,
            &req.email,
            &password_hash,
            &req.name,
            OrgRole::Admin,
        )
        .await?;

    // Generate JWT token
    let token = state.jwt.create_token(user.id, user.org_id)?;
    let refresh_token = state.jwt.create_refresh_token(user.id, user.org_id)?;

    tracing::info!(
        user_id = %user.id,
        email = %user.email,
        "User registered successfully"
    );

    Ok(HttpResponse::Created().json(serde_json::json!({
        "user": UserResponse::from(user),
        "token": token,
        "refresh_token": refresh_token,
    })))
}

async fn login(
    state: web::Data<Arc<AppState>>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, Error> {
    // Validate request
    req.validate()
        .map_err(|e| Error::BadRequest(format!("Validation failed: {}", e)))?;

    // Get user by email
    let user = state
        .db
        .get_user_by_email(&req.email)
        .await?
        .ok_or_else(|| {
            // Use constant-time response to prevent timing attacks
            tracing::warn!(email = %req.email, "Login attempt with non-existent email");
            Error::Unauthorized("Invalid email or password".to_string())
        })?;

    // Verify password using Argon2
    let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|e| {
        tracing::error!(user_id = %user.id, error = %e, "Invalid password hash in database");
        Error::Internal("Authentication error".to_string())
    })?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| {
            tracing::warn!(
                user_id = %user.id,
                email = %user.email,
                "Failed login attempt - invalid password"
            );
            Error::Unauthorized("Invalid email or password".to_string())
        })?;

    // Generate JWT tokens
    let token = state.jwt.create_token(user.id, user.org_id)?;
    let refresh_token = state.jwt.create_refresh_token(user.id, user.org_id)?;

    tracing::info!(
        user_id = %user.id,
        email = %user.email,
        "User logged in successfully"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user": UserResponse::from(user),
        "token": token,
        "refresh_token": refresh_token,
    })))
}

async fn refresh_token(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    // Extract and validate refresh token
    let token = extract_token(&req)?;
    let claims = state.jwt.validate_token(&token)?;

    // Issue new tokens
    let user_id = claims.user_id()?;
    let org_id = claims.org_id()?;

    let new_token = state.jwt.create_token(user_id, org_id)?;
    let new_refresh_token = state.jwt.create_refresh_token(user_id, org_id)?;

    tracing::info!(
        user_id = %user_id,
        "Token refreshed successfully"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "token": new_token,
        "refresh_token": new_refresh_token,
    })))
}

async fn me(state: web::Data<Arc<AppState>>, req: HttpRequest) -> Result<HttpResponse, Error> {
    let (user_id, _org_id) = get_auth_context(&state, &req)?;

    let user = state.db.get_user(user_id).await?;

    Ok(HttpResponse::Ok().json(UserResponse::from(user)))
}
