pub mod auth;
mod canvases;
mod dashboards;
mod datasources;
mod health;
mod queries;
mod runs;
mod schedules;
mod visualizations;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(health::configure)
            .configure(auth::configure)
            .configure(datasources::configure)
            .configure(queries::configure)
            .configure(runs::configure)
            .configure(dashboards::configure)
            .configure(visualizations::configure)
            .configure(schedules::configure)
            .configure(canvases::configure),
    );
}
