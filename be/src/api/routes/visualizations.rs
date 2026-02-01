use crate::AppState;
use crate::permissions::{get_user_context, require_permission, Permission};
use actix_web::{HttpRequest, HttpResponse, web};
use loupe::Error;
use loupe::models::{
    CreateVisualizationRequest, UpdateVisualizationRequest, VisualizationResponse,
};
use loupe::validation::validate_request;
use loupe::{PaginatedResponse, PaginationParams};
use std::sync::Arc;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/visualizations")
            .route("", web::get().to(list_visualizations))
            .route("", web::post().to(create_visualization))
            .route("/{id}", web::get().to(get_visualization))
            .route("/{id}", web::put().to(update_visualization))
            .route("/{id}", web::delete().to(delete_visualization)),
    );
}

#[derive(serde::Deserialize)]
pub struct ListVisualizationsQuery {
    query_id: Option<Uuid>,
    #[serde(flatten)]
    pagination: PaginationParams,
}

async fn list_visualizations(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    query: web::Query<ListVisualizationsQuery>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;

    let mut pagination = query.pagination.clone();
    pagination.validate();

    let (vizs, total) = state
        .db
        .list_visualizations_paginated(org_id, query.query_id, pagination.limit, pagination.offset)
        .await?;

    let items: Vec<VisualizationResponse> = vizs.into_iter().map(Into::into).collect();

    let paginated = PaginatedResponse::new(items, total, &pagination);
    Ok(HttpResponse::Ok().json(paginated))
}

async fn create_visualization(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    body: web::Json<CreateVisualizationRequest>,
) -> Result<HttpResponse, Error> {
    let (user_id, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Editor)?;

    // Validate request
    validate_request(&*body)?;

    // Verify query exists
    state.db.get_query(body.query_id, org_id).await?;

    let tags = serde_json::to_value(&body.tags).unwrap_or_default();

    let viz = state
        .db
        .create_visualization(
            org_id,
            body.query_id,
            &body.name,
            body.chart_type,
            &body.config,
            &tags,
            user_id,
        )
        .await?;

    Ok(HttpResponse::Created().json(VisualizationResponse::from(viz)))
}

async fn get_visualization(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;

    let id = path.into_inner();
    let viz = state.db.get_visualization(id, org_id).await?;
    Ok(HttpResponse::Ok().json(VisualizationResponse::from(viz)))
}

async fn delete_visualization(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Editor)?;

    let id = path.into_inner();
    state.db.delete_visualization(id, org_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn update_visualization(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateVisualizationRequest>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Editor)?;

    // Validate request
    validate_request(&*body)?;

    let id = path.into_inner();

    // If changing query, verify the new query exists and belongs to org
    if let Some(query_id) = body.query_id {
        state.db.get_query(query_id, org_id).await?;
    }

    let tags = body.tags.as_ref().map(|t| serde_json::to_value(t).unwrap());

    let viz = state
        .db
        .update_visualization(
            id,
            org_id,
            body.query_id,
            body.name.as_deref(),
            body.chart_type,
            body.config.as_ref(),
            tags.as_ref(),
        )
        .await?;

    Ok(HttpResponse::Ok().json(VisualizationResponse::from(viz)))
}
