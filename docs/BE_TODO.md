# Backend Architecture & Quality Improvements

This document tracks backend improvements to make Loupe more robust, secure, and maintainable.

## Status Legend

- ⚡ Quick win (< 2 hours)
- 🎯 High impact
- 🔧 Medium effort
- 🔒 Security
- 🧪 Testing
- 🚀 Performance
- 📊 Observability

---

## Critical Security & Validation 🔒🎯

These are high-priority security and data integrity improvements.

### 1. Input Validation & Sanitization ✅

- [x] Audit all route handlers for input validation
- [x] Validate email format in auth endpoints
- [x] Validate password strength (min 8 chars)
- [x] Validate name length (1-255 chars)
- [x] Sanitize user-provided SQL in queries (via SQL parser)
- [x] Add comprehensive request validation middleware (validator crate)
- [x] Add length limits on all string inputs
- [x] Validate datasource connection strings
- [x] Add custom validators for common patterns
- [x] Add JSON schema validation for complex payloads
- [x] Test with malformed/malicious inputs

**Status:** ✅ **COMPLETE** - Comprehensive validation implemented across all endpoints with validator crate

**Implementation Details:**

- Created [validation.rs](../be/src/common/validation.rs) with custom validators
- Added validation attributes to all request DTOs:
  - Auth: Email format, password strength (8-128 chars), name length
  - Datasources: Name validation, connection string validation (scheme check, length limits, injection pattern detection)
  - Queries: Name, description, SQL length limits, parameter/tag array limits
  - Dashboards: Name, description, tags, tile positioning and sizing
  - Visualizations: Name, tags
  - Schedules: Name, cron expression validation, tags
  - Canvases: Name, time preset, node/edge validation
- Custom validators implemented:
  - `validate_connection_string`: Checks for protocol scheme, length limits, SQL injection patterns
  - `validate_cron_expression`: Validates cron syntax using cron crate
  - `validate_name`: Alphanumeric + allowed punctuation, 1-255 chars
  - `validate_description`: Max 2000 chars
  - `validate_sql_length`: 1-100,000 chars
- All create/update endpoints now call `validate_request()` before processing
- 12 unit tests for validation logic, all passing

### 2. SQL Injection Prevention ✅

- [x] Audit all raw SQL queries for injection risks
- [x] Use parameterized queries everywhere
- [x] Review dynamic query building in connector modules
- [x] Add SQL parser and validator ([sql_validator.rs](../be/src/common/sql_validator.rs))
- [x] Block non-SELECT statements (INSERT, UPDATE, DELETE, DROP, etc.)
- [x] Block dangerous functions (pg_read_file, dblink, etc.)
- [x] Test with SQL injection payloads
- [x] Document safe query patterns
- [x] Review [connectors/postgres.rs](../be/src/common/connectors/postgres.rs) for injection risks

**Status:** ✅ **COMPLETE** - SQL validation implemented with sqlparser-rs

### 3. Authentication & Authorization ✅

- [x] Review password hashing in [routes/auth.rs](../be/src/api/routes/auth.rs) - Using Argon2
- [x] Add JWT token generation and validation ([jwt.rs](../be/src/common/jwt.rs))
- [x] Add token refresh mechanism (POST /auth/refresh)
- [x] Add token expiration (24h configurable)
- [x] Add cryptographic signing with secret key
- [x] Add email/password validation
- [x] Add audit logging for auth events
- [x] Prevent unauthorized access to other users' data (org_id checks)
- [x] Add rate limiting on all endpoints (100 req/min global)
- [x] Implement role-based access control (RBAC) - Enforced on all core routes
- [ ] Add account lockout after failed attempts
- [ ] Implement secure password reset flow
- [ ] Implement proper session management/revocation

**Status:** ✅ **MOSTLY COMPLETE** - JWT auth secure with RBAC, advanced features pending

### 4. Error Handling & Information Disclosure ✅

- [x] Review error types in [common/error.rs](../be/src/common/error.rs)
- [x] Prevent leaking sensitive info in error messages
- [x] Add structured error responses (JSON with error_id)
- [x] Log detailed errors server-side only
- [x] Return generic errors to clients for server errors
- [x] Add error IDs for correlation/debugging
- [x] Implement proper 4xx vs 5xx status codes
- [x] Add context to errors without exposing internals
- [x] Test error responses don't leak paths/queries

**Status:** ✅ **COMPLETE** - Error disclosure fixed, logging implemented

### 5. Database Connection Security ✅

- [x] Review connection string handling
- [x] Ensure SSL/TLS for database connections
- [x] Implement connection pooling limits
- [x] Add connection timeout configuration
- [x] Validate DATABASE_URL format
- [x] Prevent connection string exposure in logs
- [ ] Add database credential rotation support (future enhancement)
- [x] Review SQLx pool configuration
- [x] Add connection health checks

**Status:** ✅ **COMPLETE** - Production-ready database connection security implemented

**Implementation Details:**

- Created `DatabaseConfig` struct with comprehensive connection settings:
  - Min/max connections: Configurable via `DB_MIN_CONNECTIONS` (default: 2) and `DB_MAX_CONNECTIONS` (default: 10)
  - Connection timeout: 10 seconds
  - Idle timeout: 10 minutes (closes idle connections to free resources)
  - Max lifetime: 30 minutes (prevents long-lived connection issues)
  - Acquire timeout: 5 seconds (prevents deadlocks)
  - Test before acquire: Enabled (validates connections before use)
- SSL/TLS Configuration:
  - Development: `PgSslMode::Prefer` (use SSL if available, fallback to plain)
  - Production: `PgSslMode::Require` (require SSL, fail if unavailable)
  - Configurable via `DB_SSL_MODE` environment variable (disable, allow, prefer, require, verify-ca, verify-full)
- DATABASE_URL Validation:
  - Validates format on startup (must start with postgres:// or postgresql://)
  - Checks minimum length requirements
  - Prevents empty or malformed URLs
- Security improvements:
  - Connection errors don't expose DATABASE_URL in logs
  - Sanitized error messages for client responses
  - Detailed errors logged server-side only
- Health check endpoint enhanced:
  - `/health` now tests actual database connectivity
  - Returns 503 Service Unavailable if database is down
  - Returns connection status in JSON response
- Environment variable configuration:
  - `APP_ENV=production` automatically enables stricter SSL requirements
  - All pool settings configurable via environment variables

**Files Modified:**

- [db/mod.rs](../be/src/common/db/mod.rs) - Enhanced connection logic and configuration
- [health.rs](../be/src/api/routes/health.rs) - Added database connectivity check

---

## API Design & Consistency 🎯

### 6. REST API Standards

- [ ] Audit all endpoints for REST conventions
- [ ] Standardize response formats
- [ ] Add consistent pagination pattern
- [ ] Add consistent filtering/sorting pattern
- [ ] Implement HATEOAS links (optional)
- [ ] Version API endpoints (e.g., `/api/v1/`)
- [ ] Document API with OpenAPI/Swagger
- [ ] Add request/response examples
- [ ] Implement consistent error response format
- [ ] Add HTTP caching headers where appropriate

**Current routes to audit:**

- [auth.rs](../be/src/api/routes/auth.rs)
- [canvases.rs](../be/src/api/routes/canvases.rs)
- [dashboards.rs](../be/src/api/routes/dashboards.rs)
- [datasources.rs](../be/src/api/routes/datasources.rs)
- [queries.rs](../be/src/api/routes/queries.rs)
- [runs.rs](../be/src/api/routes/runs.rs)
- [schedules.rs](../be/src/api/routes/schedules.rs)
- [visualizations.rs](../be/src/api/routes/visualizations.rs)

### 7. Request/Response Validation

- [ ] Add request DTOs for all endpoints
- [ ] Add response DTOs for all endpoints
- [ ] Implement serde validation rules
- [ ] Add custom validators for business logic
- [ ] Validate date ranges
- [ ] Validate cron expressions in schedules
- [ ] Add field-level validation messages
- [ ] Test validation edge cases
- [ ] Document validation rules

### 8. Pagination Implementation ✅

- [x] Design consistent pagination pattern
- [x] Add `limit` and `offset` parameters
- [ ] Add `cursor`-based pagination (optional, future enhancement)
- [x] Return total count in responses
- [x] Add pagination metadata (page, per_page, total, has_next, has_prev, total_pages)
- [x] Implement in dashboard list endpoint
- [x] Implement in query list endpoint
- [x] Implement in run history endpoint
- [x] Implement in visualization list endpoint
- [x] Implement in schedule list endpoint
- [x] Implement in canvas list endpoint
- [x] Implement in datasource list endpoint
- [x] Implement in organization users list endpoint
- [x] Add default and max page sizes (DEFAULT_PAGE_SIZE=20, MAX_PAGE_SIZE=100)
- [x] Update frontend API clients and views
- [x] Document pagination in API docs

**Status:** ✅ **COMPLETE** - Comprehensive pagination implemented across all list endpoints

**Implementation Details:**

- Created [pagination.rs](../be/src/common/pagination.rs) module with reusable types:
  - `PaginationParams`: Query parameters with limit (default: 20), offset (default: 0)
  - `PaginatedResponse<T>`: Generic wrapper with items, total, page, per_page, total_pages, has_next, has_prev
  - Constants: DEFAULT_PAGE_SIZE (20), MAX_PAGE_SIZE (100)
  - Automatic validation and clamping of limit and offset parameters
  - Computed page number from offset/limit for easier navigation
- Database layer enhancements ([db/mod.rs](../be/src/common/db/mod.rs)):
  - Added 7 paginated methods: `list_*_paginated(org_id, limit, offset)`
  - Each method returns `(Vec<T>, i64)` - items and total count
  - Uses SQL LIMIT and OFFSET for efficient pagination
  - Separate COUNT query for total count
- Route handlers updated (8 endpoints):
  - [dashboards.rs](../be/src/api/routes/dashboards.rs) - GET /dashboards
  - [queries.rs](../be/src/api/routes/queries.rs) - GET /queries
  - [runs.rs](../be/src/api/routes/runs.rs) - GET /runs (preserves query_id filter)
  - [visualizations.rs](../be/src/api/routes/visualizations.rs) - GET /visualizations (preserves query_id filter)
  - [schedules.rs](../be/src/api/routes/schedules.rs) - GET /schedules
  - [canvases.rs](../be/src/api/routes/canvases.rs) - GET /canvases
  - [datasources.rs](../be/src/api/routes/datasources.rs) - GET /datasources
  - [organizations.rs](../be/src/api/routes/organizations.rs) - GET /organizations/users
- Frontend updates:
  - Created [pagination.ts](../fe/src/types/pagination.ts) with TypeScript types
  - Updated 7 API clients to return `PaginatedResponse<T>` and accept `PaginationParams`
  - Updated 11 Vue views to use `response.items` from paginated responses
- Testing:
  - 6 unit tests for pagination module (defaults, validation, page calculation, metadata)
  - All tests passing
  - Backend compilation successful

**Files Modified:**

- Backend:
  - [pagination.rs](../be/src/common/pagination.rs) - New pagination module
  - [db/mod.rs](../be/src/common/db/mod.rs) - Added 7 paginated database methods
  - [mod.rs](../be/src/common/mod.rs) - Export pagination types
  - 8 route handler files - Updated to use pagination
- Frontend:
  - [pagination.ts](../fe/src/types/pagination.ts) - New TypeScript types
  - [index.ts](../fe/src/types/index.ts) - Export pagination types
  - 7 API client files - Updated to handle PaginatedResponse
  - 11 Vue view files - Updated to use .items from responses

### 9. Filtering & Sorting ✅

- [x] Design query parameter schema
- [x] Add filter by created_at, updated_at (date range for runs)
- [x] Add filter by status (for runs)
- [x] Add filter by datasource_id, query_id, enabled
- [x] Add sort by multiple fields (name, created_at, updated_at, started_at, completed_at, next_run_at)
- [x] Add search/text filtering (ILIKE searches with pg_trgm indexes)
- [x] Validate filter parameters (whitelist-based column validation)
- [x] Prevent SQL injection in filters (parameterized queries, validated column names)
- [x] Add filter combination logic (conditional SQL branching)
- [x] Add tag filtering (JSONB containment)
- [x] Create database migration for text search indexes (pg_trgm GIN indexes)
- [x] Update frontend API clients with new filter types

**Status:** ✅ **COMPLETE** - Comprehensive filtering and sorting implemented across all 7 endpoints

**Implementation Details:**

- Created [filtering.rs](../be/src/common/filtering.rs) module with reusable types:
  - `SortParams`: Validates sort column against whitelist, supports asc/desc direction
  - `SearchParams`: Sanitizes search terms, adds ILIKE pattern wrapping, length limits
  - `DateRangeParams`: Optional start_date and end_date for time-based filtering
  - `parse_tags()`: Parses comma-separated tags, enforces length/count limits
  - `SortableColumns`: Whitelists for each endpoint preventing SQL injection
- Database layer updates ([db/mod.rs](../be/src/common/db/mod.rs)):
  - Updated 7 `list_*_paginated()` functions with filtering and sorting parameters
  - Uses conditional SQL pattern matching to build queries safely
  - All user input bound via `.bind()`, never string interpolation
  - Column names validated against whitelists before use in ORDER BY clauses
- Route handlers updated (7 endpoints):
  - [dashboards.rs](../be/src/api/routes/dashboards.rs) - Search: name/description, Tags filter, Sort: name/created_at/updated_at
  - [queries.rs](../be/src/api/routes/queries.rs) - Search: name/description/sql, Datasource filter, Tags filter, Sort: name/created_at/updated_at
  - [runs.rs](../be/src/api/routes/runs.rs) - Query ID filter, Status enum filter, Date range filter, Sort: created_at/started_at/completed_at
  - [visualizations.rs](../be/src/api/routes/visualizations.rs) - Search: name, Query ID filter, Tags filter, Sort: name/created_at/updated_at
  - [schedules.rs](../be/src/api/routes/schedules.rs) - Search: name, Tags filter, Enabled boolean filter, Sort: name/next_run_at/created_at/updated_at
  - [datasources.rs](../be/src/api/routes/datasources.rs) - Search: name, Sort: name/created_at/updated_at
  - [canvases.rs](../be/src/api/routes/canvases.rs) - Search: name, Tags filter, Sort: name/created_at/updated_at
- Database migration ([20260131000000_add_text_search_indexes.up.sql](../be/migrations/20260131000000_add_text_search_indexes.up.sql)):
  - Enabled pg_trgm extension for trigram similarity searches
  - Created GIN indexes on name columns for all 7 entities
  - Created GIN indexes on queries.description and queries.sql
  - Indexes provide fast ILIKE searches without full table scans
- Frontend updates:
  - Added filter types to [api.ts](../fe/src/types/api.ts): DashboardFilterParams, QueryFilterParams, RunFilterParams, VisualizationFilterParams, ScheduleFilterParams, DatasourceFilterParams, CanvasFilterParams
  - Updated 6 API clients to accept new filter parameters
  - Added index signatures to filter types for API client compatibility
- Security features:
  - Column name whitelist validation prevents ORDER BY injection
  - All filter values bound via parameterized queries
  - Search terms length-limited to 200 characters
  - Tags limited to 10 tags, 50 chars each
  - Status enum validated against allowed values
- Testing:
  - 16 unit tests for filtering module (SQL injection prevention, validation, sanitization)
  - Backend compilation successful
  - Frontend type-checking successful

**Files Created:**

- [filtering.rs](../be/src/common/filtering.rs) - Core filtering module
- [20260131000000_add_text_search_indexes.up.sql](../be/migrations/20260131000000_add_text_search_indexes.up.sql) - Database indexes migration
- [20260131000000_add_text_search_indexes.down.sql](../be/migrations/20260131000000_add_text_search_indexes.down.sql) - Migration rollback

**Files Modified:**

- Backend: [mod.rs](../be/src/common/mod.rs), [db/mod.rs](../be/src/common/db/mod.rs), 7 route handler files
- Frontend: [api.ts](../fe/src/types/api.ts), [pagination.ts](../fe/src/types/pagination.ts), 6 API client files, [QueriesView.vue](../fe/src/views/QueriesView.vue)

---

## Testing & Quality 🧪

### 10. Unit Test Coverage

- [ ] Audit existing tests in [tests/](../be/tests/)
- [ ] Add tests for all model methods
- [ ] Add tests for validation logic
- [ ] Add tests for error handling
- [ ] Add tests for auth logic
- [ ] Add tests for connectors
- [ ] Target 80%+ code coverage
- [ ] Add coverage reporting (tarpaulin)
- [ ] Run tests in CI/CD
- [ ] Add code coverage badges

**Current test files:**

- [api_tests.rs](../be/tests/api_tests.rs)
- [connector_tests.rs](../be/tests/connector_tests.rs)
- [db_tests.rs](../be/tests/db_tests.rs)

### 11. Integration Tests

- [ ] Test complete API workflows
- [ ] Test auth flow end-to-end
- [ ] Test dashboard creation → query → visualization
- [ ] Test schedule → run workflow
- [ ] Test error scenarios
- [ ] Test concurrent requests
- [ ] Use testcontainers for isolation
- [ ] Add API client tests
- [ ] Test database migrations
- [ ] Test rollback scenarios

### 12. Load & Performance Testing

- [ ] Set up load testing framework (k6, wrk)
- [ ] Test API endpoint performance
- [ ] Test database query performance
- [ ] Test connection pool under load
- [ ] Identify bottlenecks
- [ ] Test runner throughput
- [ ] Test scheduler performance
- [ ] Document performance benchmarks
- [ ] Set performance budgets

### 13. Property-Based Testing

- [ ] Add proptest for models
- [ ] Generate random valid inputs
- [ ] Test invariants hold
- [ ] Test serialization/deserialization
- [ ] Test validation rules
- [ ] Find edge cases automatically
- [ ] Add fuzzing for critical paths

---

## Database & Migrations 🔧

### 14. Schema Review

- [ ] Review initial migration [20260111000000_initial.up.sql](../be/migrations/20260111000000_initial.up.sql)
- [ ] Review canvas migration [20260128000000_canvases.up.sql](../be/migrations/20260128000000_canvases.up.sql)
- [ ] Add missing indexes for common queries
- [ ] Add foreign key constraints
- [ ] Add CHECK constraints for validation
- [ ] Add NOT NULL where appropriate
- [ ] Review column types for efficiency
- [ ] Add database-level defaults
- [ ] Document schema design decisions

### 15. Migration Best Practices

- [ ] Test all migrations forward and backward
- [ ] Add migration testing in CI
- [ ] Document breaking changes
- [ ] Add data migration scripts where needed
- [ ] Test migrations on production-size data
- [ ] Add migration versioning strategy
- [ ] Handle concurrent migrations
- [ ] Add migration rollback procedures
- [ ] Document manual intervention steps

### 16. Query Optimization

- [ ] Identify N+1 query problems
- [ ] Add eager loading where needed
- [ ] Review slow query logs
- [ ] Add database indexes strategically
- [ ] Use EXPLAIN ANALYZE for complex queries
- [ ] Add query result caching
- [ ] Optimize JOIN queries
- [ ] Add partial indexes where applicable
- [ ] Monitor query performance in production

### 17. Data Integrity

- [ ] Add database constraints
- [ ] Add unique constraints
- [ ] Add cascade delete rules
- [ ] Handle orphaned records
- [ ] Add data validation at DB level
- [ ] Implement soft deletes (optional)
- [ ] Add audit columns (created_at, updated_at)
- [ ] Add data consistency checks
- [ ] Test referential integrity

---

## Performance & Scalability 🚀

### 18. Connection Pooling

- [ ] Review SQLx pool configuration
- [ ] Set appropriate pool size limits
- [ ] Add connection timeout handling
- [ ] Monitor pool utilization
- [ ] Add pool metrics
- [ ] Handle pool exhaustion gracefully
- [ ] Test under high concurrency
- [ ] Document pool tuning guidelines

### 19. Caching Strategy

- [ ] Identify cacheable endpoints
- [ ] Add Redis/in-memory cache
- [ ] Cache dashboard metadata
- [ ] Cache query results (with TTL)
- [ ] Add cache invalidation strategy
- [ ] Add cache headers (ETags, Last-Modified)
- [ ] Implement cache warming
- [ ] Monitor cache hit rates
- [ ] Document caching policies

### 20. Background Job Processing

- [ ] Review runner architecture [runner/main.rs](../be/src/runner/main.rs)
- [ ] Add job queue (consider sidekiq-style system)
- [ ] Add job retry logic with backoff
- [ ] Add job timeout handling
- [ ] Add dead letter queue
- [ ] Monitor job processing metrics
- [ ] Add job priority levels
- [ ] Implement graceful shutdown
- [ ] Add job cancellation support
- [ ] Document job lifecycle

### 21. Rate Limiting ✅

- [x] Add rate limiting middleware (actix-governor)
- [x] Rate limit by IP address (PeerIpKeyExtractor)
- [x] Return proper 429 status codes (automatic)
- [x] Add Retry-After headers (automatic)
- [x] Implement token bucket algorithm (via actix-governor)
- [x] Global rate limit: 100 requests/minute per IP
- [ ] Add different limits per endpoint
- [ ] Rate limit by user/API key
- [ ] Add rate limit bypass for internal services
- [ ] Document rate limits in API docs
- [ ] Monitor rate limit violations

**Status:** ✅ **COMPLETE** - Global rate limiting implemented, endpoint-specific limits pending

### 22. Query Execution Safety

- [ ] Add query timeout limits
- [ ] Add query result size limits
- [ ] Prevent runaway queries
- [ ] Add query cost estimation
- [ ] Implement query queue
- [ ] Add concurrent query limits per user
- [ ] Add query cancellation support
- [ ] Log slow queries
- [ ] Add query execution monitoring

---

## Observability & Operations 📊

### 23. Structured Logging

- [ ] Review tracing configuration
- [ ] Add structured log format (JSON)
- [ ] Add correlation IDs to requests
- [ ] Log all API requests
- [ ] Log authentication events
- [ ] Log database queries (debug mode)
- [ ] Add log levels appropriately
- [ ] Remove sensitive data from logs
- [ ] Add request duration logging
- [ ] Configure log rotation

**Current logging:** Uses tracing crate, needs enhancement

### 24. Metrics & Monitoring

- [ ] Add Prometheus metrics
- [ ] Track request count by endpoint
- [ ] Track request duration (p50, p95, p99)
- [ ] Track error rates
- [ ] Track database query metrics
- [ ] Track connection pool metrics
- [ ] Track job queue length
- [ ] Track runner/scheduler health
- [ ] Add custom business metrics
- [ ] Create Grafana dashboards

### 25. Health Checks

- [ ] Enhance [routes/health.rs](../be/src/api/routes/health.rs)
- [ ] Add database connectivity check
- [ ] Add dependency health checks
- [ ] Add liveness endpoint
- [ ] Add readiness endpoint
- [ ] Add startup probe
- [ ] Check critical services
- [ ] Return detailed health status
- [ ] Add health check monitoring

### 26. Distributed Tracing

- [ ] Add OpenTelemetry support
- [ ] Trace requests across services
- [ ] Trace database queries
- [ ] Add span attributes
- [ ] Track error traces
- [ ] Set up trace sampling
- [ ] Integrate with Jaeger/Zipkin
- [ ] Add trace context propagation
- [ ] Document tracing setup

### 27. Error Tracking

- [ ] Integrate Sentry or similar
- [ ] Track unhandled errors
- [ ] Add error context (user, request)
- [ ] Set up error alerts
- [ ] Add error fingerprinting
- [ ] Track error trends
- [ ] Add source maps for debugging
- [ ] Configure error sampling
- [ ] Set up error notifications

---

## Code Organization & Architecture 🔧

### 28. Module Structure

- [ ] Review module organization
- [ ] Separate domain logic from API
- [ ] Create service layer
- [ ] Create repository layer
- [ ] Implement dependency injection
- [ ] Add trait-based abstractions
- [ ] Separate read/write models (CQRS-lite)
- [ ] Document architecture patterns
- [ ] Add architecture decision records (ADRs)

**Current structure:**

```
be/src/
├── api/          # API server and routes
├── common/       # Shared models, DB, config
├── runner/       # Job execution service
└── scheduler/    # Job scheduling service
```

### 29. Error Handling Patterns

- [ ] Review [common/error.rs](../be/src/common/error.rs)
- [ ] Use thiserror consistently
- [ ] Add error context with anyhow
- [ ] Create domain-specific error types
- [ ] Add error conversion traits
- [ ] Document error handling patterns
- [ ] Add error recovery strategies
- [ ] Distinguish retriable vs non-retriable errors

### 30. Configuration Management

- [ ] Review [common/config.rs](../be/src/common/config.rs)
- [ ] Use typed configuration
- [ ] Add config validation on startup
- [ ] Support multiple environments
- [ ] Add config file support (TOML/YAML)
- [ ] Document all config options
- [ ] Add config defaults
- [ ] Support config hot-reload (where safe)
- [ ] Add config schema

### 31. Async Patterns

- [ ] Review tokio runtime configuration
- [ ] Use async-trait consistently
- [ ] Avoid blocking in async contexts
- [ ] Add proper error propagation
- [ ] Use appropriate task spawning
- [ ] Handle cancellation properly
- [ ] Add timeout handling
- [ ] Document async patterns
- [ ] Profile async performance

---

## Security Hardening 🔒

### 32. CORS Configuration

- [ ] Review actix-cors settings
- [ ] Restrict allowed origins
- [ ] Add environment-based CORS config
- [ ] Document CORS policy
- [ ] Test CORS preflight requests
- [ ] Add credential handling
- [ ] Restrict allowed methods
- [ ] Add exposed headers

### 33. Security Headers

- [ ] Add Content-Security-Policy
- [ ] Add X-Frame-Options
- [ ] Add X-Content-Type-Options
- [ ] Add Strict-Transport-Security (HSTS)
- [ ] Add X-XSS-Protection
- [ ] Add Referrer-Policy
- [ ] Add Permissions-Policy
- [ ] Test header configuration
- [ ] Document security headers

### 34. Secrets Management

- [ ] Use environment variables for secrets
- [ ] Add secrets validation on startup
- [ ] Support secrets from files (Docker secrets)
- [ ] Support secrets from vault
- [ ] Never log secrets
- [ ] Add secrets rotation support
- [ ] Document secrets management
- [ ] Add secrets scanning in CI

### 35. Dependency Security

- [ ] Run cargo audit regularly
- [ ] Add dependabot/renovate
- [ ] Keep dependencies updated
- [ ] Review security advisories
- [ ] Pin dependency versions
- [ ] Audit transitive dependencies
- [ ] Add license checking
- [ ] Document dependency policy

### 36. Data Encryption

- [ ] Encrypt sensitive data at rest
- [ ] Encrypt connections (TLS)
- [ ] Add field-level encryption for sensitive fields
- [ ] Review password storage (argon2)
- [ ] Add encryption key management
- [ ] Document encryption strategy
- [ ] Add data masking in logs
- [ ] Implement secure key derivation

---

## API Documentation 📚

### 37. OpenAPI/Swagger Spec

- [ ] Generate OpenAPI specification
- [ ] Document all endpoints
- [ ] Add request/response schemas
- [ ] Add authentication documentation
- [ ] Add example requests/responses
- [ ] Add error response documentation
- [ ] Host interactive API docs (Swagger UI)
- [ ] Keep docs in sync with code
- [ ] Version API documentation

### 38. Code Documentation

- [ ] Add rustdoc comments to public APIs
- [ ] Document modules with //!
- [ ] Add usage examples
- [ ] Document error conditions
- [ ] Document panics
- [ ] Document safety requirements
- [ ] Generate and publish docs
- [ ] Add inline comments for complex logic

### 39. Developer Onboarding

- [ ] Create comprehensive README
- [ ] Add setup instructions
- [ ] Document local development
- [ ] Add troubleshooting guide
- [ ] Document testing procedures
- [ ] Add contribution guidelines
- [ ] Create development runbook
- [ ] Add architecture diagrams

---

## DevOps & Deployment 🚀

### 40. Containerization

- [ ] Create optimized Dockerfile
- [ ] Use multi-stage builds
- [ ] Add docker-compose for local dev
- [ ] Minimize image size
- [ ] Add health checks in container
- [ ] Document container deployment
- [ ] Add .dockerignore
- [ ] Test container startup
- [ ] Add container security scanning

### 41. CI/CD Pipeline

- [ ] Set up GitHub Actions / GitLab CI
- [ ] Run tests on every commit
- [ ] Run lints (clippy)
- [ ] Run security audit
- [ ] Run code formatting check (rustfmt)
- [ ] Build Docker images
- [ ] Push to container registry
- [ ] Add deployment automation
- [ ] Add rollback procedures

### 42. Environment Configuration

- [ ] Define dev/staging/prod environments
- [ ] Add environment-specific configs
- [ ] Document environment differences
- [ ] Add environment variable validation
- [ ] Support 12-factor app principles
- [ ] Add configuration templates
- [ ] Document deployment process

### 43. Database Migrations in Production

- [ ] Add migration strategy documentation
- [ ] Test migrations on production-like data
- [ ] Add migration rollback plan
- [ ] Implement zero-downtime migrations
- [ ] Add migration monitoring
- [ ] Document migration procedures
- [ ] Add backup before migration
- [ ] Test migration in staging

---

## Data Management 📊

### 44. Backup & Recovery

- [ ] Implement database backup strategy
- [ ] Add automated backups
- [ ] Test backup restoration
- [ ] Document recovery procedures
- [ ] Add point-in-time recovery
- [ ] Store backups securely
- [ ] Add backup monitoring
- [ ] Document RTO/RPO

### 45. Data Retention & Cleanup

- [ ] Define data retention policies
- [ ] Add old run cleanup job
- [ ] Add query result archival
- [ ] Implement soft delete
- [ ] Add data export functionality
- [ ] Document retention periods
- [ ] Add compliance considerations
- [ ] Monitor database growth

### 46. Query Result Storage

- [ ] Design result storage strategy
- [ ] Add result compression
- [ ] Add result expiration
- [ ] Implement result pagination
- [ ] Add result caching
- [ ] Handle large result sets
- [ ] Add result export formats
- [ ] Monitor storage usage

---

## Multi-Tenancy (Future) 🏗️

### 47. Tenant Isolation

- [ ] Design tenant architecture
- [ ] Add tenant_id to all tables
- [ ] Add row-level security
- [ ] Isolate tenant data
- [ ] Add tenant creation workflow
- [ ] Add tenant limits/quotas
- [ ] Test cross-tenant access prevention
- [ ] Document tenant model

### 48. Organization Management

- [x] Add organization model (exists in database)
- [x] Add role-based permissions (RBAC implemented - see [RBAC_IMPLEMENTATION.md](./RBAC_IMPLEMENTATION.md))
- [ ] Add organization management API (create, update, settings)
- [x] Add team/user management API (list users, update roles, remove users)
- [ ] Add invitation system
- [ ] Add usage tracking per org
- [ ] Add billing integration (future)

**Status:** ⚠️ **MOSTLY COMPLETE** - RBAC enforced, user management implemented, org settings pending

**Files:**

- [permissions.rs](../be/src/api/permissions.rs) - RBAC implementation
- [organizations.rs](../be/src/api/routes/organizations.rs) - User management API
- [RBAC_IMPLEMENTATION.md](./RBAC_IMPLEMENTATION.md) - Complete documentation

**Implemented Endpoints:**

- `GET /organizations/users` - List all users in organization
- `PUT /organizations/users/:id/role` - Update user role (Admin only)
- `DELETE /organizations/users/:id` - Remove user from organization (Admin only)

---

## Progress Tracking

**Started:** 2026-01-11
**Last Updated:** 2026-01-31

**Critical Security:** 5/5 (100%) ✅ - Input Validation ✅, SQL Injection ✅, Auth & RBAC ✅, Error Handling ✅, DB Connection ✅
**API Design:** 2/4 (50%) - Pagination Implementation ✅, Filtering & Sorting ✅
**Testing:** 0/4 (0%)
**Database:** 0/4 (0%)
**Performance:** 1/5 (20%) - Rate Limiting ✅
**Observability:** 0/5 (0%)
**Code Organization:** 0/4 (0%)
**Security Hardening:** 0/5 (0%)
**Documentation:** 0/3 (0%)
**DevOps:** 0/4 (0%)
**Data Management:** 0/3 (0%)

**Overall Progress:** 8/48 major tasks (16.7%)

---

## Notes

Add any notes, decisions, or discussions here:

- Consider adding API versioning before v1 release
- Evaluate using diesel vs SQLx tradeoffs
- Review auth strategy (JWT vs sessions)
- Consider GraphQL for complex queries
-
-

---

## Related Documents

- [Frontend TODO](./FE_TODO.md)
- [Product Brief](./BRIEF.md)
- [Agents & Workflows](./AGENTS.md)
- [API Documentation](./API.md) (to be created)
- [Deployment Guide](./DEPLOYMENT.md) (to be created)
