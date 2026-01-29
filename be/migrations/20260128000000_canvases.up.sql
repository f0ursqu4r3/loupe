-- Canvases schema for semantic data analysis
-- Canvases (collections of query nodes and notes)
CREATE TABLE
    canvases (
        id UUID PRIMARY KEY,
        org_id UUID NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
        name TEXT NOT NULL,
        -- Time range settings
        time_preset TEXT NOT NULL DEFAULT '7d' CHECK (
            time_preset IN (
                'now',
                '1h',
                '3h',
                '6h',
                '12h',
                '24h',
                '7d',
                '30d',
                '90d',
                'custom'
            )
        ),
        time_offset BIGINT NOT NULL DEFAULT 0,
        time_custom_start TIMESTAMPTZ,
        time_custom_end TIMESTAMPTZ,
        live BOOLEAN NOT NULL DEFAULT FALSE,
        created_by UUID NOT NULL REFERENCES users (id),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

CREATE INDEX idx_canvases_org_id ON canvases (org_id);

CREATE INDEX idx_canvases_created_by ON canvases (created_by);

-- Canvas Nodes (query or note nodes within a canvas)
CREATE TABLE
    canvas_nodes (
        id UUID PRIMARY KEY,
        canvas_id UUID NOT NULL REFERENCES canvases (id) ON DELETE CASCADE,
        node_type TEXT NOT NULL CHECK (node_type IN ('query', 'note')),
        title TEXT NOT NULL,
        pos_x DOUBLE PRECISION NOT NULL DEFAULT 0,
        pos_y DOUBLE PRECISION NOT NULL DEFAULT 0,
        width DOUBLE PRECISION NOT NULL DEFAULT 280,
        height DOUBLE PRECISION NOT NULL DEFAULT 160,
        -- Flexible metadata: datasourceId, sql, viz, vizConfig, text, etc.
        meta JSONB NOT NULL DEFAULT '{}',
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

CREATE INDEX idx_canvas_nodes_canvas_id ON canvas_nodes (canvas_id);

-- Canvas Edges (connections between nodes with semantic relationships)
CREATE TABLE
    canvas_edges (
        id UUID PRIMARY KEY,
        canvas_id UUID NOT NULL REFERENCES canvases (id) ON DELETE CASCADE,
        from_node_id UUID NOT NULL REFERENCES canvas_nodes (id) ON DELETE CASCADE,
        to_node_id UUID NOT NULL REFERENCES canvas_nodes (id) ON DELETE CASCADE,
        label TEXT NOT NULL CHECK (
            label IN (
                'motivates',
                'explains',
                'contradicts',
                'supports',
                'derives_from',
                'questions'
            )
        ),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

CREATE INDEX idx_canvas_edges_canvas_id ON canvas_edges (canvas_id);

CREATE INDEX idx_canvas_edges_from_node ON canvas_edges (from_node_id);

CREATE INDEX idx_canvas_edges_to_node ON canvas_edges (to_node_id);

-- Prevent duplicate edges between same nodes
CREATE UNIQUE INDEX idx_canvas_edges_unique ON canvas_edges (from_node_id, to_node_id);
