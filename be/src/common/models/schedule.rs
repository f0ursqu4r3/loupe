use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A schedule for periodic query refresh
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Schedule {
    pub id: Uuid,
    pub org_id: Uuid,
    pub query_id: Uuid,
    pub name: String,
    /// Cron expression (e.g., "0 */15 * * *" for every 15 minutes)
    pub cron_expression: String,
    /// JSON object of parameter values to use
    pub parameters: serde_json::Value,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// DTOs
#[derive(Debug, Deserialize)]
pub struct CreateScheduleRequest {
    pub query_id: Uuid,
    pub name: String,
    pub cron_expression: String,
    #[serde(default)]
    pub parameters: serde_json::Value,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct UpdateScheduleRequest {
    pub name: Option<String>,
    pub cron_expression: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ScheduleResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub query_id: Uuid,
    pub name: String,
    pub cron_expression: String,
    pub parameters: serde_json::Value,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Schedule> for ScheduleResponse {
    fn from(s: Schedule) -> Self {
        Self {
            id: s.id,
            org_id: s.org_id,
            query_id: s.query_id,
            name: s.name,
            cron_expression: s.cron_expression,
            parameters: s.parameters,
            enabled: s.enabled,
            last_run_at: s.last_run_at,
            next_run_at: s.next_run_at,
            created_by: s.created_by,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}
