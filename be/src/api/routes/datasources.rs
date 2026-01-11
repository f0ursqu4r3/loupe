use crate::AppState;
use crate::routes::auth::get_auth_context;
use actix_web::{HttpRequest, HttpResponse, web};
use loupe::Error;
use loupe::connectors::{Connector, PostgresConnector};
use loupe::models::{
    ConnectionTestResult, CreateDatasourceRequest, DatasourceResponse, DatasourceType,
    UpdateDatasourceRequest,
};
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

async fn list_datasources(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_auth_context(&req)?;
    let datasources = state.db.list_datasources(org_id).await?;
    let response: Vec<DatasourceResponse> = datasources.into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(response))
}

async fn create_datasource(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    body: web::Json<CreateDatasourceRequest>,
) -> Result<HttpResponse, Error> {
    let (user_id, org_id) = get_auth_context(&req)?;

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
    let (_, org_id) = get_auth_context(&req)?;
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
    let (_, org_id) = get_auth_context(&req)?;
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
    let (_, org_id) = get_auth_context(&req)?;
    let id = path.into_inner();
    state.db.delete_datasource(id, org_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn test_connection(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_auth_context(&req)?;
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
    let (_, org_id) = get_auth_context(&req)?;
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
