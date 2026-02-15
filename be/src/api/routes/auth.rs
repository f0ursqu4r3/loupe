use crate::AppState;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{HttpRequest, HttpResponse, web};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::Utc;
use loupe::Error;
use loupe::models::{AuthResponse, CreateUserRequest, LoginRequest, OrgRole, RefreshTokenResponse, UserResponse};
use loupe::validation::validate_request;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use uuid::Uuid;

/// Pre-computed Argon2 hash used to burn CPU on login attempts for non-existent
/// accounts so the response time is indistinguishable from a real password check.
static DUMMY_PASSWORD_HASH: LazyLock<String> = LazyLock::new(|| {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(b"dummy-password-for-timing-equalization", &salt)
        .expect("Failed to generate dummy password hash")
        .to_string()
});

const FAILED_LOGIN_LIMIT: u32 = 5;
const FAILED_LOGIN_WINDOW_SECS: u64 = 15 * 60;
const BASE_ACCOUNT_LOCKOUT_SECS: u64 = 15 * 60;
const MAX_ACCOUNT_LOCKOUT_SECS: u64 = 24 * 60 * 60;
const LOCKOUT_ESCALATION_WINDOW_SECS: u64 = 24 * 60 * 60;
const REGISTER_RATE_LIMIT_PER_HOUR: u64 = 3;
const LOGIN_RATE_LIMIT_BURST: u32 = 5;
const LOGIN_SECONDS_PER_REQUEST: u64 = 15 * 60 / LOGIN_RATE_LIMIT_BURST as u64;

pub fn configure(cfg: &mut web::ServiceConfig) {
    // Endpoint-specific rate limits for authentication endpoints
    let register_rate_conf = GovernorConfigBuilder::default()
        .requests_per_hour(REGISTER_RATE_LIMIT_PER_HOUR)
        .burst_size(REGISTER_RATE_LIMIT_PER_HOUR as u32)
        .finish()
        .expect("valid register rate limit configuration");

    let login_rate_conf = GovernorConfigBuilder::default()
        .seconds_per_request(LOGIN_SECONDS_PER_REQUEST)
        .burst_size(LOGIN_RATE_LIMIT_BURST)
        .finish()
        .expect("valid login rate limit configuration");

    cfg.service(
        web::scope("/auth")
            .service(
                web::resource("/register")
                    .wrap(Governor::new(&register_rate_conf))
                    .route(web::post().to(register)),
            )
            .service(
                web::resource("/login")
                    .wrap(Governor::new(&login_rate_conf))
                    .route(web::post().to(login)),
            )
            .route("/logout", web::post().to(logout))
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

fn hash_auth_subject(subject: &str) -> String {
    let normalized = subject.trim().to_lowercase();
    let digest = ring::digest::digest(&ring::digest::SHA256, normalized.as_bytes());
    hex::encode(digest)
}

fn revoked_token_key(jti: &str) -> String {
    format!("auth:revoked_token:{jti}")
}

fn failed_login_attempts_key(email: &str) -> String {
    format!("auth:login_attempts:{}", hash_auth_subject(email))
}

fn account_lockout_until_key(email: &str) -> String {
    format!("auth:account_lockout_until:{}", hash_auth_subject(email))
}

fn lockout_level_key(email: &str) -> String {
    format!("auth:account_lockout_level:{}", hash_auth_subject(email))
}

fn lockout_duration_secs(level: u32) -> u64 {
    let shift = level.saturating_sub(1).min(16);
    let multiplier = 1u64.checked_shl(shift).unwrap_or(u64::MAX);
    BASE_ACCOUNT_LOCKOUT_SECS
        .saturating_mul(multiplier)
        .min(MAX_ACCOUNT_LOCKOUT_SECS)
}

async fn is_token_revoked(cache: &loupe::CacheManager, jti: &str) -> Result<bool, Error> {
    cache
        .get::<bool>(&revoked_token_key(jti))
        .await
        .map(|val| val.unwrap_or(false))
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to check token revocation state");
            Error::Internal("Authentication service unavailable".to_string())
        })
}

async fn revoke_token(cache: &loupe::CacheManager, jti: &str, expires_at: i64) -> Result<(), Error> {
    let ttl_secs = (expires_at - Utc::now().timestamp()).max(1) as u64;
    cache
        .set_with_ttl(
            &revoked_token_key(jti),
            &true,
            Duration::from_secs(ttl_secs),
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, jti = %jti, "Failed to revoke token");
            Error::Internal("Failed to complete logout".to_string())
        })
}

async fn is_login_locked(cache: &loupe::CacheManager, email: &str) -> Result<Option<u64>, Error> {
    let now = Utc::now().timestamp();
    cache
        .get::<i64>(&account_lockout_until_key(email))
        .await
        .map(|val| match val {
            Some(until_ts) if until_ts > now => Some((until_ts - now) as u64),
            _ => None,
        })
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to check login lockout state");
            Error::Internal("Authentication service unavailable".to_string())
        })
}

async fn record_failed_login_attempt(cache: &loupe::CacheManager, email: &str) -> Result<(), Error> {
    let attempts_key = failed_login_attempts_key(email);

    // Atomic increment — no read-then-write race.
    let attempts = cache
        .increment(&attempts_key, Duration::from_secs(FAILED_LOGIN_WINDOW_SECS))
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to increment login attempt counter");
            Error::Internal("Authentication service unavailable".to_string())
        })? as u32;

    if attempts >= FAILED_LOGIN_LIMIT {
        let lockout_lk = lockout_level_key(email);

        // Escalation level is low-contention — get-then-set is acceptable here.
        let lockout_level = cache
            .get::<u32>(&lockout_lk)
            .await
            .map(|val| val.unwrap_or(0).saturating_add(1))
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to read lockout escalation level");
                Error::Internal("Authentication service unavailable".to_string())
            })?;

        let lockout_secs = lockout_duration_secs(lockout_level);
        let unlock_at = Utc::now().timestamp().saturating_add(lockout_secs as i64);

        cache
            .set_with_ttl(
                &account_lockout_until_key(email),
                &unlock_at,
                Duration::from_secs(lockout_secs),
            )
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to set account lockout");
                Error::Internal("Authentication service unavailable".to_string())
            })?;

        cache
            .set_with_ttl(
                &lockout_lk,
                &lockout_level,
                Duration::from_secs(LOCKOUT_ESCALATION_WINDOW_SECS),
            )
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to update lockout escalation level");
                Error::Internal("Authentication service unavailable".to_string())
            })?;

        cache.delete(&attempts_key).await.map_err(|e| {
            tracing::error!(error = %e, "Failed to clear login attempt counter");
            Error::Internal("Authentication service unavailable".to_string())
        })?;
    }

    Ok(())
}

async fn clear_failed_login_attempts(cache: &loupe::CacheManager, email: &str) -> Result<(), Error> {
    cache
        .delete(&account_lockout_until_key(email))
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to clear account lockout state");
            Error::Internal("Authentication service unavailable".to_string())
        })?;

    cache
        .delete(&lockout_level_key(email))
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to clear lockout escalation level");
            Error::Internal("Authentication service unavailable".to_string())
        })?;

    cache
        .delete(&failed_login_attempts_key(email))
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to clear login attempt counter");
            Error::Internal("Authentication service unavailable".to_string())
        })
}

async fn validate_token_with_revocation(state: &AppState, token: &str) -> Result<loupe::Claims, Error> {
    let claims = state.jwt.validate_token(token)?;

    if is_token_revoked(&state.cache, &claims.jti).await? {
        tracing::warn!(jti = %claims.jti, "Attempt to use revoked token");
        return Err(Error::Unauthorized(
            "Invalid or expired token".to_string(),
        ));
    }

    Ok(claims)
}

/// Get current user and org from request using JWT
pub async fn get_auth_context(state: &AppState, req: &HttpRequest) -> Result<(Uuid, Uuid), Error> {
    let token = extract_token(req)?;
    let claims = validate_token_with_revocation(state, &token).await?;
    let user_id = claims.user_id()?;
    let org_id = claims.org_id()?;
    Ok((user_id, org_id))
}

async fn register(
    state: web::Data<Arc<AppState>>,
    req: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, Error> {
    validate_request(&*req)?;

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

    Ok(HttpResponse::Created().json(AuthResponse {
        user: UserResponse::from(user),
        token,
        refresh_token,
    }))
}

async fn login(
    state: web::Data<Arc<AppState>>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, Error> {
    validate_request(&*req)?;

    if let Some(retry_after_secs) = is_login_locked(&state.cache, &req.email).await? {
        tracing::warn!(email = %req.email, "Login blocked due to account lockout");
        return Ok(HttpResponse::TooManyRequests()
            .insert_header(("Retry-After", retry_after_secs.to_string()))
            .json(serde_json::json!({
                "error": {
                    "type": "rate_limited",
                    "message": "Too many failed login attempts. Please try again later."
                }
            })));
    }

    // Get user by email
    let user = match state.db.get_user_by_email(&req.email).await? {
        Some(user) => user,
        None => {
            // Run Argon2 verify against a dummy hash so the response time is
            // indistinguishable from a real password check (prevents user enumeration
            // via timing side-channel).
            let dummy = PasswordHash::new(&DUMMY_PASSWORD_HASH)
                .expect("DUMMY_PASSWORD_HASH is a valid Argon2 hash");
            let _ = Argon2::default().verify_password(req.password.as_bytes(), &dummy);

            record_failed_login_attempt(&state.cache, &req.email).await?;
            tracing::warn!(email = %req.email, "Login attempt with non-existent email");
            return Err(Error::Unauthorized("Invalid email or password".to_string()));
        }
    };

    // Verify password using Argon2
    let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|e| {
        tracing::error!(user_id = %user.id, error = %e, "Invalid password hash in database");
        Error::Internal("Authentication error".to_string())
    })?;

    if Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        record_failed_login_attempt(&state.cache, &req.email).await?;
        tracing::warn!(
            user_id = %user.id,
            email = %user.email,
            "Failed login attempt - invalid password"
        );
        return Err(Error::Unauthorized("Invalid email or password".to_string()));
    }

    clear_failed_login_attempts(&state.cache, &req.email).await?;

    // Generate JWT tokens
    let token = state.jwt.create_token(user.id, user.org_id)?;
    let refresh_token = state.jwt.create_refresh_token(user.id, user.org_id)?;

    tracing::info!(
        user_id = %user.id,
        email = %user.email,
        "User logged in successfully"
    );

    Ok(HttpResponse::Ok().json(AuthResponse {
        user: UserResponse::from(user),
        token,
        refresh_token,
    }))
}

async fn logout(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let token = extract_token(&req)?;
    let claims = validate_token_with_revocation(&state, &token).await?;
    revoke_token(&state.cache, &claims.jti, claims.exp).await?;

    tracing::info!(
        user_id = %claims.sub,
        jti = %claims.jti,
        "Token revoked via logout"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}

async fn refresh_token(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    // Extract and validate refresh token
    let token = extract_token(&req)?;
    let claims = validate_token_with_revocation(&state, &token).await?;

    // Issue new tokens
    let user_id = claims.user_id()?;
    let org_id = claims.org_id()?;

    let new_token = state.jwt.create_token(user_id, org_id)?;
    let new_refresh_token = state.jwt.create_refresh_token(user_id, org_id)?;

    tracing::info!(
        user_id = %user_id,
        "Token refreshed successfully"
    );

    Ok(HttpResponse::Ok().json(RefreshTokenResponse {
        token: new_token,
        refresh_token: new_refresh_token,
    }))
}

async fn me(state: web::Data<Arc<AppState>>, req: HttpRequest) -> Result<HttpResponse, Error> {
    let (user_id, _org_id) = get_auth_context(&state, &req).await?;

    let user = state.db.get_user(user_id).await?;

    Ok(HttpResponse::Ok().json(UserResponse::from(user)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_auth_subject_is_stable_and_not_plaintext() {
        let email = "User@Example.com";
        let hash1 = hash_auth_subject(email);
        let hash2 = hash_auth_subject("user@example.com");

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
        assert!(!hash1.contains("user@example.com"));
    }

    #[test]
    fn test_cache_key_builders() {
        let jti = "1234";
        let email = "user@example.com";

        assert_eq!(revoked_token_key(jti), "auth:revoked_token:1234");
        assert!(failed_login_attempts_key(email).starts_with("auth:login_attempts:"));
        assert!(account_lockout_until_key(email).starts_with("auth:account_lockout_until:"));
        assert!(lockout_level_key(email).starts_with("auth:account_lockout_level:"));
    }

    #[test]
    fn test_lockout_duration_escalation() {
        assert_eq!(lockout_duration_secs(1), BASE_ACCOUNT_LOCKOUT_SECS);
        assert_eq!(lockout_duration_secs(2), BASE_ACCOUNT_LOCKOUT_SECS * 2);
        assert_eq!(lockout_duration_secs(3), BASE_ACCOUNT_LOCKOUT_SECS * 4);
        assert_eq!(lockout_duration_secs(20), MAX_ACCOUNT_LOCKOUT_SECS);
    }
}
