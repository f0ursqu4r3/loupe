use crate::AppState;
use actix_web::{web, HttpResponse};
use loupe::models::{CreateScheduleRequest, ScheduleResponse};
use loupe::Error;
use std::sync::Arc;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/schedules")
            .route("", web::get().to(list_schedules))
            .route("", web::post().to(create_schedule)),
    );
}

fn get_current_user() -> (Uuid, Uuid) {
    (
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
    )
}

async fn list_schedules(state: web::Data<Arc<AppState>>) -> Result<HttpResponse, Error> {
    let (_, org_id) = get_current_user();
    let schedules = state.db.list_schedules(org_id).await?;
    let response: Vec<ScheduleResponse> = schedules.into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(response))
}

async fn create_schedule(
    state: web::Data<Arc<AppState>>,
    req: web::Json<CreateScheduleRequest>,
) -> Result<HttpResponse, Error> {
    let (user_id, org_id) = get_current_user();

    // Verify query exists
    state.db.get_query(req.query_id, org_id).await?;

    let schedule = state
        .db
        .create_schedule(
            org_id,
            req.query_id,
            &req.name,
            &req.cron_expression,
            &req.parameters,
            req.enabled,
            user_id,
        )
        .await?;

    Ok(HttpResponse::Created().json(ScheduleResponse::from(schedule)))
}
