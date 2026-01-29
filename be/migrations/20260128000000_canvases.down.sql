-- Reverse canvases schema
-- Drop indexes and tables in reverse order of creation
DROP INDEX IF EXISTS idx_canvas_edges_unique;

DROP INDEX IF EXISTS idx_canvas_edges_to_node;

DROP INDEX IF EXISTS idx_canvas_edges_from_node;

DROP INDEX IF EXISTS idx_canvas_edges_canvas_id;

DROP TABLE IF EXISTS canvas_edges;

DROP INDEX IF EXISTS idx_canvas_nodes_canvas_id;

DROP TABLE IF EXISTS canvas_nodes;

DROP INDEX IF EXISTS idx_canvases_created_by;

DROP INDEX IF EXISTS idx_canvases_org_id;

DROP TABLE IF EXISTS canvases;
