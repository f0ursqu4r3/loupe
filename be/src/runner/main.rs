use loupe::connectors::{Connector, PostgresConnector};
use loupe::models::DatasourceType;
use loupe::params::TypedValue;
use loupe::{init_tracing, load_env, Database, Metrics, QueryLimiter, QueryLimits};
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tokio::sync::broadcast;
use tokio::task::JoinSet;

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const MAX_CONCURRENT_RUNS: usize = 4;
const SLOW_QUERY_THRESHOLD_MS: i64 = 1000; // Log queries slower than 1 second
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30); // Grace period for in-flight tasks

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

    // Create shutdown channel
    let (shutdown_tx, _) = broadcast::channel(1);
    let mut shutdown_rx = shutdown_tx.subscribe();

    // Spawn shutdown signal handler
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        shutdown_signal().await;
        tracing::info!("Shutdown signal received, initiating graceful shutdown...");
        let _ = shutdown_tx_clone.send(());
    });

    tracing::info!("Runner ready, polling for jobs...");

    // Task set for tracking spawned jobs
    let mut tasks = JoinSet::new();

    // Main polling loop
    loop {
        tokio::select! {
            // Check for shutdown signal
            _ = shutdown_rx.recv() => {
                tracing::info!("Shutting down, waiting for {} in-flight tasks...", tasks.len());

                // Stop accepting new work, wait for existing tasks to complete
                let shutdown_start = std::time::Instant::now();
                while !tasks.is_empty() {
                    if shutdown_start.elapsed() > SHUTDOWN_TIMEOUT {
                        tracing::warn!(
                            "Shutdown timeout reached, aborting {} remaining tasks",
                            tasks.len()
                        );
                        tasks.shutdown().await;
                        break;
                    }

                    tokio::select! {
                        Some(result) = tasks.join_next() => {
                            if let Err(e) = result {
                                tracing::error!("Task panicked during shutdown: {}", e);
                            }
                        }
                        _ = tokio::time::sleep(Duration::from_millis(100)) => {}
                    }
                }

                tracing::info!("Graceful shutdown complete");
                return Ok(());
            }

            // Clean up completed tasks
            Some(result) = tasks.join_next(), if !tasks.is_empty() => {
                if let Err(e) = result {
                    tracing::error!("Task panicked: {}", e);
                }
            }

            // Claim new work if under concurrency limit
            _ = tokio::time::sleep(POLL_INTERVAL), if tasks.len() < MAX_CONCURRENT_RUNS => {
                // First try to claim a retry run (prioritize retries)
                let run_result = db.claim_retry_run(&runner_id).await;
                let (run, run_type) = match run_result {
                    Ok(Some(retry_run)) => {
                        tracing::info!(
                            "Claimed retry run {} (attempt {})",
                            retry_run.id,
                            retry_run.retry_count + 1
                        );
                        metrics.jobs_claimed_total.with_label_values(&["retry"]).inc();
                        (Some(retry_run), "retry")
                    }
                    Ok(None) => {
                        // No retries available, try claiming a new run
                        match db.claim_run(&runner_id).await {
                            Ok(Some(new_run)) => {
                                tracing::info!("Claimed new run {}", new_run.id);
                                metrics.jobs_claimed_total.with_label_values(&["new"]).inc();
                                (Some(new_run), "new")
                            }
                            Ok(None) => (None, "none"),
                            Err(e) => {
                                tracing::error!("Error claiming run: {}", e);
                                (None, "none")
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error claiming retry run: {}", e);
                        (None, "none")
                    }
                };

                // If we got a run (either retry or new), execute it
                if let Some(run) = run {
                    let db_clone = db.clone();
                    let metrics_clone = metrics.clone();
                    let limiter_clone = query_limiter.clone();
                    let run_id = run.id;

                    // Spawn task to execute the run
                    tasks.spawn(async move {
                        let result = execute_run(&db_clone, &metrics_clone, &limiter_clone, &run).await;
                        if let Err(e) = result {
                            tracing::error!("Run {} failed: {}", run_id, e);
                        }
                    });
                }
            }
        }
    }
}

/// Wait for SIGTERM or SIGINT (Ctrl+C)
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
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

            // Determine if error is retryable (connection errors, database unavailable, etc.)
            let is_retryable = is_retryable_error(&error_msg);

            // Check if it was a timeout
            if error_msg.contains("timed out") {
                metrics.query_executions_total.with_label_values(&["timeout"]).inc();
                metrics.query_timeouts_total.inc();

                // Try to schedule retry for timeouts
                if is_retryable {
                    match db.schedule_retry(run.id, &format!("Query timed out: {}", error_msg)).await? {
                        Some(retry_run) => {
                            tracing::warn!(
                                run_id = %run.id,
                                query_id = %run.query_id,
                                retry_count = retry_run.retry_count,
                                next_retry_at = ?retry_run.next_retry_at,
                                "Run timed out, scheduled for retry"
                            );
                        }
                        None => {
                            // Max retries exceeded, move to dead letter queue
                            db.timeout_run(run.id).await?;
                            db.move_to_dead_letter_queue(run.id).await?;
                            tracing::error!(
                                run_id = %run.id,
                                query_id = %run.query_id,
                                "Run timed out, max retries exceeded, moved to dead letter queue"
                            );
                        }
                    }
                } else {
                    db.timeout_run(run.id).await?;
                    tracing::warn!(
                        run_id = %run.id,
                        query_id = %run.query_id,
                        "Run timed out (non-retryable)"
                    );
                }
            } else if is_retryable {
                // Try to schedule retry for retryable errors
                match db.schedule_retry(run.id, &error_msg).await? {
                    Some(retry_run) => {
                        metrics.query_executions_total.with_label_values(&["retry_scheduled"]).inc();
                        tracing::warn!(
                            run_id = %run.id,
                            query_id = %run.query_id,
                            retry_count = retry_run.retry_count,
                            max_retries = retry_run.max_retries,
                            next_retry_at = ?retry_run.next_retry_at,
                            error = %error_msg,
                            "Run failed, scheduled for retry"
                        );
                    }
                    None => {
                        // Max retries exceeded, move to dead letter queue
                        metrics.query_executions_total.with_label_values(&["failed_permanent"]).inc();
                        db.fail_run(run.id, &error_msg).await?;
                        db.move_to_dead_letter_queue(run.id).await?;
                        tracing::error!(
                            run_id = %run.id,
                            query_id = %run.query_id,
                            error = %error_msg,
                            "Run failed permanently (max retries exceeded), moved to dead letter queue"
                        );
                    }
                }
            } else {
                // Non-retryable error (SQL syntax error, invalid credentials, etc.)
                metrics.query_executions_total.with_label_values(&["failed"]).inc();
                db.fail_run(run.id, &error_msg).await?;
                tracing::error!(
                    run_id = %run.id,
                    query_id = %run.query_id,
                    error = %error_msg,
                    "Run failed (non-retryable error)"
                );
            }

            Ok(())
        }
    };

    execution_result
}

/// Determine if an error is retryable
///
/// Retryable errors include:
/// - Connection errors (network issues, database unavailable)
/// - Transient errors (deadlocks, timeout on acquire)
/// - Resource exhaustion (too many connections)
///
/// Non-retryable errors include:
/// - SQL syntax errors
/// - Invalid credentials
/// - Permission denied
/// - Data type mismatches
fn is_retryable_error(error_msg: &str) -> bool {
    let lower = error_msg.to_lowercase();

    // Retryable errors
    if lower.contains("connection")
        || lower.contains("network")
        || lower.contains("timeout")
        || lower.contains("unavailable")
        || lower.contains("deadlock")
        || lower.contains("too many connections")
        || lower.contains("could not connect")
        || lower.contains("connection refused")
        || lower.contains("connection reset")
    {
        return true;
    }

    // Non-retryable errors
    if lower.contains("syntax error")
        || lower.contains("permission denied")
        || lower.contains("authentication failed")
        || lower.contains("invalid credentials")
        || lower.contains("does not exist")
        || lower.contains("type mismatch")
        || lower.contains("constraint violation")
    {
        return false;
    }

    // Default to non-retryable for unknown errors
    false
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
