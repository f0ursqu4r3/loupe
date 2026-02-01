use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// A dashboard containing multiple tiles
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Dashboard {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    /// JSON array of global parameter bindings
    pub parameters: serde_json::Value,
    /// JSON array of tag strings
    pub tags: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A tile on a dashboard (displays a visualization)
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Tile {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub visualization_id: Uuid,
    pub title: Option<String>,
    /// Grid position x
    pub pos_x: i32,
    /// Grid position y
    pub pos_y: i32,
    /// Grid width
    pub width: i32,
    /// Grid height
    pub height: i32,
    /// JSON object mapping viz params to dashboard params
    pub parameter_bindings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// DTOs
#[derive(Debug, Deserialize, Validate)]
pub struct CreateDashboardRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    #[validate(custom(function = "crate::validation::validate_name", message = "Name contains invalid characters"))]
    pub name: String,

    #[validate(length(max = 2000, message = "Description must be less than 2000 characters"))]
    #[validate(custom(function = "crate::validation::validate_description"))]
    pub description: Option<String>,

    #[serde(default)]
    pub parameters: serde_json::Value,

    #[serde(default)]
    #[validate(length(max = 50, message = "Maximum 50 tags allowed"))]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDashboardRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    #[validate(custom(function = "crate::validation::validate_name", message = "Name contains invalid characters"))]
    pub name: Option<String>,

    #[validate(length(max = 2000, message = "Description must be less than 2000 characters"))]
    #[validate(custom(function = "crate::validation::validate_description"))]
    pub description: Option<String>,

    pub parameters: Option<serde_json::Value>,

    #[validate(length(max = 50, message = "Maximum 50 tags allowed"))]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTileRequest {
    pub visualization_id: Uuid,

    #[validate(length(max = 255, message = "Title must be less than 255 characters"))]
    pub title: Option<String>,

    #[serde(default)]
    #[validate(range(min = -1000, max = 1000, message = "Position must be between -1000 and 1000"))]
    pub pos_x: i32,

    #[serde(default)]
    #[validate(range(min = -1000, max = 1000, message = "Position must be between -1000 and 1000"))]
    pub pos_y: i32,

    #[serde(default = "default_tile_size")]
    #[validate(range(min = 1, max = 100, message = "Width must be between 1 and 100"))]
    pub width: i32,

    #[serde(default = "default_tile_size")]
    #[validate(range(min = 1, max = 100, message = "Height must be between 1 and 100"))]
    pub height: i32,

    #[serde(default)]
    pub parameter_bindings: serde_json::Value,
}

fn default_tile_size() -> i32 {
    4
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTileRequest {
    #[validate(length(max = 255, message = "Title must be less than 255 characters"))]
    pub title: Option<String>,

    #[validate(range(min = -1000, max = 1000, message = "Position must be between -1000 and 1000"))]
    pub pos_x: Option<i32>,

    #[validate(range(min = -1000, max = 1000, message = "Position must be between -1000 and 1000"))]
    pub pos_y: Option<i32>,

    #[validate(range(min = 1, max = 100, message = "Width must be between 1 and 100"))]
    pub width: Option<i32>,

    #[validate(range(min = 1, max = 100, message = "Height must be between 1 and 100"))]
    pub height: Option<i32>,

    pub parameter_bindings: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TileResponse {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub visualization_id: Uuid,
    pub title: Option<String>,
    pub pos_x: i32,
    pub pos_y: i32,
    pub width: i32,
    pub height: i32,
    pub parameter_bindings: serde_json::Value,
}

impl From<Tile> for TileResponse {
    fn from(t: Tile) -> Self {
        Self {
            id: t.id,
            dashboard_id: t.dashboard_id,
            visualization_id: t.visualization_id,
            title: t.title,
            pos_x: t.pos_x,
            pos_y: t.pos_y,
            width: t.width,
            height: t.height,
            parameter_bindings: t.parameter_bindings,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub parameters: serde_json::Value,
    pub tags: Vec<String>,
    pub tiles: Vec<TileResponse>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
