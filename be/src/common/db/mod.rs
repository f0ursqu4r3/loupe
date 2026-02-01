pub mod span_helpers;

use crate::error::{Error, Result};
use crate::models::*;
use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

/// Connection pool statistics
#[derive(Debug, Clone, Copy)]
pub struct PoolStats {
    /// Number of connections currently in use
    pub connections_active: u32,
    /// Number of idle connections available in the pool
    pub connections_idle: u32,
    /// Maximum number of connections allowed in the pool
    pub connections_max: u32,
}

/// Database connection configuration
pub struct DatabaseConfig {
    /// Minimum number of connections in the pool
    pub min_connections: u32,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Connection timeout (time to wait for a connection to be established)
    pub connect_timeout: Duration,
    /// Idle timeout (time before closing idle connections)
    pub idle_timeout: Option<Duration>,
    /// Max lifetime for a connection
    pub max_lifetime: Option<Duration>,
    /// Acquire timeout (time to wait for a connection from the pool)
    pub acquire_timeout: Duration,
    /// SSL mode for the connection
    pub ssl_mode: PgSslMode,
    /// Test connections before giving them to the application
    pub test_before_acquire: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            min_connections: 2,
            max_connections: 10,
            connect_timeout: Duration::from_secs(10),
            idle_timeout: Some(Duration::from_secs(600)), // 10 minutes
            max_lifetime: Some(Duration::from_secs(1800)), // 30 minutes
            acquire_timeout: Duration::from_secs(5),
            ssl_mode: PgSslMode::Prefer, // Prefer SSL, fallback to plain if unavailable
            test_before_acquire: true,
        }
    }
}

impl DatabaseConfig {
    /// Create a production-ready configuration with stricter settings
    pub fn production() -> Self {
        Self {
            ssl_mode: PgSslMode::Require, // Require SSL in production
            ..Default::default()
        }
    }

    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "local".to_string());

        let mut config = if env == "prod" || env == "production" {
            Self::production()
        } else {
            Self::default()
        };

        // Override from environment variables if present
        if let Ok(max_str) = std::env::var("DB_MAX_CONNECTIONS") {
            if let Ok(max) = max_str.parse() {
                config.max_connections = max;
            }
        }

        if let Ok(min_str) = std::env::var("DB_MIN_CONNECTIONS") {
            if let Ok(min) = min_str.parse() {
                config.min_connections = min;
            }
        }

        if let Ok(ssl_str) = std::env::var("DB_SSL_MODE") {
            config.ssl_mode = match ssl_str.to_lowercase().as_str() {
                "disable" => PgSslMode::Disable,
                "allow" => PgSslMode::Allow,
                "prefer" => PgSslMode::Prefer,
                "require" => PgSslMode::Require,
                "verify-ca" => PgSslMode::VerifyCa,
                "verify-full" => PgSslMode::VerifyFull,
                _ => PgSslMode::Prefer,
            };
        }

        config
    }
}

impl Database {
    /// Connect to the database with default configuration
    pub async fn connect(database_url: &str) -> Result<Self> {
        Self::connect_with_config(database_url, DatabaseConfig::from_env()).await
    }

    /// Connect to the database with custom configuration
    pub async fn connect_with_config(database_url: &str, config: DatabaseConfig) -> Result<Self> {
        // Validate DATABASE_URL format
        Self::validate_database_url(database_url)?;

        // Parse connection options from URL
        let options = PgConnectOptions::from_str(database_url)
            .map_err(|e| Error::BadRequest(format!("Invalid DATABASE_URL format: {}", e)))?
            .ssl_mode(config.ssl_mode);

        // Build pool with configuration
        let pool = PgPoolOptions::new()
            .min_connections(config.min_connections)
            .max_connections(config.max_connections)
            .acquire_timeout(config.acquire_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .test_before_acquire(config.test_before_acquire)
            .connect_with(options)
            .await
            .map_err(|e| {
                // Don't include the connection string in the error message
                tracing::error!("Database connection failed: {}", e);
                Error::Database("Failed to connect to database. Check configuration and network.".to_string())
            })?;

        tracing::info!(
            "Database connection pool initialized (min: {}, max: {}, SSL: {:?})",
            config.min_connections,
            config.max_connections,
            config.ssl_mode
        );

        Ok(Self { pool })
    }

    /// Validate DATABASE_URL format without exposing it in logs
    fn validate_database_url(url: &str) -> Result<()> {
        // Basic validation
        if url.is_empty() {
            return Err(Error::BadRequest("DATABASE_URL cannot be empty".to_string()));
        }

        if !url.starts_with("postgres://") && !url.starts_with("postgresql://") {
            return Err(Error::BadRequest(
                "DATABASE_URL must start with postgres:// or postgresql://".to_string(),
            ));
        }

        // Check minimum length (protocol + host + db name should be at least 20 chars)
        if url.len() < 20 {
            return Err(Error::BadRequest("DATABASE_URL is too short".to_string()));
        }

        Ok(())
    }

    /// Check if the database connection is healthy
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("Database health check failed: {}", e);
                Error::Database("Database health check failed".to_string())
            })?;

        Ok(())
    }

    /// Check if all migrations have been applied
    pub async fn check_migrations_applied(&self) -> Result<bool> {
        // Query the _sqlx_migrations table to check if all migrations are applied
        // If the table doesn't exist, migrations haven't been run
        let result = sqlx::query(
            "SELECT COUNT(*) as count FROM _sqlx_migrations WHERE success = true"
        )
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(_)) => {
                // Table exists and we can query it - migrations have been applied
                // We could compare count with expected migrations, but for simplicity
                // we'll just check if the table exists and has successful migrations
                Ok(true)
            }
            Ok(None) => {
                // Table exists but no rows - unlikely but consider it as no migrations
                Ok(false)
            }
            Err(e) => {
                // Table probably doesn't exist or query failed
                tracing::warn!("Migration status check failed: {}", e);
                // Return false instead of error to avoid breaking health checks
                Ok(false)
            }
        }
    }

    /// Get connection pool statistics
    ///
    /// Returns statistics about the current state of the connection pool,
    /// including active connections, idle connections, and pool size.
    pub fn pool_stats(&self) -> PoolStats {
        let size = self.pool.size() as u32;
        let idle = self.pool.num_idle() as u32;
        let active = size.saturating_sub(idle);

        PoolStats {
            connections_active: active,
            connections_idle: idle,
            connections_max: self.pool.options().get_max_connections(),
        }
    }

    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| Error::Database(format!("Migration failed: {}", e)))?;
        Ok(())
    }

    // ==================== Organizations ====================

    pub async fn create_organization(&self, name: &str) -> Result<Organization> {
        let org = sqlx::query_as::<_, Organization>(
            r#"
            INSERT INTO organizations (id, name, created_at, updated_at)
            VALUES ($1, $2, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        Ok(org)
    }

    pub async fn get_organization(&self, id: Uuid) -> Result<Organization> {
        let org = sqlx::query_as::<_, Organization>("SELECT * FROM organizations WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(org)
    }

    // ==================== Users ====================

    pub async fn create_user(
        &self,
        org_id: Uuid,
        email: &str,
        password_hash: &str,
        name: &str,
        role: OrgRole,
    ) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, org_id, email, password_hash, name, role, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(email)
        .bind(password_hash)
        .bind(name)
        .bind(role)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    pub async fn get_user(&self, id: Uuid) -> Result<User> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }

    // ==================== Organization User Management ====================

    /// List all users in an organization
    pub async fn list_organization_users(&self, org_id: Uuid) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE org_id = $1 ORDER BY created_at DESC",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    pub async fn list_organization_users_paginated(
        &self,
        org_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<User>, i64)> {
        // Get paginated results
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE org_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(org_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        // Get total count
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users WHERE org_id = $1",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok((users, total.0))
    }

    /// Get a specific user within an organization (for verification)
    pub async fn get_user_in_organization(&self, user_id: Uuid, org_id: Uuid) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1 AND org_id = $2",
        )
        .bind(user_id)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound("User not found in this organization".into()))?;

        Ok(user)
    }

    /// Update a user's role within an organization
    pub async fn update_user_role(
        &self,
        user_id: Uuid,
        org_id: Uuid,
        new_role: OrgRole,
    ) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET role = $3, updated_at = NOW()
            WHERE id = $1 AND org_id = $2
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(org_id)
        .bind(new_role)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound("User not found in this organization".into()))?;

        Ok(user)
    }

    /// Remove a user from an organization
    pub async fn remove_user_from_organization(&self, user_id: Uuid, org_id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1 AND org_id = $2")
            .bind(user_id)
            .bind(org_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound("User not found in this organization".into()));
        }

        Ok(())
    }

    // ==================== Datasources ====================

    pub async fn create_datasource(
        &self,
        org_id: Uuid,
        name: &str,
        ds_type: DatasourceType,
        connection_string_encrypted: &str,
        created_by: Uuid,
    ) -> Result<Datasource> {
        let ds = sqlx::query_as::<_, Datasource>(
            r#"
            INSERT INTO datasources (id, org_id, name, ds_type, connection_string_encrypted, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(name)
        .bind(ds_type)
        .bind(connection_string_encrypted)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(ds)
    }

    pub async fn get_datasource(&self, id: Uuid, org_id: Uuid) -> Result<Datasource> {
        let ds = sqlx::query_as::<_, Datasource>(
            "SELECT * FROM datasources WHERE id = $1 AND org_id = $2",
        )
        .bind(id)
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ds)
    }

    pub async fn list_datasources(&self, org_id: Uuid) -> Result<Vec<Datasource>> {
        let datasources = sqlx::query_as::<_, Datasource>(
            "SELECT * FROM datasources WHERE org_id = $1 ORDER BY created_at DESC",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(datasources)
    }

    pub async fn list_datasources_paginated(
        &self,
        org_id: Uuid,
        search: Option<String>,
        sort_column: &str,
        sort_direction: &str,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Datasource>, i64)> {
        let order_by = format!("{} {}", sort_column, sort_direction);

        // Build query based on search filter
        let datasources = if let Some(pattern) = search.as_ref() {
            sqlx::query_as(&format!(
                "SELECT * FROM datasources WHERE org_id = $1 AND name ILIKE $2 ORDER BY {} LIMIT $3 OFFSET $4",
                order_by
            ))
            .bind(org_id)
            .bind(pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(&format!(
                "SELECT * FROM datasources WHERE org_id = $1 ORDER BY {} LIMIT $2 OFFSET $3",
                order_by
            ))
            .bind(org_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        // Get total count with same filter
        let total: (i64,) = if let Some(pattern) = search.as_ref() {
            sqlx::query_as("SELECT COUNT(*) FROM datasources WHERE org_id = $1 AND name ILIKE $2")
                .bind(org_id)
                .bind(pattern)
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_as("SELECT COUNT(*) FROM datasources WHERE org_id = $1")
                .bind(org_id)
                .fetch_one(&self.pool)
                .await?
        };

        Ok((datasources, total.0))
    }

    pub async fn update_datasource(
        &self,
        id: Uuid,
        org_id: Uuid,
        name: Option<&str>,
        connection_string_encrypted: Option<&str>,
    ) -> Result<Datasource> {
        let ds = sqlx::query_as::<_, Datasource>(
            r#"
            UPDATE datasources 
            SET name = COALESCE($3, name),
                connection_string_encrypted = COALESCE($4, connection_string_encrypted),
                updated_at = NOW()
            WHERE id = $1 AND org_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(name)
        .bind(connection_string_encrypted)
        .fetch_one(&self.pool)
        .await?;

        Ok(ds)
    }

    pub async fn delete_datasource(&self, id: Uuid, org_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM datasources WHERE id = $1 AND org_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // ==================== Queries ====================

    pub async fn create_query(
        &self,
        org_id: Uuid,
        datasource_id: Uuid,
        name: &str,
        description: Option<&str>,
        sql: &str,
        parameters: &serde_json::Value,
        tags: &serde_json::Value,
        timeout_seconds: i32,
        max_rows: i32,
        created_by: Uuid,
    ) -> Result<Query> {
        let query = sqlx::query_as::<_, Query>(
            r#"
            INSERT INTO queries (id, org_id, datasource_id, name, description, sql, parameters, tags, timeout_seconds, max_rows, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(datasource_id)
        .bind(name)
        .bind(description)
        .bind(sql)
        .bind(parameters)
        .bind(tags)
        .bind(timeout_seconds)
        .bind(max_rows)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(query)
    }

    pub async fn get_query(&self, id: Uuid, org_id: Uuid) -> Result<Query> {
        let query =
            sqlx::query_as::<_, Query>("SELECT * FROM queries WHERE id = $1 AND org_id = $2")
                .bind(id)
                .bind(org_id)
                .fetch_one(&self.pool)
                .await?;

        Ok(query)
    }

    pub async fn list_queries(&self, org_id: Uuid) -> Result<Vec<Query>> {
        let queries = sqlx::query_as::<_, Query>(
            "SELECT * FROM queries WHERE org_id = $1 ORDER BY created_at DESC",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(queries)
    }

    pub async fn list_queries_paginated(
        &self,
        org_id: Uuid,
        search: Option<String>,
        datasource_id: Option<Uuid>,
        tags: Option<Vec<String>>,
        sort_column: &str,
        sort_direction: &str,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Query>, i64)> {
        let order_by = format!("{} {}", sort_column, sort_direction);

        // Build query based on filter combinations
        let queries = match (search.as_ref(), datasource_id, tags.as_ref()) {
            (None, None, None) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM queries WHERE org_id = $1 ORDER BY {} LIMIT $2 OFFSET $3",
                    order_by
                ))
                .bind(org_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(pattern), None, None) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM queries
                     WHERE org_id = $1 AND (name ILIKE $2 OR description ILIKE $2 OR sql ILIKE $2)
                     ORDER BY {} LIMIT $3 OFFSET $4",
                    order_by
                ))
                .bind(org_id)
                .bind(pattern)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(ds_id), None) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM queries WHERE org_id = $1 AND datasource_id = $2 ORDER BY {} LIMIT $3 OFFSET $4",
                    order_by
                ))
                .bind(org_id)
                .bind(ds_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (None, None, Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as(&format!(
                    "SELECT * FROM queries WHERE org_id = $1 AND tags @> $2 ORDER BY {} LIMIT $3 OFFSET $4",
                    order_by
                ))
                .bind(org_id)
                .bind(tags_json)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(pattern), Some(ds_id), None) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM queries
                     WHERE org_id = $1 AND datasource_id = $2
                     AND (name ILIKE $3 OR description ILIKE $3 OR sql ILIKE $3)
                     ORDER BY {} LIMIT $4 OFFSET $5",
                    order_by
                ))
                .bind(org_id)
                .bind(ds_id)
                .bind(pattern)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(pattern), None, Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as(&format!(
                    "SELECT * FROM queries
                     WHERE org_id = $1 AND (name ILIKE $2 OR description ILIKE $2 OR sql ILIKE $2)
                     AND tags @> $3
                     ORDER BY {} LIMIT $4 OFFSET $5",
                    order_by
                ))
                .bind(org_id)
                .bind(pattern)
                .bind(tags_json)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(ds_id), Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as(&format!(
                    "SELECT * FROM queries
                     WHERE org_id = $1 AND datasource_id = $2 AND tags @> $3
                     ORDER BY {} LIMIT $4 OFFSET $5",
                    order_by
                ))
                .bind(org_id)
                .bind(ds_id)
                .bind(tags_json)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(pattern), Some(ds_id), Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as(&format!(
                    "SELECT * FROM queries
                     WHERE org_id = $1 AND datasource_id = $2
                     AND (name ILIKE $3 OR description ILIKE $3 OR sql ILIKE $3)
                     AND tags @> $4
                     ORDER BY {} LIMIT $5 OFFSET $6",
                    order_by
                ))
                .bind(org_id)
                .bind(ds_id)
                .bind(pattern)
                .bind(tags_json)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
        };

        // Get total count with same filter conditions
        let total: (i64,) = match (search.as_ref(), datasource_id, tags.as_ref()) {
            (None, None, None) => {
                sqlx::query_as("SELECT COUNT(*) FROM queries WHERE org_id = $1")
                    .bind(org_id)
                    .fetch_one(&self.pool)
                    .await?
            }
            (Some(pattern), None, None) => {
                sqlx::query_as(
                    "SELECT COUNT(*) FROM queries
                     WHERE org_id = $1 AND (name ILIKE $2 OR description ILIKE $2 OR sql ILIKE $2)",
                )
                .bind(org_id)
                .bind(pattern)
                .fetch_one(&self.pool)
                .await?
            }
            (None, Some(ds_id), None) => {
                sqlx::query_as(
                    "SELECT COUNT(*) FROM queries WHERE org_id = $1 AND datasource_id = $2",
                )
                .bind(org_id)
                .bind(ds_id)
                .fetch_one(&self.pool)
                .await?
            }
            (None, None, Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as("SELECT COUNT(*) FROM queries WHERE org_id = $1 AND tags @> $2")
                    .bind(org_id)
                    .bind(tags_json)
                    .fetch_one(&self.pool)
                    .await?
            }
            (Some(pattern), Some(ds_id), None) => {
                sqlx::query_as(
                    "SELECT COUNT(*) FROM queries
                     WHERE org_id = $1 AND datasource_id = $2
                     AND (name ILIKE $3 OR description ILIKE $3 OR sql ILIKE $3)",
                )
                .bind(org_id)
                .bind(ds_id)
                .bind(pattern)
                .fetch_one(&self.pool)
                .await?
            }
            (Some(pattern), None, Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as(
                    "SELECT COUNT(*) FROM queries
                     WHERE org_id = $1 AND (name ILIKE $2 OR description ILIKE $2 OR sql ILIKE $2)
                     AND tags @> $3",
                )
                .bind(org_id)
                .bind(pattern)
                .bind(tags_json)
                .fetch_one(&self.pool)
                .await?
            }
            (None, Some(ds_id), Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as(
                    "SELECT COUNT(*) FROM queries
                     WHERE org_id = $1 AND datasource_id = $2 AND tags @> $3",
                )
                .bind(org_id)
                .bind(ds_id)
                .bind(tags_json)
                .fetch_one(&self.pool)
                .await?
            }
            (Some(pattern), Some(ds_id), Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as(
                    "SELECT COUNT(*) FROM queries
                     WHERE org_id = $1 AND datasource_id = $2
                     AND (name ILIKE $3 OR description ILIKE $3 OR sql ILIKE $3)
                     AND tags @> $4",
                )
                .bind(org_id)
                .bind(ds_id)
                .bind(pattern)
                .bind(tags_json)
                .fetch_one(&self.pool)
                .await?
            }
        };

        Ok((queries, total.0))
    }

    pub async fn update_query(
        &self,
        id: Uuid,
        org_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        sql: Option<&str>,
        parameters: Option<&serde_json::Value>,
        tags: Option<&serde_json::Value>,
        timeout_seconds: Option<i32>,
        max_rows: Option<i32>,
    ) -> Result<Query> {
        let query = sqlx::query_as::<_, Query>(
            r#"
            UPDATE queries 
            SET name = COALESCE($3, name),
                description = COALESCE($4, description),
                sql = COALESCE($5, sql),
                parameters = COALESCE($6, parameters),
                tags = COALESCE($7, tags),
                timeout_seconds = COALESCE($8, timeout_seconds),
                max_rows = COALESCE($9, max_rows),
                updated_at = NOW()
            WHERE id = $1 AND org_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(name)
        .bind(description)
        .bind(sql)
        .bind(parameters)
        .bind(tags)
        .bind(timeout_seconds)
        .bind(max_rows)
        .fetch_one(&self.pool)
        .await?;

        Ok(query)
    }

    pub async fn delete_query(&self, id: Uuid, org_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM queries WHERE id = $1 AND org_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // ==================== Runs ====================

    pub async fn create_run(
        &self,
        org_id: Uuid,
        query_id: Uuid,
        datasource_id: Uuid,
        executed_sql: &str,
        parameters: &serde_json::Value,
        timeout_seconds: i32,
        max_rows: i32,
        created_by: Uuid,
    ) -> Result<Run> {
        let run = sqlx::query_as::<_, Run>(
            r#"
            INSERT INTO runs (id, org_id, query_id, datasource_id, executed_sql, parameters, status, timeout_seconds, max_rows, created_by, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, 'queued', $7, $8, $9, NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(query_id)
        .bind(datasource_id)
        .bind(executed_sql)
        .bind(parameters)
        .bind(timeout_seconds)
        .bind(max_rows)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(run)
    }

    pub async fn get_run(&self, id: Uuid, org_id: Uuid) -> Result<Run> {
        let run = sqlx::query_as::<_, Run>("SELECT * FROM runs WHERE id = $1 AND org_id = $2")
            .bind(id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(run)
    }

    pub async fn list_runs(&self, org_id: Uuid, query_id: Option<Uuid>) -> Result<Vec<Run>> {
        let runs = if let Some(qid) = query_id {
            sqlx::query_as::<_, Run>(
                "SELECT * FROM runs WHERE org_id = $1 AND query_id = $2 ORDER BY created_at DESC LIMIT 100",
            )
            .bind(org_id)
            .bind(qid)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Run>(
                "SELECT * FROM runs WHERE org_id = $1 ORDER BY created_at DESC LIMIT 100",
            )
            .bind(org_id)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(runs)
    }

    pub async fn list_runs_paginated(
        &self,
        org_id: Uuid,
        query_id: Option<Uuid>,
        status: Option<String>,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
        sort_column: &str,
        sort_direction: &str,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Run>, i64)> {
        let order_by = format!("{} {}", sort_column, sort_direction);

        // Build date range condition
        let has_date_filter = start_date.is_some() || end_date.is_some();

        // Get paginated results
        let runs = match (query_id, status.as_ref(), has_date_filter) {
            (None, None, false) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM runs WHERE org_id = $1 ORDER BY {} LIMIT $2 OFFSET $3",
                    order_by
                ))
                .bind(org_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(qid), None, false) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM runs WHERE org_id = $1 AND query_id = $2 ORDER BY {} LIMIT $3 OFFSET $4",
                    order_by
                ))
                .bind(org_id)
                .bind(qid)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(st), false) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM runs WHERE org_id = $1 AND status = $2 ORDER BY {} LIMIT $3 OFFSET $4",
                    order_by
                ))
                .bind(org_id)
                .bind(st)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (None, None, true) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM runs WHERE org_id = $1 AND created_at >= $2 AND created_at <= $3 ORDER BY {} LIMIT $4 OFFSET $5",
                    order_by
                ))
                .bind(org_id)
                .bind(start_date.unwrap_or_else(|| chrono::DateTime::UNIX_EPOCH))
                .bind(end_date.unwrap_or_else(chrono::Utc::now))
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(qid), Some(st), false) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM runs WHERE org_id = $1 AND query_id = $2 AND status = $3 ORDER BY {} LIMIT $4 OFFSET $5",
                    order_by
                ))
                .bind(org_id)
                .bind(qid)
                .bind(st)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(qid), None, true) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM runs WHERE org_id = $1 AND query_id = $2 AND created_at >= $3 AND created_at <= $4 ORDER BY {} LIMIT $5 OFFSET $6",
                    order_by
                ))
                .bind(org_id)
                .bind(qid)
                .bind(start_date.unwrap_or_else(|| chrono::DateTime::UNIX_EPOCH))
                .bind(end_date.unwrap_or_else(chrono::Utc::now))
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(st), true) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM runs WHERE org_id = $1 AND status = $2 AND created_at >= $3 AND created_at <= $4 ORDER BY {} LIMIT $5 OFFSET $6",
                    order_by
                ))
                .bind(org_id)
                .bind(st)
                .bind(start_date.unwrap_or_else(|| chrono::DateTime::UNIX_EPOCH))
                .bind(end_date.unwrap_or_else(chrono::Utc::now))
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(qid), Some(st), true) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM runs WHERE org_id = $1 AND query_id = $2 AND status = $3 AND created_at >= $4 AND created_at <= $5 ORDER BY {} LIMIT $6 OFFSET $7",
                    order_by
                ))
                .bind(org_id)
                .bind(qid)
                .bind(st)
                .bind(start_date.unwrap_or_else(|| chrono::DateTime::UNIX_EPOCH))
                .bind(end_date.unwrap_or_else(chrono::Utc::now))
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
        };

        // Get total count with same filter conditions
        let total: (i64,) = match (query_id, status.as_ref(), has_date_filter) {
            (None, None, false) => {
                sqlx::query_as("SELECT COUNT(*) FROM runs WHERE org_id = $1")
                    .bind(org_id)
                    .fetch_one(&self.pool)
                    .await?
            }
            (Some(qid), None, false) => {
                sqlx::query_as("SELECT COUNT(*) FROM runs WHERE org_id = $1 AND query_id = $2")
                    .bind(org_id)
                    .bind(qid)
                    .fetch_one(&self.pool)
                    .await?
            }
            (None, Some(st), false) => {
                sqlx::query_as("SELECT COUNT(*) FROM runs WHERE org_id = $1 AND status = $2")
                    .bind(org_id)
                    .bind(st)
                    .fetch_one(&self.pool)
                    .await?
            }
            (None, None, true) => {
                sqlx::query_as(
                    "SELECT COUNT(*) FROM runs WHERE org_id = $1 AND created_at >= $2 AND created_at <= $3",
                )
                .bind(org_id)
                .bind(start_date.unwrap_or_else(|| chrono::DateTime::UNIX_EPOCH))
                .bind(end_date.unwrap_or_else(chrono::Utc::now))
                .fetch_one(&self.pool)
                .await?
            }
            (Some(qid), Some(st), false) => {
                sqlx::query_as(
                    "SELECT COUNT(*) FROM runs WHERE org_id = $1 AND query_id = $2 AND status = $3",
                )
                .bind(org_id)
                .bind(qid)
                .bind(st)
                .fetch_one(&self.pool)
                .await?
            }
            (Some(qid), None, true) => {
                sqlx::query_as(
                    "SELECT COUNT(*) FROM runs WHERE org_id = $1 AND query_id = $2 AND created_at >= $3 AND created_at <= $4",
                )
                .bind(org_id)
                .bind(qid)
                .bind(start_date.unwrap_or_else(|| chrono::DateTime::UNIX_EPOCH))
                .bind(end_date.unwrap_or_else(chrono::Utc::now))
                .fetch_one(&self.pool)
                .await?
            }
            (None, Some(st), true) => {
                sqlx::query_as(
                    "SELECT COUNT(*) FROM runs WHERE org_id = $1 AND status = $2 AND created_at >= $3 AND created_at <= $4",
                )
                .bind(org_id)
                .bind(st)
                .bind(start_date.unwrap_or_else(|| chrono::DateTime::UNIX_EPOCH))
                .bind(end_date.unwrap_or_else(chrono::Utc::now))
                .fetch_one(&self.pool)
                .await?
            }
            (Some(qid), Some(st), true) => {
                sqlx::query_as(
                    "SELECT COUNT(*) FROM runs WHERE org_id = $1 AND query_id = $2 AND status = $3 AND created_at >= $4 AND created_at <= $5",
                )
                .bind(org_id)
                .bind(qid)
                .bind(st)
                .bind(start_date.unwrap_or_else(|| chrono::DateTime::UNIX_EPOCH))
                .bind(end_date.unwrap_or_else(chrono::Utc::now))
                .fetch_one(&self.pool)
                .await?
            }
        };

        Ok((runs, total.0))
    }

    /// Claim a queued run for execution (called by runner)
    pub async fn claim_run(&self, runner_id: &str) -> Result<Option<Run>> {
        let run = sqlx::query_as::<_, Run>(
            r#"
            UPDATE runs 
            SET status = 'running', runner_id = $1, started_at = NOW()
            WHERE id = (
                SELECT id FROM runs 
                WHERE status = 'queued' 
                ORDER BY created_at ASC 
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING *
            "#,
        )
        .bind(runner_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(run)
    }

    /// Complete a run with success
    pub async fn complete_run(&self, id: Uuid, _result_id: Uuid) -> Result<Run> {
        let run = sqlx::query_as::<_, Run>(
            r#"
            UPDATE runs 
            SET status = 'completed', completed_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(run)
    }

    /// Fail a run with an error message
    pub async fn fail_run(&self, id: Uuid, error_message: &str) -> Result<Run> {
        let run = sqlx::query_as::<_, Run>(
            r#"
            UPDATE runs 
            SET status = 'failed', completed_at = NOW(), error_message = $2
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(error_message)
        .fetch_one(&self.pool)
        .await?;

        Ok(run)
    }

    /// Timeout a run
    pub async fn timeout_run(&self, id: Uuid) -> Result<Run> {
        let run = sqlx::query_as::<_, Run>(
            r#"
            UPDATE runs
            SET status = 'timeout', completed_at = NOW(), error_message = 'Query execution timed out'
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(run)
    }

    pub async fn cancel_run(&self, id: Uuid) -> Result<Run> {
        let run = sqlx::query_as::<_, Run>(
            r#"
            UPDATE runs
            SET status = 'cancelled', completed_at = NOW(), error_message = 'Query execution cancelled by user'
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(run)
    }

    /// Schedule a run for retry with exponential backoff
    ///
    /// Uses exponential backoff: base_delay * 2^retry_count
    /// Example: 30s, 60s, 120s, 240s, ...
    pub async fn schedule_retry(&self, id: Uuid, error_message: &str) -> Result<Option<Run>> {
        let run = sqlx::query_as::<_, Run>(
            r#"
            UPDATE runs
            SET status = 'failed',
                error_message = $2,
                retry_count = retry_count + 1,
                next_retry_at = CASE
                    WHEN retry_count < max_retries THEN
                        NOW() + (INTERVAL '30 seconds' * POWER(2, retry_count))
                    ELSE
                        NULL
                END
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(error_message)
        .fetch_one(&self.pool)
        .await?;

        // Return Some if retry is scheduled, None if max retries exceeded
        if run.next_retry_at.is_some() {
            Ok(Some(run))
        } else {
            Ok(None)
        }
    }

    /// Claim a run that's ready for retry
    ///
    /// Finds runs where next_retry_at is in the past and status is 'failed'
    pub async fn claim_retry_run(&self, runner_id: &str) -> Result<Option<Run>> {
        let run = sqlx::query_as::<_, Run>(
            r#"
            UPDATE runs
            SET status = 'queued',
                runner_id = $1,
                next_retry_at = NULL,
                error_message = NULL
            WHERE id = (
                SELECT id FROM runs
                WHERE status = 'failed'
                  AND next_retry_at IS NOT NULL
                  AND next_retry_at <= NOW()
                ORDER BY next_retry_at ASC
                FOR UPDATE SKIP LOCKED
                LIMIT 1
            )
            RETURNING *
            "#,
        )
        .bind(runner_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(run)
    }

    /// Get queue depth statistics
    ///
    /// Returns (pending_jobs, retry_jobs, dead_letter_jobs)
    pub async fn get_queue_stats(&self) -> Result<(i64, i64, i64)> {
        // Count pending jobs (status = queued)
        let pending: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM runs WHERE status = 'queued'"
        )
        .fetch_one(&self.pool)
        .await?;

        // Count retry jobs (status = failed with next_retry_at set)
        let retry: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM runs WHERE status = 'failed' AND next_retry_at IS NOT NULL"
        )
        .fetch_one(&self.pool)
        .await?;

        // Count dead letter queue
        let dead_letter: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM run_failures"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok((pending.0, retry.0, dead_letter.0))
    }

    /// Move a permanently failed run to the dead letter queue
    ///
    /// This should be called when a run exceeds max retries.
    /// The run is copied to run_failures and deleted from runs.
    pub async fn move_to_dead_letter_queue(&self, run_id: Uuid) -> Result<()> {
        // Get the run first
        let run = sqlx::query_as::<_, crate::models::Run>(
            "SELECT * FROM runs WHERE id = $1"
        )
        .bind(run_id)
        .fetch_one(&self.pool)
        .await?;

        // Insert into dead letter queue
        sqlx::query(
            r#"
            INSERT INTO run_failures (
                run_id, org_id, query_id, datasource_id, executed_sql, parameters,
                error_message, retry_count, max_retries, first_failed_at, last_failed_at,
                created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), $11)
            "#,
        )
        .bind(run.id)
        .bind(run.org_id)
        .bind(run.query_id)
        .bind(run.datasource_id)
        .bind(&run.executed_sql)
        .bind(&run.parameters)
        .bind(run.error_message.as_deref().unwrap_or("Unknown error"))
        .bind(run.retry_count)
        .bind(run.max_retries)
        .bind(run.started_at.unwrap_or(run.created_at))
        .bind(run.created_by)
        .execute(&self.pool)
        .await?;

        // Delete from runs table
        sqlx::query("DELETE FROM runs WHERE id = $1")
            .bind(run_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // ==================== Run Results ====================

    pub async fn create_run_result(
        &self,
        run_id: Uuid,
        columns: &serde_json::Value,
        rows: &serde_json::Value,
        row_count: i64,
        byte_count: i64,
        execution_time_ms: i64,
    ) -> Result<RunResult> {
        let result = sqlx::query_as::<_, RunResult>(
            r#"
            INSERT INTO run_results (id, run_id, columns, rows, row_count, byte_count, execution_time_ms, created_at, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW() + INTERVAL '7 days')
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(run_id)
        .bind(columns)
        .bind(rows)
        .bind(row_count)
        .bind(byte_count)
        .bind(execution_time_ms)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_run_result(&self, run_id: Uuid) -> Result<RunResult> {
        let result = sqlx::query_as::<_, RunResult>("SELECT * FROM run_results WHERE run_id = $1")
            .bind(run_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    // ==================== Visualizations ====================

    pub async fn create_visualization(
        &self,
        org_id: Uuid,
        query_id: Uuid,
        name: &str,
        chart_type: ChartType,
        config: &serde_json::Value,
        tags: &serde_json::Value,
        created_by: Uuid,
    ) -> Result<Visualization> {
        let viz = sqlx::query_as::<_, Visualization>(
            r#"
            INSERT INTO visualizations (id, org_id, query_id, name, chart_type, config, tags, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(query_id)
        .bind(name)
        .bind(chart_type)
        .bind(config)
        .bind(tags)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(viz)
    }

    pub async fn get_visualization(&self, id: Uuid, org_id: Uuid) -> Result<Visualization> {
        let viz = sqlx::query_as::<_, Visualization>(
            "SELECT * FROM visualizations WHERE id = $1 AND org_id = $2",
        )
        .bind(id)
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(viz)
    }

    pub async fn list_visualizations(
        &self,
        org_id: Uuid,
        query_id: Option<Uuid>,
    ) -> Result<Vec<Visualization>> {
        let vizs = if let Some(qid) = query_id {
            sqlx::query_as::<_, Visualization>(
                "SELECT * FROM visualizations WHERE org_id = $1 AND query_id = $2 ORDER BY created_at DESC",
            )
            .bind(org_id)
            .bind(qid)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Visualization>(
                "SELECT * FROM visualizations WHERE org_id = $1 ORDER BY created_at DESC",
            )
            .bind(org_id)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(vizs)
    }

    pub async fn list_visualizations_paginated(
        &self,
        org_id: Uuid,
        search: Option<String>,
        query_id: Option<Uuid>,
        tags: Option<Vec<String>>,
        sort_column: &str,
        sort_direction: &str,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Visualization>, i64)> {
        let order_by = format!("{} {}", sort_column, sort_direction);

        // Build query based on filter combinations
        let vizs = match (search.as_ref(), query_id, tags.as_ref()) {
            (None, None, None) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM visualizations WHERE org_id = $1 ORDER BY {} LIMIT $2 OFFSET $3",
                    order_by
                ))
                .bind(org_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(pattern), None, None) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM visualizations WHERE org_id = $1 AND name ILIKE $2 ORDER BY {} LIMIT $3 OFFSET $4",
                    order_by
                ))
                .bind(org_id)
                .bind(pattern)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(qid), None) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM visualizations WHERE org_id = $1 AND query_id = $2 ORDER BY {} LIMIT $3 OFFSET $4",
                    order_by
                ))
                .bind(org_id)
                .bind(qid)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (None, None, Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as(&format!(
                    "SELECT * FROM visualizations WHERE org_id = $1 AND tags @> $2 ORDER BY {} LIMIT $3 OFFSET $4",
                    order_by
                ))
                .bind(org_id)
                .bind(tags_json)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(pattern), Some(qid), None) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM visualizations WHERE org_id = $1 AND query_id = $2 AND name ILIKE $3 ORDER BY {} LIMIT $4 OFFSET $5",
                    order_by
                ))
                .bind(org_id)
                .bind(qid)
                .bind(pattern)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(pattern), None, Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as(&format!(
                    "SELECT * FROM visualizations WHERE org_id = $1 AND name ILIKE $2 AND tags @> $3 ORDER BY {} LIMIT $4 OFFSET $5",
                    order_by
                ))
                .bind(org_id)
                .bind(pattern)
                .bind(tags_json)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(qid), Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as(&format!(
                    "SELECT * FROM visualizations WHERE org_id = $1 AND query_id = $2 AND tags @> $3 ORDER BY {} LIMIT $4 OFFSET $5",
                    order_by
                ))
                .bind(org_id)
                .bind(qid)
                .bind(tags_json)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(pattern), Some(qid), Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as(&format!(
                    "SELECT * FROM visualizations WHERE org_id = $1 AND query_id = $2 AND name ILIKE $3 AND tags @> $4 ORDER BY {} LIMIT $5 OFFSET $6",
                    order_by
                ))
                .bind(org_id)
                .bind(qid)
                .bind(pattern)
                .bind(tags_json)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
        };

        // Get total count with same filter conditions
        let total: (i64,) = match (search.as_ref(), query_id, tags.as_ref()) {
            (None, None, None) => {
                sqlx::query_as("SELECT COUNT(*) FROM visualizations WHERE org_id = $1")
                    .bind(org_id)
                    .fetch_one(&self.pool)
                    .await?
            }
            (Some(pattern), None, None) => {
                sqlx::query_as("SELECT COUNT(*) FROM visualizations WHERE org_id = $1 AND name ILIKE $2")
                    .bind(org_id)
                    .bind(pattern)
                    .fetch_one(&self.pool)
                    .await?
            }
            (None, Some(qid), None) => {
                sqlx::query_as("SELECT COUNT(*) FROM visualizations WHERE org_id = $1 AND query_id = $2")
                    .bind(org_id)
                    .bind(qid)
                    .fetch_one(&self.pool)
                    .await?
            }
            (None, None, Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as("SELECT COUNT(*) FROM visualizations WHERE org_id = $1 AND tags @> $2")
                    .bind(org_id)
                    .bind(tags_json)
                    .fetch_one(&self.pool)
                    .await?
            }
            (Some(pattern), Some(qid), None) => {
                sqlx::query_as("SELECT COUNT(*) FROM visualizations WHERE org_id = $1 AND query_id = $2 AND name ILIKE $3")
                    .bind(org_id)
                    .bind(qid)
                    .bind(pattern)
                    .fetch_one(&self.pool)
                    .await?
            }
            (Some(pattern), None, Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as("SELECT COUNT(*) FROM visualizations WHERE org_id = $1 AND name ILIKE $2 AND tags @> $3")
                    .bind(org_id)
                    .bind(pattern)
                    .bind(tags_json)
                    .fetch_one(&self.pool)
                    .await?
            }
            (None, Some(qid), Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as("SELECT COUNT(*) FROM visualizations WHERE org_id = $1 AND query_id = $2 AND tags @> $3")
                    .bind(org_id)
                    .bind(qid)
                    .bind(tags_json)
                    .fetch_one(&self.pool)
                    .await?
            }
            (Some(pattern), Some(qid), Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as("SELECT COUNT(*) FROM visualizations WHERE org_id = $1 AND query_id = $2 AND name ILIKE $3 AND tags @> $4")
                    .bind(org_id)
                    .bind(qid)
                    .bind(pattern)
                    .bind(tags_json)
                    .fetch_one(&self.pool)
                    .await?
            }
        };

        Ok((vizs, total.0))
    }

    pub async fn delete_visualization(&self, id: Uuid, org_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM visualizations WHERE id = $1 AND org_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_visualization(
        &self,
        id: Uuid,
        org_id: Uuid,
        query_id: Option<Uuid>,
        name: Option<&str>,
        chart_type: Option<ChartType>,
        config: Option<&serde_json::Value>,
        tags: Option<&serde_json::Value>,
    ) -> Result<Visualization> {
        let viz = sqlx::query_as::<_, Visualization>(
            r#"
            UPDATE visualizations
            SET query_id = COALESCE($3, query_id),
                name = COALESCE($4, name),
                chart_type = COALESCE($5, chart_type),
                config = COALESCE($6, config),
                tags = COALESCE($7, tags),
                updated_at = NOW()
            WHERE id = $1 AND org_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(query_id)
        .bind(name)
        .bind(chart_type)
        .bind(config)
        .bind(tags)
        .fetch_one(&self.pool)
        .await?;

        Ok(viz)
    }

    // ==================== Dashboards ====================

    pub async fn create_dashboard(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        parameters: &serde_json::Value,
        tags: &serde_json::Value,
        created_by: Uuid,
    ) -> Result<Dashboard> {
        let dashboard = sqlx::query_as::<_, Dashboard>(
            r#"
            INSERT INTO dashboards (id, org_id, name, description, parameters, tags, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(name)
        .bind(description)
        .bind(parameters)
        .bind(tags)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(dashboard)
    }

    pub async fn get_dashboard(&self, id: Uuid, org_id: Uuid) -> Result<Dashboard> {
        let dashboard = sqlx::query_as::<_, Dashboard>(
            "SELECT * FROM dashboards WHERE id = $1 AND org_id = $2",
        )
        .bind(id)
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(dashboard)
    }

    pub async fn list_dashboards(&self, org_id: Uuid) -> Result<Vec<Dashboard>> {
        let dashboards = sqlx::query_as::<_, Dashboard>(
            "SELECT * FROM dashboards WHERE org_id = $1 ORDER BY created_at DESC",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(dashboards)
    }

    pub async fn list_dashboards_paginated(
        &self,
        org_id: Uuid,
        search: Option<String>,
        tags: Option<Vec<String>>,
        sort_column: &str,
        sort_direction: &str,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Dashboard>, i64)> {
        // Build ORDER BY clause (safe - validated by SortParams)
        let order_by = format!("{} {}", sort_column, sort_direction);

        // Get paginated results with conditional filtering
        let dashboards = match (search.as_ref(), tags.as_ref()) {
            // No filters
            (None, None) => {
                sqlx::query_as::<_, Dashboard>(&format!(
                    "SELECT * FROM dashboards WHERE org_id = $1 ORDER BY {} LIMIT $2 OFFSET $3",
                    order_by
                ))
                .bind(org_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            // Search only
            (Some(pattern), None) => {
                sqlx::query_as::<_, Dashboard>(&format!(
                    "SELECT * FROM dashboards
                     WHERE org_id = $1
                     AND (name ILIKE $2 OR description ILIKE $2)
                     ORDER BY {} LIMIT $3 OFFSET $4",
                    order_by
                ))
                .bind(org_id)
                .bind(pattern)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            // Tags only (use GIN index with @> operator)
            (None, Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as::<_, Dashboard>(&format!(
                    "SELECT * FROM dashboards
                     WHERE org_id = $1
                     AND tags @> $2
                     ORDER BY {} LIMIT $3 OFFSET $4",
                    order_by
                ))
                .bind(org_id)
                .bind(tags_json)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            // Both search and tags
            (Some(pattern), Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as::<_, Dashboard>(&format!(
                    "SELECT * FROM dashboards
                     WHERE org_id = $1
                     AND (name ILIKE $2 OR description ILIKE $2)
                     AND tags @> $3
                     ORDER BY {} LIMIT $4 OFFSET $5",
                    order_by
                ))
                .bind(org_id)
                .bind(pattern)
                .bind(tags_json)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
        };

        // Get total count with same filters
        let total: (i64,) = match (search.as_ref(), tags.as_ref()) {
            (None, None) => {
                sqlx::query_as("SELECT COUNT(*) FROM dashboards WHERE org_id = $1")
                    .bind(org_id)
                    .fetch_one(&self.pool)
                    .await?
            }
            (Some(pattern), None) => {
                sqlx::query_as(
                    "SELECT COUNT(*) FROM dashboards
                     WHERE org_id = $1
                     AND (name ILIKE $2 OR description ILIKE $2)",
                )
                .bind(org_id)
                .bind(pattern)
                .fetch_one(&self.pool)
                .await?
            }
            (None, Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as(
                    "SELECT COUNT(*) FROM dashboards
                     WHERE org_id = $1
                     AND tags @> $2",
                )
                .bind(org_id)
                .bind(tags_json)
                .fetch_one(&self.pool)
                .await?
            }
            (Some(pattern), Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as(
                    "SELECT COUNT(*) FROM dashboards
                     WHERE org_id = $1
                     AND (name ILIKE $2 OR description ILIKE $2)
                     AND tags @> $3",
                )
                .bind(org_id)
                .bind(pattern)
                .bind(tags_json)
                .fetch_one(&self.pool)
                .await?
            }
        };

        Ok((dashboards, total.0))
    }

    pub async fn delete_dashboard(&self, id: Uuid, org_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM dashboards WHERE id = $1 AND org_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_dashboard(
        &self,
        id: Uuid,
        org_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        parameters: Option<&serde_json::Value>,
        tags: Option<&serde_json::Value>,
    ) -> Result<Dashboard> {
        let dashboard = sqlx::query_as::<_, Dashboard>(
            r#"
            UPDATE dashboards
            SET name = COALESCE($3, name),
                description = COALESCE($4, description),
                parameters = COALESCE($5, parameters),
                tags = COALESCE($6, tags),
                updated_at = NOW()
            WHERE id = $1 AND org_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(name)
        .bind(description)
        .bind(parameters)
        .bind(tags)
        .fetch_one(&self.pool)
        .await?;

        Ok(dashboard)
    }

    // ==================== Tiles ====================

    pub async fn create_tile(
        &self,
        dashboard_id: Uuid,
        visualization_id: Uuid,
        title: Option<&str>,
        pos_x: i32,
        pos_y: i32,
        width: i32,
        height: i32,
        parameter_bindings: &serde_json::Value,
    ) -> Result<Tile> {
        let tile = sqlx::query_as::<_, Tile>(
            r#"
            INSERT INTO tiles (id, dashboard_id, visualization_id, title, pos_x, pos_y, width, height, parameter_bindings, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(dashboard_id)
        .bind(visualization_id)
        .bind(title)
        .bind(pos_x)
        .bind(pos_y)
        .bind(width)
        .bind(height)
        .bind(parameter_bindings)
        .fetch_one(&self.pool)
        .await?;

        Ok(tile)
    }

    pub async fn list_tiles(&self, dashboard_id: Uuid) -> Result<Vec<Tile>> {
        let tiles = sqlx::query_as::<_, Tile>(
            "SELECT * FROM tiles WHERE dashboard_id = $1 ORDER BY pos_y, pos_x",
        )
        .bind(dashboard_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(tiles)
    }

    pub async fn delete_tile(&self, id: Uuid, dashboard_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM tiles WHERE id = $1 AND dashboard_id = $2")
            .bind(id)
            .bind(dashboard_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_tile(
        &self,
        id: Uuid,
        dashboard_id: Uuid,
        title: Option<&str>,
        pos_x: Option<i32>,
        pos_y: Option<i32>,
        width: Option<i32>,
        height: Option<i32>,
        parameter_bindings: Option<&serde_json::Value>,
    ) -> Result<Tile> {
        let tile = sqlx::query_as::<_, Tile>(
            r#"
            UPDATE tiles
            SET title = COALESCE($3, title),
                pos_x = COALESCE($4, pos_x),
                pos_y = COALESCE($5, pos_y),
                width = COALESCE($6, width),
                height = COALESCE($7, height),
                parameter_bindings = COALESCE($8, parameter_bindings),
                updated_at = NOW()
            WHERE id = $1 AND dashboard_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(dashboard_id)
        .bind(title)
        .bind(pos_x)
        .bind(pos_y)
        .bind(width)
        .bind(height)
        .bind(parameter_bindings)
        .fetch_one(&self.pool)
        .await?;

        Ok(tile)
    }

    // ==================== Schedules ====================

    /// Calculate the next run time from a cron expression
    /// Handles both 5-part (standard cron) and 6-part (with seconds) expressions
    fn calculate_next_run(
        cron_expression: &str,
        enabled: bool,
    ) -> Option<chrono::DateTime<chrono::Utc>> {
        if !enabled {
            return None;
        }

        use std::str::FromStr;

        // The cron crate expects 6 or 7 parts (with seconds)
        // Standard cron uses 5 parts: minute hour day-of-month month day-of-week
        // Convert 5-part to 6-part by prepending "0 " for seconds
        let parts: Vec<&str> = cron_expression.split_whitespace().collect();
        let expr = if parts.len() == 5 {
            format!("0 {}", cron_expression)
        } else {
            cron_expression.to_string()
        };

        let schedule = cron::Schedule::from_str(&expr).ok()?;
        schedule.upcoming(chrono::Utc).next()
    }

    pub async fn create_schedule(
        &self,
        org_id: Uuid,
        query_id: Uuid,
        name: &str,
        cron_expression: &str,
        parameters: &serde_json::Value,
        tags: &serde_json::Value,
        enabled: bool,
        created_by: Uuid,
    ) -> Result<Schedule> {
        let next_run_at = Self::calculate_next_run(cron_expression, enabled);

        let schedule = sqlx::query_as::<_, Schedule>(
            r#"
            INSERT INTO schedules (id, org_id, query_id, name, cron_expression, parameters, tags, enabled, next_run_at, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(query_id)
        .bind(name)
        .bind(cron_expression)
        .bind(parameters)
        .bind(tags)
        .bind(enabled)
        .bind(next_run_at)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(schedule)
    }

    pub async fn list_schedules(&self, org_id: Uuid) -> Result<Vec<Schedule>> {
        let schedules = sqlx::query_as::<_, Schedule>(
            "SELECT * FROM schedules WHERE org_id = $1 ORDER BY created_at DESC",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(schedules)
    }

    pub async fn list_schedules_paginated(
        &self,
        org_id: Uuid,
        search: Option<String>,
        tags: Option<Vec<String>>,
        enabled: Option<bool>,
        sort_column: &str,
        sort_direction: &str,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Schedule>, i64)> {
        let order_by = format!("{} {}", sort_column, sort_direction);

        // Build query based on filter combinations
        let schedules = match (search.as_ref(), tags.as_ref(), enabled) {
            (None, None, None) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM schedules WHERE org_id = $1 ORDER BY {} LIMIT $2 OFFSET $3",
                    order_by
                ))
                .bind(org_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(pattern), None, None) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM schedules WHERE org_id = $1 AND name ILIKE $2 ORDER BY {} LIMIT $3 OFFSET $4",
                    order_by
                ))
                .bind(org_id)
                .bind(pattern)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(tag_list), None) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as(&format!(
                    "SELECT * FROM schedules WHERE org_id = $1 AND tags @> $2 ORDER BY {} LIMIT $3 OFFSET $4",
                    order_by
                ))
                .bind(org_id)
                .bind(tags_json)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (None, None, Some(en)) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM schedules WHERE org_id = $1 AND enabled = $2 ORDER BY {} LIMIT $3 OFFSET $4",
                    order_by
                ))
                .bind(org_id)
                .bind(en)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(pattern), Some(tag_list), None) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as(&format!(
                    "SELECT * FROM schedules WHERE org_id = $1 AND name ILIKE $2 AND tags @> $3 ORDER BY {} LIMIT $4 OFFSET $5",
                    order_by
                ))
                .bind(org_id)
                .bind(pattern)
                .bind(tags_json)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(pattern), None, Some(en)) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM schedules WHERE org_id = $1 AND name ILIKE $2 AND enabled = $3 ORDER BY {} LIMIT $4 OFFSET $5",
                    order_by
                ))
                .bind(org_id)
                .bind(pattern)
                .bind(en)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(tag_list), Some(en)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as(&format!(
                    "SELECT * FROM schedules WHERE org_id = $1 AND tags @> $2 AND enabled = $3 ORDER BY {} LIMIT $4 OFFSET $5",
                    order_by
                ))
                .bind(org_id)
                .bind(tags_json)
                .bind(en)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(pattern), Some(tag_list), Some(en)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as(&format!(
                    "SELECT * FROM schedules WHERE org_id = $1 AND name ILIKE $2 AND tags @> $3 AND enabled = $4 ORDER BY {} LIMIT $5 OFFSET $6",
                    order_by
                ))
                .bind(org_id)
                .bind(pattern)
                .bind(tags_json)
                .bind(en)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
        };

        // Get total count with same filter conditions
        let total: (i64,) = match (search.as_ref(), tags.as_ref(), enabled) {
            (None, None, None) => {
                sqlx::query_as("SELECT COUNT(*) FROM schedules WHERE org_id = $1")
                    .bind(org_id)
                    .fetch_one(&self.pool)
                    .await?
            }
            (Some(pattern), None, None) => {
                sqlx::query_as("SELECT COUNT(*) FROM schedules WHERE org_id = $1 AND name ILIKE $2")
                    .bind(org_id)
                    .bind(pattern)
                    .fetch_one(&self.pool)
                    .await?
            }
            (None, Some(tag_list), None) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as("SELECT COUNT(*) FROM schedules WHERE org_id = $1 AND tags @> $2")
                    .bind(org_id)
                    .bind(tags_json)
                    .fetch_one(&self.pool)
                    .await?
            }
            (None, None, Some(en)) => {
                sqlx::query_as("SELECT COUNT(*) FROM schedules WHERE org_id = $1 AND enabled = $2")
                    .bind(org_id)
                    .bind(en)
                    .fetch_one(&self.pool)
                    .await?
            }
            (Some(pattern), Some(tag_list), None) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as("SELECT COUNT(*) FROM schedules WHERE org_id = $1 AND name ILIKE $2 AND tags @> $3")
                    .bind(org_id)
                    .bind(pattern)
                    .bind(tags_json)
                    .fetch_one(&self.pool)
                    .await?
            }
            (Some(pattern), None, Some(en)) => {
                sqlx::query_as("SELECT COUNT(*) FROM schedules WHERE org_id = $1 AND name ILIKE $2 AND enabled = $3")
                    .bind(org_id)
                    .bind(pattern)
                    .bind(en)
                    .fetch_one(&self.pool)
                    .await?
            }
            (None, Some(tag_list), Some(en)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as("SELECT COUNT(*) FROM schedules WHERE org_id = $1 AND tags @> $2 AND enabled = $3")
                    .bind(org_id)
                    .bind(tags_json)
                    .bind(en)
                    .fetch_one(&self.pool)
                    .await?
            }
            (Some(pattern), Some(tag_list), Some(en)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as("SELECT COUNT(*) FROM schedules WHERE org_id = $1 AND name ILIKE $2 AND tags @> $3 AND enabled = $4")
                    .bind(org_id)
                    .bind(pattern)
                    .bind(tags_json)
                    .bind(en)
                    .fetch_one(&self.pool)
                    .await?
            }
        };

        Ok((schedules, total.0))
    }

    pub async fn get_due_schedules(&self) -> Result<Vec<Schedule>> {
        let schedules = sqlx::query_as::<_, Schedule>(
            "SELECT * FROM schedules WHERE enabled = true AND (next_run_at IS NULL OR next_run_at <= NOW())",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(schedules)
    }

    pub async fn update_schedule_last_run(
        &self,
        id: Uuid,
        cron_expression: &str,
        enabled: bool,
    ) -> Result<()> {
        let next_run_at = Self::calculate_next_run(cron_expression, enabled);

        sqlx::query(
            "UPDATE schedules SET last_run_at = NOW(), next_run_at = $2, updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .bind(next_run_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_schedule(&self, id: Uuid, org_id: Uuid) -> Result<Schedule> {
        let schedule =
            sqlx::query_as::<_, Schedule>("SELECT * FROM schedules WHERE id = $1 AND org_id = $2")
                .bind(id)
                .bind(org_id)
                .fetch_optional(&self.pool)
                .await?
                .ok_or_else(|| Error::NotFound("Schedule not found".into()))?;

        Ok(schedule)
    }

    pub async fn update_schedule(
        &self,
        id: Uuid,
        org_id: Uuid,
        name: Option<&str>,
        cron_expression: Option<&str>,
        parameters: Option<&serde_json::Value>,
        tags: Option<&serde_json::Value>,
        enabled: Option<bool>,
    ) -> Result<Schedule> {
        // Get existing schedule to determine current values for next_run calculation
        let existing = self.get_schedule(id, org_id).await?;

        let new_cron = cron_expression.unwrap_or(&existing.cron_expression);
        let new_enabled = enabled.unwrap_or(existing.enabled);
        let next_run_at = Self::calculate_next_run(new_cron, new_enabled);

        // Use a simpler approach: update all fields, using COALESCE for optional ones
        let schedule = sqlx::query_as::<_, Schedule>(
            r#"
            UPDATE schedules SET
                name = COALESCE($3, name),
                cron_expression = COALESCE($4, cron_expression),
                parameters = COALESCE($5, parameters),
                tags = COALESCE($6, tags),
                enabled = COALESCE($7, enabled),
                next_run_at = $8,
                updated_at = NOW()
            WHERE id = $1 AND org_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(name)
        .bind(cron_expression)
        .bind(parameters)
        .bind(tags)
        .bind(enabled)
        .bind(next_run_at)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound("Schedule not found".into()))?;

        Ok(schedule)
    }

    pub async fn delete_schedule(&self, id: Uuid, org_id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM schedules WHERE id = $1 AND org_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound("Schedule not found".into()));
        }

        Ok(())
    }

    pub async fn enable_schedule(&self, id: Uuid, org_id: Uuid) -> Result<Schedule> {
        // First get the schedule to access the cron expression
        let existing = self.get_schedule(id, org_id).await?;
        let next_run_at = Self::calculate_next_run(&existing.cron_expression, true);

        let schedule = sqlx::query_as::<_, Schedule>(
            "UPDATE schedules SET enabled = true, next_run_at = $3, updated_at = NOW() WHERE id = $1 AND org_id = $2 RETURNING *",
        )
        .bind(id)
        .bind(org_id)
        .bind(next_run_at)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound("Schedule not found".into()))?;

        Ok(schedule)
    }

    pub async fn disable_schedule(&self, id: Uuid, org_id: Uuid) -> Result<Schedule> {
        let schedule = sqlx::query_as::<_, Schedule>(
            "UPDATE schedules SET enabled = false, next_run_at = NULL, updated_at = NOW() WHERE id = $1 AND org_id = $2 RETURNING *",
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound("Schedule not found".into()))?;

        Ok(schedule)
    }

    // ==================== Canvases ====================

    pub async fn create_canvas(
        &self,
        org_id: Uuid,
        name: &str,
        time_preset: &str,
        time_offset: i64,
        time_custom_start: Option<chrono::DateTime<chrono::Utc>>,
        time_custom_end: Option<chrono::DateTime<chrono::Utc>>,
        live: bool,
        created_by: Uuid,
    ) -> Result<Canvas> {
        let canvas = sqlx::query_as::<_, Canvas>(
            r#"
            INSERT INTO canvases (id, org_id, name, time_preset, time_offset, time_custom_start, time_custom_end, live, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(name)
        .bind(time_preset)
        .bind(time_offset)
        .bind(time_custom_start)
        .bind(time_custom_end)
        .bind(live)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(canvas)
    }

    pub async fn get_canvas(&self, id: Uuid, org_id: Uuid) -> Result<Canvas> {
        let canvas = sqlx::query_as::<_, Canvas>(
            "SELECT * FROM canvases WHERE id = $1 AND org_id = $2",
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound("Canvas not found".into()))?;

        Ok(canvas)
    }

    pub async fn list_canvases(&self, org_id: Uuid) -> Result<Vec<Canvas>> {
        let canvases = sqlx::query_as::<_, Canvas>(
            "SELECT * FROM canvases WHERE org_id = $1 ORDER BY updated_at DESC",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(canvases)
    }

    pub async fn list_canvases_paginated(
        &self,
        org_id: Uuid,
        search: Option<String>,
        tags: Option<Vec<String>>,
        sort_column: &str,
        sort_direction: &str,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Canvas>, i64)> {
        let order_by = format!("{} {}", sort_column, sort_direction);

        // Build query based on filter combinations
        let canvases = match (search.as_ref(), tags.as_ref()) {
            (None, None) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM canvases WHERE org_id = $1 ORDER BY {} LIMIT $2 OFFSET $3",
                    order_by
                ))
                .bind(org_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(pattern), None) => {
                sqlx::query_as(&format!(
                    "SELECT * FROM canvases WHERE org_id = $1 AND name ILIKE $2 ORDER BY {} LIMIT $3 OFFSET $4",
                    order_by
                ))
                .bind(org_id)
                .bind(pattern)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as(&format!(
                    "SELECT * FROM canvases WHERE org_id = $1 AND tags @> $2 ORDER BY {} LIMIT $3 OFFSET $4",
                    order_by
                ))
                .bind(org_id)
                .bind(tags_json)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(pattern), Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as(&format!(
                    "SELECT * FROM canvases WHERE org_id = $1 AND name ILIKE $2 AND tags @> $3 ORDER BY {} LIMIT $4 OFFSET $5",
                    order_by
                ))
                .bind(org_id)
                .bind(pattern)
                .bind(tags_json)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
        };

        // Get total count with same filter conditions
        let total: (i64,) = match (search.as_ref(), tags.as_ref()) {
            (None, None) => {
                sqlx::query_as("SELECT COUNT(*) FROM canvases WHERE org_id = $1")
                    .bind(org_id)
                    .fetch_one(&self.pool)
                    .await?
            }
            (Some(pattern), None) => {
                sqlx::query_as("SELECT COUNT(*) FROM canvases WHERE org_id = $1 AND name ILIKE $2")
                    .bind(org_id)
                    .bind(pattern)
                    .fetch_one(&self.pool)
                    .await?
            }
            (None, Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as("SELECT COUNT(*) FROM canvases WHERE org_id = $1 AND tags @> $2")
                    .bind(org_id)
                    .bind(tags_json)
                    .fetch_one(&self.pool)
                    .await?
            }
            (Some(pattern), Some(tag_list)) => {
                let tags_json = serde_json::to_value(tag_list).expect("Vec<String> to JSON should never fail");
                sqlx::query_as("SELECT COUNT(*) FROM canvases WHERE org_id = $1 AND name ILIKE $2 AND tags @> $3")
                    .bind(org_id)
                    .bind(pattern)
                    .bind(tags_json)
                    .fetch_one(&self.pool)
                    .await?
            }
        };

        Ok((canvases, total.0))
    }

    pub async fn update_canvas(
        &self,
        id: Uuid,
        org_id: Uuid,
        name: Option<&str>,
        time_preset: Option<&str>,
        time_offset: Option<i64>,
        time_custom_start: Option<chrono::DateTime<chrono::Utc>>,
        time_custom_end: Option<chrono::DateTime<chrono::Utc>>,
        live: Option<bool>,
    ) -> Result<Canvas> {
        let canvas = sqlx::query_as::<_, Canvas>(
            r#"
            UPDATE canvases
            SET name = COALESCE($3, name),
                time_preset = COALESCE($4, time_preset),
                time_offset = COALESCE($5, time_offset),
                time_custom_start = COALESCE($6, time_custom_start),
                time_custom_end = COALESCE($7, time_custom_end),
                live = COALESCE($8, live),
                updated_at = NOW()
            WHERE id = $1 AND org_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(name)
        .bind(time_preset)
        .bind(time_offset)
        .bind(time_custom_start)
        .bind(time_custom_end)
        .bind(live)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound("Canvas not found".into()))?;

        Ok(canvas)
    }

    pub async fn delete_canvas(&self, id: Uuid, org_id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM canvases WHERE id = $1 AND org_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound("Canvas not found".into()));
        }

        Ok(())
    }

    // ==================== Canvas Nodes ====================

    pub async fn create_canvas_node(
        &self,
        canvas_id: Uuid,
        node_type: &str,
        title: &str,
        pos_x: f64,
        pos_y: f64,
        width: f64,
        height: f64,
        meta: &serde_json::Value,
    ) -> Result<CanvasNode> {
        let node = sqlx::query_as::<_, CanvasNode>(
            r#"
            INSERT INTO canvas_nodes (id, canvas_id, node_type, title, pos_x, pos_y, width, height, meta, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(canvas_id)
        .bind(node_type)
        .bind(title)
        .bind(pos_x)
        .bind(pos_y)
        .bind(width)
        .bind(height)
        .bind(meta)
        .fetch_one(&self.pool)
        .await?;

        Ok(node)
    }

    pub async fn get_canvas_node(&self, id: Uuid, canvas_id: Uuid) -> Result<CanvasNode> {
        let node = sqlx::query_as::<_, CanvasNode>(
            "SELECT * FROM canvas_nodes WHERE id = $1 AND canvas_id = $2",
        )
        .bind(id)
        .bind(canvas_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound("Canvas node not found".into()))?;

        Ok(node)
    }

    pub async fn list_canvas_nodes(&self, canvas_id: Uuid) -> Result<Vec<CanvasNode>> {
        let nodes = sqlx::query_as::<_, CanvasNode>(
            "SELECT * FROM canvas_nodes WHERE canvas_id = $1 ORDER BY created_at ASC",
        )
        .bind(canvas_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(nodes)
    }

    pub async fn update_canvas_node(
        &self,
        id: Uuid,
        canvas_id: Uuid,
        title: Option<&str>,
        pos_x: Option<f64>,
        pos_y: Option<f64>,
        width: Option<f64>,
        height: Option<f64>,
        meta: Option<&serde_json::Value>,
    ) -> Result<CanvasNode> {
        let node = sqlx::query_as::<_, CanvasNode>(
            r#"
            UPDATE canvas_nodes
            SET title = COALESCE($3, title),
                pos_x = COALESCE($4, pos_x),
                pos_y = COALESCE($5, pos_y),
                width = COALESCE($6, width),
                height = COALESCE($7, height),
                meta = COALESCE($8, meta),
                updated_at = NOW()
            WHERE id = $1 AND canvas_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(canvas_id)
        .bind(title)
        .bind(pos_x)
        .bind(pos_y)
        .bind(width)
        .bind(height)
        .bind(meta)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound("Canvas node not found".into()))?;

        Ok(node)
    }

    pub async fn delete_canvas_node(&self, id: Uuid, canvas_id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM canvas_nodes WHERE id = $1 AND canvas_id = $2")
            .bind(id)
            .bind(canvas_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound("Canvas node not found".into()));
        }

        Ok(())
    }

    // ==================== Canvas Edges ====================

    pub async fn create_canvas_edge(
        &self,
        canvas_id: Uuid,
        from_node_id: Uuid,
        to_node_id: Uuid,
        label: &str,
    ) -> Result<CanvasEdge> {
        let edge = sqlx::query_as::<_, CanvasEdge>(
            r#"
            INSERT INTO canvas_edges (id, canvas_id, from_node_id, to_node_id, label, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(canvas_id)
        .bind(from_node_id)
        .bind(to_node_id)
        .bind(label)
        .fetch_one(&self.pool)
        .await?;

        Ok(edge)
    }

    pub async fn get_canvas_edge(&self, id: Uuid, canvas_id: Uuid) -> Result<CanvasEdge> {
        let edge = sqlx::query_as::<_, CanvasEdge>(
            "SELECT * FROM canvas_edges WHERE id = $1 AND canvas_id = $2",
        )
        .bind(id)
        .bind(canvas_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound("Canvas edge not found".into()))?;

        Ok(edge)
    }

    pub async fn list_canvas_edges(&self, canvas_id: Uuid) -> Result<Vec<CanvasEdge>> {
        let edges = sqlx::query_as::<_, CanvasEdge>(
            "SELECT * FROM canvas_edges WHERE canvas_id = $1 ORDER BY created_at ASC",
        )
        .bind(canvas_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(edges)
    }

    pub async fn update_canvas_edge(
        &self,
        id: Uuid,
        canvas_id: Uuid,
        label: Option<&str>,
    ) -> Result<CanvasEdge> {
        let edge = sqlx::query_as::<_, CanvasEdge>(
            r#"
            UPDATE canvas_edges
            SET label = COALESCE($3, label),
                updated_at = NOW()
            WHERE id = $1 AND canvas_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(canvas_id)
        .bind(label)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound("Canvas edge not found".into()))?;

        Ok(edge)
    }

    pub async fn delete_canvas_edge(&self, id: Uuid, canvas_id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM canvas_edges WHERE id = $1 AND canvas_id = $2")
            .bind(id)
            .bind(canvas_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound("Canvas edge not found".into()));
        }

        Ok(())
    }
}
