use crate::common::error::Error;
use validator::{Validate, ValidationError};

/// Validates a request DTO using the validator crate
pub fn validate_request<T: Validate>(data: &T) -> Result<(), Error> {
    data.validate().map_err(|e| {
        let messages: Vec<String> = e
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                let field = field.to_string();
                errors
                    .iter()
                    .map(move |err| format!("{}: {}", field, err.message.as_ref().unwrap_or(&std::borrow::Cow::Borrowed("validation failed"))))
            })
            .collect();

        Error::BadRequest(messages.join(", "))
    })
}

/// Custom validator for connection strings
pub fn validate_connection_string(value: &str) -> Result<(), ValidationError> {
    // Basic check - must contain scheme and host
    if !value.contains("://") {
        return Err(ValidationError::new("invalid_connection_string")
            .with_message("Connection string must include a protocol scheme (e.g., postgresql://)".into()));
    }

    // Check for common SQL injection patterns in connection strings
    let dangerous_patterns = ["';", "--", "/*", "*/", "xp_", "sp_", "exec(", "execute("];
    for pattern in dangerous_patterns {
        if value.to_lowercase().contains(pattern) {
            return Err(ValidationError::new("suspicious_connection_string")
                .with_message("Connection string contains suspicious patterns".into()));
        }
    }

    // Check minimum length
    if value.len() < 10 {
        return Err(ValidationError::new("connection_string_too_short")
            .with_message("Connection string is too short".into()));
    }

    // Check maximum length (prevent DoS via huge strings)
    if value.len() > 2048 {
        return Err(ValidationError::new("connection_string_too_long")
            .with_message("Connection string exceeds maximum length of 2048 characters".into()));
    }

    Ok(())
}

/// Custom validator for cron expressions
pub fn validate_cron_expression(value: &str) -> Result<(), ValidationError> {
    use cron::Schedule;
    use std::str::FromStr;

    Schedule::from_str(value)
        .map(|_| ())
        .map_err(|_| {
            ValidationError::new("invalid_cron_expression")
                .with_message("Invalid cron expression format".into())
        })
}

/// Custom validator for name fields (alphanumeric, spaces, hyphens, underscores)
pub fn validate_name(value: &str) -> Result<(), ValidationError> {
    if value.is_empty() {
        return Err(ValidationError::new("name_required")
            .with_message("Name cannot be empty".into()));
    }

    if value.len() > 255 {
        return Err(ValidationError::new("name_too_long")
            .with_message("Name exceeds maximum length of 255 characters".into()));
    }

    // Allow alphanumeric, spaces, hyphens, underscores, and common punctuation
    let valid = value.chars().all(|c| {
        c.is_alphanumeric() || c.is_whitespace() || matches!(c, '-' | '_' | '.' | '(' | ')' | ',' | ':')
    });

    if !valid {
        return Err(ValidationError::new("invalid_name_characters")
            .with_message("Name contains invalid characters".into()));
    }

    Ok(())
}

/// Custom validator for description fields
pub fn validate_description(value: &str) -> Result<(), ValidationError> {
    if value.len() > 2000 {
        return Err(ValidationError::new("description_too_long")
            .with_message("Description exceeds maximum length of 2000 characters".into()));
    }

    Ok(())
}

/// Custom validator for SQL queries (basic length and pattern check)
pub fn validate_sql_length(value: &str) -> Result<(), ValidationError> {
    if value.is_empty() {
        return Err(ValidationError::new("sql_required")
            .with_message("SQL query cannot be empty".into()));
    }

    if value.len() > 100_000 {
        return Err(ValidationError::new("sql_too_long")
            .with_message("SQL query exceeds maximum length of 100,000 characters".into()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_connection_string_valid() {
        assert!(validate_connection_string("postgresql://user:pass@localhost:5432/db").is_ok());
        assert!(validate_connection_string("mysql://localhost/database").is_ok());
    }

    #[test]
    fn test_validate_connection_string_no_scheme() {
        assert!(validate_connection_string("localhost:5432").is_err());
    }

    #[test]
    fn test_validate_connection_string_too_short() {
        assert!(validate_connection_string("pg://").is_err());
    }

    #[test]
    fn test_validate_connection_string_sql_injection() {
        assert!(validate_connection_string("postgresql://user';DROP TABLE--@localhost/db").is_err());
    }

    #[test]
    fn test_validate_name_valid() {
        assert!(validate_name("My Dashboard").is_ok());
        assert!(validate_name("query-123_test").is_ok());
        assert!(validate_name("Sales Report (Q4)").is_ok());
    }

    #[test]
    fn test_validate_name_empty() {
        assert!(validate_name("").is_err());
    }

    #[test]
    fn test_validate_name_too_long() {
        let long_name = "a".repeat(256);
        assert!(validate_name(&long_name).is_err());
    }

    #[test]
    fn test_validate_description_valid() {
        assert!(validate_description("This is a valid description").is_ok());
    }

    #[test]
    fn test_validate_description_too_long() {
        let long_desc = "a".repeat(2001);
        assert!(validate_description(&long_desc).is_err());
    }

    #[test]
    fn test_validate_sql_length_valid() {
        assert!(validate_sql_length("SELECT * FROM users").is_ok());
    }

    #[test]
    fn test_validate_sql_length_empty() {
        assert!(validate_sql_length("").is_err());
    }

    #[test]
    fn test_validate_sql_length_too_long() {
        let long_sql = "SELECT * FROM users WHERE ".to_string() + &"a".repeat(100_000);
        assert!(validate_sql_length(&long_sql).is_err());
    }
}
