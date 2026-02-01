use crate::AppState;
use crate::routes::auth::get_auth_context;
use actix_web::{HttpRequest, HttpResponse, web};
use loupe::{Error, SqlValidator};
use loupe::models::{
    CreateQueryRequest, ImportQueriesRequest, ImportQueriesResponse, QueryExport, QueryResponse,
    UpdateQueryRequest,
};
use std::sync::Arc;
use validator::Validate;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/queries")
            .route("", web::get().to(list_queries))
            .route("", web::post().to(create_query))
            .route("/export", web::get().to(export_queries))
            .route("/import", web::post().to(import_queries))
            .route("/{id}", web::get().to(get_query))
            .route("/{id}", web::put().to(update_query))
            .route("/{id}", web::delete().to(delete_query)),
    );
}

async fn list_queries(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_auth_context(&state, &req)?;
    let queries = state.db.list_queries(org_id).await?;
    let response: Vec<QueryResponse> = queries.into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(response))
}

async fn create_query(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    body: web::Json<CreateQueryRequest>,
) -> Result<HttpResponse, Error> {
    let (user_id, org_id) = get_auth_context(&state, &req)?;

    // Validate request input
    body.validate()
        .map_err(|e| Error::BadRequest(format!("Validation failed: {}", e)))?;

    // Verify datasource exists and belongs to org
    state.db.get_datasource(body.datasource_id, org_id).await?;

    // SECURITY: Validate SQL to prevent injection attacks
    let validator = SqlValidator::new();
    validator.validate(&body.sql)?;

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

    tracing::info!(
        query_id = %query.id,
        user_id = %user_id,
        "Query created and validated successfully"
    );

    Ok(HttpResponse::Created().json(QueryResponse::from(query)))
}

async fn get_query(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_auth_context(&state, &req)?;
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
    let (_, org_id) = get_auth_context(&state, &req)?;
    let id = path.into_inner();

    // Validate request input
    body.validate()
        .map_err(|e| Error::BadRequest(format!("Validation failed: {}", e)))?;

    // SECURITY: Validate SQL if it's being updated
    if let Some(ref sql) = body.sql {
        let validator = SqlValidator::new();
        validator.validate(sql)?;
    }

    let parameters = body
        .parameters
        .as_ref()
        .map(|p| serde_json::to_value(p).unwrap());

    let tags = body.tags.as_ref().map(|t| serde_json::to_value(t).unwrap());

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
    let (_, org_id) = get_auth_context(&state, &req)?;
    let id = path.into_inner();
    state.db.delete_query(id, org_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn export_queries(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_auth_context(&state, &req)?;

    let queries = state.db.list_queries(org_id).await?;
    let datasources = state.db.list_datasources(org_id).await?;

    // Create a map of datasource ID to name for lookup
    let ds_map: std::collections::HashMap<Uuid, String> =
        datasources.into_iter().map(|ds| (ds.id, ds.name)).collect();

    let exports: Vec<QueryExport> = queries
        .into_iter()
        .map(|q| {
            let parameters: Vec<loupe::models::ParamDef> =
                serde_json::from_value(q.parameters).unwrap_or_default();
            let tags: Vec<String> = serde_json::from_value(q.tags).unwrap_or_default();
            QueryExport {
                name: q.name,
                description: q.description,
                sql: q.sql,
                parameters,
                timeout_seconds: q.timeout_seconds,
                max_rows: q.max_rows,
                tags,
                datasource_name: ds_map.get(&q.datasource_id).cloned(),
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(exports))
}

async fn import_queries(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    body: web::Json<ImportQueriesRequest>,
) -> Result<HttpResponse, Error> {
    let (user_id, org_id) = get_auth_context(&state, &req)?;

    // Verify datasource exists and belongs to org
    state.db.get_datasource(body.datasource_id, org_id).await?;

    // Get existing query names for deduplication
    let existing_queries = state.db.list_queries(org_id).await?;
    let existing_names: std::collections::HashSet<String> =
        existing_queries.into_iter().map(|q| q.name).collect();

    let mut imported = 0;
    let mut skipped = 0;
    let mut skipped_names = Vec::new();
    let validator = SqlValidator::new();

    for query in &body.queries {
        // Check for duplicate
        if existing_names.contains(&query.name) {
            if body.skip_duplicates {
                skipped += 1;
                skipped_names.push(query.name.clone());
                continue;
            }
        }

        // SECURITY: Validate SQL for each imported query
        validator.validate(&query.sql)?;

        let parameters = serde_json::to_value(&query.parameters).unwrap_or_default();
        let tags = serde_json::to_value(&query.tags).unwrap_or_default();

        state
            .db
            .create_query(
                org_id,
                body.datasource_id,
                &query.name,
                query.description.as_deref(),
                &query.sql,
                &parameters,
                &tags,
                query.timeout_seconds,
                query.max_rows,
                user_id,
            )
            .await?;

        imported += 1;
    }

    Ok(HttpResponse::Ok().json(ImportQueriesResponse {
        imported,
        skipped,
        skipped_names,
    }))
}
