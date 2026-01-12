use crate::error::{Error, Result};
use crate::models::*;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .map_err(|e| Error::Database(format!("Failed to connect: {}", e)))?;

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| Error::Database(format!("Migration failed: {}", e)))?;
        Ok(())
    }

    // ==================== Organizations ====================

    pub async fn create_organization(&self, name: &str) -> Result<Organization> {
        let org = sqlx::query_as::<_, Organization>(
            r#"
            INSERT INTO organizations (id, name, created_at, updated_at)
            VALUES ($1, $2, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        Ok(org)
    }

    pub async fn get_organization(&self, id: Uuid) -> Result<Organization> {
        let org = sqlx::query_as::<_, Organization>("SELECT * FROM organizations WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(org)
    }

    // ==================== Users ====================

    pub async fn create_user(
        &self,
        org_id: Uuid,
        email: &str,
        password_hash: &str,
        name: &str,
        role: OrgRole,
    ) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, org_id, email, password_hash, name, role, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(email)
        .bind(password_hash)
        .bind(name)
        .bind(role)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    pub async fn get_user(&self, id: Uuid) -> Result<User> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }

    // ==================== Datasources ====================

    pub async fn create_datasource(
        &self,
        org_id: Uuid,
        name: &str,
        ds_type: DatasourceType,
        connection_string_encrypted: &str,
        created_by: Uuid,
    ) -> Result<Datasource> {
        let ds = sqlx::query_as::<_, Datasource>(
            r#"
            INSERT INTO datasources (id, org_id, name, ds_type, connection_string_encrypted, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(name)
        .bind(ds_type)
        .bind(connection_string_encrypted)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(ds)
    }

    pub async fn get_datasource(&self, id: Uuid, org_id: Uuid) -> Result<Datasource> {
        let ds = sqlx::query_as::<_, Datasource>(
            "SELECT * FROM datasources WHERE id = $1 AND org_id = $2",
        )
        .bind(id)
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ds)
    }

    pub async fn list_datasources(&self, org_id: Uuid) -> Result<Vec<Datasource>> {
        let datasources = sqlx::query_as::<_, Datasource>(
            "SELECT * FROM datasources WHERE org_id = $1 ORDER BY created_at DESC",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(datasources)
    }

    pub async fn update_datasource(
        &self,
        id: Uuid,
        org_id: Uuid,
        name: Option<&str>,
        connection_string_encrypted: Option<&str>,
    ) -> Result<Datasource> {
        let ds = sqlx::query_as::<_, Datasource>(
            r#"
            UPDATE datasources 
            SET name = COALESCE($3, name),
                connection_string_encrypted = COALESCE($4, connection_string_encrypted),
                updated_at = NOW()
            WHERE id = $1 AND org_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(name)
        .bind(connection_string_encrypted)
        .fetch_one(&self.pool)
        .await?;

        Ok(ds)
    }

    pub async fn delete_datasource(&self, id: Uuid, org_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM datasources WHERE id = $1 AND org_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // ==================== Queries ====================

    pub async fn create_query(
        &self,
        org_id: Uuid,
        datasource_id: Uuid,
        name: &str,
        description: Option<&str>,
        sql: &str,
        parameters: &serde_json::Value,
        tags: &serde_json::Value,
        timeout_seconds: i32,
        max_rows: i32,
        created_by: Uuid,
    ) -> Result<Query> {
        let query = sqlx::query_as::<_, Query>(
            r#"
            INSERT INTO queries (id, org_id, datasource_id, name, description, sql, parameters, tags, timeout_seconds, max_rows, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(datasource_id)
        .bind(name)
        .bind(description)
        .bind(sql)
        .bind(parameters)
        .bind(tags)
        .bind(timeout_seconds)
        .bind(max_rows)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(query)
    }

    pub async fn get_query(&self, id: Uuid, org_id: Uuid) -> Result<Query> {
        let query =
            sqlx::query_as::<_, Query>("SELECT * FROM queries WHERE id = $1 AND org_id = $2")
                .bind(id)
                .bind(org_id)
                .fetch_one(&self.pool)
                .await?;

        Ok(query)
    }

    pub async fn list_queries(&self, org_id: Uuid) -> Result<Vec<Query>> {
        let queries = sqlx::query_as::<_, Query>(
            "SELECT * FROM queries WHERE org_id = $1 ORDER BY created_at DESC",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(queries)
    }

    pub async fn update_query(
        &self,
        id: Uuid,
        org_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        sql: Option<&str>,
        parameters: Option<&serde_json::Value>,
        tags: Option<&serde_json::Value>,
        timeout_seconds: Option<i32>,
        max_rows: Option<i32>,
    ) -> Result<Query> {
        let query = sqlx::query_as::<_, Query>(
            r#"
            UPDATE queries 
            SET name = COALESCE($3, name),
                description = COALESCE($4, description),
                sql = COALESCE($5, sql),
                parameters = COALESCE($6, parameters),
                tags = COALESCE($7, tags),
                timeout_seconds = COALESCE($8, timeout_seconds),
                max_rows = COALESCE($9, max_rows),
                updated_at = NOW()
            WHERE id = $1 AND org_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(name)
        .bind(description)
        .bind(sql)
        .bind(parameters)
        .bind(tags)
        .bind(timeout_seconds)
        .bind(max_rows)
        .fetch_one(&self.pool)
        .await?;

        Ok(query)
    }

    pub async fn delete_query(&self, id: Uuid, org_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM queries WHERE id = $1 AND org_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // ==================== Runs ====================

    pub async fn create_run(
        &self,
        org_id: Uuid,
        query_id: Uuid,
        datasource_id: Uuid,
        executed_sql: &str,
        parameters: &serde_json::Value,
        timeout_seconds: i32,
        max_rows: i32,
        created_by: Uuid,
    ) -> Result<Run> {
        let run = sqlx::query_as::<_, Run>(
            r#"
            INSERT INTO runs (id, org_id, query_id, datasource_id, executed_sql, parameters, status, timeout_seconds, max_rows, created_by, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, 'queued', $7, $8, $9, NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(query_id)
        .bind(datasource_id)
        .bind(executed_sql)
        .bind(parameters)
        .bind(timeout_seconds)
        .bind(max_rows)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(run)
    }

    pub async fn get_run(&self, id: Uuid, org_id: Uuid) -> Result<Run> {
        let run = sqlx::query_as::<_, Run>("SELECT * FROM runs WHERE id = $1 AND org_id = $2")
            .bind(id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(run)
    }

    pub async fn list_runs(&self, org_id: Uuid, query_id: Option<Uuid>) -> Result<Vec<Run>> {
        let runs = if let Some(qid) = query_id {
            sqlx::query_as::<_, Run>(
                "SELECT * FROM runs WHERE org_id = $1 AND query_id = $2 ORDER BY created_at DESC LIMIT 100",
            )
            .bind(org_id)
            .bind(qid)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Run>(
                "SELECT * FROM runs WHERE org_id = $1 ORDER BY created_at DESC LIMIT 100",
            )
            .bind(org_id)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(runs)
    }

    /// Claim a queued run for execution (called by runner)
    pub async fn claim_run(&self, runner_id: &str) -> Result<Option<Run>> {
        let run = sqlx::query_as::<_, Run>(
            r#"
            UPDATE runs 
            SET status = 'running', runner_id = $1, started_at = NOW()
            WHERE id = (
                SELECT id FROM runs 
                WHERE status = 'queued' 
                ORDER BY created_at ASC 
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING *
            "#,
        )
        .bind(runner_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(run)
    }

    /// Complete a run with success
    pub async fn complete_run(&self, id: Uuid, _result_id: Uuid) -> Result<Run> {
        let run = sqlx::query_as::<_, Run>(
            r#"
            UPDATE runs 
            SET status = 'completed', completed_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(run)
    }

    /// Fail a run with an error message
    pub async fn fail_run(&self, id: Uuid, error_message: &str) -> Result<Run> {
        let run = sqlx::query_as::<_, Run>(
            r#"
            UPDATE runs 
            SET status = 'failed', completed_at = NOW(), error_message = $2
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(error_message)
        .fetch_one(&self.pool)
        .await?;

        Ok(run)
    }

    /// Timeout a run
    pub async fn timeout_run(&self, id: Uuid) -> Result<Run> {
        let run = sqlx::query_as::<_, Run>(
            r#"
            UPDATE runs 
            SET status = 'timeout', completed_at = NOW(), error_message = 'Query execution timed out'
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(run)
    }

    // ==================== Run Results ====================

    pub async fn create_run_result(
        &self,
        run_id: Uuid,
        columns: &serde_json::Value,
        rows: &serde_json::Value,
        row_count: i64,
        byte_count: i64,
        execution_time_ms: i64,
    ) -> Result<RunResult> {
        let result = sqlx::query_as::<_, RunResult>(
            r#"
            INSERT INTO run_results (id, run_id, columns, rows, row_count, byte_count, execution_time_ms, created_at, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW() + INTERVAL '7 days')
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(run_id)
        .bind(columns)
        .bind(rows)
        .bind(row_count)
        .bind(byte_count)
        .bind(execution_time_ms)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_run_result(&self, run_id: Uuid) -> Result<RunResult> {
        let result = sqlx::query_as::<_, RunResult>("SELECT * FROM run_results WHERE run_id = $1")
            .bind(run_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    // ==================== Visualizations ====================

    pub async fn create_visualization(
        &self,
        org_id: Uuid,
        query_id: Uuid,
        name: &str,
        chart_type: ChartType,
        config: &serde_json::Value,
        tags: &serde_json::Value,
        created_by: Uuid,
    ) -> Result<Visualization> {
        let viz = sqlx::query_as::<_, Visualization>(
            r#"
            INSERT INTO visualizations (id, org_id, query_id, name, chart_type, config, tags, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(query_id)
        .bind(name)
        .bind(chart_type)
        .bind(config)
        .bind(tags)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(viz)
    }

    pub async fn get_visualization(&self, id: Uuid, org_id: Uuid) -> Result<Visualization> {
        let viz = sqlx::query_as::<_, Visualization>(
            "SELECT * FROM visualizations WHERE id = $1 AND org_id = $2",
        )
        .bind(id)
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(viz)
    }

    pub async fn list_visualizations(
        &self,
        org_id: Uuid,
        query_id: Option<Uuid>,
    ) -> Result<Vec<Visualization>> {
        let vizs = if let Some(qid) = query_id {
            sqlx::query_as::<_, Visualization>(
                "SELECT * FROM visualizations WHERE org_id = $1 AND query_id = $2 ORDER BY created_at DESC",
            )
            .bind(org_id)
            .bind(qid)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Visualization>(
                "SELECT * FROM visualizations WHERE org_id = $1 ORDER BY created_at DESC",
            )
            .bind(org_id)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(vizs)
    }

    pub async fn delete_visualization(&self, id: Uuid, org_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM visualizations WHERE id = $1 AND org_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_visualization(
        &self,
        id: Uuid,
        org_id: Uuid,
        query_id: Option<Uuid>,
        name: Option<&str>,
        chart_type: Option<ChartType>,
        config: Option<&serde_json::Value>,
        tags: Option<&serde_json::Value>,
    ) -> Result<Visualization> {
        let viz = sqlx::query_as::<_, Visualization>(
            r#"
            UPDATE visualizations
            SET query_id = COALESCE($3, query_id),
                name = COALESCE($4, name),
                chart_type = COALESCE($5, chart_type),
                config = COALESCE($6, config),
                tags = COALESCE($7, tags),
                updated_at = NOW()
            WHERE id = $1 AND org_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(query_id)
        .bind(name)
        .bind(chart_type)
        .bind(config)
        .bind(tags)
        .fetch_one(&self.pool)
        .await?;

        Ok(viz)
    }

    // ==================== Dashboards ====================

    pub async fn create_dashboard(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        parameters: &serde_json::Value,
        tags: &serde_json::Value,
        created_by: Uuid,
    ) -> Result<Dashboard> {
        let dashboard = sqlx::query_as::<_, Dashboard>(
            r#"
            INSERT INTO dashboards (id, org_id, name, description, parameters, tags, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(name)
        .bind(description)
        .bind(parameters)
        .bind(tags)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(dashboard)
    }

    pub async fn get_dashboard(&self, id: Uuid, org_id: Uuid) -> Result<Dashboard> {
        let dashboard = sqlx::query_as::<_, Dashboard>(
            "SELECT * FROM dashboards WHERE id = $1 AND org_id = $2",
        )
        .bind(id)
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(dashboard)
    }

    pub async fn list_dashboards(&self, org_id: Uuid) -> Result<Vec<Dashboard>> {
        let dashboards = sqlx::query_as::<_, Dashboard>(
            "SELECT * FROM dashboards WHERE org_id = $1 ORDER BY created_at DESC",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(dashboards)
    }

    pub async fn delete_dashboard(&self, id: Uuid, org_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM dashboards WHERE id = $1 AND org_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_dashboard(
        &self,
        id: Uuid,
        org_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        parameters: Option<&serde_json::Value>,
        tags: Option<&serde_json::Value>,
    ) -> Result<Dashboard> {
        let dashboard = sqlx::query_as::<_, Dashboard>(
            r#"
            UPDATE dashboards
            SET name = COALESCE($3, name),
                description = COALESCE($4, description),
                parameters = COALESCE($5, parameters),
                tags = COALESCE($6, tags),
                updated_at = NOW()
            WHERE id = $1 AND org_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(name)
        .bind(description)
        .bind(parameters)
        .bind(tags)
        .fetch_one(&self.pool)
        .await?;

        Ok(dashboard)
    }

    // ==================== Tiles ====================

    pub async fn create_tile(
        &self,
        dashboard_id: Uuid,
        visualization_id: Uuid,
        title: Option<&str>,
        pos_x: i32,
        pos_y: i32,
        width: i32,
        height: i32,
        parameter_bindings: &serde_json::Value,
    ) -> Result<Tile> {
        let tile = sqlx::query_as::<_, Tile>(
            r#"
            INSERT INTO tiles (id, dashboard_id, visualization_id, title, pos_x, pos_y, width, height, parameter_bindings, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(dashboard_id)
        .bind(visualization_id)
        .bind(title)
        .bind(pos_x)
        .bind(pos_y)
        .bind(width)
        .bind(height)
        .bind(parameter_bindings)
        .fetch_one(&self.pool)
        .await?;

        Ok(tile)
    }

    pub async fn list_tiles(&self, dashboard_id: Uuid) -> Result<Vec<Tile>> {
        let tiles = sqlx::query_as::<_, Tile>(
            "SELECT * FROM tiles WHERE dashboard_id = $1 ORDER BY pos_y, pos_x",
        )
        .bind(dashboard_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(tiles)
    }

    pub async fn delete_tile(&self, id: Uuid, dashboard_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM tiles WHERE id = $1 AND dashboard_id = $2")
            .bind(id)
            .bind(dashboard_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_tile(
        &self,
        id: Uuid,
        dashboard_id: Uuid,
        title: Option<&str>,
        pos_x: Option<i32>,
        pos_y: Option<i32>,
        width: Option<i32>,
        height: Option<i32>,
        parameter_bindings: Option<&serde_json::Value>,
    ) -> Result<Tile> {
        let tile = sqlx::query_as::<_, Tile>(
            r#"
            UPDATE tiles
            SET title = COALESCE($3, title),
                pos_x = COALESCE($4, pos_x),
                pos_y = COALESCE($5, pos_y),
                width = COALESCE($6, width),
                height = COALESCE($7, height),
                parameter_bindings = COALESCE($8, parameter_bindings),
                updated_at = NOW()
            WHERE id = $1 AND dashboard_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(dashboard_id)
        .bind(title)
        .bind(pos_x)
        .bind(pos_y)
        .bind(width)
        .bind(height)
        .bind(parameter_bindings)
        .fetch_one(&self.pool)
        .await?;

        Ok(tile)
    }

    // ==================== Schedules ====================

    /// Calculate the next run time from a cron expression
    /// Handles both 5-part (standard cron) and 6-part (with seconds) expressions
    fn calculate_next_run(
        cron_expression: &str,
        enabled: bool,
    ) -> Option<chrono::DateTime<chrono::Utc>> {
        if !enabled {
            return None;
        }

        use std::str::FromStr;

        // The cron crate expects 6 or 7 parts (with seconds)
        // Standard cron uses 5 parts: minute hour day-of-month month day-of-week
        // Convert 5-part to 6-part by prepending "0 " for seconds
        let parts: Vec<&str> = cron_expression.split_whitespace().collect();
        let expr = if parts.len() == 5 {
            format!("0 {}", cron_expression)
        } else {
            cron_expression.to_string()
        };

        let schedule = cron::Schedule::from_str(&expr).ok()?;
        schedule.upcoming(chrono::Utc).next()
    }

    pub async fn create_schedule(
        &self,
        org_id: Uuid,
        query_id: Uuid,
        name: &str,
        cron_expression: &str,
        parameters: &serde_json::Value,
        tags: &serde_json::Value,
        enabled: bool,
        created_by: Uuid,
    ) -> Result<Schedule> {
        let next_run_at = Self::calculate_next_run(cron_expression, enabled);

        let schedule = sqlx::query_as::<_, Schedule>(
            r#"
            INSERT INTO schedules (id, org_id, query_id, name, cron_expression, parameters, tags, enabled, next_run_at, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(query_id)
        .bind(name)
        .bind(cron_expression)
        .bind(parameters)
        .bind(tags)
        .bind(enabled)
        .bind(next_run_at)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(schedule)
    }

    pub async fn list_schedules(&self, org_id: Uuid) -> Result<Vec<Schedule>> {
        let schedules = sqlx::query_as::<_, Schedule>(
            "SELECT * FROM schedules WHERE org_id = $1 ORDER BY created_at DESC",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(schedules)
    }

    pub async fn get_due_schedules(&self) -> Result<Vec<Schedule>> {
        let schedules = sqlx::query_as::<_, Schedule>(
            "SELECT * FROM schedules WHERE enabled = true AND (next_run_at IS NULL OR next_run_at <= NOW())",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(schedules)
    }

    pub async fn update_schedule_last_run(
        &self,
        id: Uuid,
        cron_expression: &str,
        enabled: bool,
    ) -> Result<()> {
        let next_run_at = Self::calculate_next_run(cron_expression, enabled);

        sqlx::query(
            "UPDATE schedules SET last_run_at = NOW(), next_run_at = $2, updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .bind(next_run_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_schedule(&self, id: Uuid, org_id: Uuid) -> Result<Schedule> {
        let schedule =
            sqlx::query_as::<_, Schedule>("SELECT * FROM schedules WHERE id = $1 AND org_id = $2")
                .bind(id)
                .bind(org_id)
                .fetch_optional(&self.pool)
                .await?
                .ok_or_else(|| Error::NotFound("Schedule not found".into()))?;

        Ok(schedule)
    }

    pub async fn update_schedule(
        &self,
        id: Uuid,
        org_id: Uuid,
        name: Option<&str>,
        cron_expression: Option<&str>,
        parameters: Option<&serde_json::Value>,
        tags: Option<&serde_json::Value>,
        enabled: Option<bool>,
    ) -> Result<Schedule> {
        // Get existing schedule to determine current values for next_run calculation
        let existing = self.get_schedule(id, org_id).await?;

        let new_cron = cron_expression.unwrap_or(&existing.cron_expression);
        let new_enabled = enabled.unwrap_or(existing.enabled);
        let next_run_at = Self::calculate_next_run(new_cron, new_enabled);

        // Use a simpler approach: update all fields, using COALESCE for optional ones
        let schedule = sqlx::query_as::<_, Schedule>(
            r#"
            UPDATE schedules SET
                name = COALESCE($3, name),
                cron_expression = COALESCE($4, cron_expression),
                parameters = COALESCE($5, parameters),
                tags = COALESCE($6, tags),
                enabled = COALESCE($7, enabled),
                next_run_at = $8,
                updated_at = NOW()
            WHERE id = $1 AND org_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(name)
        .bind(cron_expression)
        .bind(parameters)
        .bind(tags)
        .bind(enabled)
        .bind(next_run_at)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound("Schedule not found".into()))?;

        Ok(schedule)
    }

    pub async fn delete_schedule(&self, id: Uuid, org_id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM schedules WHERE id = $1 AND org_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound("Schedule not found".into()));
        }

        Ok(())
    }

    pub async fn enable_schedule(&self, id: Uuid, org_id: Uuid) -> Result<Schedule> {
        // First get the schedule to access the cron expression
        let existing = self.get_schedule(id, org_id).await?;
        let next_run_at = Self::calculate_next_run(&existing.cron_expression, true);

        let schedule = sqlx::query_as::<_, Schedule>(
            "UPDATE schedules SET enabled = true, next_run_at = $3, updated_at = NOW() WHERE id = $1 AND org_id = $2 RETURNING *",
        )
        .bind(id)
        .bind(org_id)
        .bind(next_run_at)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound("Schedule not found".into()))?;

        Ok(schedule)
    }

    pub async fn disable_schedule(&self, id: Uuid, org_id: Uuid) -> Result<Schedule> {
        let schedule = sqlx::query_as::<_, Schedule>(
            "UPDATE schedules SET enabled = false, next_run_at = NULL, updated_at = NOW() WHERE id = $1 AND org_id = $2 RETURNING *",
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound("Schedule not found".into()))?;

        Ok(schedule)
    }
}
