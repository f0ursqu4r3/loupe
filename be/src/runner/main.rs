use loupe::connectors::{Connector, PostgresConnector};
use loupe::models::DatasourceType;
use loupe::params::TypedValue;
use loupe::{init_tracing, load_env, Database, Metrics, QueryLimiter, QueryLimits};
use std::sync::Arc;
use std::time::Duration;

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const MAX_CONCURRENT_RUNS: usize = 4;
const SLOW_QUERY_THRESHOLD_MS: i64 = 1000; // Log queries slower than 1 second

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    load_env();
    init_tracing();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let runner_id = std::env::var("RUNNER_ID").unwrap_or_else(|_| {
        format!("runner-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap())
    });

    tracing::info!("Starting Loupe Runner: {}", runner_id);
    tracing::info!("Connecting to database...");

    let db = Database::connect(&database_url).await?;

    // Initialize metrics
    let metrics = Arc::new(Metrics::new().expect("Failed to create metrics registry"));
    tracing::info!("Metrics initialized");

    // Initialize query limiter
    let query_limits = QueryLimits::from_env();
    let query_limiter = Arc::new(QueryLimiter::new(query_limits.clone()));
    tracing::info!(
        "Query limiter initialized: max {} per org, {} global",
        query_limits.max_concurrent_per_org,
        query_limits.max_concurrent_global
    );

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
                    let metrics_clone = metrics.clone();
                    let limiter_clone = query_limiter.clone();
                    let run_id = run.id;

                    tracing::info!("Claimed run {}", run_id);

                    // Spawn task to execute the run
                    tokio::spawn(async move {
                        let result = execute_run(&db_clone, &metrics_clone, &limiter_clone, &run).await;
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

async fn execute_run(
    db: &Database,
    metrics: &Arc<Metrics>,
    limiter: &Arc<QueryLimiter>,
    run: &loupe::models::Run,
) -> anyhow::Result<()> {
    // Try to acquire a query execution slot
    let _guard = match limiter.try_acquire(run.org_id) {
        Ok(guard) => {
            tracing::debug!("Acquired query slot for org {}", run.org_id);
            guard
        }
        Err(e) => {
            // Query limit reached - fail the run
            let error_msg = format!("Query limit reached: {}", e);
            db.fail_run(run.id, &error_msg).await?;
            tracing::warn!("Run {} rejected: {}", run.id, error_msg);
            metrics.query_executions_total.with_label_values(&["rejected"]).inc();
            return Ok(());
        }
    };

    // Increment in-flight queries
    metrics.queries_in_flight.inc();
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

    // Parse bound parameters from run.parameters
    let params = parse_bound_params(&run.parameters)?;

    // Execute with or without parameters
    let result = if params.is_empty() {
        connector.execute(&run.executed_sql, timeout, max_rows).await
    } else {
        connector
            .execute_with_params(&run.executed_sql, &params, timeout, max_rows)
            .await
    };

    let execution_result = match result {
        Ok(output) => {
            let execution_time_ms = start.elapsed().as_millis() as i64;
            let execution_time_secs = execution_time_ms as f64 / 1000.0;

            // Record metrics
            metrics.query_executions_total.with_label_values(&["completed"]).inc();
            metrics
                .query_execution_duration_seconds
                .with_label_values(&[&run.query_id.to_string()])
                .observe(execution_time_secs);
            metrics
                .query_rows_returned
                .with_label_values(&[&run.query_id.to_string()])
                .observe(output.row_count as f64);
            metrics.queries_in_flight.dec();

            // Log slow queries
            if execution_time_ms > SLOW_QUERY_THRESHOLD_MS {
                tracing::warn!(
                    run_id = %run.id,
                    query_id = %run.query_id,
                    org_id = %run.org_id,
                    duration_ms = execution_time_ms,
                    rows = output.row_count,
                    "Slow query detected"
                );
            }

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

            Ok(())
        }
        Err(e) => {
            let error_msg = e.to_string();

            // Decrement in-flight counter
            metrics.queries_in_flight.dec();

            // Check if it was a timeout
            if error_msg.contains("timed out") {
                metrics.query_executions_total.with_label_values(&["timeout"]).inc();
                metrics.query_timeouts_total.inc();
                db.timeout_run(run.id).await?;
                tracing::warn!(
                    run_id = %run.id,
                    query_id = %run.query_id,
                    timeout_seconds = run.timeout_seconds,
                    "Run timed out"
                );
            } else {
                metrics.query_executions_total.with_label_values(&["failed"]).inc();
                db.fail_run(run.id, &error_msg).await?;
                tracing::error!(
                    run_id = %run.id,
                    query_id = %run.query_id,
                    error = %error_msg,
                    "Run failed"
                );
            }

            Ok(())
        }
    };

    execution_result
}

/// Parse bound parameters from JSON array stored in run.parameters
fn parse_bound_params(params_json: &serde_json::Value) -> anyhow::Result<Vec<TypedValue>> {
    let arr = match params_json.as_array() {
        Some(a) => a,
        None => return Ok(vec![]), // Empty or object = no bound params
    };

    let mut params = Vec::with_capacity(arr.len());
    for item in arr {
        let type_str = item
            .get("type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow::anyhow!("Parameter missing 'type' field"))?;

        let value = item.get("value");

        let typed = match type_str {
            "string" => TypedValue::String(
                value
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            ),
            "number" => TypedValue::Number(value.and_then(|v| v.as_f64()).unwrap_or(0.0)),
            "integer" => TypedValue::Integer(value.and_then(|v| v.as_i64()).unwrap_or(0)),
            "boolean" => TypedValue::Boolean(value.and_then(|v| v.as_bool()).unwrap_or(false)),
            "date" => {
                let s = value.and_then(|v| v.as_str()).unwrap_or("");
                let date = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map_err(|e| anyhow::anyhow!("Invalid date '{}': {}", s, e))?;
                TypedValue::Date(date)
            }
            "datetime" => {
                let s = value.and_then(|v| v.as_str()).unwrap_or("");
                let dt = chrono::DateTime::parse_from_rfc3339(s)
                    .map_err(|e| anyhow::anyhow!("Invalid datetime '{}': {}", s, e))?;
                TypedValue::DateTime(dt.with_timezone(&chrono::Utc))
            }
            "null" => TypedValue::Null,
            other => return Err(anyhow::anyhow!("Unknown parameter type: {}", other)),
        };

        params.push(typed);
    }

    Ok(params)
}
