use crate::AppState;
use crate::routes::auth::get_auth_context;
use actix_web::{HttpRequest, HttpResponse, web};
use loupe::Error;
use loupe::models::{CreateScheduleRequest, ScheduleResponse};
use std::sync::Arc;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/schedules")
            .route("", web::get().to(list_schedules))
            .route("", web::post().to(create_schedule)),
    );
}

async fn list_schedules(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_auth_context(&req)?;
    let schedules = state.db.list_schedules(org_id).await?;
    let response: Vec<ScheduleResponse> = schedules.into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(response))
}

async fn create_schedule(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    body: web::Json<CreateScheduleRequest>,
) -> Result<HttpResponse, Error> {
    let (user_id, org_id) = get_auth_context(&req)?;

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
