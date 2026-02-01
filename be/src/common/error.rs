use actix_web::{HttpResponse, ResponseError};
use std::fmt;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    // Client errors (safe to show details)
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),

    // Server errors (hide details from client)
    Internal(String),
    Database(String),
    Connection(String),
    QueryExecution(String),
    Timeout(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotFound(msg) => write!(f, "Not found: {}", msg),
            Error::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            Error::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            Error::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            Error::Conflict(msg) => write!(f, "Conflict: {}", msg),
            Error::Internal(msg) => write!(f, "Internal error: {}", msg),
            Error::Database(msg) => write!(f, "Database error: {}", msg),
            Error::Connection(msg) => write!(f, "Connection error: {}", msg),
            Error::QueryExecution(msg) => write!(f, "Query execution error: {}", msg),
            Error::Timeout(msg) => write!(f, "Timeout: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        let error_id = Uuid::new_v4();

        let (status, error_type, client_message) = match self {
            // Client errors - safe to show details
            Error::NotFound(msg) => (
                actix_web::http::StatusCode::NOT_FOUND,
                "not_found",
                msg.clone(),
            ),
            Error::BadRequest(msg) => (
                actix_web::http::StatusCode::BAD_REQUEST,
                "bad_request",
                msg.clone(),
            ),
            Error::Unauthorized(msg) => (
                actix_web::http::StatusCode::UNAUTHORIZED,
                "unauthorized",
                msg.clone(),
            ),
            Error::Forbidden(msg) => (
                actix_web::http::StatusCode::FORBIDDEN,
                "forbidden",
                msg.clone(),
            ),
            Error::Conflict(msg) => (
                actix_web::http::StatusCode::CONFLICT,
                "conflict",
                msg.clone(),
            ),

            // Server errors - hide details from client, log server-side
            Error::Internal(msg) => {
                tracing::error!(
                    error_id = %error_id,
                    error = %msg,
                    "Internal server error"
                );
                (
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "An internal error occurred. Please contact support with this error ID.".to_string(),
                )
            }
            Error::Database(msg) => {
                tracing::error!(
                    error_id = %error_id,
                    error = %msg,
                    "Database error"
                );
                (
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "database_error",
                    "A database error occurred. Please try again later.".to_string(),
                )
            }
            Error::Connection(msg) => {
                tracing::error!(
                    error_id = %error_id,
                    error = %msg,
                    "Connection error"
                );
                (
                    actix_web::http::StatusCode::BAD_GATEWAY,
                    "connection_error",
                    "Failed to connect to external service. Please try again later.".to_string(),
                )
            }
            Error::QueryExecution(msg) => {
                tracing::error!(
                    error_id = %error_id,
                    error = %msg,
                    "Query execution error"
                );
                (
                    actix_web::http::StatusCode::BAD_REQUEST,
                    "query_error",
                    "Query execution failed. Please check your query syntax.".to_string(),
                )
            }
            Error::Timeout(msg) => {
                tracing::warn!(
                    error_id = %error_id,
                    error = %msg,
                    "Request timeout"
                );
                (
                    actix_web::http::StatusCode::GATEWAY_TIMEOUT,
                    "timeout",
                    "The operation timed out. Please try again or reduce query complexity.".to_string(),
                )
            }
        };

        HttpResponse::build(status).json(serde_json::json!({
            "error": {
                "type": error_type,
                "message": client_message,
                "error_id": error_id.to_string(),
            }
        }))
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Error::NotFound("Record not found".to_string()),
            sqlx::Error::Database(e) => {
                // Log full database error server-side
                tracing::error!(error = %e, "Database error occurred");
                // Return generic error to client
                Error::Database("Database operation failed".to_string())
            }
            _ => {
                // Log full error server-side
                tracing::error!(error = %err, "Database error occurred");
                // Return generic error to client
                Error::Database("Database operation failed".to_string())
            }
        }
    }
}

impl From<argon2::password_hash::Error> for Error {
    fn from(err: argon2::password_hash::Error) -> Self {
        // Never expose password hashing details
        tracing::error!(error = %err, "Password hashing error");
        Error::Internal("Authentication error".to_string())
    }
}
