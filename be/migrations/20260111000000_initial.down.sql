-- Reverse initial schema
-- Drop indexes and tables in reverse order of creation
DROP INDEX IF EXISTS idx_schedules_tags;

DROP INDEX IF EXISTS idx_schedules_next_run_at;

DROP INDEX IF EXISTS idx_schedules_enabled;

DROP INDEX IF EXISTS idx_schedules_org_id;

DROP TABLE IF EXISTS schedules;

DROP INDEX IF EXISTS idx_tiles_dashboard_id;

DROP TABLE IF EXISTS tiles;

DROP INDEX IF EXISTS idx_dashboards_tags;

DROP INDEX IF EXISTS idx_dashboards_org_id;

DROP TABLE IF EXISTS dashboards;

DROP INDEX IF EXISTS idx_visualizations_tags;

DROP INDEX IF EXISTS idx_visualizations_query_id;

DROP INDEX IF EXISTS idx_visualizations_org_id;

DROP TABLE IF EXISTS visualizations;

DROP INDEX IF EXISTS idx_run_results_expires_at;

DROP INDEX IF EXISTS idx_run_results_run_id;

DROP TABLE IF EXISTS run_results;

DROP INDEX IF EXISTS idx_runs_created_at;

DROP INDEX IF EXISTS idx_runs_status;

DROP INDEX IF EXISTS idx_runs_query_id;

DROP INDEX IF EXISTS idx_runs_org_id;

DROP TABLE IF EXISTS runs;

DROP INDEX IF EXISTS idx_queries_tags;

DROP INDEX IF EXISTS idx_queries_datasource_id;

DROP INDEX IF EXISTS idx_queries_org_id;

DROP TABLE IF EXISTS queries;

DROP INDEX IF EXISTS idx_datasources_org_id;

DROP TABLE IF EXISTS datasources;

DROP INDEX IF EXISTS idx_users_email;

DROP INDEX IF EXISTS idx_users_org_id;

DROP TABLE IF EXISTS users;

DROP TABLE IF EXISTS organizations;
