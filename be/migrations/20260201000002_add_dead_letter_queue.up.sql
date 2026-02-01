-- Dead Letter Queue for permanently failed runs

CREATE TABLE run_failures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id UUID NOT NULL,
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    query_id UUID NOT NULL,
    datasource_id UUID NOT NULL,
    executed_sql TEXT NOT NULL,
    parameters JSONB NOT NULL DEFAULT '{}',
    error_message TEXT NOT NULL,
    retry_count INTEGER NOT NULL,
    max_retries INTEGER NOT NULL,
    first_failed_at TIMESTAMPTZ NOT NULL,
    last_failed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for querying
CREATE INDEX idx_run_failures_org_id ON run_failures(org_id);
CREATE INDEX idx_run_failures_query_id ON run_failures(query_id);
CREATE INDEX idx_run_failures_created_at ON run_failures(created_at DESC);

-- Comments
COMMENT ON TABLE run_failures IS 'Dead letter queue for runs that exceeded max retries';
COMMENT ON COLUMN run_failures.run_id IS 'Original run ID (may no longer exist in runs table)';
COMMENT ON COLUMN run_failures.first_failed_at IS 'Timestamp when the run first failed';
COMMENT ON COLUMN run_failures.last_failed_at IS 'Timestamp of the last retry attempt';
