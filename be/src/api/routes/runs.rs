use crate::AppState;
use crate::permissions::{get_user_context, require_permission, Permission};
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{HttpRequest, HttpResponse, web};
use loupe::{Error, SqlValidator};
use loupe::filtering::{ListParams, SortableColumns};
use loupe::models::{
    CreateRunRequest, ExecuteAdHocRequest, ParamDef, RunResponse, RunResultResponse, RunStatus,
};
use loupe::params::{ParamSchema, bind_params};
use loupe::PaginatedResponse;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

const EXECUTE_RATE_LIMIT_PER_MINUTE: u64 = 100;
const EXECUTE_RATE_LIMIT_BURST: u32 = 25;

pub fn configure(cfg: &mut web::ServiceConfig) {
    // Endpoint-specific limit for ad-hoc SQL execution to contain abuse and expensive workloads.
    let execute_rate_conf = GovernorConfigBuilder::default()
        .requests_per_minute(EXECUTE_RATE_LIMIT_PER_MINUTE)
        .burst_size(EXECUTE_RATE_LIMIT_BURST)
        .finish()
        .expect("valid execute rate limit configuration");

    cfg.service(
        web::scope("/runs")
            .route("", web::get().to(list_runs))
            .route("", web::post().to(create_run))
            .service(
                web::resource("/execute")
                    .wrap(Governor::new(&execute_rate_conf))
                    .route(web::post().to(execute_adhoc)),
            )
            .route("/{id}", web::get().to(get_run))
            .route("/{id}/result", web::get().to(get_run_result))
            .route("/{id}/cancel", web::post().to(cancel_run)),
    );
}

#[derive(serde::Deserialize)]
pub struct ListRunsQuery {
    // Filter parameters
    query_id: Option<Uuid>,
    status: Option<String>,

    // Date range parameters
    start_date: Option<chrono::DateTime<chrono::Utc>>,
    end_date: Option<chrono::DateTime<chrono::Utc>>,

    // Sort parameters
    sort_by: Option<String>,
    sort_direction: Option<String>,

    // Pagination parameters
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    20
}

async fn list_runs(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    query: web::Query<ListRunsQuery>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;

    let lp = ListParams::parse(
        query.limit, query.offset,
        query.sort_by.clone(), query.sort_direction.clone(),
        None, None,
        SortableColumns::RUNS, "created_at",
    );

    // Validate status enum if provided
    let valid_statuses = ["queued", "running", "completed", "failed", "cancelled", "timeout"];
    let status = query.status.as_ref().and_then(|s| {
        let lower = s.to_lowercase();
        if valid_statuses.contains(&lower.as_str()) {
            Some(lower)
        } else {
            None
        }
    });

    // Validate date range
    loupe::validation::validate_date_range(query.start_date, query.end_date)
        .map_err(|e| Error::BadRequest(
            e.message.map(|m| m.to_string()).unwrap_or_else(|| "Invalid date range".to_string())
        ))?;

    let (runs, total) = state
        .db
        .list_runs_paginated(
            org_id,
            query.query_id,
            status,
            query.start_date,
            query.end_date,
            &lp.sort_column,
            &lp.sort_direction,
            lp.pagination.limit,
            lp.pagination.offset,
        )
        .await?;

    let items: Vec<RunResponse> = runs.into_iter().map(Into::into).collect();

    let paginated = PaginatedResponse::new(items, total, &lp.pagination);
    Ok(HttpResponse::Ok().json(paginated))
}

async fn create_run(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    body: web::Json<CreateRunRequest>,
) -> Result<HttpResponse, Error> {
    let (user_id, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;  // Anyone can run saved queries

    // Get the query
    let query = state.db.get_query(body.query_id, org_id).await?;

    // Use query defaults or request overrides
    let timeout = body.timeout_seconds.unwrap_or(query.timeout_seconds);
    let max_rows = body.max_rows.unwrap_or(query.max_rows);

    // Parse query's parameter schema
    let param_defs: Vec<ParamDef> =
        serde_json::from_value(query.parameters.clone()).unwrap_or_default();

    // Convert to ParamSchema for binding
    let schema: Vec<ParamSchema> = param_defs
        .iter()
        .map(|p| ParamSchema {
            name: p.name.clone(),
            param_type: p.param_type.clone(),
            required: p.required,
            default: p.default.clone(),
        })
        .collect();

    // Convert request params (JSON object) to HashMap
    let param_values: HashMap<String, serde_json::Value> = body
        .parameters
        .as_object()
        .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();

    // Bind parameters: validate types and convert $name to $1, $2, ...
    let (executed_sql, bound_values) = if schema.is_empty() {
        // No parameters, use SQL as-is
        (query.sql.clone(), serde_json::json!([]))
    } else {
        let bound = bind_params(&query.sql, &schema, &param_values)?;
        // Store the typed values in order for the runner
        let values_json: Vec<serde_json::Value> = bound
            .values
            .iter()
            .map(|tv| match tv {
                loupe::TypedValue::String(s) => serde_json::json!({"type": "string", "value": s}),
                loupe::TypedValue::Number(n) => serde_json::json!({"type": "number", "value": n}),
                loupe::TypedValue::Integer(i) => serde_json::json!({"type": "integer", "value": i}),
                loupe::TypedValue::Boolean(b) => serde_json::json!({"type": "boolean", "value": b}),
                loupe::TypedValue::Date(d) => {
                    serde_json::json!({"type": "date", "value": d.to_string()})
                }
                loupe::TypedValue::DateTime(dt) => {
                    serde_json::json!({"type": "datetime", "value": dt.to_rfc3339()})
                }
                loupe::TypedValue::Null => serde_json::json!({"type": "null", "value": null}),
            })
            .collect();
        (bound.sql, serde_json::json!(values_json))
    };

    // Create the run (status = queued)
    let run = state
        .db
        .create_run(
            org_id,
            query.id,
            query.datasource_id,
            &executed_sql,
            &bound_values,
            timeout,
            max_rows,
            user_id,
        )
        .await?;

    Ok(HttpResponse::Created().json(RunResponse::from(run)))
}

async fn execute_adhoc(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    body: web::Json<ExecuteAdHocRequest>,
) -> Result<HttpResponse, Error> {
    let (user_id, org_id, role) = get_user_context(&state, &req).await?;
    // SECURITY: Only editors and admins can execute ad-hoc SQL
    require_permission(role, Permission::Editor)?;

    // SECURITY: Validate SQL to prevent injection attacks
    // This is CRITICAL - validate BEFORE storing or executing
    let validator = SqlValidator::new();
    validator.validate(&body.sql)?;

    // Verify datasource exists
    let datasource = state.db.get_datasource(body.datasource_id, org_id).await?;

    // For ad-hoc queries, no parameter schema is defined (raw SQL only)
    // Create an ephemeral query
    let query = state
        .db
        .create_query(
            org_id,
            datasource.id,
            "_adhoc",
            None,
            &body.sql,
            &serde_json::json!([]),
            &serde_json::json!([]), // empty tags for adhoc
            body.timeout_seconds,
            body.max_rows,
            user_id,
        )
        .await?;

    // Create the run with raw SQL (no parameter binding for ad-hoc)
    let run = state
        .db
        .create_run(
            org_id,
            query.id,
            datasource.id,
            &body.sql,
            &serde_json::json!([]), // Empty params array
            body.timeout_seconds,
            body.max_rows,
            user_id,
        )
        .await?;

    tracing::info!(
        run_id = %run.id,
        user_id = %user_id,
        "Ad-hoc query validated and queued for execution"
    );

    Ok(HttpResponse::Created().json(RunResponse::from(run)))
}

async fn get_run(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;

    let id = path.into_inner();
    let run = state.db.get_run(id, org_id).await?;
    Ok(HttpResponse::Ok().json(RunResponse::from(run)))
}

async fn get_run_result(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, _, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;

    let run_id = path.into_inner();
    let result = state.db.get_run_result(run_id).await?;
    Ok(HttpResponse::Ok().json(RunResultResponse::from(result)))
}

/// POST /api/v1/runs/{id}/cancel - Cancel a running query
///
/// Cancels a query that is currently queued or running.
/// Only queries in 'queued' or 'running' status can be cancelled.
///
/// Returns 200 if successfully cancelled, 400 if already completed/failed.
async fn cancel_run(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Editor)?;

    let run_id = path.into_inner();

    // Get the run to check its status
    let run = state.db.get_run(run_id, org_id).await?;

    // Check if run can be cancelled
    match run.status {
        RunStatus::Queued | RunStatus::Running => {
            // Cancel the run
            state.db.cancel_run(run_id).await?;

            tracing::info!(
                run_id = %run_id,
                query_id = %run.query_id,
                org_id = %org_id,
                "Run cancelled by user"
            );

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Run cancelled successfully",
                "run_id": run_id,
            })))
        }
        RunStatus::Completed | RunStatus::Failed | RunStatus::Timeout | RunStatus::Cancelled => {
            Err(Error::BadRequest(format!(
                "Cannot cancel run with status '{:?}'",
                run.status
            )))
        }
    }
}
