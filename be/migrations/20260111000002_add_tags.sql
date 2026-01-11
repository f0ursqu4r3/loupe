-- Add tags column to queries, visualizations, dashboards, and schedules
ALTER TABLE queries
ADD COLUMN tags JSONB NOT NULL DEFAULT '[]';

ALTER TABLE visualizations
ADD COLUMN tags JSONB NOT NULL DEFAULT '[]';

ALTER TABLE dashboards
ADD COLUMN tags JSONB NOT NULL DEFAULT '[]';

ALTER TABLE schedules
ADD COLUMN tags JSONB NOT NULL DEFAULT '[]';

-- Create GIN indexes for efficient tag queries
CREATE INDEX idx_queries_tags ON queries USING GIN (tags);

CREATE INDEX idx_visualizations_tags ON visualizations USING GIN (tags);

CREATE INDEX idx_dashboards_tags ON dashboards USING GIN (tags);

CREATE INDEX idx_schedules_tags ON schedules USING GIN (tags);
