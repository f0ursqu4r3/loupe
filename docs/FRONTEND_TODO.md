# Frontend TODO

## P0

[x] Fix datetime parameter handling: `datetime-local` values and defaults need coercion to RFC3339 (or backend accepts `YYYY-MM-DDTHH:mm`) to avoid validation failures and incorrect runs.

## P1

[ ] Reload editor views on route param change (`/dashboards/:id`, `/queries/:id`, `/schedules/:id`, `/visualizations/:id`) to prevent stale state when navigating between items.
[ ] Add cancellation/guarding for long-poll loops in query and dashboard views to avoid updates after unmount and reduce stray API calls.

## P2

[ ] Normalize date/time formatting helpers across views to use locale-based utilities and avoid hard-coded `en-US` formatters.
