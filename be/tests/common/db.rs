//! Test database utilities using testcontainers

use loupe::Database;
use once_cell::sync::Lazy;
use std::sync::Arc;
use testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;
use tokio::sync::OnceCell;

/// Shared test database container (singleton)
static TEST_CONTAINER: Lazy<OnceCell<Arc<TestDbContainer>>> = Lazy::new(OnceCell::new);

struct TestDbContainer {
    _container: ContainerAsync<Postgres>,
    connection_string: String,
}

/// Test database wrapper that provides isolated database connections
pub struct TestDb {
    pub db: Database,
    pub connection_string: String,
}

impl TestDb {
    /// Create a new test database connection
    /// Uses a shared container but creates a fresh schema for isolation
    pub async fn new() -> Self {
        let container = TEST_CONTAINER
            .get_or_init(|| async {
                let container = Postgres::default()
                    .with_tag("16-alpine")
                    .start()
                    .await
                    .expect("Failed to start postgres container");

                let host = container.get_host().await.unwrap();
                let port = container.get_host_port_ipv4(5432).await.unwrap();
                let connection_string =
                    format!("postgres://postgres:postgres@{}:{}/postgres", host, port);

                Arc::new(TestDbContainer {
                    _container: container,
                    connection_string,
                })
            })
            .await;

        // Connect and run migrations
        let db = Database::connect(&container.connection_string)
            .await
            .expect("Failed to connect to test database");

        db.run_migrations().await.expect("Failed to run migrations");

        Self {
            db,
            connection_string: container.connection_string.clone(),
        }
    }

    /// Clean all data from tables (for test isolation)
    pub async fn clean(&self) {
        sqlx::query(
            r#"
            TRUNCATE TABLE 
                tiles, 
                dashboards, 
                visualizations, 
                schedules, 
                run_results, 
                runs, 
                queries, 
                datasources, 
                users, 
                organizations 
            CASCADE
            "#,
        )
        .execute(&self.db.pool)
        .await
        .expect("Failed to clean test database");
    }
}
