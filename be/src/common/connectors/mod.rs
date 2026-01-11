mod postgres;

pub use postgres::PostgresConnector;

use crate::error::Result;
use crate::models::ColumnDef;
use crate::params::TypedValue;
use async_trait::async_trait;
use std::time::Duration;

/// Result of executing a query
#[derive(Debug)]
pub struct QueryOutput {
    pub columns: Vec<ColumnDef>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
    pub execution_time: Duration,
}

/// Connector trait for database connections
#[async_trait]
pub trait Connector: Send + Sync {
    /// Test if the connection is valid
    async fn test_connection(&self) -> Result<Duration>;

    /// Execute a query and return results (no parameters)
    async fn execute(
        &self,
        sql: &str,
        timeout: Duration,
        max_rows: usize,
    ) -> Result<QueryOutput>;

    /// Execute a query with bound parameters
    async fn execute_with_params(
        &self,
        sql: &str,
        params: &[TypedValue],
        timeout: Duration,
        max_rows: usize,
    ) -> Result<QueryOutput>;

    /// Get schema information (tables, columns)
    async fn get_schema(&self) -> Result<Vec<TableSchema>>;
}

#[derive(Debug, serde::Serialize)]
pub struct TableSchema {
    pub schema: String,
    pub name: String,
    pub columns: Vec<ColumnSchema>,
}

#[derive(Debug, serde::Serialize)]
pub struct ColumnSchema {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
}
