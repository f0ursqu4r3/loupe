use actix_web::{HttpResponse, ResponseError};
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    // Client errors
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),

    // Server errors
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
        let (status, error_type) = match self {
            Error::NotFound(_) => (actix_web::http::StatusCode::NOT_FOUND, "not_found"),
            Error::BadRequest(_) => (actix_web::http::StatusCode::BAD_REQUEST, "bad_request"),
            Error::Unauthorized(_) => (actix_web::http::StatusCode::UNAUTHORIZED, "unauthorized"),
            Error::Forbidden(_) => (actix_web::http::StatusCode::FORBIDDEN, "forbidden"),
            Error::Conflict(_) => (actix_web::http::StatusCode::CONFLICT, "conflict"),
            Error::Internal(_) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
            ),
            Error::Database(_) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "database_error",
            ),
            Error::Connection(_) => (actix_web::http::StatusCode::BAD_GATEWAY, "connection_error"),
            Error::QueryExecution(_) => (actix_web::http::StatusCode::BAD_REQUEST, "query_error"),
            Error::Timeout(_) => (actix_web::http::StatusCode::GATEWAY_TIMEOUT, "timeout"),
        };

        HttpResponse::build(status).json(serde_json::json!({
            "error": {
                "type": error_type,
                "message": self.to_string()
            }
        }))
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Error::NotFound("Record not found".to_string()),
            sqlx::Error::Database(e) => Error::Database(e.to_string()),
            _ => Error::Database(err.to_string()),
        }
    }
}

impl From<argon2::password_hash::Error> for Error {
    fn from(err: argon2::password_hash::Error) -> Self {
        Error::Internal(format!("Password hashing error: {}", err))
    }
}
