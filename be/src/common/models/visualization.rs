use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ChartType {
    Table,
    Line,
    Bar,
    SingleStat,
}

/// A visualization configuration for a query result
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Visualization {
    pub id: Uuid,
    pub org_id: Uuid,
    pub query_id: Uuid,
    pub name: String,
    pub chart_type: ChartType,
    /// JSON config for chart (axes, colors, etc.)
    pub config: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// DTOs
#[derive(Debug, Deserialize)]
pub struct CreateVisualizationRequest {
    pub query_id: Uuid,
    pub name: String,
    pub chart_type: ChartType,
    #[serde(default)]
    pub config: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVisualizationRequest {
    pub name: Option<String>,
    pub chart_type: Option<ChartType>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct VisualizationResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub query_id: Uuid,
    pub name: String,
    pub chart_type: ChartType,
    pub config: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Visualization> for VisualizationResponse {
    fn from(v: Visualization) -> Self {
        Self {
            id: v.id,
            org_id: v.org_id,
            query_id: v.query_id,
            name: v.name,
            chart_type: v.chart_type,
            config: v.config,
            created_by: v.created_by,
            created_at: v.created_at,
            updated_at: v.updated_at,
        }
    }
}
