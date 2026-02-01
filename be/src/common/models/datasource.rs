use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DatasourceType {
    Postgres,
}

/// A datasource connection definition
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Datasource {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub ds_type: DatasourceType,
    /// Encrypted connection string
    #[serde(skip_serializing)]
    pub connection_string_encrypted: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// DTOs
#[derive(Debug, Deserialize, Validate)]
pub struct CreateDatasourceRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    #[validate(custom(function = "crate::validation::validate_name", message = "Name contains invalid characters"))]
    pub name: String,

    #[serde(default = "default_ds_type")]
    pub ds_type: DatasourceType,

    #[validate(custom(function = "crate::validation::validate_connection_string", message = "Invalid connection string"))]
    pub connection_string: String,
}

fn default_ds_type() -> DatasourceType {
    DatasourceType::Postgres
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDatasourceRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    #[validate(custom(function = "crate::validation::validate_name", message = "Name contains invalid characters"))]
    pub name: Option<String>,

    #[validate(custom(function = "crate::validation::validate_connection_string", message = "Invalid connection string"))]
    pub connection_string: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DatasourceResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub ds_type: DatasourceType,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Datasource> for DatasourceResponse {
    fn from(ds: Datasource) -> Self {
        Self {
            id: ds.id,
            org_id: ds.org_id,
            name: ds.name,
            ds_type: ds.ds_type,
            created_by: ds.created_by,
            created_at: ds.created_at,
            updated_at: ds.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
    pub latency_ms: Option<u64>,
}
