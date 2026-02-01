-- Add text search indexes for efficient ILIKE queries
-- Using pg_trgm (trigram) extension for fast similarity searches

CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Dashboards: name searches
CREATE INDEX IF NOT EXISTS idx_dashboards_name_trgm ON dashboards USING gin (name gin_trgm_ops);

-- Queries: name, description, and SQL searches
CREATE INDEX IF NOT EXISTS idx_queries_name_trgm ON queries USING gin (name gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_queries_description_trgm ON queries USING gin (description gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_queries_sql_trgm ON queries USING gin (sql gin_trgm_ops);

-- Visualizations: name searches
CREATE INDEX IF NOT EXISTS idx_visualizations_name_trgm ON visualizations USING gin (name gin_trgm_ops);

-- Schedules: name searches
CREATE INDEX IF NOT EXISTS idx_schedules_name_trgm ON schedules USING gin (name gin_trgm_ops);

-- Datasources: name searches
CREATE INDEX IF NOT EXISTS idx_datasources_name_trgm ON datasources USING gin (name gin_trgm_ops);

-- Canvases: name searches
CREATE INDEX IF NOT EXISTS idx_canvases_name_trgm ON canvases USING gin (name gin_trgm_ops);
