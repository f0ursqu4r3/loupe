# Brief

## Product

A standalone, self-hosted BI tool with a Rust backend (separate API and runner binaries) and a Vue frontend. Initial release supports **Postgres only** as a query datasource.

## Goals

- Provide a full BI workflow: connect datasource → write queries → visualize → assemble dashboards → schedule refreshes.
- Separate concerns cleanly: API handles auth/metadata; runners handle execution.
- Be reliable under load: strong timeouts, concurrency controls, and deterministic run history.
- Stay extensible: connectors are a first-class abstraction from day 1.

## Non-Goals (v1)

- Multiple datasources beyond Postgres.
- Public sharing / embeds.
- Alerts (Slack/email/webhook).
- Complex permissions (per-object ACL). Start with coarse org roles.
- Pivot tables / OLAP modeling / spreadsheet-like UI.
- “Warehouse-scale” query optimization (pushdown is delegated to Postgres).

## Target Users

Internal operators and engineers running ops dashboards for:

- Trad
- ohub (Canadian outages)

## Core Concepts

- **Datasource**: a connection definition (Postgres only in v1).
- **Query**: saved SQL + parameter schema + default run options.
- **Run**: an execution instance of a query with concrete parameters and options.
- **Result**: persisted output of a run (schema + rows + stats) with retention rules.
- **Visualization**: a rendering config for a result (chart type + mapping).
- **Dashboard**: grid layout composed of tiles (viz + query + bindings).
- **Schedule**: periodic refresh rule for a query (and/or dashboard tiles).

## Architecture Overview

### Binaries

- **api** (Actix-Web)
  - Auth/session
  - Metadata CRUD (datasources, queries, dashboards, visualizations, schedules)
  - Enqueue runs
  - Serve run status + results
  - Enforce permissions

- **runner** (worker)
  - Dequeue runs
  - Execute SQL via connector
  - Enforce timeouts/limits/concurrency
  - Persist results + run status
  - Best-effort cancellation

### Primary Store

- **Postgres metadata database** (source of truth)
  - users, orgs, roles
  - datasources (secrets encrypted)
  - queries, dashboards, tiles, visualizations
  - runs and result pointers/metadata

### Results Storage

- Results must be readable by the API.
- Default approach:
  - Small/medium results: stored in Postgres (compressed blob) with TTL.
  - Large results: stored in object storage (S3/R2/MinIO) with a pointer in Postgres.
- A local SQLite cache may exist inside the runner *only as a cache*, not the system of record.

## Execution Model

- The API never executes queries inline.
- Every execution is a **Run**:
  1. API creates `run` row (status=queued)
  2. Runner claims and executes
  3. Runner writes result payload + stats and sets final status
  4. UI polls or subscribes to updates

## Safety & Limits (v1 requirements)

- Server-side timeouts (default + per-run override within max).
- Server-side max rows and max bytes.
- Concurrency caps:
  - global per runner
  - per datasource
- Parameter binding must be safe:
  - Prefer prepared statements for values
  - No string interpolation for arbitrary values

## Connector Strategy

- Implement a connector interface in the runner.
- v1 ships with **PostgresConnector** only.
- The connector boundary is designed to support future “plugins”:
  - in-process connectors initially
  - optional evolution to out-of-process plugins via RPC without changing API contracts

## Frontend (Vue) Scope

- Monaco-based SQL editor
- Schema browser (basic introspection)
- Results table (virtualized)
- Visualizations (minimum set):
  - time series line
  - bar
  - single stat
  - table
- Dashboard builder:
  - grid layout
  - tile parameter bindings
  - edit/view modes
- Schedules UI for query refresh

## Milestones (high-level)

1. Metadata schema + migrations
2. API auth + basic RBAC
3. Datasource CRUD + Postgres connection test
4. Query CRUD + typed parameters
5. Run queueing + runner execution + results persistence
6. UI: editor + run + results
7. Dashboards + tiles
8. Visualizations + config UI
9. Scheduling + refresh history
10. Hardening: limits, cancellation, observability

## Success Criteria

- Can build and maintain internal ops dashboards for Trad and ohub.
- Scheduled refresh is stable and observable.
- Run history is trustworthy (status, timing, errors, row/byte counts).
- Adding a second connector does not require redesigning core execution flow.
