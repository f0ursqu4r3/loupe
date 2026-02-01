-- Remove retry support from runs table

DROP INDEX IF EXISTS idx_runs_retry_ready;

ALTER TABLE runs
DROP COLUMN IF EXISTS next_retry_at,
DROP COLUMN IF EXISTS max_retries,
DROP COLUMN IF EXISTS retry_count;
