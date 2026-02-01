/// Database tracing span helpers
///
/// This module provides helpers for creating tracing spans around database operations.
/// SQLx automatically emits trace events for queries.
///
/// These utilities allow adding custom spans around database operations for better observability.

use tracing::Span;

/// Macro to create a database operation span
///
/// # Examples
///
/// ```rust,ignore
/// use loupe::db_span;
///
/// async fn get_user(db: &Database, user_id: Uuid) -> Result<User> {
///     let _span = db_span!("get_user", user_id = %user_id);
///     sqlx::query_as("SELECT * FROM users WHERE id = $1")
///         .bind(user_id)
///         .fetch_one(&db.pool)
///         .await
/// }
/// ```
#[macro_export]
macro_rules! db_span {
    ($operation:expr) => {
        tracing::info_span!("db.operation", operation = $operation, db.system = "postgresql")
    };
    ($operation:expr, $($field:tt)*) => {
        tracing::info_span!(
            "db.operation",
            operation = $operation,
            db.system = "postgresql",
            $($field)*
        )
    };
}

/// Helper to create a span for a database query
///
/// This is useful when you want to add custom attributes to a query span
/// beyond what sqlx provides automatically.
pub fn query_span(operation: &str) -> Span {
    tracing::info_span!(
        "db.query",
        operation = operation,
        db.system = "postgresql",
        otel.kind = "client",
    )
}

/// Helper to create a span for a database transaction
pub fn transaction_span(name: &str) -> Span {
    tracing::info_span!(
        "db.transaction",
        transaction = name,
        db.system = "postgresql",
        otel.kind = "client",
    )
}

/// Example instrumented function showing best practices
///
/// ```rust,ignore
/// #[instrument(
///     name = "db.get_dashboard",
///     skip(db),
///     fields(
///         db.system = "postgresql",
///         db.operation = "SELECT",
///         dashboard.id = %dashboard_id
///     )
/// )]
/// async fn get_dashboard(db: &Database, dashboard_id: Uuid) -> Result<Dashboard> {
///     sqlx::query_as("SELECT * FROM dashboards WHERE id = $1")
///         .bind(dashboard_id)
///         .fetch_one(&db.pool)
///         .await
///         .map_err(Error::from)
/// }
/// ```

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_span_creation() {
        let span = query_span("SELECT users");
        assert_eq!(span.metadata().map(|m| m.name()), Some("db.query"));
    }

    #[test]
    fn test_transaction_span_creation() {
        let span = transaction_span("create_dashboard");
        assert_eq!(span.metadata().map(|m| m.name()), Some("db.transaction"));
    }
}
