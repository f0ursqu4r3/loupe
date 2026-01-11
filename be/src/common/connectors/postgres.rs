use super::{ColumnSchema, Connector, QueryOutput, TableSchema};
use crate::error::{Error, Result};
use crate::models::ColumnDef;
use crate::params::TypedValue;
use async_trait::async_trait;
use sqlx::postgres::{PgArguments, PgPoolOptions};
use sqlx::{Arguments, Column, PgPool, Row, TypeInfo};
use std::time::{Duration, Instant};

pub struct PostgresConnector {
    pool: PgPool,
}

impl PostgresConnector {
    pub async fn new(connection_string: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(10))
            .connect(connection_string)
            .await
            .map_err(|e| Error::Connection(format!("Failed to connect to Postgres: {}", e)))?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl Connector for PostgresConnector {
    async fn test_connection(&self) -> Result<Duration> {
        let start = Instant::now();
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Connection(format!("Connection test failed: {}", e)))?;
        Ok(start.elapsed())
    }

    async fn execute(
        &self,
        sql: &str,
        timeout: Duration,
        max_rows: usize,
    ) -> Result<QueryOutput> {
        let start = Instant::now();

        // Wrap query with timeout and limit
        let limited_sql = format!(
            "SELECT * FROM ({}) AS _q LIMIT {}",
            sql.trim().trim_end_matches(';'),
            max_rows
        );

        let rows = tokio::time::timeout(timeout, sqlx::query(&limited_sql).fetch_all(&self.pool))
            .await
            .map_err(|_| Error::Timeout(format!("Query timed out after {:?}", timeout)))?
            .map_err(|e| Error::QueryExecution(e.to_string()))?;

        let execution_time = start.elapsed();

        if rows.is_empty() {
            return Ok(QueryOutput {
                columns: vec![],
                rows: vec![],
                row_count: 0,
                execution_time,
            });
        }

        // Extract column information
        let columns: Vec<ColumnDef> = rows[0]
            .columns()
            .iter()
            .map(|c| ColumnDef {
                name: c.name().to_string(),
                data_type: c.type_info().name().to_string(),
            })
            .collect();

        // Extract row data
        let result_rows: Vec<Vec<serde_json::Value>> = rows
            .iter()
            .map(|row| {
                columns
                    .iter()
                    .enumerate()
                    .map(|(i, col)| pg_value_to_json(row, i, &col.data_type))
                    .collect()
            })
            .collect();

        let row_count = result_rows.len();

        Ok(QueryOutput {
            columns,
            rows: result_rows,
            row_count,
            execution_time,
        })
    }

    async fn execute_with_params(
        &self,
        sql: &str,
        params: &[TypedValue],
        timeout: Duration,
        max_rows: usize,
    ) -> Result<QueryOutput> {
        let start = Instant::now();

        // Wrap query with limit
        let limited_sql = format!(
            "SELECT * FROM ({}) AS _q LIMIT {}",
            sql.trim().trim_end_matches(';'),
            max_rows
        );

        // Build arguments
        let mut args = PgArguments::default();
        for param in params {
            match param {
                TypedValue::String(s) => args.add(s.as_str()).map_err(|e| Error::BadRequest(e.to_string()))?,
                TypedValue::Number(n) => args.add(*n).map_err(|e| Error::BadRequest(e.to_string()))?,
                TypedValue::Integer(i) => args.add(*i).map_err(|e| Error::BadRequest(e.to_string()))?,
                TypedValue::Boolean(b) => args.add(*b).map_err(|e| Error::BadRequest(e.to_string()))?,
                TypedValue::Date(d) => args.add(*d).map_err(|e| Error::BadRequest(e.to_string()))?,
                TypedValue::DateTime(dt) => args.add(*dt).map_err(|e| Error::BadRequest(e.to_string()))?,
                TypedValue::Null => {
                    // For null, we need to bind as Option<String>
                    let null_val: Option<String> = None;
                    args.add(null_val).map_err(|e| Error::BadRequest(e.to_string()))?;
                }
            }
        }

        let rows = tokio::time::timeout(
            timeout,
            sqlx::query_with(&limited_sql, args).fetch_all(&self.pool),
        )
        .await
        .map_err(|_| Error::Timeout(format!("Query timed out after {:?}", timeout)))?
        .map_err(|e| Error::QueryExecution(e.to_string()))?;

        let execution_time = start.elapsed();

        if rows.is_empty() {
            return Ok(QueryOutput {
                columns: vec![],
                rows: vec![],
                row_count: 0,
                execution_time,
            });
        }

        // Extract column information
        let columns: Vec<ColumnDef> = rows[0]
            .columns()
            .iter()
            .map(|c| ColumnDef {
                name: c.name().to_string(),
                data_type: c.type_info().name().to_string(),
            })
            .collect();

        // Extract row data
        let result_rows: Vec<Vec<serde_json::Value>> = rows
            .iter()
            .map(|row| {
                columns
                    .iter()
                    .enumerate()
                    .map(|(i, col)| pg_value_to_json(row, i, &col.data_type))
                    .collect()
            })
            .collect();

        let row_count = result_rows.len();

        Ok(QueryOutput {
            columns,
            rows: result_rows,
            row_count,
            execution_time,
        })
    }

    async fn get_schema(&self) -> Result<Vec<TableSchema>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                table_schema,
                table_name,
                column_name,
                data_type,
                is_nullable
            FROM information_schema.columns
            WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
            ORDER BY table_schema, table_name, ordinal_position
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::QueryExecution(format!("Failed to get schema: {}", e)))?;

        let mut tables: Vec<TableSchema> = Vec::new();
        let mut current_table: Option<(String, String)> = None;

        for row in rows {
            let schema: String = row.get("table_schema");
            let table: String = row.get("table_name");
            let column_name: String = row.get("column_name");
            let data_type: String = row.get("data_type");
            let is_nullable: String = row.get("is_nullable");

            let col = ColumnSchema {
                name: column_name,
                data_type,
                is_nullable: is_nullable == "YES",
            };

            match &current_table {
                Some((s, t)) if s == &schema && t == &table => {
                    if let Some(last) = tables.last_mut() {
                        last.columns.push(col);
                    }
                }
                _ => {
                    tables.push(TableSchema {
                        schema: schema.clone(),
                        name: table.clone(),
                        columns: vec![col],
                    });
                    current_table = Some((schema, table));
                }
            }
        }

        Ok(tables)
    }
}

fn pg_value_to_json(row: &sqlx::postgres::PgRow, idx: usize, type_name: &str) -> serde_json::Value {
    // Try common types
    match type_name.to_uppercase().as_str() {
        "INT2" | "INT4" | "INT8" | "SMALLINT" | "INTEGER" | "BIGINT" => {
            row.try_get::<i64, _>(idx)
                .map(serde_json::Value::from)
                .unwrap_or(serde_json::Value::Null)
        }
        "FLOAT4" | "FLOAT8" | "REAL" | "DOUBLE PRECISION" | "NUMERIC" | "DECIMAL" => {
            row.try_get::<f64, _>(idx)
                .map(|v| serde_json::json!(v))
                .unwrap_or(serde_json::Value::Null)
        }
        "BOOL" | "BOOLEAN" => row
            .try_get::<bool, _>(idx)
            .map(serde_json::Value::from)
            .unwrap_or(serde_json::Value::Null),
        "TEXT" | "VARCHAR" | "CHAR" | "NAME" | "BPCHAR" => row
            .try_get::<String, _>(idx)
            .map(serde_json::Value::from)
            .unwrap_or(serde_json::Value::Null),
        "UUID" => row
            .try_get::<uuid::Uuid, _>(idx)
            .map(|v| serde_json::Value::from(v.to_string()))
            .unwrap_or(serde_json::Value::Null),
        "TIMESTAMP" | "TIMESTAMPTZ" => row
            .try_get::<chrono::DateTime<chrono::Utc>, _>(idx)
            .map(|v| serde_json::Value::from(v.to_rfc3339()))
            .unwrap_or(serde_json::Value::Null),
        "DATE" => row
            .try_get::<chrono::NaiveDate, _>(idx)
            .map(|v| serde_json::Value::from(v.to_string()))
            .unwrap_or(serde_json::Value::Null),
        "JSON" | "JSONB" => row
            .try_get::<serde_json::Value, _>(idx)
            .unwrap_or(serde_json::Value::Null),
        _ => {
            // Fallback: try as string
            row.try_get::<String, _>(idx)
                .map(serde_json::Value::from)
                .unwrap_or(serde_json::Value::Null)
        }
    }
}
