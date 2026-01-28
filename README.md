# Loupe

Loupe is a self-hosted BI tool with a Rust backend (separate API, runner, and scheduler binaries) and a Vue frontend. v1 targets Postgres only.

## Repo layout

- `be/` Rust backend (API, runner, scheduler)
- `fe/` Vue frontend (Vite)
- `docs/` Project docs and notes

## Quick start (dev)

1) Start Postgres and set `DATABASE_URL`.

2) Run the API (auto-runs migrations on startup):

    ```bash
    cd be
    cargo run --bin loupe-api
    ```

3) Run the runner (executes queued runs):

    ```bash
    cd be
    cargo run --bin loupe-runner
    ```

4) Run the scheduler (enqueues scheduled runs):

    ```bash
    cd be
    cargo run --bin loupe-scheduler
    ```

5) Run the frontend:

    ```bash
    cd fe
    bun install
    bun run dev
    ```

Vite proxies `/api` to `http://localhost:8080` by default.

## Environment variables

Backend:

- `DATABASE_URL` (required)
- `API_HOST` (default `127.0.0.1`)
- `API_PORT` (default `8080`)
- `RUST_LOG` (default `info,sqlx=warn`)

Runner:

- `DATABASE_URL` (required)
- `RUNNER_ID` (optional; auto-generated if omitted)
- `RUST_LOG` (default `info,sqlx=warn`)

Scheduler:

- `DATABASE_URL` (required)
- `SCHEDULER_ID` (optional; auto-generated if omitted)
- `SCHEDULER_POLL_INTERVAL_SECONDS` (default `10`)
- `RUST_LOG` (default `info,sqlx=warn`)

## Notes

- Schedules only fire when the scheduler is running.
- Runs are executed by the runner; the API only queues runs.

## Docs

- `docs/BRIEF.md` product and architecture overview
- `docs/AGENTS.md` ownership and workflows
