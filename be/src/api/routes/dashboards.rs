use crate::AppState;
use crate::routes::auth::get_auth_context;
use actix_web::{HttpRequest, HttpResponse, web};
use loupe::Error;
use loupe::models::{
    CreateDashboardRequest, CreateTileRequest, DashboardResponse, TileResponse,
    UpdateDashboardRequest, UpdateTileRequest,
};
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

async fn list_dashboards(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_auth_context(&req)?;
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
    req: HttpRequest,
    body: web::Json<CreateDashboardRequest>,
) -> Result<HttpResponse, Error> {
    let (user_id, org_id) = get_auth_context(&req)?;

    let dashboard = state
        .db
        .create_dashboard(
            org_id,
            &body.name,
            body.description.as_deref(),
            &body.parameters,
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
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_auth_context(&req)?;
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
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateDashboardRequest>,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_auth_context(&req)?;
    let id = path.into_inner();

    let dashboard = state
        .db
        .update_dashboard(
            id,
            org_id,
            body.name.as_deref(),
            body.description.as_deref(),
            body.parameters.as_ref(),
        )
        .await?;
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
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_auth_context(&req)?;
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
    let (_, org_id) = get_auth_context(&req)?;
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
    path: web::Path<TilePathParams>,
) -> Result<HttpResponse, Error> {
    let params = path.into_inner();
    state.db.delete_tile(params.tile_id, params.id).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn update_tile(
    state: web::Data<Arc<AppState>>,
    path: web::Path<TilePathParams>,
    body: web::Json<UpdateTileRequest>,
) -> Result<HttpResponse, Error> {
    let params = path.into_inner();

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
