use crate::AppState;
use crate::permissions::{get_user_context, require_permission, Permission};
use actix_web::{HttpRequest, HttpResponse, web};
use loupe::Error;
use loupe::filtering::{parse_tags, SearchParams, SortParams, SortableColumns};
use loupe::models::{
    CreateDashboardRequest, CreateTileRequest, DashboardResponse, TileResponse,
    UpdateDashboardRequest, UpdateTileRequest,
};
use loupe::{PaginatedResponse, PaginationParams};
use loupe::validation::validate_request;
use std::sync::Arc;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/dashboards")
            .route("", web::get().to(list_dashboards))
            .route("", web::post().to(create_dashboard))
            .route("/{id}", web::get().to(get_dashboard))
            .route("/{id}", web::put().to(update_dashboard))
            .route("/{id}", web::delete().to(delete_dashboard))
            .route("/{id}/tiles", web::post().to(create_tile))
            .route("/{id}/tiles/{tile_id}", web::put().to(update_tile))
            .route("/{id}/tiles/{tile_id}", web::delete().to(delete_tile)),
    );
}

#[derive(serde::Deserialize)]
pub struct ListDashboardsQuery {
    /// Search in name and description
    #[serde(flatten)]
    pub search: SearchParams,

    /// Filter by tags (comma-separated: "analytics,prod")
    pub tags: Option<String>,

    #[serde(flatten)]
    pub sort: SortParams,

    #[serde(flatten)]
    pub pagination: PaginationParams,
}

async fn list_dashboards(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    query: web::Query<ListDashboardsQuery>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;

    // Validate pagination
    let mut pagination = query.pagination.clone();
    pagination.validate();

    // Validate and build sort parameters
    let (sort_column, sort_direction) = query.sort.validate_and_build(
        SortableColumns::DASHBOARDS,
        "created_at", // default
    );

    // Parse tags filter (comma-separated to Vec)
    let tags = query
        .tags
        .as_ref()
        .map(|t| parse_tags(t))
        .filter(|v| !v.is_empty());

    // Get search pattern
    let search = query.search.get_pattern();

    // Call database layer with filters
    let (dashboards, total) = state
        .db
        .list_dashboards_paginated(
            org_id,
            search,
            tags,
            &sort_column,
            &sort_direction,
            pagination.limit,
            pagination.offset,
        )
        .await?;

    let mut items = Vec::new();
    for dashboard in dashboards {
        let tiles = state.db.list_tiles(dashboard.id).await?;
        let tags: Vec<String> = serde_json::from_value(dashboard.tags).unwrap_or_default();
        items.push(DashboardResponse {
            id: dashboard.id,
            org_id: dashboard.org_id,
            name: dashboard.name,
            description: dashboard.description,
            parameters: dashboard.parameters,
            tags,
            tiles: tiles.into_iter().map(Into::into).collect(),
            created_by: dashboard.created_by,
            created_at: dashboard.created_at,
            updated_at: dashboard.updated_at,
        });
    }

    let paginated = PaginatedResponse::new(items, total, &pagination);
    Ok(HttpResponse::Ok().json(paginated))
}

async fn create_dashboard(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    body: web::Json<CreateDashboardRequest>,
) -> Result<HttpResponse, Error> {
    let (user_id, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Editor)?;

    // Validate request
    validate_request(&*body)?;

    let tags_json = serde_json::to_value(&body.tags).unwrap_or_default();

    let dashboard = state
        .db
        .create_dashboard(
            org_id,
            &body.name,
            body.description.as_deref(),
            &body.parameters,
            &tags_json,
            user_id,
        )
        .await?;

    let tags: Vec<String> = serde_json::from_value(dashboard.tags).unwrap_or_default();

    Ok(HttpResponse::Created().json(DashboardResponse {
        id: dashboard.id,
        org_id: dashboard.org_id,
        name: dashboard.name,
        description: dashboard.description,
        parameters: dashboard.parameters,
        tags,
        tiles: vec![],
        created_by: dashboard.created_by,
        created_at: dashboard.created_at,
        updated_at: dashboard.updated_at,
    }))
}

async fn get_dashboard(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;

    let id = path.into_inner();
    let dashboard = state.db.get_dashboard(id, org_id).await?;
    let tiles = state.db.list_tiles(dashboard.id).await?;
    let tags: Vec<String> = serde_json::from_value(dashboard.tags).unwrap_or_default();

    Ok(HttpResponse::Ok().json(DashboardResponse {
        id: dashboard.id,
        org_id: dashboard.org_id,
        name: dashboard.name,
        description: dashboard.description,
        parameters: dashboard.parameters,
        tags,
        tiles: tiles.into_iter().map(Into::into).collect(),
        created_by: dashboard.created_by,
        created_at: dashboard.created_at,
        updated_at: dashboard.updated_at,
    }))
}

async fn update_dashboard(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateDashboardRequest>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Editor)?;

    // Validate request
    validate_request(&*body)?;

    let id = path.into_inner();

    let tags = body
        .tags
        .as_ref()
        .map(|t| serde_json::to_value(t).unwrap());

    let dashboard = state
        .db
        .update_dashboard(
            id,
            org_id,
            body.name.as_deref(),
            body.description.as_deref(),
            body.parameters.as_ref(),
            tags.as_ref(),
        )
        .await?;
    let tiles = state.db.list_tiles(dashboard.id).await?;
    let tags: Vec<String> = serde_json::from_value(dashboard.tags).unwrap_or_default();

    Ok(HttpResponse::Ok().json(DashboardResponse {
        id: dashboard.id,
        org_id: dashboard.org_id,
        name: dashboard.name,
        description: dashboard.description,
        parameters: dashboard.parameters,
        tags,
        tiles: tiles.into_iter().map(Into::into).collect(),
        created_by: dashboard.created_by,
        created_at: dashboard.created_at,
        updated_at: dashboard.updated_at,
    }))
}

async fn delete_dashboard(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Editor)?;

    let id = path.into_inner();
    state.db.delete_dashboard(id, org_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn create_tile(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<CreateTileRequest>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Editor)?;

    // Validate request
    validate_request(&*body)?;

    let dashboard_id = path.into_inner();

    // Verify dashboard exists
    state.db.get_dashboard(dashboard_id, org_id).await?;

    // Verify visualization exists
    state
        .db
        .get_visualization(body.visualization_id, org_id)
        .await?;

    let tile = state
        .db
        .create_tile(
            dashboard_id,
            body.visualization_id,
            body.title.as_deref(),
            body.pos_x,
            body.pos_y,
            body.width,
            body.height,
            &body.parameter_bindings,
        )
        .await?;

    Ok(HttpResponse::Created().json(TileResponse::from(tile)))
}

#[derive(serde::Deserialize)]
pub struct TilePathParams {
    id: Uuid,
    tile_id: Uuid,
}

async fn delete_tile(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<TilePathParams>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Editor)?;

    let params = path.into_inner();

    // Verify dashboard belongs to this org
    state.db.get_dashboard(params.id, org_id).await?;

    state.db.delete_tile(params.tile_id, params.id).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn update_tile(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<TilePathParams>,
    body: web::Json<UpdateTileRequest>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Editor)?;

    // Validate request
    validate_request(&*body)?;

    let params = path.into_inner();

    // Verify dashboard belongs to this org
    state.db.get_dashboard(params.id, org_id).await?;

    let tile = state
        .db
        .update_tile(
            params.tile_id,
            params.id,
            body.title.as_deref(),
            body.pos_x,
            body.pos_y,
            body.width,
            body.height,
            body.parameter_bindings.as_ref(),
        )
        .await?;

    Ok(HttpResponse::Ok().json(TileResponse::from(tile)))
}
