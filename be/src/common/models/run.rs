use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

/// An execution instance of a query
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Run {
    pub id: Uuid,
    pub org_id: Uuid,
    pub query_id: Uuid,
    pub datasource_id: Uuid,
    /// The SQL that was actually executed (with parameters resolved)
    pub executed_sql: String,
    /// JSON object of parameter values
    pub parameters: serde_json::Value,
    pub status: RunStatus,
    /// Runner instance that claimed this run
    pub runner_id: Option<String>,
    pub timeout_seconds: i32,
    pub max_rows: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    /// Number of times this run has been retried
    pub retry_count: i32,
    /// Maximum number of retry attempts allowed
    pub max_retries: i32,
    /// Timestamp when this run is eligible for retry
    pub next_retry_at: Option<DateTime<Utc>>,
}

/// A run that exceeded max retries and was moved to the dead letter queue
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct RunFailure {
    pub id: Uuid,
    pub run_id: Uuid,
    pub org_id: Uuid,
    pub query_id: Uuid,
    pub datasource_id: Uuid,
    pub executed_sql: String,
    pub parameters: serde_json::Value,
    pub error_message: String,
    pub retry_count: i32,
    pub max_retries: i32,
    pub first_failed_at: DateTime<Utc>,
    pub last_failed_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// The result of a completed run
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct RunResult {
    pub id: Uuid,
    pub run_id: Uuid,
    /// JSON array of column definitions
    pub columns: serde_json::Value,
    /// JSON array of row arrays
    pub rows: serde_json::Value,
    pub row_count: i64,
    pub byte_count: i64,
    pub execution_time_ms: i64,
    pub created_at: DateTime<Utc>,
    /// TTL for cleanup
    pub expires_at: Option<DateTime<Utc>>,
}

// DTOs
#[derive(Debug, Deserialize)]
pub struct CreateRunRequest {
    pub query_id: Uuid,
    #[serde(default)]
    pub parameters: serde_json::Value,
    pub timeout_seconds: Option<i32>,
    pub max_rows: Option<i32>,
}

/// Request to execute ad-hoc SQL (creates ephemeral query + run)
#[derive(Debug, Deserialize)]
pub struct ExecuteAdHocRequest {
    pub datasource_id: Uuid,
    pub sql: String,
    #[serde(default)]
    pub parameters: serde_json::Value,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: i32,
    #[serde(default = "default_max_rows")]
    pub max_rows: i32,
}

fn default_timeout() -> i32 {
    30
}

fn default_max_rows() -> i32 {
    10000
}

#[derive(Debug, Serialize)]
pub struct RunResponse {
    pub id: Uuid,
    pub query_id: Uuid,
    pub status: RunStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<Run> for RunResponse {
    fn from(r: Run) -> Self {
        Self {
            id: r.id,
            query_id: r.query_id,
            status: r.status,
            started_at: r.started_at,
            completed_at: r.completed_at,
            error_message: r.error_message,
            created_at: r.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: String,
}

#[derive(Debug, Serialize)]
pub struct RunResultResponse {
    pub run_id: Uuid,
    pub columns: Vec<ColumnDef>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: i64,
    pub execution_time_ms: i64,
}

impl From<RunResult> for RunResultResponse {
    fn from(r: RunResult) -> Self {
        let columns = serde_json::from_value(r.columns.clone()).unwrap_or_else(|e| {
            tracing::error!("Failed to deserialize columns for run {}: {}", r.run_id, e);
            vec![]
        });
        let rows = serde_json::from_value(r.rows.clone()).unwrap_or_else(|e| {
            tracing::error!("Failed to deserialize rows for run {}: {}", r.run_id, e);
            vec![]
        });
        Self {
            run_id: r.run_id,
            columns,
            rows,
            row_count: r.row_count,
            execution_time_ms: r.execution_time_ms,
        }
    }
}
