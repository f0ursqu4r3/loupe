-- Remove text search indexes

DROP INDEX IF EXISTS idx_canvases_name_trgm;
DROP INDEX IF EXISTS idx_datasources_name_trgm;
DROP INDEX IF EXISTS idx_schedules_name_trgm;
DROP INDEX IF EXISTS idx_visualizations_name_trgm;
DROP INDEX IF EXISTS idx_queries_sql_trgm;
DROP INDEX IF EXISTS idx_queries_description_trgm;
DROP INDEX IF EXISTS idx_queries_name_trgm;
DROP INDEX IF EXISTS idx_dashboards_name_trgm;

-- Note: We don't drop the pg_trgm extension as other parts of the system might use it
