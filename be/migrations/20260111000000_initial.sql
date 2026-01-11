-- Initial schema for Loupe BI
-- Organizations
CREATE TABLE
    organizations (
        id UUID PRIMARY KEY,
        name TEXT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

-- Users
CREATE TABLE
    users (
        id UUID PRIMARY KEY,
        org_id UUID NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
        email TEXT NOT NULL UNIQUE,
        password_hash TEXT NOT NULL,
        name TEXT NOT NULL,
        role TEXT NOT NULL DEFAULT 'viewer' CHECK (role IN ('admin', 'editor', 'viewer')),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

CREATE INDEX idx_users_org_id ON users (org_id);

CREATE INDEX idx_users_email ON users (email);

-- Datasources
CREATE TABLE
    datasources (
        id UUID PRIMARY KEY,
        org_id UUID NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
        name TEXT NOT NULL,
        ds_type TEXT NOT NULL DEFAULT 'postgres' CHECK (ds_type IN ('postgres')),
        connection_string_encrypted TEXT NOT NULL,
        created_by UUID NOT NULL REFERENCES users (id),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

CREATE INDEX idx_datasources_org_id ON datasources (org_id);

-- Queries
CREATE TABLE
    queries (
        id UUID PRIMARY KEY,
        org_id UUID NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
        datasource_id UUID NOT NULL REFERENCES datasources (id) ON DELETE CASCADE,
        name TEXT NOT NULL,
        description TEXT,
        SQL TEXT NOT NULL,
        parameters JSONB NOT NULL DEFAULT '[]',
        timeout_seconds INTEGER NOT NULL DEFAULT 30,
        max_rows INTEGER NOT NULL DEFAULT 10000,
        created_by UUID NOT NULL REFERENCES users (id),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

CREATE INDEX idx_queries_org_id ON queries (org_id);

CREATE INDEX idx_queries_datasource_id ON queries (datasource_id);

-- Runs
CREATE TABLE
    runs (
        id UUID PRIMARY KEY,
        org_id UUID NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
        query_id UUID NOT NULL REFERENCES queries (id) ON DELETE CASCADE,
        datasource_id UUID NOT NULL REFERENCES datasources (id) ON DELETE CASCADE,
        executed_sql TEXT NOT NULL,
        parameters JSONB NOT NULL DEFAULT '{}',
        status TEXT NOT NULL DEFAULT 'queued' CHECK (status IN ('queued', 'running', 'completed', 'failed', 'cancelled', 'timeout')),
        runner_id TEXT,
        timeout_seconds INTEGER NOT NULL DEFAULT 30,
        max_rows INTEGER NOT NULL DEFAULT 10000,
        started_at TIMESTAMPTZ,
        completed_at TIMESTAMPTZ,
        error_message TEXT,
        created_by UUID NOT NULL REFERENCES users (id),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

CREATE INDEX idx_runs_org_id ON runs (org_id);

CREATE INDEX idx_runs_query_id ON runs (query_id);

CREATE INDEX idx_runs_status ON runs (status);

CREATE INDEX idx_runs_created_at ON runs (created_at);

-- Run Results
CREATE TABLE
    run_results (
        id UUID PRIMARY KEY,
        run_id UUID NOT NULL UNIQUE REFERENCES runs (id) ON DELETE CASCADE,
        columns JSONB NOT NULL,
        ROWS JSONB NOT NULL,
        row_count BIGINT NOT NULL,
        byte_count BIGINT NOT NULL,
        execution_time_ms BIGINT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        expires_at TIMESTAMPTZ
    );

CREATE INDEX idx_run_results_run_id ON run_results (run_id);

CREATE INDEX idx_run_results_expires_at ON run_results (expires_at);

-- Visualizations
CREATE TABLE
    visualizations (
        id UUID PRIMARY KEY,
        org_id UUID NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
        query_id UUID NOT NULL REFERENCES queries (id) ON DELETE CASCADE,
        name TEXT NOT NULL,
        chart_type TEXT NOT NULL CHECK (chart_type IN ('table', 'line', 'bar', 'single_stat')),
        config JSONB NOT NULL DEFAULT '{}',
        created_by UUID NOT NULL REFERENCES users (id),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

CREATE INDEX idx_visualizations_org_id ON visualizations (org_id);

CREATE INDEX idx_visualizations_query_id ON visualizations (query_id);

-- Dashboards
CREATE TABLE
    dashboards (
        id UUID PRIMARY KEY,
        org_id UUID NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
        name TEXT NOT NULL,
        description TEXT,
        parameters JSONB NOT NULL DEFAULT '[]',
        created_by UUID NOT NULL REFERENCES users (id),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

CREATE INDEX idx_dashboards_org_id ON dashboards (org_id);

-- Tiles
CREATE TABLE
    tiles (
        id UUID PRIMARY KEY,
        dashboard_id UUID NOT NULL REFERENCES dashboards (id) ON DELETE CASCADE,
        visualization_id UUID NOT NULL REFERENCES visualizations (id) ON DELETE CASCADE,
        title TEXT,
        pos_x INTEGER NOT NULL DEFAULT 0,
        pos_y INTEGER NOT NULL DEFAULT 0,
        width INTEGER NOT NULL DEFAULT 4,
        height INTEGER NOT NULL DEFAULT 4,
        parameter_bindings JSONB NOT NULL DEFAULT '{}',
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

CREATE INDEX idx_tiles_dashboard_id ON tiles (dashboard_id);

-- Schedules
CREATE TABLE
    schedules (
        id UUID PRIMARY KEY,
        org_id UUID NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
        query_id UUID NOT NULL REFERENCES queries (id) ON DELETE CASCADE,
        name TEXT NOT NULL,
        cron_expression TEXT NOT NULL,
        parameters JSONB NOT NULL DEFAULT '{}',
        enabled BOOLEAN NOT NULL DEFAULT TRUE,
        last_run_at TIMESTAMPTZ,
        next_run_at TIMESTAMPTZ,
        created_by UUID NOT NULL REFERENCES users (id),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

CREATE INDEX idx_schedules_org_id ON schedules (org_id);

CREATE INDEX idx_schedules_enabled ON schedules (enabled);

CREATE INDEX idx_schedules_next_run_at ON schedules (next_run_at);
