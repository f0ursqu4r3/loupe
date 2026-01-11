-- Add 'pie' chart type to visualizations
ALTER TABLE visualizations
DROP CONSTRAINT visualizations_chart_type_check;

ALTER TABLE visualizations ADD CONSTRAINT visualizations_chart_type_check CHECK (chart_type IN ('table', 'line', 'bar', 'pie', 'single_stat'));
