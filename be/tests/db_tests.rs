//! Database layer integration tests
//! 
//! These tests use testcontainers to spin up a real PostgreSQL instance.

use loupe::db::Database;
use loupe::models::*;
use testcontainers::{runners::AsyncRunner, ContainerAsync};
use testcontainers_modules::postgres::Postgres;
use uuid::Uuid;

/// Test helper that provides a database connected to a fresh Postgres container
struct TestDb {
    db: Database,
    #[allow(dead_code)]
    container: ContainerAsync<Postgres>,
}

impl TestDb {
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

        db.run_migrations()
            .await
            .expect("Failed to run migrations");

        Self { db, container }
    }

    fn database(&self) -> &Database {
        &self.db
    }
}

mod organization_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_organization() {
        let test_db = TestDb::new().await;
        let db = test_db.database();

        let org = db.create_organization("Test Org").await.unwrap();

        assert_eq!(org.name, "Test Org");
        assert!(!org.id.is_nil());
    }

    #[tokio::test]
    async fn test_get_organization() {
        let test_db = TestDb::new().await;
        let db = test_db.database();

        let created = db.create_organization("My Company").await.unwrap();
        let fetched = db.get_organization(created.id).await.unwrap();

        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.name, "My Company");
    }

    #[tokio::test]
    async fn test_get_nonexistent_organization() {
        let test_db = TestDb::new().await;
        let db = test_db.database();

        let result = db.get_organization(Uuid::new_v4()).await;
        assert!(result.is_err());
    }
}

mod user_tests {
    use super::*;

    async fn setup() -> (TestDb, Organization) {
        let test_db = TestDb::new().await;
        let org = test_db.database().create_organization("Test Org").await.unwrap();
        (test_db, org)
    }

    #[tokio::test]
    async fn test_create_user() {
        let (test_db, org) = setup().await;
        let db = test_db.database();

        let user = db
            .create_user(
                org.id,
                "test@example.com",
                "hashed_password",
                "Test User",
                OrgRole::Admin,
            )
            .await
            .unwrap();

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.name, "Test User");
        assert_eq!(user.role, OrgRole::Admin);
        assert_eq!(user.org_id, org.id);
    }

    #[tokio::test]
    async fn test_get_user_by_email() {
        let (test_db, org) = setup().await;
        let db = test_db.database();

        let created = db
            .create_user(org.id, "lookup@example.com", "hash", "Lookup User", OrgRole::Viewer)
            .await
            .unwrap();

        let found = db.get_user_by_email("lookup@example.com").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, created.id);

        let not_found = db.get_user_by_email("nonexistent@example.com").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_get_user_by_id() {
        let (test_db, org) = setup().await;
        let db = test_db.database();

        let created = db
            .create_user(org.id, "byid@example.com", "hash", "ById User", OrgRole::Editor)
            .await
            .unwrap();

        let fetched = db.get_user(created.id).await.unwrap();
        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.email, "byid@example.com");
    }

    #[tokio::test]
    async fn test_duplicate_email_fails() {
        let (test_db, org) = setup().await;
        let db = test_db.database();

        db.create_user(org.id, "duplicate@example.com", "hash", "User 1", OrgRole::Viewer)
            .await
            .unwrap();

        let result = db
            .create_user(org.id, "duplicate@example.com", "hash", "User 2", OrgRole::Viewer)
            .await;

        assert!(result.is_err());
    }
}

mod datasource_tests {
    use super::*;

    async fn setup() -> (TestDb, Organization, User) {
        let test_db = TestDb::new().await;
        let db = test_db.database();
        let org = db.create_organization("Test Org").await.unwrap();
        let user = db
            .create_user(org.id, "admin@example.com", "hash", "Admin", OrgRole::Admin)
            .await
            .unwrap();
        (test_db, org, user)
    }

    #[tokio::test]
    async fn test_create_datasource() {
        let (test_db, org, user) = setup().await;
        let db = test_db.database();

        let ds = db
            .create_datasource(
                org.id,
                "Production DB",
                DatasourceType::Postgres,
                "encrypted_conn_string",
                user.id,
            )
            .await
            .unwrap();

        assert_eq!(ds.name, "Production DB");
        assert_eq!(ds.ds_type, DatasourceType::Postgres);
        assert_eq!(ds.org_id, org.id);
        assert_eq!(ds.created_by, user.id);
    }

    #[tokio::test]
    async fn test_list_datasources() {
        let (test_db, org, user) = setup().await;
        let db = test_db.database();

        db.create_datasource(org.id, "DS 1", DatasourceType::Postgres, "conn1", user.id)
            .await
            .unwrap();
        db.create_datasource(org.id, "DS 2", DatasourceType::Postgres, "conn2", user.id)
            .await
            .unwrap();

        let list = db.list_datasources(org.id).await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_update_datasource() {
        let (test_db, org, user) = setup().await;
        let db = test_db.database();

        let ds = db
            .create_datasource(org.id, "Original", DatasourceType::Postgres, "conn", user.id)
            .await
            .unwrap();

        let updated = db
            .update_datasource(ds.id, org.id, Some("Renamed"), None)
            .await
            .unwrap();

        assert_eq!(updated.name, "Renamed");
        assert_eq!(updated.connection_string_encrypted, "conn"); // unchanged
    }

    #[tokio::test]
    async fn test_delete_datasource() {
        let (test_db, org, user) = setup().await;
        let db = test_db.database();

        let ds = db
            .create_datasource(org.id, "ToDelete", DatasourceType::Postgres, "conn", user.id)
            .await
            .unwrap();

        db.delete_datasource(ds.id, org.id).await.unwrap();

        let list = db.list_datasources(org.id).await.unwrap();
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn test_datasource_org_isolation() {
        let (test_db, org1, user) = setup().await;
        let db = test_db.database();
        let org2 = db.create_organization("Other Org").await.unwrap();

        let ds = db
            .create_datasource(org1.id, "Org1 DS", DatasourceType::Postgres, "conn", user.id)
            .await
            .unwrap();

        // Can access with correct org
        let fetched = db.get_datasource(ds.id, org1.id).await;
        assert!(fetched.is_ok());

        // Cannot access with wrong org
        let wrong_org = db.get_datasource(ds.id, org2.id).await;
        assert!(wrong_org.is_err());
    }
}

mod query_tests {
    use super::*;

    async fn setup() -> (TestDb, Organization, User, Datasource) {
        let test_db = TestDb::new().await;
        let db = test_db.database();
        let org = db.create_organization("Test Org").await.unwrap();
        let user = db
            .create_user(org.id, "admin@example.com", "hash", "Admin", OrgRole::Admin)
            .await
            .unwrap();
        let ds = db
            .create_datasource(org.id, "Test DS", DatasourceType::Postgres, "conn", user.id)
            .await
            .unwrap();
        (test_db, org, user, ds)
    }

    #[tokio::test]
    async fn test_create_query() {
        let (test_db, org, user, ds) = setup().await;
        let db = test_db.database();

        let query = db
            .create_query(
                org.id,
                ds.id,
                "Active Users",
                Some("Count of active users"),
                "SELECT COUNT(*) FROM users WHERE active = true",
                &serde_json::json!([]),
                30,
                10000,
                user.id,
            )
            .await
            .unwrap();

        assert_eq!(query.name, "Active Users");
        assert_eq!(query.description.as_deref(), Some("Count of active users"));
        assert_eq!(query.timeout_seconds, 30);
        assert_eq!(query.max_rows, 10000);
    }

    #[tokio::test]
    async fn test_query_with_parameters() {
        let (test_db, org, user, ds) = setup().await;
        let db = test_db.database();

        let params = serde_json::json!([
            {"name": "start_date", "param_type": "date", "required": true},
            {"name": "limit", "param_type": "number", "default": 100}
        ]);

        let query = db
            .create_query(
                org.id, ds.id, "Parameterized Query", None,
                "SELECT * FROM events WHERE date > $1 LIMIT $2",
                &params, 60, 5000, user.id,
            )
            .await
            .unwrap();

        assert_eq!(query.parameters, params);
    }

    #[tokio::test]
    async fn test_update_query() {
        let (test_db, org, user, ds) = setup().await;
        let db = test_db.database();

        let query = db
            .create_query(
                org.id, ds.id, "Original Name", None,
                "SELECT 1", &serde_json::json!([]), 30, 10000, user.id,
            )
            .await
            .unwrap();

        let updated = db
            .update_query(
                query.id, org.id,
                Some("New Name"),
                Some("Added description"),
                Some("SELECT 2"),
                None, Some(60), None,
            )
            .await
            .unwrap();

        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.description.as_deref(), Some("Added description"));
        assert_eq!(updated.sql, "SELECT 2");
        assert_eq!(updated.timeout_seconds, 60);
        assert_eq!(updated.max_rows, 10000); // unchanged
    }

    #[tokio::test]
    async fn test_delete_query() {
        let (test_db, org, user, ds) = setup().await;
        let db = test_db.database();

        let query = db
            .create_query(
                org.id, ds.id, "ToDelete", None,
                "SELECT 1", &serde_json::json!([]), 30, 10000, user.id,
            )
            .await
            .unwrap();

        db.delete_query(query.id, org.id).await.unwrap();

        let list = db.list_queries(org.id).await.unwrap();
        assert!(list.is_empty());
    }
}

mod run_tests {
    use super::*;

    async fn setup() -> (TestDb, Organization, User, Datasource, Query) {
        let test_db = TestDb::new().await;
        let db = test_db.database();
        let org = db.create_organization("Test Org").await.unwrap();
        let user = db
            .create_user(org.id, "admin@example.com", "hash", "Admin", OrgRole::Admin)
            .await
            .unwrap();
        let ds = db
            .create_datasource(org.id, "Test DS", DatasourceType::Postgres, "conn", user.id)
            .await
            .unwrap();
        let query = db
            .create_query(
                org.id, ds.id, "Test Query", None,
                "SELECT 1", &serde_json::json!([]), 30, 10000, user.id,
            )
            .await
            .unwrap();
        (test_db, org, user, ds, query)
    }

    #[tokio::test]
    async fn test_create_run() {
        let (test_db, org, user, ds, query) = setup().await;
        let db = test_db.database();

        let run = db
            .create_run(
                org.id,
                query.id,
                ds.id,
                "SELECT 1",
                &serde_json::json!({}),
                30,
                10000,
                user.id,
            )
            .await
            .unwrap();

        assert_eq!(run.status, RunStatus::Queued);
        assert_eq!(run.query_id, query.id);
        assert_eq!(run.created_by, user.id);
    }

    #[tokio::test]
    async fn test_claim_run() {
        let (test_db, org, user, ds, query) = setup().await;
        let db = test_db.database();

        let run = db
            .create_run(
                org.id, query.id, ds.id,
                "SELECT 1", &serde_json::json!({}), 30, 10000, user.id,
            )
            .await
            .unwrap();

        let runner_id = "runner-1";
        let claimed = db.claim_run(runner_id).await.unwrap();

        assert!(claimed.is_some());
        let claimed = claimed.unwrap();
        assert_eq!(claimed.id, run.id);
        assert_eq!(claimed.status, RunStatus::Running);
        assert_eq!(claimed.runner_id.as_deref(), Some(runner_id));
    }

    #[tokio::test]
    async fn test_claim_empty_queue() {
        let test_db = TestDb::new().await;
        let db = test_db.database();

        let claimed = db.claim_run("runner-1").await.unwrap();
        assert!(claimed.is_none());
    }

    #[tokio::test]
    async fn test_complete_run() {
        let (test_db, org, user, ds, query) = setup().await;
        let db = test_db.database();

        let run = db
            .create_run(
                org.id, query.id, ds.id,
                "SELECT 1", &serde_json::json!({}), 30, 10000, user.id,
            )
            .await
            .unwrap();

        // Claim it first
        db.claim_run("runner-1").await.unwrap();

        // Store a result
        let result = db.create_run_result(
            run.id,
            &serde_json::json!([{"name": "col", "data_type": "INT4"}]),
            &serde_json::json!([[1]]),
            1, 4, 5
        ).await.unwrap();

        let completed = db
            .complete_run(run.id, result.id)
            .await
            .unwrap();

        assert_eq!(completed.status, RunStatus::Completed);
        assert!(completed.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_fail_run() {
        let (test_db, org, user, ds, query) = setup().await;
        let db = test_db.database();

        let run = db
            .create_run(
                org.id, query.id, ds.id,
                "SELECT 1", &serde_json::json!({}), 30, 10000, user.id,
            )
            .await
            .unwrap();

        // Claim it first
        db.claim_run("runner-1").await.unwrap();

        let failed = db
            .fail_run(run.id, "Connection refused")
            .await
            .unwrap();

        assert_eq!(failed.status, RunStatus::Failed);
        assert_eq!(failed.error_message.as_deref(), Some("Connection refused"));
    }
}

mod run_result_tests {
    use super::*;

    async fn setup_with_run() -> (TestDb, Organization, Run) {
        let test_db = TestDb::new().await;
        let db = test_db.database();
        let org = db.create_organization("Test Org").await.unwrap();
        let user = db
            .create_user(org.id, "admin@example.com", "hash", "Admin", OrgRole::Admin)
            .await
            .unwrap();
        let ds = db
            .create_datasource(org.id, "Test DS", DatasourceType::Postgres, "conn", user.id)
            .await
            .unwrap();
        let query = db
            .create_query(
                org.id, ds.id, "Test Query", None,
                "SELECT 1", &serde_json::json!([]), 30, 10000, user.id,
            )
            .await
            .unwrap();
        let run = db
            .create_run(
                org.id, query.id, ds.id,
                "SELECT 1 as num", &serde_json::json!({}), 30, 10000, user.id,
            )
            .await
            .unwrap();
        (test_db, org, run)
    }

    #[tokio::test]
    async fn test_store_run_result() {
        let (test_db, _org, run) = setup_with_run().await;
        let db = test_db.database();

        let columns = serde_json::json!([{"name": "num", "data_type": "INT4"}]);
        let rows = serde_json::json!([[1]]);

        let result = db
            .create_run_result(run.id, &columns, &rows, 1, 8, 5)
            .await
            .unwrap();

        assert_eq!(result.run_id, run.id);
        assert_eq!(result.row_count, 1);
        assert_eq!(result.byte_count, 8);
        assert_eq!(result.execution_time_ms, 5);
    }

    #[tokio::test]
    async fn test_get_run_result() {
        let (test_db, org, run) = setup_with_run().await;
        let db = test_db.database();

        let columns = serde_json::json!([{"name": "id", "data_type": "INT8"}]);
        let rows = serde_json::json!([[1], [2], [3]]);

        db.create_run_result(run.id, &columns, &rows, 3, 24, 10)
            .await
            .unwrap();

        let fetched = db.get_run_result(run.id).await.unwrap();
        assert_eq!(fetched.row_count, 3);
        assert_eq!(fetched.columns, columns);
    }
}

mod visualization_tests {
    use super::*;

    async fn setup() -> (TestDb, Organization, User, Query) {
        let test_db = TestDb::new().await;
        let db = test_db.database();
        let org = db.create_organization("Test Org").await.unwrap();
        let user = db
            .create_user(org.id, "admin@example.com", "hash", "Admin", OrgRole::Admin)
            .await
            .unwrap();
        let ds = db
            .create_datasource(org.id, "Test DS", DatasourceType::Postgres, "conn", user.id)
            .await
            .unwrap();
        let query = db
            .create_query(
                org.id, ds.id, "Test Query", None,
                "SELECT date, count FROM metrics", &serde_json::json!([]), 30, 10000, user.id,
            )
            .await
            .unwrap();
        (test_db, org, user, query)
    }

    #[tokio::test]
    async fn test_create_visualization() {
        let (test_db, org, user, query) = setup().await;
        let db = test_db.database();

        let config = serde_json::json!({"x": "date", "y": "count"});
        let viz = db
            .create_visualization(
                org.id,
                query.id,
                "Daily Metrics",
                ChartType::Line,
                &config,
                user.id,
            )
            .await
            .unwrap();

        assert_eq!(viz.name, "Daily Metrics");
        assert_eq!(viz.chart_type, ChartType::Line);
        assert_eq!(viz.config, config);
    }

    #[tokio::test]
    async fn test_list_visualizations_for_query() {
        let (test_db, org, user, query) = setup().await;
        let db = test_db.database();

        db.create_visualization(org.id, query.id, "Table View", ChartType::Table, &serde_json::json!({}), user.id)
            .await.unwrap();
        db.create_visualization(org.id, query.id, "Line Chart", ChartType::Line, &serde_json::json!({}), user.id)
            .await.unwrap();

        let list = db.list_visualizations(org.id, Some(query.id)).await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_get_visualization() {
        let (test_db, org, user, query) = setup().await;
        let db = test_db.database();

        let viz = db
            .create_visualization(org.id, query.id, "My Viz", ChartType::Bar, &serde_json::json!({}), user.id)
            .await.unwrap();

        let fetched = db.get_visualization(viz.id, org.id).await.unwrap();
        assert_eq!(fetched.name, "My Viz");
        assert_eq!(fetched.chart_type, ChartType::Bar);
    }

    #[tokio::test]
    async fn test_delete_visualization() {
        let (test_db, org, user, query) = setup().await;
        let db = test_db.database();

        let viz = db
            .create_visualization(org.id, query.id, "To Delete", ChartType::Table, &serde_json::json!({}), user.id)
            .await.unwrap();

        db.delete_visualization(viz.id, org.id).await.unwrap();

        let list = db.list_visualizations(org.id, Some(query.id)).await.unwrap();
        assert!(list.is_empty());
    }
}

mod dashboard_tests {
    use super::*;

    async fn setup() -> (TestDb, Organization, User, Visualization) {
        let test_db = TestDb::new().await;
        let db = test_db.database();
        let org = db.create_organization("Test Org").await.unwrap();
        let user = db
            .create_user(org.id, "admin@example.com", "hash", "Admin", OrgRole::Admin)
            .await.unwrap();
        let ds = db
            .create_datasource(org.id, "Test DS", DatasourceType::Postgres, "conn", user.id)
            .await.unwrap();
        let query = db
            .create_query(
                org.id, ds.id, "Test Query", None,
                "SELECT 1", &serde_json::json!([]), 30, 10000, user.id,
            )
            .await.unwrap();
        let viz = db
            .create_visualization(org.id, query.id, "Test Viz", ChartType::Table, &serde_json::json!({}), user.id)
            .await.unwrap();
        (test_db, org, user, viz)
    }

    #[tokio::test]
    async fn test_create_dashboard() {
        let (test_db, org, user, _viz) = setup().await;
        let db = test_db.database();

        let dash = db
            .create_dashboard(org.id, "Executive Dashboard", Some("Weekly metrics"), &serde_json::json!({}), user.id)
            .await.unwrap();

        assert_eq!(dash.name, "Executive Dashboard");
        assert_eq!(dash.description.as_deref(), Some("Weekly metrics"));
    }

    #[tokio::test]
    async fn test_add_tile_to_dashboard() {
        let (test_db, org, user, viz) = setup().await;
        let db = test_db.database();

        let dash = db
            .create_dashboard(org.id, "Test Dashboard", None, &serde_json::json!({}), user.id)
            .await.unwrap();

        let tile = db
            .create_tile(
                dash.id,
                viz.id,
                Some("My Widget"),
                0, 0, 6, 4,
                &serde_json::json!({}),
            )
            .await.unwrap();

        assert_eq!(tile.dashboard_id, dash.id);
        assert_eq!(tile.visualization_id, viz.id);
        assert_eq!(tile.title.as_deref(), Some("My Widget"));
        assert_eq!(tile.width, 6);
        assert_eq!(tile.height, 4);
    }

    #[tokio::test]
    async fn test_list_dashboard_tiles() {
        let (test_db, org, user, viz) = setup().await;
        let db = test_db.database();

        let dash = db.create_dashboard(org.id, "Multi-tile Dashboard", None, &serde_json::json!({}), user.id).await.unwrap();

        db.create_tile(dash.id, viz.id, Some("Tile 1"), 0, 0, 6, 4, &serde_json::json!({})).await.unwrap();
        db.create_tile(dash.id, viz.id, Some("Tile 2"), 6, 0, 6, 4, &serde_json::json!({})).await.unwrap();

        let tiles = db.list_tiles(dash.id).await.unwrap();
        assert_eq!(tiles.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_tile() {
        let (test_db, org, user, viz) = setup().await;
        let db = test_db.database();

        let dash = db.create_dashboard(org.id, "Dashboard", None, &serde_json::json!({}), user.id).await.unwrap();
        let tile = db.create_tile(dash.id, viz.id, None, 0, 0, 4, 4, &serde_json::json!({})).await.unwrap();

        db.delete_tile(tile.id, dash.id).await.unwrap();

        let tiles = db.list_tiles(dash.id).await.unwrap();
        assert!(tiles.is_empty());
    }

    #[tokio::test]
    async fn test_delete_dashboard() {
        let (test_db, org, user, _viz) = setup().await;
        let db = test_db.database();

        let dash = db.create_dashboard(org.id, "To Delete", None, &serde_json::json!({}), user.id).await.unwrap();

        db.delete_dashboard(dash.id, org.id).await.unwrap();

        let dashboards = db.list_dashboards(org.id).await.unwrap();
        assert!(dashboards.is_empty());
    }
}

mod schedule_tests {
    use super::*;
    use chrono::{Duration, Utc};

    async fn setup() -> (TestDb, Organization, User, Query) {
        let test_db = TestDb::new().await;
        let db = test_db.database();
        let org = db.create_organization("Test Org").await.unwrap();
        let user = db
            .create_user(org.id, "admin@example.com", "hash", "Admin", OrgRole::Admin)
            .await.unwrap();
        let ds = db
            .create_datasource(org.id, "Test DS", DatasourceType::Postgres, "conn", user.id)
            .await.unwrap();
        let query = db
            .create_query(
                org.id, ds.id, "Scheduled Query", None,
                "SELECT NOW()", &serde_json::json!([]), 30, 10000, user.id,
            )
            .await.unwrap();
        (test_db, org, user, query)
    }

    #[tokio::test]
    async fn test_create_schedule() {
        let (test_db, org, user, query) = setup().await;
        let db = test_db.database();

        let schedule = db
            .create_schedule(
                org.id,
                query.id,
                "Every Hour",
                "0 * * * *",
                &serde_json::json!({}),
                true,
                user.id,
            )
            .await.unwrap();

        assert_eq!(schedule.name, "Every Hour");
        assert_eq!(schedule.cron_expression, "0 * * * *");
        assert!(schedule.enabled);
    }

    #[tokio::test]
    async fn test_list_schedules() {
        let (test_db, org, user, query) = setup().await;
        let db = test_db.database();

        db.create_schedule(org.id, query.id, "Hourly", "0 * * * *", &serde_json::json!({}), true, user.id).await.unwrap();
        db.create_schedule(org.id, query.id, "Daily", "0 0 * * *", &serde_json::json!({}), false, user.id).await.unwrap();

        let schedules = db.list_schedules(org.id).await.unwrap();
        assert_eq!(schedules.len(), 2);
    }

    #[tokio::test]
    async fn test_update_schedule_last_run() {
        let (test_db, org, user, query) = setup().await;
        let db = test_db.database();

        let schedule = db
            .create_schedule(org.id, query.id, "To Track", "0 * * * *", &serde_json::json!({}), true, user.id)
            .await.unwrap();

        let next_run = Utc::now() + Duration::hours(1);
        db.update_schedule_last_run(schedule.id, next_run).await.unwrap();

        // Verify by re-listing (since we don't have get_schedule)
        let schedules = db.list_schedules(org.id).await.unwrap();
        let updated = schedules.iter().find(|s| s.id == schedule.id).unwrap();
        assert!(updated.last_run_at.is_some());
    }

    #[tokio::test]
    async fn test_get_due_schedules() {
        let (test_db, org, user, query) = setup().await;
        let db = test_db.database();

        // Create an enabled schedule with null next_run_at (should be due)
        db.create_schedule(org.id, query.id, "Should Be Due", "0 * * * *", &serde_json::json!({}), true, user.id).await.unwrap();
        
        // Create a disabled schedule (should not be due)
        db.create_schedule(org.id, query.id, "Disabled", "0 * * * *", &serde_json::json!({}), false, user.id).await.unwrap();

        let due = db.get_due_schedules().await.unwrap();
        // At least one should be due (the enabled one with null next_run_at)
        assert!(!due.is_empty());
        assert!(due.iter().all(|s| s.enabled));
    }
}
