use loupe::connectors::{Connector, PostgresConnector};
use loupe::models::DatasourceType;
use loupe::Database;
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const MAX_CONCURRENT_RUNS: usize = 4;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,sqlx=warn".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let runner_id = std::env::var("RUNNER_ID").unwrap_or_else(|_| {
        format!("runner-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap())
    });

    tracing::info!("Starting Loupe Runner: {}", runner_id);
    tracing::info!("Connecting to database...");

    let db = Database::connect(&database_url).await?;

    tracing::info!("Runner ready, polling for jobs...");

    // Simple polling loop
    let mut active_tasks = 0;

    loop {
        // Only claim new work if under concurrency limit
        if active_tasks < MAX_CONCURRENT_RUNS {
            match db.claim_run(&runner_id).await {
                Ok(Some(run)) => {
                    active_tasks += 1;
                    let db_clone = db.clone();
                    let run_id = run.id;

                    tracing::info!("Claimed run {}", run_id);

                    // Spawn task to execute the run
                    tokio::spawn(async move {
                        let result = execute_run(&db_clone, &run).await;
                        if let Err(e) = result {
                            tracing::error!("Run {} failed: {}", run_id, e);
                        }
                    });

                    // Decrement will happen when task completes
                    // For simplicity, we'll just continue polling
                    // A proper implementation would track task handles
                    active_tasks -= 1;
                }
                Ok(None) => {
                    // No work available, wait before polling again
                    tokio::time::sleep(POLL_INTERVAL).await;
                }
                Err(e) => {
                    tracing::error!("Error claiming run: {}", e);
                    tokio::time::sleep(POLL_INTERVAL).await;
                }
            }
        } else {
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }
}

async fn execute_run(db: &Database, run: &loupe::models::Run) -> anyhow::Result<()> {
    let start = std::time::Instant::now();

    // Get the datasource
    let datasource = db.get_datasource(run.datasource_id, run.org_id).await?;

    // Create connector based on type
    let connector: Box<dyn Connector> = match datasource.ds_type {
        DatasourceType::Postgres => {
            Box::new(PostgresConnector::new(&datasource.connection_string_encrypted).await?)
        }
    };

    // Execute the query with timeout and row limit
    let timeout = Duration::from_secs(run.timeout_seconds as u64);
    let max_rows = run.max_rows as usize;

    match connector.execute(&run.executed_sql, timeout, max_rows).await {
        Ok(output) => {
            let execution_time_ms = start.elapsed().as_millis() as i64;

            // Serialize results
            let columns = serde_json::to_value(&output.columns)?;
            let rows = serde_json::to_value(&output.rows)?;
            let byte_count = serde_json::to_string(&rows)?.len() as i64;

            // Store result
            let result = db
                .create_run_result(
                    run.id,
                    &columns,
                    &rows,
                    output.row_count as i64,
                    byte_count,
                    execution_time_ms,
                )
                .await?;

            // Mark run as completed
            db.complete_run(run.id, result.id).await?;

            tracing::info!(
                "Run {} completed: {} rows in {}ms",
                run.id,
                output.row_count,
                execution_time_ms
            );
        }
        Err(e) => {
            let error_msg = e.to_string();

            // Check if it was a timeout
            if error_msg.contains("timed out") {
                db.timeout_run(run.id).await?;
                tracing::warn!("Run {} timed out", run.id);
            } else {
                db.fail_run(run.id, &error_msg).await?;
                tracing::error!("Run {} failed: {}", run.id, error_msg);
            }
        }
    }

    Ok(())
}
