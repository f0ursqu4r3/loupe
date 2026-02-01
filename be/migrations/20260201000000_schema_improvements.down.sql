-- Rollback schema improvements migration

-- Remove added column
ALTER TABLE canvases DROP COLUMN IF EXISTS tags;

-- Remove CHECK constraints (in reverse order)
ALTER TABLE canvases DROP CONSTRAINT IF EXISTS check_canvases_time_offset;
ALTER TABLE run_results DROP CONSTRAINT IF EXISTS check_run_results_execution_time;
ALTER TABLE run_results DROP CONSTRAINT IF EXISTS check_run_results_byte_count;
ALTER TABLE run_results DROP CONSTRAINT IF EXISTS check_run_results_row_count;
ALTER TABLE canvas_nodes DROP CONSTRAINT IF EXISTS check_canvas_nodes_size;
ALTER TABLE canvas_nodes DROP CONSTRAINT IF EXISTS check_canvas_nodes_position;
ALTER TABLE tiles DROP CONSTRAINT IF EXISTS check_tiles_size;
ALTER TABLE tiles DROP CONSTRAINT IF EXISTS check_tiles_position;
ALTER TABLE runs DROP CONSTRAINT IF EXISTS check_runs_max_rows_range;
ALTER TABLE runs DROP CONSTRAINT IF EXISTS check_runs_timeout_range;
ALTER TABLE queries DROP CONSTRAINT IF EXISTS check_queries_max_rows_range;
ALTER TABLE queries DROP CONSTRAINT IF EXISTS check_queries_timeout_range;

-- Remove partial indexes
DROP INDEX IF EXISTS idx_runs_failed;
DROP INDEX IF EXISTS idx_runs_active;

-- Remove composite indexes
DROP INDEX IF EXISTS idx_runs_query_status_created;
DROP INDEX IF EXISTS idx_schedules_enabled_next_run;
DROP INDEX IF EXISTS idx_runs_org_status_created;

-- Remove filtering indexes
DROP INDEX IF EXISTS idx_canvas_nodes_type;
DROP INDEX IF EXISTS idx_tiles_visualization_id;
DROP INDEX IF EXISTS idx_schedules_query_id;
DROP INDEX IF EXISTS idx_runs_datasource_id;

-- Remove sorting indexes
DROP INDEX IF EXISTS idx_runs_completed_at;
DROP INDEX IF EXISTS idx_runs_started_at;
DROP INDEX IF EXISTS idx_canvases_updated_at;
DROP INDEX IF EXISTS idx_canvases_created_at;
DROP INDEX IF EXISTS idx_canvases_name;
DROP INDEX IF EXISTS idx_canvases_tags;
DROP INDEX IF EXISTS idx_datasources_updated_at;
DROP INDEX IF EXISTS idx_datasources_created_at;
DROP INDEX IF EXISTS idx_datasources_name;
DROP INDEX IF EXISTS idx_schedules_updated_at;
DROP INDEX IF EXISTS idx_schedules_created_at;
DROP INDEX IF EXISTS idx_schedules_name;
DROP INDEX IF EXISTS idx_visualizations_updated_at;
DROP INDEX IF EXISTS idx_visualizations_created_at;
DROP INDEX IF EXISTS idx_visualizations_name;
DROP INDEX IF EXISTS idx_queries_updated_at;
DROP INDEX IF EXISTS idx_queries_created_at;
DROP INDEX IF EXISTS idx_queries_name;
DROP INDEX IF EXISTS idx_dashboards_updated_at;
DROP INDEX IF EXISTS idx_dashboards_created_at;
DROP INDEX IF EXISTS idx_dashboards_name;
