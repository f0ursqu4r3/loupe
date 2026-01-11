# Agents

This repo uses “agents” as owned domains of work. Each agent corresponds to a technical area with clear responsibilities and interfaces.

## Agent: API (Actix-Web)

### Owns (API)

- Authentication (session/token)
- Authorization (org + coarse roles)
- Metadata CRUD APIs:
  - datasources
  - queries
  - dashboards, tiles
  - visualizations
  - schedules
- Run orchestration:
  - create run record
  - enqueue run request
  - expose run status + results
- Validation of user input (SQL stored as-is; params validated against schema)

### Does NOT own (API)

- Executing queries
- Database driver/SDK dependencies for connectors
- Any long-running work

### Interfaces (API)

- DB: Postgres metadata DB
- Queue: `runs` table (or queue abstraction) used by runners to claim work
- HTTP: REST/JSON (or OpenAPI-defined) endpoints used by the frontend

## Agent: Runner (Worker)

### Owns (Runner)

- Claiming queued runs
- Executing SQL through connector interface
- Enforcing resource limits:
  - timeout
  - max rows/bytes
  - concurrency caps (global + per datasource)
- Persisting results payload and run stats
- Best-effort cancellation

### Does NOT own (Runner)

- User auth flows
- Dashboard composition logic
- Visualization rendering

### Interfaces (Runner)

- DB: Postgres metadata DB (reads run definitions, writes status/results)
- Connector: `Connector` interface (v1: PostgresConnector)
- Optional: object storage client (for large results)

## Agent: Connector (Postgres v1)

### Owns (Connector)

- Postgres connectivity and execution details:
  - connection pooling
  - statement timeouts
  - prepared statements / safe binding
  - schema introspection

### Contract (Connector, conceptual)

- `test_connection(config) -> ok/error`
- `introspect(config) -> schemas/tables/columns`
- `execute(config, sql, params, options) -> result_stream_or_pages`
- `cancel(handle) -> best_effort`

## Agent: Frontend (Vue)

### Owns (Frontend)

- Query editor UI (Monaco)
- Parameter forms (typed)
- Results viewing
- Visualization configuration UI
- Dashboard layout editor
- Schedule configuration UI

### Does NOT own (Frontend)

- Query execution logic (other than triggering runs)
- Persistence rules
- Any secret handling

### Interfaces (Frontend)

- API endpoints for metadata, runs, results, and schedules

## Agent: Storage & Migrations

### Owns (Storage & Migrations)

- Postgres schema design
- Migrations (idempotent, forward-only)
- Encryption strategy for secrets (datasource passwords)
- Retention/TTL policies for results
- Indexing and query performance for run/result lookup

## Agent: Observability

### Owns (Observability)

- Logging format + correlation IDs (`run_id`)
- Metrics:
  - queue depth
  - run duration
  - success/failure rates
  - per-connector timing
- Tracing boundaries:
  - API request → run creation
  - runner claim → execute → persist

## Operating Conventions

### Source of truth

- Metadata + run state lives in Postgres.
- Results are served through the API; runners must write results to shared storage.

### Run lifecycle (states)

- `queued` → `running` → `succeeded | failed | canceled`
- Runner is responsible for transitions after `queued`.

### Failure rules

- Every failure must produce:
  - stable status (`failed`)
  - error message (sanitized)
  - timing and resource stats
- Runner retries are explicit (do not auto-retry without a policy).

### Security rules

- Secrets are encrypted at rest in the metadata DB.
- Never send datasource secrets to the frontend.
- Parameter binding must not allow injection via values.

### Extensibility rule

- API must remain connector-agnostic.
- Runner may evolve connector implementation from in-process to RPC plugin without changing:
  - run record schema
  - API endpoints
  - frontend contracts

## Development Workflow

### Local services

- Postgres (metadata DB)
- Optional object storage (MinIO) if enabled

### Suggested make targets (example)

- `make api`        # run API in dev
- `make runner`     # run a runner in dev
- `make web`        # run Vue dev server
- `make test`       # unit/integration tests
- `make migrate`    # apply DB migrations

(Define these in the repo; keep targets stable for CI.)

## CI Expectations

- Format + lint (Rust + JS)
- Unit tests
- Integration tests for:
  - Postgres connector (happy path + timeout + cancel best-effort)
  - runner claiming/executing/persisting
- Migration checks (can apply cleanly from empty DB)

## Glossary

- **BI**: Business Intelligence; queries + models + visualizations + dashboards + scheduling.
- **Connector**: datasource adapter; v1: Postgres.
- **Run**: execution instance tracked as durable state.
- **Result**: persisted output with retention and size limits.
