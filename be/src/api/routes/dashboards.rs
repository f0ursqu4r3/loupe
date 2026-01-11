use crate::AppState;
use actix_web::{web, HttpResponse};
use loupe::models::{
    CreateDashboardRequest, CreateTileRequest, DashboardResponse, TileResponse,
    UpdateDashboardRequest,
};
use loupe::Error;
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
            .route("/{id}/tiles/{tile_id}", web::delete().to(delete_tile)),
    );
}

fn get_current_user() -> (Uuid, Uuid) {
    (
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
    )
}

async fn list_dashboards(state: web::Data<Arc<AppState>>) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_current_user();
    let dashboards = state.db.list_dashboards(org_id).await?;

    let mut response = Vec::new();
    for dashboard in dashboards {
        let tiles = state.db.list_tiles(dashboard.id).await?;
        response.push(DashboardResponse {
            id: dashboard.id,
            org_id: dashboard.org_id,
            name: dashboard.name,
            description: dashboard.description,
            parameters: dashboard.parameters,
            tiles: tiles.into_iter().map(Into::into).collect(),
            created_by: dashboard.created_by,
            created_at: dashboard.created_at,
            updated_at: dashboard.updated_at,
        });
    }

    Ok(HttpResponse::Ok().json(response))
}

async fn create_dashboard(
    state: web::Data<Arc<AppState>>,
    req: web::Json<CreateDashboardRequest>,
) -> Result<HttpResponse, Error> {
    let (user_id, org_id) = get_current_user();

    let dashboard = state
        .db
        .create_dashboard(
            org_id,
            &req.name,
            req.description.as_deref(),
            &req.parameters,
            user_id,
        )
        .await?;

    Ok(HttpResponse::Created().json(DashboardResponse {
        id: dashboard.id,
        org_id: dashboard.org_id,
        name: dashboard.name,
        description: dashboard.description,
        parameters: dashboard.parameters,
        tiles: vec![],
        created_by: dashboard.created_by,
        created_at: dashboard.created_at,
        updated_at: dashboard.updated_at,
    }))
}

async fn get_dashboard(
    state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_current_user();
    let id = path.into_inner();
    let dashboard = state.db.get_dashboard(id, org_id).await?;
    let tiles = state.db.list_tiles(dashboard.id).await?;

    Ok(HttpResponse::Ok().json(DashboardResponse {
        id: dashboard.id,
        org_id: dashboard.org_id,
        name: dashboard.name,
        description: dashboard.description,
        parameters: dashboard.parameters,
        tiles: tiles.into_iter().map(Into::into).collect(),
        created_by: dashboard.created_by,
        created_at: dashboard.created_at,
        updated_at: dashboard.updated_at,
    }))
}

async fn update_dashboard(
    state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
    _req: web::Json<UpdateDashboardRequest>,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_current_user();
    let id = path.into_inner();
    
    // For now, just return the existing dashboard
    // TODO: implement update
    let dashboard = state.db.get_dashboard(id, org_id).await?;
    let tiles = state.db.list_tiles(dashboard.id).await?;

    Ok(HttpResponse::Ok().json(DashboardResponse {
        id: dashboard.id,
        org_id: dashboard.org_id,
        name: dashboard.name,
        description: dashboard.description,
        parameters: dashboard.parameters,
        tiles: tiles.into_iter().map(Into::into).collect(),
        created_by: dashboard.created_by,
        created_at: dashboard.created_at,
        updated_at: dashboard.updated_at,
    }))
}

async fn delete_dashboard(
    state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_current_user();
    let id = path.into_inner();
    state.db.delete_dashboard(id, org_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn create_tile(
    state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
    req: web::Json<CreateTileRequest>,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_current_user();
    let dashboard_id = path.into_inner();

    // Verify dashboard exists
    state.db.get_dashboard(dashboard_id, org_id).await?;

    // Verify visualization exists
    state.db.get_visualization(req.visualization_id, org_id).await?;

    let tile = state
        .db
        .create_tile(
            dashboard_id,
            req.visualization_id,
            req.title.as_deref(),
            req.pos_x,
            req.pos_y,
            req.width,
            req.height,
            &req.parameter_bindings,
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
    path: web::Path<TilePathParams>,
) -> Result<HttpResponse, Error> {
    let params = path.into_inner();
    state.db.delete_tile(params.tile_id, params.id).await?;
    Ok(HttpResponse::NoContent().finish())
}
