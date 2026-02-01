use crate::AppState;
use crate::permissions::{get_user_context, require_permission, Permission};
use actix_web::{HttpRequest, HttpResponse, web};
use loupe::Error;
use loupe::connectors::{Connector, PostgresConnector};
use loupe::filtering::{SearchParams, SortParams, SortableColumns};
use loupe::models::{
    ConnectionTestResult, CreateDatasourceRequest, DatasourceResponse, DatasourceType,
    UpdateDatasourceRequest,
};
use loupe::validation::validate_request;
use loupe::{PaginatedResponse, PaginationParams};
use std::sync::Arc;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/datasources")
            .route("", web::get().to(list_datasources))
            .route("", web::post().to(create_datasource))
            .route("/{id}", web::get().to(get_datasource))
            .route("/{id}", web::put().to(update_datasource))
            .route("/{id}", web::delete().to(delete_datasource))
            .route("/{id}/test", web::post().to(test_connection))
            .route("/{id}/schema", web::get().to(get_schema)),
    );
}

#[derive(serde::Deserialize)]
pub struct ListDatasourcesQuery {
    #[serde(flatten)]
    pub search: SearchParams,

    #[serde(flatten)]
    pub sort: SortParams,

    #[serde(flatten)]
    pub pagination: PaginationParams,
}

async fn list_datasources(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    query: web::Query<ListDatasourcesQuery>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;

    let mut pagination = query.pagination.clone();
    pagination.validate();

    // Validate and build sort params
    let (sort_column, sort_direction) = query.sort.validate_and_build(
        SortableColumns::DATASOURCES,
        "created_at",
    );

    // Get search pattern
    let search = query.search.get_pattern();

    let (datasources, total) = state
        .db
        .list_datasources_paginated(
            org_id,
            search,
            &sort_column,
            &sort_direction,
            pagination.limit,
            pagination.offset,
        )
        .await?;

    let items: Vec<DatasourceResponse> = datasources.into_iter().map(Into::into).collect();

    let paginated = PaginatedResponse::new(items, total, &pagination);
    Ok(HttpResponse::Ok().json(paginated))
}

async fn create_datasource(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    body: web::Json<CreateDatasourceRequest>,
) -> Result<HttpResponse, Error> {
    let (user_id, org_id, role) = get_user_context(&state, &req).await?;
    // SECURITY: Only admins can create datasources (contains sensitive connection strings)
    require_permission(role, Permission::Admin)?;

    // Validate request
    validate_request(&*body)?;

    // In production, encrypt the connection string
    let encrypted = &body.connection_string; // TODO: actual encryption

    let datasource = state
        .db
        .create_datasource(org_id, &body.name, body.ds_type, encrypted, user_id)
        .await?;

    Ok(HttpResponse::Created().json(DatasourceResponse::from(datasource)))
}

async fn get_datasource(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;

    let id = path.into_inner();
    let datasource = state.db.get_datasource(id, org_id).await?;
    Ok(HttpResponse::Ok().json(DatasourceResponse::from(datasource)))
}

async fn update_datasource(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateDatasourceRequest>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Admin)?;

    // Validate request
    validate_request(&*body)?;

    let id = path.into_inner();

    let encrypted = body.connection_string.as_deref(); // TODO: encryption

    let datasource = state
        .db
        .update_datasource(id, org_id, body.name.as_deref(), encrypted)
        .await?;

    Ok(HttpResponse::Ok().json(DatasourceResponse::from(datasource)))
}

async fn delete_datasource(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Admin)?;

    let id = path.into_inner();
    state.db.delete_datasource(id, org_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn test_connection(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;

    let id = path.into_inner();
    let datasource = state.db.get_datasource(id, org_id).await?;

    // Decrypt connection string (TODO: actual decryption)
    let conn_str = &datasource.connection_string_encrypted;

    match datasource.ds_type {
        DatasourceType::Postgres => match PostgresConnector::new(conn_str).await {
            Ok(connector) => match connector.test_connection().await {
                Ok(latency) => Ok(HttpResponse::Ok().json(ConnectionTestResult {
                    success: true,
                    message: "Connection successful".to_string(),
                    latency_ms: Some(latency.as_millis() as u64),
                })),
                Err(e) => Ok(HttpResponse::Ok().json(ConnectionTestResult {
                    success: false,
                    message: e.to_string(),
                    latency_ms: None,
                })),
            },
            Err(e) => Ok(HttpResponse::Ok().json(ConnectionTestResult {
                success: false,
                message: e.to_string(),
                latency_ms: None,
            })),
        },
    }
}

async fn get_schema(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;

    let id = path.into_inner();
    let datasource = state.db.get_datasource(id, org_id).await?;

    let conn_str = &datasource.connection_string_encrypted;

    match datasource.ds_type {
        DatasourceType::Postgres => {
            let connector = PostgresConnector::new(conn_str).await?;
            let schema = connector.get_schema().await?;
            Ok(HttpResponse::Ok().json(schema))
        }
    }
}
