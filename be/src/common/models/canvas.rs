use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ===== Canvas =====

/// A canvas for semantic data analysis
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Canvas {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub time_preset: String,
    pub time_offset: i64,
    pub time_custom_start: Option<DateTime<Utc>>,
    pub time_custom_end: Option<DateTime<Utc>>,
    pub live: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCanvasRequest {
    pub name: String,
    #[serde(default = "default_time_preset")]
    pub time_preset: String,
    #[serde(default)]
    pub time_offset: i64,
    pub time_custom_start: Option<DateTime<Utc>>,
    pub time_custom_end: Option<DateTime<Utc>>,
    #[serde(default)]
    pub live: bool,
}

fn default_time_preset() -> String {
    "7d".to_string()
}

#[derive(Debug, Deserialize)]
pub struct UpdateCanvasRequest {
    pub name: Option<String>,
    pub time_preset: Option<String>,
    pub time_offset: Option<i64>,
    pub time_custom_start: Option<DateTime<Utc>>,
    pub time_custom_end: Option<DateTime<Utc>>,
    pub live: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CanvasResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub time_preset: String,
    pub time_offset: i64,
    pub time_custom_start: Option<DateTime<Utc>>,
    pub time_custom_end: Option<DateTime<Utc>>,
    pub live: bool,
    pub nodes: Vec<CanvasNodeResponse>,
    pub edges: Vec<CanvasEdgeResponse>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Canvas {
    pub fn into_response(self, nodes: Vec<CanvasNodeResponse>, edges: Vec<CanvasEdgeResponse>) -> CanvasResponse {
        CanvasResponse {
            id: self.id,
            org_id: self.org_id,
            name: self.name,
            time_preset: self.time_preset,
            time_offset: self.time_offset,
            time_custom_start: self.time_custom_start,
            time_custom_end: self.time_custom_end,
            live: self.live,
            nodes,
            edges,
            created_by: self.created_by,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

// ===== Canvas Node =====

/// A node within a canvas (query or note)
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct CanvasNode {
    pub id: Uuid,
    pub canvas_id: Uuid,
    pub node_type: String,
    pub title: String,
    pub pos_x: f64,
    pub pos_y: f64,
    pub width: f64,
    pub height: f64,
    pub meta: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCanvasNodeRequest {
    pub node_type: String,
    pub title: String,
    #[serde(default)]
    pub pos_x: f64,
    #[serde(default)]
    pub pos_y: f64,
    #[serde(default = "default_width")]
    pub width: f64,
    #[serde(default = "default_height")]
    pub height: f64,
    #[serde(default)]
    pub meta: serde_json::Value,
}

fn default_width() -> f64 {
    280.0
}

fn default_height() -> f64 {
    160.0
}

#[derive(Debug, Deserialize)]
pub struct UpdateCanvasNodeRequest {
    pub title: Option<String>,
    pub pos_x: Option<f64>,
    pub pos_y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct CanvasNodeResponse {
    pub id: Uuid,
    pub canvas_id: Uuid,
    pub node_type: String,
    pub title: String,
    pub pos_x: f64,
    pub pos_y: f64,
    pub width: f64,
    pub height: f64,
    pub meta: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CanvasNode> for CanvasNodeResponse {
    fn from(n: CanvasNode) -> Self {
        Self {
            id: n.id,
            canvas_id: n.canvas_id,
            node_type: n.node_type,
            title: n.title,
            pos_x: n.pos_x,
            pos_y: n.pos_y,
            width: n.width,
            height: n.height,
            meta: n.meta,
            created_at: n.created_at,
            updated_at: n.updated_at,
        }
    }
}

// ===== Canvas Edge =====

/// An edge connecting two nodes with a semantic relationship
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct CanvasEdge {
    pub id: Uuid,
    pub canvas_id: Uuid,
    pub from_node_id: Uuid,
    pub to_node_id: Uuid,
    pub label: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCanvasEdgeRequest {
    pub from_node_id: Uuid,
    pub to_node_id: Uuid,
    #[serde(default = "default_label")]
    pub label: String,
}

fn default_label() -> String {
    "motivates".to_string()
}

#[derive(Debug, Deserialize)]
pub struct UpdateCanvasEdgeRequest {
    pub label: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CanvasEdgeResponse {
    pub id: Uuid,
    pub canvas_id: Uuid,
    pub from_node_id: Uuid,
    pub to_node_id: Uuid,
    pub label: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CanvasEdge> for CanvasEdgeResponse {
    fn from(e: CanvasEdge) -> Self {
        Self {
            id: e.id,
            canvas_id: e.canvas_id,
            from_node_id: e.from_node_id,
            to_node_id: e.to_node_id,
            label: e.label,
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}
