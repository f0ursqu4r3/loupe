use crate::AppState;
use crate::permissions::{get_user_context, require_permission, Permission};
use actix_web::{HttpRequest, HttpResponse, web};
use loupe::Error;
use loupe::filtering::{parse_tags, SearchParams, SortParams, SortableColumns};
use loupe::models::{CreateScheduleRequest, ScheduleResponse, TriggerScheduleResponse, UpdateScheduleRequest};
use loupe::validation::validate_request;
use loupe::{PaginatedResponse, PaginationParams};
use std::sync::Arc;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/schedules")
            .route("", web::get().to(list_schedules))
            .route("", web::post().to(create_schedule))
            .route("/{id}", web::get().to(get_schedule))
            .route("/{id}", web::patch().to(update_schedule))
            .route("/{id}", web::delete().to(delete_schedule))
            .route("/{id}/enable", web::post().to(enable_schedule))
            .route("/{id}/disable", web::post().to(disable_schedule))
            .route("/{id}/trigger", web::post().to(trigger_schedule)),
    );
}

#[derive(serde::Deserialize)]
pub struct ListSchedulesQuery {
    // Search parameter
    pub search: Option<String>,

    // Filter parameters
    pub tags: Option<String>,
    pub enabled: Option<bool>,

    // Sort parameters
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,

    // Pagination parameters
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

async fn list_schedules(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    query: web::Query<ListSchedulesQuery>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;

    let mut pagination = PaginationParams {
        limit: query.limit,
        offset: query.offset,
    };
    pagination.validate();

    // Validate and build sort params
    let sort = SortParams {
        sort_by: query.sort_by.clone(),
        sort_direction: query.sort_direction.clone(),
    };
    let (sort_column, sort_direction) = sort.validate_and_build(
        SortableColumns::SCHEDULES,
        "created_at",
    );

    // Parse tags filter
    let tags = query.tags.as_ref().map(|t| parse_tags(t)).filter(|v| !v.is_empty());

    // Get search pattern
    let search_params = SearchParams {
        search: query.search.clone(),
    };
    let search = search_params.get_pattern();

    let (schedules, total) = state
        .db
        .list_schedules_paginated(
            org_id,
            search,
            tags,
            query.enabled,
            &sort_column,
            &sort_direction,
            pagination.limit,
            pagination.offset,
        )
        .await?;

    let items: Vec<ScheduleResponse> = schedules.into_iter().map(Into::into).collect();

    let paginated = PaginatedResponse::new(items, total, &pagination);
    Ok(HttpResponse::Ok().json(paginated))
}

async fn get_schedule(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;

    let id = path.into_inner();
    let schedule = state.db.get_schedule(id, org_id).await?;
    Ok(HttpResponse::Ok().json(ScheduleResponse::from(schedule)))
}

async fn create_schedule(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    body: web::Json<CreateScheduleRequest>,
) -> Result<HttpResponse, Error> {
    let (user_id, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Editor)?;

    // Validate request
    validate_request(&*body)?;

    // Verify query exists
    state.db.get_query(body.query_id, org_id).await?;

    let tags = serde_json::to_value(&body.tags).unwrap_or_default();

    let schedule = state
        .db
        .create_schedule(
            org_id,
            body.query_id,
            &body.name,
            &body.cron_expression,
            &body.parameters,
            &tags,
            body.enabled,
            user_id,
        )
        .await?;

    Ok(HttpResponse::Created().json(ScheduleResponse::from(schedule)))
}

async fn update_schedule(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateScheduleRequest>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Editor)?;

    // Validate request
    validate_request(&*body)?;

    let id = path.into_inner();

    let tags = body
        .tags
        .as_ref()
        .map(|t| serde_json::to_value(t).unwrap_or_default());

    let schedule = state
        .db
        .update_schedule(
            id,
            org_id,
            body.name.as_deref(),
            body.cron_expression.as_deref(),
            body.parameters.as_ref(),
            tags.as_ref(),
            body.enabled,
        )
        .await?;

    Ok(HttpResponse::Ok().json(ScheduleResponse::from(schedule)))
}

async fn delete_schedule(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Editor)?;

    let id = path.into_inner();
    state.db.delete_schedule(id, org_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn enable_schedule(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Editor)?;

    let id = path.into_inner();
    let schedule = state.db.enable_schedule(id, org_id).await?;
    Ok(HttpResponse::Ok().json(ScheduleResponse::from(schedule)))
}

async fn disable_schedule(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Editor)?;

    let id = path.into_inner();
    let schedule = state.db.disable_schedule(id, org_id).await?;
    Ok(HttpResponse::Ok().json(ScheduleResponse::from(schedule)))
}

async fn trigger_schedule(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (user_id, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Editor)?;

    let id = path.into_inner();

    // Get the schedule to find the associated query
    let schedule = state.db.get_schedule(id, org_id).await?;

    // Get the query
    let query = state.db.get_query(schedule.query_id, org_id).await?;

    // Merge schedule parameters with query defaults
    let parameters = schedule.parameters;

    // Create a run for this query with the schedule's parameters
    let run = state
        .db
        .create_run(
            org_id,
            query.id,
            query.datasource_id,
            &query.sql,
            &parameters,
            query.timeout_seconds,
            query.max_rows,
            user_id,
        )
        .await?;

    // Update last_run_at and calculate next_run_at
    state
        .db
        .update_schedule_last_run(id, &schedule.cron_expression, schedule.enabled)
        .await?;

    Ok(HttpResponse::Ok().json(TriggerScheduleResponse {
        run_id: run.id,
        message: "Schedule triggered successfully".to_string(),
    }))
}
