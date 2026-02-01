//! End-to-End Workflow Integration Tests
//!
//! These tests verify complete user workflows from start to finish,
//! ensuring all components work together correctly.

use actix_web::{App, http::StatusCode, test, web};
use loupe::db::Database;
use loupe::models::*;
use serde_json::json;
use std::sync::Arc;
use testcontainers::{ContainerAsync, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;

/// Application state for workflow tests
pub struct AppState {
    pub db: Database,
}

/// Test helper providing isolated test environment
struct TestWorkflow {
    db: Database,
    #[allow(dead_code)]
    container: ContainerAsync<Postgres>,
}

impl TestWorkflow {
    async fn new() -> Self {
        let container = Postgres::default()
            .start()
            .await
            .expect("Failed to start postgres container");

        let host = container.get_host().await.expect("get host");
        let port = container.get_host_port_ipv4(5432).await.expect("get port");
        let database_url = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);

        let db = Database::connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        db.run_migrations().await.expect("Failed to run migrations");

        Self { db, container }
    }

    fn app_state(&self) -> Arc<AppState> {
        Arc::new(AppState {
            db: self.db.clone(),
        })
    }
}

mod complete_analytics_workflow {
    use super::*;

    /// Tests the complete analytics workflow:
    /// 1. Register user
    /// 2. Create datasource
    /// 3. Create multiple queries
    /// 4. Create visualizations for queries
    /// 5. Create dashboard with tiles
    /// 6. Execute queries and verify results
    #[tokio::test]
    async fn test_complete_analytics_pipeline() {
        let test = TestWorkflow::new().await;
        let db = &test.db;

        // Step 1: Setup organization and user
        let org = db.create_organization("Analytics Corp").await.unwrap();
        let user = db
            .create_user(
                org.id,
                "analyst@analytics.com",
                "hashed_password",
                "Senior Analyst",
                OrgRole::Admin,
            )
            .await
            .unwrap();

        assert_eq!(user.role, OrgRole::Admin);
        assert_eq!(user.org_id, org.id);

        // Step 2: Create datasource
        let datasource = db
            .create_datasource(
                org.id,
                "Production Analytics DB",
                DatasourceType::Postgres,
                "postgres://localhost:5432/analytics",
                user.id,
            )
            .await
            .unwrap();

        // Step 3: Create multiple queries
        let daily_users_query = db
            .create_query(
                org.id,
                datasource.id,
                "Daily Active Users",
                Some("Count of daily active users over time"),
                "SELECT date_trunc('day', login_at) as date, COUNT(DISTINCT user_id) as dau FROM logins GROUP BY date",
                &json!([]),
                &json!([]),
                30,
                10000,
                user.id,
            )
            .await
            .unwrap();

        let revenue_query = db
            .create_query(
                org.id,
                datasource.id,
                "Daily Revenue",
                Some("Total revenue per day"),
                "SELECT date, SUM(amount) as revenue FROM transactions GROUP BY date",
                &json!([]),
                &json!([]),
                30,
                10000,
                user.id,
            )
            .await
            .unwrap();

        let top_products_query = db
            .create_query(
                org.id,
                datasource.id,
                "Top 10 Products",
                Some("Best selling products this month"),
                "SELECT product_name, COUNT(*) as sales FROM orders WHERE created_at > NOW() - INTERVAL '30 days' GROUP BY product_name ORDER BY sales DESC LIMIT 10",
                &json!([]),
                &json!([]),
                60,
                10,
                user.id,
            )
            .await
            .unwrap();

        // Verify queries were created
        let all_queries = db.list_queries(org.id).await.unwrap();
        assert_eq!(all_queries.len(), 3);

        // Step 4: Create visualizations
        let dau_line_chart = db
            .create_visualization(
                org.id,
                daily_users_query.id,
                "DAU Trend Line",
                ChartType::Line,
                &json!({
                    "x_axis": "date",
                    "y_axis": "dau",
                    "title": "Daily Active Users Over Time"
                }),
                &json!([]),
                user.id,
            )
            .await
            .unwrap();

        let revenue_bar_chart = db
            .create_visualization(
                org.id,
                revenue_query.id,
                "Revenue Bar Chart",
                ChartType::Bar,
                &json!({
                    "x_axis": "date",
                    "y_axis": "revenue",
                    "title": "Daily Revenue"
                }),
                &json!([]),
                user.id,
            )
            .await
            .unwrap();

        let products_table = db
            .create_visualization(
                org.id,
                top_products_query.id,
                "Top Products Table",
                ChartType::Table,
                &json!({
                    "columns": ["product_name", "sales"]
                }),
                &json!([]),
                user.id,
            )
            .await
            .unwrap();

        // Step 5: Create dashboard and add tiles
        let dashboard = db
            .create_dashboard(
                org.id,
                "Executive Dashboard",
                Some("Key business metrics at a glance"),
                &json!({
                    "refresh_interval": 300
                }),
                &json!(["executive", "kpi"]),
                user.id,
            )
            .await
            .unwrap();

        // Add DAU chart to top-left
        let dau_tile = db
            .create_tile(
                dashboard.id,
                dau_line_chart.id,
                Some("Daily Active Users"),
                0,
                0,
                12,
                6,
                &json!({}),
            )
            .await
            .unwrap();

        // Add revenue chart to top-right
        let revenue_tile = db
            .create_tile(
                dashboard.id,
                revenue_bar_chart.id,
                Some("Revenue"),
                0,
                6,
                12,
                6,
                &json!({}),
            )
            .await
            .unwrap();

        // Add products table below
        let products_tile = db
            .create_tile(
                dashboard.id,
                products_table.id,
                Some("Top Selling Products"),
                12,
                0,
                12,
                8,
                &json!({}),
            )
            .await
            .unwrap();

        // Verify dashboard has 3 tiles
        let tiles = db.list_tiles(dashboard.id).await.unwrap();
        assert_eq!(tiles.len(), 3);
        assert_eq!(tiles[0].id, dau_tile.id);
        assert_eq!(tiles[1].id, revenue_tile.id);
        assert_eq!(tiles[2].id, products_tile.id);

        // Step 6: Create and execute query runs
        let run = db
            .create_run(
                org.id,
                daily_users_query.id,
                datasource.id,
                &daily_users_query.sql,
                &json!({}),
                30,
                10000,
                user.id,
            )
            .await
            .unwrap();

        assert_eq!(run.status, RunStatus::Queued);

        // Simulate runner claiming the run
        let claimed_run = db.claim_run("test-runner-1").await.unwrap();
        assert!(claimed_run.is_some());
        let claimed = claimed_run.unwrap();
        assert_eq!(claimed.id, run.id);
        assert_eq!(claimed.status, RunStatus::Running);

        // Simulate query execution result
        let result = db
            .create_run_result(
                run.id,
                &json!([
                    {"name": "date", "data_type": "DATE"},
                    {"name": "dau", "data_type": "INT8"}
                ]),
                &json!([
                    ["2024-01-01", 1523],
                    ["2024-01-02", 1678],
                    ["2024-01-03", 1834]
                ]),
                3,
                256,
                125,
            )
            .await
            .unwrap();

        // Complete the run
        let completed = db.complete_run(run.id, result.id).await.unwrap();
        assert_eq!(completed.status, RunStatus::Completed);
        assert!(completed.completed_at.is_some());

        // Verify result can be retrieved
        let fetched_result = db.get_run_result(run.id).await.unwrap();
        assert_eq!(fetched_result.row_count, 3);
        assert_eq!(fetched_result.execution_time_ms, 125);

        // Step 7: Verify complete workflow integrity
        let dashboards = db.list_dashboards(org.id).await.unwrap();
        assert_eq!(dashboards.len(), 1);
        assert_eq!(dashboards[0].name, "Executive Dashboard");

        let visualizations = db.list_visualizations(org.id, None).await.unwrap();
        assert_eq!(visualizations.len(), 3);

        let queries = db.list_queries(org.id).await.unwrap();
        assert_eq!(queries.len(), 3);

        let datasources = db.list_datasources(org.id).await.unwrap();
        assert_eq!(datasources.len(), 1);
    }
}

mod scheduled_query_workflow {
    use super::*;

    /// Tests scheduled query execution workflow:
    /// 1. Create query
    /// 2. Create schedule
    /// 3. Find due schedules
    /// 4. Execute scheduled query
    /// 5. Update last run time
    #[tokio::test]
    async fn test_scheduled_query_execution() {
        let test = TestWorkflow::new().await;
        let db = &test.db;

        // Setup
        let org = db.create_organization("Scheduled Org").await.unwrap();
        let user = db
            .create_user(org.id, "scheduler@example.com", "hash", "Scheduler", OrgRole::Admin)
            .await
            .unwrap();
        let datasource = db
            .create_datasource(
                org.id,
                "Analytics DB",
                DatasourceType::Postgres,
                "conn",
                user.id,
            )
            .await
            .unwrap();

        // Create a query for scheduled execution
        let query = db
            .create_query(
                org.id,
                datasource.id,
                "Hourly Metrics",
                Some("Runs every hour to collect metrics"),
                "SELECT COUNT(*) as metric_count FROM metrics WHERE created_at > NOW() - INTERVAL '1 hour'",
                &json!([]),
                &json!([]),
                30,
                1000,
                user.id,
            )
            .await
            .unwrap();

        // Create a schedule for the query (every hour)
        let schedule = db
            .create_schedule(
                org.id,
                query.id,
                "Hourly Collection",
                "0 * * * *", // Every hour at :00
                &json!({}),
                &json!(["automated", "metrics"]),
                true,
                user.id,
            )
            .await
            .unwrap();

        assert_eq!(schedule.cron_expression, "0 * * * *");
        assert!(schedule.enabled);

        // Check for due schedules (should include our new schedule)
        let due_schedules = db.get_due_schedules().await.unwrap();
        assert!(due_schedules.iter().any(|s| s.id == schedule.id));

        // Create a run for the scheduled query
        let run = db
            .create_run(
                org.id,
                query.id,
                datasource.id,
                &query.sql,
                &json!({}),
                query.timeout_seconds,
                query.max_rows,
                user.id,
            )
            .await
            .unwrap();

        // Simulate execution
        let claimed = db.claim_run("scheduler-runner").await.unwrap().unwrap();
        assert_eq!(claimed.id, run.id);

        // Simulate successful execution
        let result = db
            .create_run_result(
                run.id,
                &json!([{"name": "metric_count", "data_type": "INT8"}]),
                &json!([[42]]),
                1,
                8,
                50,
            )
            .await
            .unwrap();

        db.complete_run(run.id, result.id).await.unwrap();

        // Update the schedule's last run time
        db.update_schedule_last_run(schedule.id, &schedule.cron_expression, true)
            .await
            .unwrap();

        // Verify schedule was updated
        let schedules = db.list_schedules(org.id).await.unwrap();
        let updated_schedule = schedules.iter().find(|s| s.id == schedule.id).unwrap();
        assert!(updated_schedule.last_run_at.is_some());
        assert!(updated_schedule.next_run_at.is_some());
    }

    /// Tests disabling/enabling schedules
    #[tokio::test]
    async fn test_schedule_enable_disable() {
        let test = TestWorkflow::new().await;
        let db = &test.db;

        // Setup
        let org = db.create_organization("Toggle Org").await.unwrap();
        let user = db
            .create_user(org.id, "toggle@example.com", "hash", "User", OrgRole::Admin)
            .await
            .unwrap();
        let datasource = db
            .create_datasource(org.id, "DB", DatasourceType::Postgres, "conn", user.id)
            .await
            .unwrap();
        let query = db
            .create_query(
                org.id,
                datasource.id,
                "Test Query",
                None,
                "SELECT 1",
                &json!([]),
                &json!([]),
                30,
                1000,
                user.id,
            )
            .await
            .unwrap();

        // Create enabled schedule
        let schedule = db
            .create_schedule(
                org.id,
                query.id,
                "Toggleable",
                "0 0 * * *",
                &json!({}),
                &json!([]),
                true,
                user.id,
            )
            .await
            .unwrap();

        assert!(schedule.enabled);

        // Disable schedule
        let disabled = db
            .update_schedule(schedule.id, org.id, None, None, None, None, Some(false))
            .await
            .unwrap();

        assert!(!disabled.enabled);

        // Verify disabled schedules don't appear in due list
        let due = db.get_due_schedules().await.unwrap();
        assert!(!due.iter().any(|s| s.id == schedule.id));

        // Re-enable
        let enabled = db
            .update_schedule(schedule.id, org.id, None, None, None, None, Some(true))
            .await
            .unwrap();

        assert!(enabled.enabled);
    }
}

mod concurrent_operations {
    use super::*;
    use tokio::task::JoinSet;

    /// Tests concurrent query executions don't interfere with each other
    #[tokio::test]
    async fn test_concurrent_query_runs() {
        let test = TestWorkflow::new().await;
        let db = &test.db;

        // Setup
        let org = db.create_organization("Concurrent Org").await.unwrap();
        let user = db
            .create_user(org.id, "concurrent@example.com", "hash", "User", OrgRole::Admin)
            .await
            .unwrap();
        let datasource = db
            .create_datasource(org.id, "DB", DatasourceType::Postgres, "conn", user.id)
            .await
            .unwrap();
        let query = db
            .create_query(
                org.id,
                datasource.id,
                "Test Query",
                None,
                "SELECT 1",
                &json!([]),
                &json!([]),
                30,
                1000,
                user.id,
            )
            .await
            .unwrap();

        // Create 10 concurrent runs
        let mut run_ids = vec![];
        for _ in 0..10 {
            let run = db
                .create_run(
                    org.id,
                    query.id,
                    datasource.id,
                    &query.sql,
                    &json!({}),
                    30,
                    1000,
                    user.id,
                )
                .await
                .unwrap();
            run_ids.push(run.id);
        }

        // Simulate 10 runners claiming runs concurrently
        let mut set = JoinSet::new();
        for i in 0..10 {
            let runner_id = format!("runner-{}", i);
            let db_clone = db.clone();
            set.spawn(async move {
                db_clone.claim_run(&runner_id).await
            });
        }

        let mut claimed = vec![];
        while let Some(res) = set.join_next().await {
            if let Ok(Ok(Some(run))) = res {
                claimed.push(run.id);
            }
        }

        // All 10 runs should be claimed
        assert_eq!(claimed.len(), 10);

        // Each run should be claimed exactly once (no duplicates)
        claimed.sort();
        claimed.dedup();
        assert_eq!(claimed.len(), 10);
    }

    /// Tests concurrent dashboard updates don't cause conflicts
    #[tokio::test]
    async fn test_concurrent_tile_additions() {
        let test = TestWorkflow::new().await;
        let db = &test.db;

        // Setup
        let org = db.create_organization("Tile Org").await.unwrap();
        let user = db
            .create_user(org.id, "tiles@example.com", "hash", "User", OrgRole::Admin)
            .await
            .unwrap();
        let datasource = db
            .create_datasource(org.id, "DB", DatasourceType::Postgres, "conn", user.id)
            .await
            .unwrap();
        let query = db
            .create_query(
                org.id,
                datasource.id,
                "Q",
                None,
                "SELECT 1",
                &json!([]),
                &json!([]),
                30,
                1000,
                user.id,
            )
            .await
            .unwrap();
        let viz = db
            .create_visualization(
                org.id,
                query.id,
                "V",
                ChartType::Table,
                &json!({}),
                &json!([]),
                user.id,
            )
            .await
            .unwrap();
        let dashboard = db
            .create_dashboard(org.id, "D", None, &json!({}), &json!([]), user.id)
            .await
            .unwrap();

        // Add 10 tiles concurrently
        let mut set = JoinSet::new();
        for i in 0..10 {
            let db_clone = db.clone();
            let dashboard_id = dashboard.id;
            let viz_id = viz.id;
            set.spawn(async move {
                db_clone
                    .create_tile(
                        dashboard_id,
                        viz_id,
                        Some(&format!("Tile {}", i)),
                        i as i32,
                        0,
                        4,
                        4,
                        &json!({}),
                    )
                    .await
            });
        }

        let mut tile_count = 0;
        while let Some(res) = set.join_next().await {
            if res.unwrap().is_ok() {
                tile_count += 1;
            }
        }

        assert_eq!(tile_count, 10);

        // Verify all tiles were created
        let tiles = db.list_tiles(dashboard.id).await.unwrap();
        assert_eq!(tiles.len(), 10);
    }
}

mod error_scenarios {
    use super::*;

    /// Tests graceful handling of failed query runs
    #[tokio::test]
    async fn test_query_run_failure() {
        let test = TestWorkflow::new().await;
        let db = &test.db;

        // Setup
        let org = db.create_organization("Error Org").await.unwrap();
        let user = db
            .create_user(org.id, "error@example.com", "hash", "User", OrgRole::Admin)
            .await
            .unwrap();
        let datasource = db
            .create_datasource(org.id, "DB", DatasourceType::Postgres, "conn", user.id)
            .await
            .unwrap();
        let query = db
            .create_query(
                org.id,
                datasource.id,
                "Failing Query",
                None,
                "SELECT * FROM nonexistent_table",
                &json!([]),
                &json!([]),
                30,
                1000,
                user.id,
            )
            .await
            .unwrap();

        // Create run
        let run = db
            .create_run(
                org.id,
                query.id,
                datasource.id,
                &query.sql,
                &json!({}),
                30,
                1000,
                user.id,
            )
            .await
            .unwrap();

        // Claim and fail the run
        db.claim_run("runner-1").await.unwrap();
        let failed = db
            .fail_run(run.id, "relation \"nonexistent_table\" does not exist")
            .await
            .unwrap();

        assert_eq!(failed.status, RunStatus::Failed);
        assert!(failed.error_message.is_some());
        assert!(failed.completed_at.is_some());
        assert_eq!(failed.runner_id, Some("runner-1".to_string()));
    }

    /// Tests organization isolation - users can't access other org's data
    #[tokio::test]
    async fn test_organization_isolation() {
        let test = TestWorkflow::new().await;
        let db = &test.db;

        // Create two separate organizations
        let org1 = db.create_organization("Org 1").await.unwrap();
        let org2 = db.create_organization("Org 2").await.unwrap();

        let user1 = db
            .create_user(org1.id, "user1@org1.com", "hash", "User 1", OrgRole::Admin)
            .await
            .unwrap();
        let user2 = db
            .create_user(org2.id, "user2@org2.com", "hash", "User 2", OrgRole::Admin)
            .await
            .unwrap();

        // Create datasource in org1
        let ds1 = db
            .create_datasource(org1.id, "Org1 DS", DatasourceType::Postgres, "conn1", user1.id)
            .await
            .unwrap();

        // Try to access org1's datasource from org2 - should fail
        let wrong_org_access = db.get_datasource(ds1.id, org2.id).await;
        assert!(wrong_org_access.is_err());

        // Create dashboard in org1
        let dash1 = db
            .create_dashboard(org1.id, "Org1 Dashboard", None, &json!({}), &json!([]), user1.id)
            .await
            .unwrap();

        // Try to access org1's dashboard from org2 - should fail
        let wrong_dashboard = db.get_dashboard(dash1.id, org2.id).await;
        assert!(wrong_dashboard.is_err());

        // List operations should only return org-specific data
        let org1_datasources = db.list_datasources(org1.id).await.unwrap();
        let org2_datasources = db.list_datasources(org2.id).await.unwrap();
        assert_eq!(org1_datasources.len(), 1);
        assert_eq!(org2_datasources.len(), 0);

        let org1_dashboards = db.list_dashboards(org1.id).await.unwrap();
        let org2_dashboards = db.list_dashboards(org2.id).await.unwrap();
        assert_eq!(org1_dashboards.len(), 1);
        assert_eq!(org2_dashboards.len(), 0);
    }

    /// Tests cascade deletion - deleting query should handle dependencies
    #[tokio::test]
    async fn test_cascade_deletion() {
        let test = TestWorkflow::new().await;
        let db = &test.db;

        // Setup
        let org = db.create_organization("Cascade Org").await.unwrap();
        let user = db
            .create_user(org.id, "cascade@example.com", "hash", "User", OrgRole::Admin)
            .await
            .unwrap();
        let datasource = db
            .create_datasource(org.id, "DB", DatasourceType::Postgres, "conn", user.id)
            .await
            .unwrap();
        let query = db
            .create_query(
                org.id,
                datasource.id,
                "Query",
                None,
                "SELECT 1",
                &json!([]),
                &json!([]),
                30,
                1000,
                user.id,
            )
            .await
            .unwrap();

        // Create dependent resources
        let viz = db
            .create_visualization(
                org.id,
                query.id,
                "Viz",
                ChartType::Table,
                &json!({}),
                &json!([]),
                user.id,
            )
            .await
            .unwrap();

        let schedule = db
            .create_schedule(
                org.id,
                query.id,
                "Schedule",
                "0 * * * *",
                &json!({}),
                &json!([]),
                true,
                user.id,
            )
            .await
            .unwrap();

        // Delete the query
        db.delete_query(query.id, org.id).await.unwrap();

        // Dependent resources should be handled gracefully
        // (Either cascaded or prevented - depends on schema constraints)
        let queries = db.list_queries(org.id).await.unwrap();
        assert!(queries.is_empty());

        // Check if visualizations were cascaded
        let vizs = db.list_visualizations(org.id, Some(query.id)).await.unwrap();
        assert!(vizs.is_empty());
    }
}

mod pagination_workflow {
    use super::*;

    /// Tests pagination across large result sets
    #[tokio::test]
    async fn test_paginated_query_listing() {
        let test = TestWorkflow::new().await;
        let db = &test.db;

        // Setup
        let org = db.create_organization("Pagination Org").await.unwrap();
        let user = db
            .create_user(org.id, "paginate@example.com", "hash", "User", OrgRole::Admin)
            .await
            .unwrap();
        let datasource = db
            .create_datasource(org.id, "DB", DatasourceType::Postgres, "conn", user.id)
            .await
            .unwrap();

        // Create 25 queries
        for i in 0..25 {
            db.create_query(
                org.id,
                datasource.id,
                &format!("Query {}", i),
                None,
                "SELECT 1",
                &json!([]),
                &json!([]),
                30,
                1000,
                user.id,
            )
            .await
            .unwrap();
        }

        // Test pagination - first page (limit 10)
        let (page1, total1) = db
            .list_queries_paginated(org.id, 10, 0)
            .await
            .unwrap();

        assert_eq!(page1.len(), 10);
        assert_eq!(total1, 25);

        // Second page
        let (page2, total2) = db
            .list_queries_paginated(org.id, 10, 10)
            .await
            .unwrap();

        assert_eq!(page2.len(), 10);
        assert_eq!(total2, 25);

        // Third page (partial)
        let (page3, total3) = db
            .list_queries_paginated(org.id, 10, 20)
            .await
            .unwrap();

        assert_eq!(page3.len(), 5);
        assert_eq!(total3, 25);

        // Page beyond data
        let (page4, total4) = db
            .list_queries_paginated(org.id, 10, 30)
            .await
            .unwrap();

        assert_eq!(page4.len(), 0);
        assert_eq!(total4, 25);

        // Verify no duplicates between pages
        let page1_names: Vec<_> = page1.iter().map(|q| &q.name).collect();
        let page2_names: Vec<_> = page2.iter().map(|q| &q.name).collect();
        let page3_names: Vec<_> = page3.iter().map(|q| &q.name).collect();

        for name in &page2_names {
            assert!(!page1_names.contains(name));
        }
        for name in &page3_names {
            assert!(!page1_names.contains(name));
            assert!(!page2_names.contains(name));
        }
    }
}
