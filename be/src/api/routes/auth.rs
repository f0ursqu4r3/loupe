use crate::AppState;
use actix_web::{HttpRequest, HttpResponse, web};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use loupe::Error;
use loupe::models::{CreateUserRequest, LoginRequest, OrgRole, UserResponse};
use std::sync::Arc;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/me", web::get().to(me)),
    );
}

/// Simple token format for v1: base64(user_id:org_id)
/// In production, use JWT with proper signing
fn create_token(user_id: Uuid, org_id: Uuid) -> String {
    let payload = format!("{}:{}", user_id, org_id);
    URL_SAFE_NO_PAD.encode(payload.as_bytes())
}

/// Extract user_id and org_id from token
pub fn parse_token(token: &str) -> Result<(Uuid, Uuid), Error> {
    let decoded = URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|_| Error::Unauthorized("Invalid token".to_string()))?;
    let payload =
        String::from_utf8(decoded).map_err(|_| Error::Unauthorized("Invalid token".to_string()))?;
    let parts: Vec<&str> = payload.split(':').collect();
    if parts.len() != 2 {
        return Err(Error::Unauthorized("Invalid token".to_string()));
    }
    let user_id =
        Uuid::parse_str(parts[0]).map_err(|_| Error::Unauthorized("Invalid token".to_string()))?;
    let org_id =
        Uuid::parse_str(parts[1]).map_err(|_| Error::Unauthorized("Invalid token".to_string()))?;
    Ok((user_id, org_id))
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
            "Invalid authorization format".to_string(),
        ));
    }

    Ok(auth_header[7..].to_string())
}

/// Get current user from request
pub fn get_auth_context(req: &HttpRequest) -> Result<(Uuid, Uuid), Error> {
    let token = extract_token(req)?;
    parse_token(&token)
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

    let token = create_token(user.id, user.org_id);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user": UserResponse::from(user),
        "token": token
    })))
}

async fn me(state: web::Data<Arc<AppState>>, req: HttpRequest) -> Result<HttpResponse, Error> {
    let (user_id, _org_id) = get_auth_context(&req)?;

    let user = state.db.get_user(user_id).await?;

    Ok(HttpResponse::Ok().json(UserResponse::from(user)))
}
