use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

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
    /// JSON array of tag strings
    pub tags: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// DTOs with validation
#[derive(Debug, Deserialize, Validate)]
pub struct CreateQueryRequest {
    pub datasource_id: Uuid,

    #[validate(length(min = 1, max = 255, message = "Query name must be between 1 and 255 characters"))]
    pub name: String,

    #[validate(length(max = 2000, message = "Description must be less than 2000 characters"))]
    pub description: Option<String>,

    #[validate(length(min = 1, max = 100_000, message = "SQL must be between 1 and 100,000 characters"))]
    pub sql: String,

    #[serde(default)]
    #[validate(length(max = 50, message = "Maximum 50 parameters allowed"))]
    pub parameters: Vec<ParamDef>,

    #[serde(default = "default_timeout")]
    #[validate(range(min = 1, max = 300, message = "Timeout must be between 1 and 300 seconds"))]
    pub timeout_seconds: i32,

    #[serde(default = "default_max_rows")]
    #[validate(range(min = 1, max = 100_000, message = "Max rows must be between 1 and 100,000"))]
    pub max_rows: i32,

    #[serde(default)]
    #[validate(length(max = 50, message = "Maximum 50 tags allowed"))]
    pub tags: Vec<String>,
}

fn default_timeout() -> i32 {
    30
}

fn default_max_rows() -> i32 {
    10000
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateQueryRequest {
    #[validate(length(min = 1, max = 255, message = "Query name must be between 1 and 255 characters"))]
    pub name: Option<String>,

    #[validate(length(max = 2000, message = "Description must be less than 2000 characters"))]
    pub description: Option<String>,

    #[validate(length(min = 1, max = 100_000, message = "SQL must be between 1 and 100,000 characters"))]
    pub sql: Option<String>,

    #[validate(length(max = 50, message = "Maximum 50 parameters allowed"))]
    pub parameters: Option<Vec<ParamDef>>,

    #[validate(range(min = 1, max = 300, message = "Timeout must be between 1 and 300 seconds"))]
    pub timeout_seconds: Option<i32>,

    #[validate(range(min = 1, max = 100_000, message = "Max rows must be between 1 and 100,000"))]
    pub max_rows: Option<i32>,

    #[validate(length(max = 50, message = "Maximum 50 tags allowed"))]
    pub tags: Option<Vec<String>>,
}

/// Export format for a query (excludes org-specific IDs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryExport {
    pub name: String,
    pub description: Option<String>,
    pub sql: String,
    pub parameters: Vec<ParamDef>,
    pub timeout_seconds: i32,
    pub max_rows: i32,
    pub tags: Vec<String>,
    /// Datasource name for matching on import
    pub datasource_name: Option<String>,
}

/// Request to import queries
#[derive(Debug, Deserialize)]
pub struct ImportQueriesRequest {
    pub queries: Vec<QueryExport>,
    /// Target datasource ID for all imported queries
    pub datasource_id: Uuid,
    /// Whether to skip queries that already exist (by name)
    #[serde(default = "default_skip_duplicates")]
    pub skip_duplicates: bool,
}

fn default_skip_duplicates() -> bool {
    true
}

/// Response from import operation
#[derive(Debug, Serialize)]
pub struct ImportQueriesResponse {
    pub imported: usize,
    pub skipped: usize,
    pub skipped_names: Vec<String>,
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
    pub tags: Vec<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Query> for QueryResponse {
    fn from(q: Query) -> Self {
        let parameters: Vec<ParamDef> = serde_json::from_value(q.parameters).unwrap_or_default();
        let tags: Vec<String> = serde_json::from_value(q.tags).unwrap_or_default();
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
            tags,
            created_by: q.created_by,
            created_at: q.created_at,
            updated_at: q.updated_at,
        }
    }
}
