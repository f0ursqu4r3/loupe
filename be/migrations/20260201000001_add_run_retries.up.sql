-- Add retry support to runs table

ALTER TABLE runs
ADD COLUMN retry_count INTEGER NOT NULL DEFAULT 0,
ADD COLUMN max_retries INTEGER NOT NULL DEFAULT 3,
ADD COLUMN next_retry_at TIMESTAMPTZ NULL;

-- Index for finding runs ready for retry
CREATE INDEX idx_runs_retry_ready ON runs (next_retry_at, status)
WHERE next_retry_at IS NOT NULL AND status = 'failed';

-- Add comment
COMMENT ON COLUMN runs.retry_count IS 'Number of times this run has been retried';
COMMENT ON COLUMN runs.max_retries IS 'Maximum number of retry attempts allowed';
COMMENT ON COLUMN runs.next_retry_at IS 'Timestamp when this run is eligible for retry (NULL if not retrying)';
