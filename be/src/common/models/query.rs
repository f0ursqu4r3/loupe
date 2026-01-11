use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Parameter type for query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    String,
    Number,
    Boolean,
    Date,
    DateTime,
}

/// A parameter definition for a query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDef {
    pub name: String,
    pub param_type: ParamType,
    pub default: Option<serde_json::Value>,
    pub required: bool,
}

/// A saved SQL query
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Query {
    pub id: Uuid,
    pub org_id: Uuid,
    pub datasource_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sql: String,
    /// JSON array of ParamDef
    pub parameters: serde_json::Value,
    /// Default timeout in seconds
    pub timeout_seconds: i32,
    /// Default max rows
    pub max_rows: i32,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// DTOs
#[derive(Debug, Deserialize)]
pub struct CreateQueryRequest {
    pub datasource_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sql: String,
    #[serde(default)]
    pub parameters: Vec<ParamDef>,
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

#[derive(Debug, Deserialize)]
pub struct UpdateQueryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub sql: Option<String>,
    pub parameters: Option<Vec<ParamDef>>,
    pub timeout_seconds: Option<i32>,
    pub max_rows: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub datasource_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sql: String,
    pub parameters: Vec<ParamDef>,
    pub timeout_seconds: i32,
    pub max_rows: i32,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Query> for QueryResponse {
    fn from(q: Query) -> Self {
        let parameters: Vec<ParamDef> =
            serde_json::from_value(q.parameters).unwrap_or_default();
        Self {
            id: q.id,
            org_id: q.org_id,
            datasource_id: q.datasource_id,
            name: q.name,
            description: q.description,
            sql: q.sql,
            parameters,
            timeout_seconds: q.timeout_seconds,
            max_rows: q.max_rows,
            created_by: q.created_by,
            created_at: q.created_at,
            updated_at: q.updated_at,
        }
    }
}
