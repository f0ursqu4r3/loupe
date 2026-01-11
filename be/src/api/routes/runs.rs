use crate::AppState;
use actix_web::{web, HttpResponse};
use loupe::models::{CreateRunRequest, ExecuteAdHocRequest, RunResponse, RunResultResponse};
use loupe::Error;
use std::sync::Arc;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/runs")
            .route("", web::get().to(list_runs))
            .route("", web::post().to(create_run))
            .route("/execute", web::post().to(execute_adhoc))
            .route("/{id}", web::get().to(get_run))
            .route("/{id}/result", web::get().to(get_run_result)),
    );
}

fn get_current_user() -> (Uuid, Uuid) {
    (
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
    )
}

#[derive(serde::Deserialize)]
pub struct ListRunsQuery {
    query_id: Option<Uuid>,
}

async fn list_runs(
    state: web::Data<Arc<AppState>>,
    query: web::Query<ListRunsQuery>,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_current_user();
    let runs = state.db.list_runs(org_id, query.query_id).await?;
    let response: Vec<RunResponse> = runs.into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(response))
}

async fn create_run(
    state: web::Data<Arc<AppState>>,
    req: web::Json<CreateRunRequest>,
) -> Result<HttpResponse, Error> {
    let (user_id, org_id) = get_current_user();

    // Get the query
    let query = state.db.get_query(req.query_id, org_id).await?;

    // Use query defaults or request overrides
    let timeout = req.timeout_seconds.unwrap_or(query.timeout_seconds);
    let max_rows = req.max_rows.unwrap_or(query.max_rows);

    // Create the run (status = queued)
    let run = state
        .db
        .create_run(
            org_id,
            query.id,
            query.datasource_id,
            &query.sql,
            &req.parameters,
            timeout,
            max_rows,
            user_id,
        )
        .await?;

    Ok(HttpResponse::Created().json(RunResponse::from(run)))
}

async fn execute_adhoc(
    state: web::Data<Arc<AppState>>,
    req: web::Json<ExecuteAdHocRequest>,
) -> Result<HttpResponse, Error> {
    let (user_id, org_id) = get_current_user();

    // Verify datasource exists
    let datasource = state.db.get_datasource(req.datasource_id, org_id).await?;

    // Create an ephemeral query
    let query = state
        .db
        .create_query(
            org_id,
            datasource.id,
            "_adhoc",
            None,
            &req.sql,
            &serde_json::json!([]),
            req.timeout_seconds,
            req.max_rows,
            user_id,
        )
        .await?;

    // Create the run
    let run = state
        .db
        .create_run(
            org_id,
            query.id,
            datasource.id,
            &req.sql,
            &req.parameters,
            req.timeout_seconds,
            req.max_rows,
            user_id,
        )
        .await?;

    Ok(HttpResponse::Created().json(RunResponse::from(run)))
}

async fn get_run(
    state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_current_user();
    let id = path.into_inner();
    let run = state.db.get_run(id, org_id).await?;
    Ok(HttpResponse::Ok().json(RunResponse::from(run)))
}

async fn get_run_result(
    state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let run_id = path.into_inner();
    let result = state.db.get_run_result(run_id).await?;
    Ok(HttpResponse::Ok().json(RunResultResponse::from(result)))
}
