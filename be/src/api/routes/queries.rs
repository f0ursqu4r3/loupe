use crate::AppState;
use actix_web::{web, HttpResponse};
use loupe::models::{CreateQueryRequest, QueryResponse, UpdateQueryRequest};
use loupe::Error;
use std::sync::Arc;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/queries")
            .route("", web::get().to(list_queries))
            .route("", web::post().to(create_query))
            .route("/{id}", web::get().to(get_query))
            .route("/{id}", web::put().to(update_query))
            .route("/{id}", web::delete().to(delete_query)),
    );
}

fn get_current_user() -> (Uuid, Uuid) {
    (
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
    )
}

async fn list_queries(state: web::Data<Arc<AppState>>) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_current_user();
    let queries = state.db.list_queries(org_id).await?;
    let response: Vec<QueryResponse> = queries.into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(response))
}

async fn create_query(
    state: web::Data<Arc<AppState>>,
    req: web::Json<CreateQueryRequest>,
) -> Result<HttpResponse, Error> {
    let (user_id, org_id) = get_current_user();

    // Verify datasource exists and belongs to org
    state.db.get_datasource(req.datasource_id, org_id).await?;

    let parameters = serde_json::to_value(&req.parameters).unwrap_or_default();

    let query = state
        .db
        .create_query(
            org_id,
            req.datasource_id,
            &req.name,
            req.description.as_deref(),
            &req.sql,
            &parameters,
            req.timeout_seconds,
            req.max_rows,
            user_id,
        )
        .await?;

    Ok(HttpResponse::Created().json(QueryResponse::from(query)))
}

async fn get_query(
    state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_current_user();
    let id = path.into_inner();
    let query = state.db.get_query(id, org_id).await?;
    Ok(HttpResponse::Ok().json(QueryResponse::from(query)))
}

async fn update_query(
    state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateQueryRequest>,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_current_user();
    let id = path.into_inner();

    let parameters = req.parameters.as_ref().map(|p| serde_json::to_value(p).unwrap());

    let query = state
        .db
        .update_query(
            id,
            org_id,
            req.name.as_deref(),
            req.description.as_deref(),
            req.sql.as_deref(),
            parameters.as_ref(),
            req.timeout_seconds,
            req.max_rows,
        )
        .await?;

    Ok(HttpResponse::Ok().json(QueryResponse::from(query)))
}

async fn delete_query(
    state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_current_user();
    let id = path.into_inner();
    state.db.delete_query(id, org_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
