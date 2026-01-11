use crate::AppState;
use crate::routes::auth::get_auth_context;
use actix_web::{HttpRequest, HttpResponse, web};
use loupe::Error;
use loupe::models::{CreateQueryRequest, QueryResponse, UpdateQueryRequest};
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

async fn list_queries(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_auth_context(&req)?;
    let queries = state.db.list_queries(org_id).await?;
    let response: Vec<QueryResponse> = queries.into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(response))
}

async fn create_query(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    body: web::Json<CreateQueryRequest>,
) -> Result<HttpResponse, Error> {
    let (user_id, org_id) = get_auth_context(&req)?;

    // Verify datasource exists and belongs to org
    state.db.get_datasource(body.datasource_id, org_id).await?;

    let parameters = serde_json::to_value(&body.parameters).unwrap_or_default();
    let tags = serde_json::to_value(&body.tags).unwrap_or_default();

    let query = state
        .db
        .create_query(
            org_id,
            body.datasource_id,
            &body.name,
            body.description.as_deref(),
            &body.sql,
            &parameters,
            &tags,
            body.timeout_seconds,
            body.max_rows,
            user_id,
        )
        .await?;

    Ok(HttpResponse::Created().json(QueryResponse::from(query)))
}

async fn get_query(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_auth_context(&req)?;
    let id = path.into_inner();
    let query = state.db.get_query(id, org_id).await?;
    Ok(HttpResponse::Ok().json(QueryResponse::from(query)))
}

async fn update_query(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateQueryRequest>,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_auth_context(&req)?;
    let id = path.into_inner();

    let parameters = body
        .parameters
        .as_ref()
        .map(|p| serde_json::to_value(p).unwrap());

    let tags = body
        .tags
        .as_ref()
        .map(|t| serde_json::to_value(t).unwrap());

    let query = state
        .db
        .update_query(
            id,
            org_id,
            body.name.as_deref(),
            body.description.as_deref(),
            body.sql.as_deref(),
            parameters.as_ref(),
            tags.as_ref(),
            body.timeout_seconds,
            body.max_rows,
        )
        .await?;

    Ok(HttpResponse::Ok().json(QueryResponse::from(query)))
}

async fn delete_query(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_auth_context(&req)?;
    let id = path.into_inner();
    state.db.delete_query(id, org_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
