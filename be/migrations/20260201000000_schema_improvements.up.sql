-- Schema improvements: Add missing indexes, constraints, and optimizations
-- This migration improves data integrity and query performance

-- ====================
-- INDEXES FOR SORTING
-- ====================
-- Add indexes on commonly sorted columns (name, created_at, updated_at)

-- Dashboards
CREATE INDEX IF NOT EXISTS idx_dashboards_name ON dashboards (name);
CREATE INDEX IF NOT EXISTS idx_dashboards_created_at ON dashboards (created_at);
CREATE INDEX IF NOT EXISTS idx_dashboards_updated_at ON dashboards (updated_at);

-- Queries
CREATE INDEX IF NOT EXISTS idx_queries_name ON queries (name);
CREATE INDEX IF NOT EXISTS idx_queries_created_at ON queries (created_at);
CREATE INDEX IF NOT EXISTS idx_queries_updated_at ON queries (updated_at);

-- Visualizations
CREATE INDEX IF NOT EXISTS idx_visualizations_name ON visualizations (name);
CREATE INDEX IF NOT EXISTS idx_visualizations_created_at ON visualizations (created_at);
CREATE INDEX IF NOT EXISTS idx_visualizations_updated_at ON visualizations (updated_at);

-- Schedules
CREATE INDEX IF NOT EXISTS idx_schedules_name ON schedules (name);
CREATE INDEX IF NOT EXISTS idx_schedules_created_at ON schedules (created_at);
CREATE INDEX IF NOT EXISTS idx_schedules_updated_at ON schedules (updated_at);

-- Datasources
CREATE INDEX IF NOT EXISTS idx_datasources_name ON datasources (name);
CREATE INDEX IF NOT EXISTS idx_datasources_created_at ON datasources (created_at);
CREATE INDEX IF NOT EXISTS idx_datasources_updated_at ON datasources (updated_at);

-- Canvases
CREATE INDEX IF NOT EXISTS idx_canvases_name ON canvases (name);
CREATE INDEX IF NOT EXISTS idx_canvases_created_at ON canvases (created_at);
CREATE INDEX IF NOT EXISTS idx_canvases_updated_at ON canvases (updated_at);

-- Runs (sorting by completion times)
CREATE INDEX IF NOT EXISTS idx_runs_started_at ON runs (started_at);
CREATE INDEX IF NOT EXISTS idx_runs_completed_at ON runs (completed_at);

-- ====================
-- INDEXES FOR FILTERING
-- ====================
-- Add missing indexes for common filter patterns

-- Runs by datasource (for datasource analytics)
CREATE INDEX IF NOT EXISTS idx_runs_datasource_id ON runs (datasource_id);

-- Schedules by query (for query detail view)
CREATE INDEX IF NOT EXISTS idx_schedules_query_id ON schedules (query_id);

-- Tiles by visualization (reverse lookup)
CREATE INDEX IF NOT EXISTS idx_tiles_visualization_id ON tiles (visualization_id);

-- Canvas nodes by type (for filtering)
CREATE INDEX IF NOT EXISTS idx_canvas_nodes_type ON canvas_nodes (node_type);

-- ====================
-- COMPOSITE INDEXES
-- ====================
-- Multi-column indexes for common query patterns

-- Runs: Filter by org + status + sort by created_at (most common query)
CREATE INDEX IF NOT EXISTS idx_runs_org_status_created ON runs (org_id, status, created_at DESC);

-- Schedules: Scheduler query pattern (enabled schedules by next run time)
CREATE INDEX IF NOT EXISTS idx_schedules_enabled_next_run ON schedules (enabled, next_run_at) WHERE enabled = TRUE;

-- Runs: Filter by query + status + sort by created (query detail view)
CREATE INDEX IF NOT EXISTS idx_runs_query_status_created ON runs (query_id, status, created_at DESC);

-- ====================
-- PARTIAL INDEXES
-- ====================
-- Indexes for specific subsets of data that are frequently queried

-- Active (non-completed) runs for monitoring
CREATE INDEX IF NOT EXISTS idx_runs_active ON runs (created_at DESC) WHERE status IN ('queued', 'running');

-- Failed runs for error analysis
CREATE INDEX IF NOT EXISTS idx_runs_failed ON runs (created_at DESC) WHERE status IN ('failed', 'timeout', 'cancelled');

-- Enabled schedules (most scheduler queries only look at enabled)
-- Note: Already covered by idx_schedules_enabled_next_run composite index

-- ====================
-- CHECK CONSTRAINTS
-- ====================
-- Add data validation constraints

-- Queries: Validate timeout and max_rows ranges
ALTER TABLE queries
ADD CONSTRAINT check_queries_timeout_range CHECK (timeout_seconds > 0 AND timeout_seconds <= 3600);

ALTER TABLE queries
ADD CONSTRAINT check_queries_max_rows_range CHECK (max_rows > 0 AND max_rows <= 1000000);

-- Runs: Validate timeout and max_rows ranges
ALTER TABLE runs
ADD CONSTRAINT check_runs_timeout_range CHECK (timeout_seconds > 0 AND timeout_seconds <= 3600);

ALTER TABLE runs
ADD CONSTRAINT check_runs_max_rows_range CHECK (max_rows > 0 AND max_rows <= 1000000);

-- Tiles: Validate position and size
ALTER TABLE tiles
ADD CONSTRAINT check_tiles_position CHECK (pos_x >= 0 AND pos_y >= 0);

ALTER TABLE tiles
ADD CONSTRAINT check_tiles_size CHECK (width > 0 AND height > 0);

-- Canvas nodes: Validate position and size
ALTER TABLE canvas_nodes
ADD CONSTRAINT check_canvas_nodes_position CHECK (pos_x IS NOT NULL AND pos_y IS NOT NULL);

ALTER TABLE canvas_nodes
ADD CONSTRAINT check_canvas_nodes_size CHECK (width > 0 AND height > 0);

-- Run results: Validate counts and metrics
ALTER TABLE run_results
ADD CONSTRAINT check_run_results_row_count CHECK (row_count >= 0);

ALTER TABLE run_results
ADD CONSTRAINT check_run_results_byte_count CHECK (byte_count >= 0);

ALTER TABLE run_results
ADD CONSTRAINT check_run_results_execution_time CHECK (execution_time_ms >= 0);

-- Canvases: Validate time offset
ALTER TABLE canvases
ADD CONSTRAINT check_canvases_time_offset CHECK (time_offset >= 0);

-- ====================
-- MISSING COLUMNS
-- ====================
-- Add tags to canvases (for consistency with other entities)

ALTER TABLE canvases
ADD COLUMN IF NOT EXISTS tags JSONB NOT NULL DEFAULT '[]';

-- Add GIN index for canvas tags
CREATE INDEX IF NOT EXISTS idx_canvases_tags ON canvases USING GIN (tags);

-- ====================
-- COMMENTS
-- ====================
-- Add helpful comments to document schema design decisions

COMMENT ON COLUMN queries.timeout_seconds IS 'Query execution timeout in seconds (1-3600)';
COMMENT ON COLUMN queries.max_rows IS 'Maximum number of rows to return (1-1000000)';
COMMENT ON COLUMN runs.status IS 'Run execution status: queued, running, completed, failed, cancelled, timeout';
COMMENT ON COLUMN tiles.parameter_bindings IS 'Maps dashboard parameters to query parameters';
COMMENT ON COLUMN canvases.time_preset IS 'Predefined time range: now, 1h, 3h, 6h, 12h, 24h, 7d, 30d, 90d, custom';
COMMENT ON COLUMN canvases.live IS 'Whether the canvas auto-refreshes queries';
COMMENT ON INDEX idx_runs_org_status_created IS 'Supports filtering runs by org+status and sorting by created_at';
COMMENT ON INDEX idx_schedules_enabled_next_run IS 'Optimizes scheduler queries for enabled schedules by next run time';
COMMENT ON INDEX idx_runs_active IS 'Partial index for monitoring active (queued/running) runs';
