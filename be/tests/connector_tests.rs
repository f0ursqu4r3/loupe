//! Connector integration tests
//! 
//! These tests verify the PostgresConnector against a real Postgres instance
//! using testcontainers.

use loupe::connectors::{Connector, PostgresConnector};
use std::time::Duration;
use testcontainers::{runners::AsyncRunner, ContainerAsync};
use testcontainers_modules::postgres::Postgres;

/// Test helper that provides a Postgres container and connector
struct TestConnector {
    connector: PostgresConnector,
    connection_string: String,
    #[allow(dead_code)]
    container: ContainerAsync<Postgres>,
}

impl TestConnector {
    async fn new() -> Self {
        let container = Postgres::default()
            .start()
            .await
            .expect("Failed to start postgres container");

        let host = container.get_host().await.expect("get host");
        let port = container.get_host_port_ipv4(5432).await.expect("get port");
        let connection_string = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);

        let connector = PostgresConnector::new(&connection_string)
            .await
            .expect("Failed to create connector");

        Self {
            connector,
            connection_string,
            container,
        }
    }

    fn connector(&self) -> &PostgresConnector {
        &self.connector
    }
}

mod test_connection {
    use super::*;

    #[tokio::test]
    async fn test_connection_success() {
        let test = TestConnector::new().await;
        
        let duration = test.connector().test_connection().await.unwrap();
        assert!(duration.as_millis() < 5000); // Should connect quickly
    }

    #[tokio::test]
    async fn test_connection_failure() {
        let result = PostgresConnector::new("postgres://localhost:9999/nonexistent").await;
        assert!(result.is_err());
    }
}

mod execute_tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_select() {
        let test = TestConnector::new().await;
        
        let result = test.connector()
            .execute("SELECT 1 as num, 'hello' as greeting", Duration::from_secs(10), 100)
            .await
            .unwrap();

        assert_eq!(result.row_count, 1);
        assert_eq!(result.columns.len(), 2);
        assert_eq!(result.columns[0].name, "num");
        assert_eq!(result.columns[1].name, "greeting");
        
        let row = &result.rows[0];
        assert_eq!(row[0], serde_json::json!(1));
        assert_eq!(row[1], serde_json::json!("hello"));
    }

    #[tokio::test]
    async fn test_multiple_rows() {
        let test = TestConnector::new().await;
        
        let result = test.connector()
            .execute("SELECT * FROM generate_series(1, 5) as n", Duration::from_secs(10), 100)
            .await
            .unwrap();

        assert_eq!(result.row_count, 5);
        assert_eq!(result.columns.len(), 1);
        assert_eq!(result.columns[0].name, "n");
    }

    #[tokio::test]
    async fn test_max_rows_limit() {
        let test = TestConnector::new().await;
        
        // Generate 100 rows but limit to 10
        let result = test.connector()
            .execute("SELECT * FROM generate_series(1, 100) as n", Duration::from_secs(10), 10)
            .await
            .unwrap();

        assert_eq!(result.row_count, 10);
    }

    #[tokio::test]
    async fn test_empty_result() {
        let test = TestConnector::new().await;
        
        let result = test.connector()
            .execute("SELECT 1 WHERE false", Duration::from_secs(10), 100)
            .await
            .unwrap();

        assert_eq!(result.row_count, 0);
        assert!(result.columns.is_empty());
        assert!(result.rows.is_empty());
    }

    #[tokio::test]
    async fn test_null_values() {
        let test = TestConnector::new().await;
        
        let result = test.connector()
            .execute("SELECT NULL as null_col, 1 as num", Duration::from_secs(10), 100)
            .await
            .unwrap();

        assert_eq!(result.row_count, 1);
        assert_eq!(result.rows[0][0], serde_json::Value::Null);
        assert_eq!(result.rows[0][1], serde_json::json!(1));
    }

    #[tokio::test]
    async fn test_various_data_types() {
        let test = TestConnector::new().await;
        
        let result = test.connector()
            .execute(
                r#"
                SELECT 
                    42::int4 as int_val,
                    3.14::float8 as float_val,
                    true as bool_val,
                    'text'::text as text_val,
                    '2024-01-15'::date as date_val,
                    '{"key": "value"}'::jsonb as json_val
                "#,
                Duration::from_secs(10),
                100,
            )
            .await
            .unwrap();

        assert_eq!(result.row_count, 1);
        assert_eq!(result.columns.len(), 6);
        
        let row = &result.rows[0];
        assert_eq!(row[0], serde_json::json!(42));
        // Float might have precision differences
        assert!(row[1].as_f64().unwrap() > 3.1);
        assert_eq!(row[2], serde_json::json!(true));
        assert_eq!(row[3], serde_json::json!("text"));
    }

    #[tokio::test]
    async fn test_syntax_error() {
        let test = TestConnector::new().await;
        
        let result = test.connector()
            .execute("SELEKT 1", Duration::from_secs(10), 100)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_timeout() {
        let test = TestConnector::new().await;
        
        // pg_sleep for 2 seconds, but timeout after 100ms
        let result = test.connector()
            .execute("SELECT pg_sleep(2)", Duration::from_millis(100), 100)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("timeout") || err.contains("Timeout"));
    }

    #[tokio::test]
    async fn test_execution_time_tracked() {
        let test = TestConnector::new().await;
        
        // Small sleep to ensure measurable execution time
        let result = test.connector()
            .execute("SELECT pg_sleep(0.05), 1", Duration::from_secs(10), 100)
            .await
            .unwrap();

        // Should be at least 50ms
        assert!(result.execution_time.as_millis() >= 50);
    }
}

mod schema_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_schema_empty_database() {
        let test = TestConnector::new().await;
        
        // Default database has no user tables
        let schema = test.connector().get_schema().await.unwrap();
        // Should be empty (no user tables in fresh postgres)
        assert!(schema.iter().all(|t| t.schema != "public" || t.columns.is_empty() || t.name.starts_with("pg_")));
    }

    #[tokio::test]
    async fn test_get_schema_with_tables() {
        let test = TestConnector::new().await;
        
        // Create a test table
        test.connector()
            .execute(
                "CREATE TABLE test_users (id SERIAL PRIMARY KEY, name TEXT NOT NULL, age INT)",
                Duration::from_secs(10),
                1,
            )
            .await
            .unwrap();

        let schema = test.connector().get_schema().await.unwrap();
        
        let test_table = schema.iter().find(|t| t.name == "test_users");
        assert!(test_table.is_some(), "Should find test_users table");
        
        let table = test_table.unwrap();
        assert_eq!(table.schema, "public");
        assert!(table.columns.iter().any(|c| c.name == "id"));
        assert!(table.columns.iter().any(|c| c.name == "name"));
        assert!(table.columns.iter().any(|c| c.name == "age"));

        // Check nullable flags
        let name_col = table.columns.iter().find(|c| c.name == "name").unwrap();
        assert!(!name_col.is_nullable); // NOT NULL constraint
        
        let age_col = table.columns.iter().find(|c| c.name == "age").unwrap();
        assert!(age_col.is_nullable); // No constraint, so nullable
    }
}

mod concurrent_tests {
    use super::*;
    use tokio::task::JoinSet;

    #[tokio::test]
    async fn test_concurrent_queries() {
        let test = TestConnector::new().await;
        
        let mut set = JoinSet::new();
        
        // Run 10 concurrent queries
        for i in 0..10 {
            let conn_str = test.connection_string.clone();
            set.spawn(async move {
                let connector = PostgresConnector::new(&conn_str).await.unwrap();
                let result = connector
                    .execute(&format!("SELECT {} as num", i), Duration::from_secs(10), 100)
                    .await
                    .unwrap();
                (i, result.rows[0][0].as_i64().unwrap())
            });
        }

        let mut results = vec![];
        while let Some(res) = set.join_next().await {
            results.push(res.unwrap());
        }

        assert_eq!(results.len(), 10);
        for (i, val) in results {
            assert_eq!(i as i64, val);
        }
    }
}
