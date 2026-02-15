use loupe::{ObservabilityConfig, init_tracing, load_env, Database};
use std::time::Duration;

const DEFAULT_POLL_INTERVAL_SECS: u64 = 10;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    load_env();
    init_tracing(&ObservabilityConfig::from_env());

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let scheduler_id = std::env::var("SCHEDULER_ID").unwrap_or_else(|_| {
        format!(
            "scheduler-{}",
            uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
        )
    });
    let poll_interval = std::env::var("SCHEDULER_POLL_INTERVAL_SECONDS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(DEFAULT_POLL_INTERVAL_SECS);

    tracing::info!("Starting Loupe Scheduler: {}", scheduler_id);
    tracing::info!("Connecting to database...");

    let db = Database::connect(&database_url).await?;

    tracing::info!("Scheduler ready, polling every {}s", poll_interval);

    loop {
        if let Err(e) = poll_and_enqueue(&db).await {
            tracing::error!("Scheduler poll failed: {}", e);
        }
        tokio::time::sleep(Duration::from_secs(poll_interval)).await;
    }
}

async fn poll_and_enqueue(db: &Database) -> anyhow::Result<()> {
    let schedules = db.get_due_schedules().await?;

    if schedules.is_empty() {
        return Ok(());
    }

    tracing::info!("Found {} due schedules", schedules.len());

    for schedule in schedules {
        let schedule_id = schedule.id;
        let org_id = schedule.org_id;

        let query = match db.get_query(schedule.query_id, org_id).await {
            Ok(query) => query,
            Err(e) => {
                tracing::error!(
                    "Failed to load query {} for schedule {}: {}",
                    schedule.query_id,
                    schedule_id,
                    e
                );
                continue;
            }
        };

        let run = match db
            .create_run(
                org_id,
                query.id,
                query.datasource_id,
                &query.sql,
                &schedule.parameters,
                query.timeout_seconds,
                query.max_rows,
                schedule.created_by,
            )
            .await
        {
            Ok(run) => run,
            Err(e) => {
                tracing::error!("Failed to enqueue run for schedule {}: {}", schedule_id, e);
                continue;
            }
        };

        if let Err(e) = db
            .update_schedule_last_run(schedule_id, &schedule.cron_expression, schedule.enabled)
            .await
        {
            tracing::error!(
                "Failed to update last_run_at for schedule {}: {}",
                schedule_id,
                e
            );
            continue;
        }

        tracing::info!("Enqueued run {} for schedule {}", run.id, schedule_id);
    }

    Ok(())
}
