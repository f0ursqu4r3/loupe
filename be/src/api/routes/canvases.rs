use crate::permissions::{get_user_context, require_permission, Permission};
use crate::AppState;
use actix_web::{web, HttpRequest, HttpResponse};
use loupe::models::{
    CanvasEdgeResponse, CanvasNodeResponse, CanvasResponse, CreateCanvasEdgeRequest,
    CreateCanvasNodeRequest, CreateCanvasRequest, UpdateCanvasEdgeRequest, UpdateCanvasNodeRequest,
    UpdateCanvasRequest,
};
use loupe::validation::validate_request;
use loupe::Error;
use std::sync::Arc;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/canvases")
            .route("", web::get().to(list_canvases))
            .route("", web::post().to(create_canvas))
            .route("/{id}", web::get().to(get_canvas))
            .route("/{id}", web::put().to(update_canvas))
            .route("/{id}", web::delete().to(delete_canvas))
            // Nodes
            .route("/{id}/nodes", web::post().to(create_node))
            .route("/{id}/nodes/{node_id}", web::put().to(update_node))
            .route("/{id}/nodes/{node_id}", web::delete().to(delete_node))
            // Edges
            .route("/{id}/edges", web::post().to(create_edge))
            .route("/{id}/edges/{edge_id}", web::put().to(update_edge))
            .route("/{id}/edges/{edge_id}", web::delete().to(delete_edge)),
    );
}

// ==================== Canvas CRUD ====================

async fn list_canvases(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;

    let canvases = state.db.list_canvases(org_id).await?;

    let mut response = Vec::new();
    for canvas in canvases {
        let nodes = state.db.list_canvas_nodes(canvas.id).await?;
        let edges = state.db.list_canvas_edges(canvas.id).await?;

        response.push(canvas.into_response(
            nodes.into_iter().map(Into::into).collect(),
            edges.into_iter().map(Into::into).collect(),
        ));
    }

    Ok(HttpResponse::Ok().json(response))
}

async fn create_canvas(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    body: web::Json<CreateCanvasRequest>,
) -> Result<HttpResponse, Error> {
    let (user_id, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Editor)?;

    // Validate request
    validate_request(&*body)?;

    let canvas = state
        .db
        .create_canvas(
            org_id,
            &body.name,
            &body.time_preset,
            body.time_offset,
            body.time_custom_start,
            body.time_custom_end,
            body.live,
            user_id,
        )
        .await?;

    let response = canvas.into_response(vec![], vec![]);
    Ok(HttpResponse::Created().json(response))
}

async fn get_canvas(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;
    let id = path.into_inner();

    let canvas = state.db.get_canvas(id, org_id).await?;
    let nodes = state.db.list_canvas_nodes(canvas.id).await?;
    let edges = state.db.list_canvas_edges(canvas.id).await?;

    let response = canvas.into_response(
        nodes.into_iter().map(Into::into).collect(),
        edges.into_iter().map(Into::into).collect(),
    );

    Ok(HttpResponse::Ok().json(response))
}

async fn update_canvas(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateCanvasRequest>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;

    // Validate request
    validate_request(&*body)?;

    let id = path.into_inner();

    let canvas = state
        .db
        .update_canvas(
            id,
            org_id,
            body.name.as_deref(),
            body.time_preset.as_deref(),
            body.time_offset,
            body.time_custom_start,
            body.time_custom_end,
            body.live,
        )
        .await?;

    let nodes = state.db.list_canvas_nodes(canvas.id).await?;
    let edges = state.db.list_canvas_edges(canvas.id).await?;

    let response = canvas.into_response(
        nodes.into_iter().map(Into::into).collect(),
        edges.into_iter().map(Into::into).collect(),
    );

    Ok(HttpResponse::Ok().json(response))
}

async fn delete_canvas(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;
    let id = path.into_inner();

    state.db.delete_canvas(id, org_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

// ==================== Node CRUD ====================

#[derive(serde::Deserialize)]
pub struct NodePathParams {
    id: Uuid,
    node_id: Uuid,
}

async fn create_node(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<CreateCanvasNodeRequest>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;

    // Validate request
    validate_request(&*body)?;

    let canvas_id = path.into_inner();

    // Verify canvas belongs to this org
    state.db.get_canvas(canvas_id, org_id).await?;

    let node = state
        .db
        .create_canvas_node(
            canvas_id,
            &body.node_type,
            &body.title,
            body.pos_x,
            body.pos_y,
            body.width,
            body.height,
            &body.meta,
        )
        .await?;

    Ok(HttpResponse::Created().json(CanvasNodeResponse::from(node)))
}

async fn update_node(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<NodePathParams>,
    body: web::Json<UpdateCanvasNodeRequest>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;

    // Validate request
    validate_request(&*body)?;

    let params = path.into_inner();

    // Verify canvas belongs to this org
    state.db.get_canvas(params.id, org_id).await?;

    let node = state
        .db
        .update_canvas_node(
            params.node_id,
            params.id,
            body.title.as_deref(),
            body.pos_x,
            body.pos_y,
            body.width,
            body.height,
            body.meta.as_ref(),
        )
        .await?;

    Ok(HttpResponse::Ok().json(CanvasNodeResponse::from(node)))
}

async fn delete_node(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<NodePathParams>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;
    let params = path.into_inner();

    // Verify canvas belongs to this org
    state.db.get_canvas(params.id, org_id).await?;

    state.db.delete_canvas_node(params.node_id, params.id).await?;
    Ok(HttpResponse::NoContent().finish())
}

// ==================== Edge CRUD ====================

#[derive(serde::Deserialize)]
pub struct EdgePathParams {
    id: Uuid,
    edge_id: Uuid,
}

async fn create_edge(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<CreateCanvasEdgeRequest>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;

    // Validate request
    validate_request(&*body)?;

    let canvas_id = path.into_inner();

    // Verify canvas belongs to this org
    state.db.get_canvas(canvas_id, org_id).await?;

    // Verify both nodes exist in this canvas
    state.db.get_canvas_node(body.from_node_id, canvas_id).await?;
    state.db.get_canvas_node(body.to_node_id, canvas_id).await?;

    let edge = state
        .db
        .create_canvas_edge(canvas_id, body.from_node_id, body.to_node_id, &body.label)
        .await?;

    Ok(HttpResponse::Created().json(CanvasEdgeResponse::from(edge)))
}

async fn update_edge(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<EdgePathParams>,
    body: web::Json<UpdateCanvasEdgeRequest>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;

    // Validate request
    validate_request(&*body)?;

    let params = path.into_inner();

    // Verify canvas belongs to this org
    state.db.get_canvas(params.id, org_id).await?;

    let edge = state
        .db
        .update_canvas_edge(params.edge_id, params.id, body.label.as_deref())
        .await?;

    Ok(HttpResponse::Ok().json(CanvasEdgeResponse::from(edge)))
}

async fn delete_edge(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<EdgePathParams>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;
    let params = path.into_inner();

    // Verify canvas belongs to this org
    state.db.get_canvas(params.id, org_id).await?;

    state.db.delete_canvas_edge(params.edge_id, params.id).await?;
    Ok(HttpResponse::NoContent().finish())
}
