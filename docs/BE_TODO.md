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

### 1. Input Validation & Sanitization

- [ ] Audit all route handlers for input validation
- [ ] Add request validation middleware
- [ ] Validate query parameters in [routes/queries.rs](../be/src/api/routes/queries.rs)
- [ ] Validate datasource connection strings
- [ ] Add length limits on all string inputs
- [ ] Validate UUIDs before database queries
- [ ] Add custom validators for common patterns
- [ ] Sanitize user-provided SQL in queries
- [ ] Add JSON schema validation for complex payloads
- [ ] Test with malformed/malicious inputs

**Priority:** High - Prevents injection attacks and data corruption

### 2. SQL Injection Prevention

- [ ] Audit all raw SQL queries for injection risks
- [ ] Use parameterized queries everywhere
- [ ] Review dynamic query building in connector modules
- [ ] Add query builder abstraction for complex queries
- [ ] Test with SQL injection payloads
- [ ] Document safe query patterns
- [ ] Add linting rules for raw SQL
- [ ] Review [connectors/postgres.rs](../be/src/common/connectors/postgres.rs) for injection risks

**Critical:** User-provided queries must be sandboxed/validated

### 3. Authentication & Authorization

- [ ] Review password hashing in [routes/auth.rs](../be/src/api/routes/auth.rs)
- [ ] Implement proper session management
- [ ] Add JWT token generation and validation
- [ ] Add token refresh mechanism
- [ ] Implement role-based access control (RBAC)
- [ ] Add resource ownership checks
- [ ] Prevent unauthorized access to other users' data
- [ ] Add rate limiting on auth endpoints
- [ ] Add account lockout after failed attempts
- [ ] Implement secure password reset flow
- [ ] Add audit logging for auth events

**Current status:** Basic auth exists, needs hardening

### 4. Error Handling & Information Disclosure

- [ ] Review error types in [common/error.rs](../be/src/common/error.rs)
- [ ] Prevent leaking sensitive info in error messages
- [ ] Add structured error responses
- [ ] Log detailed errors server-side only
- [ ] Return generic errors to clients
- [ ] Add error codes for client-side handling
- [ ] Implement proper 4xx vs 5xx status codes
- [ ] Add context to errors without exposing internals
- [ ] Test error responses don't leak paths/queries

### 5. Database Connection Security

- [ ] Review connection string handling
- [ ] Ensure SSL/TLS for database connections
- [ ] Implement connection pooling limits
- [ ] Add connection timeout configuration
- [ ] Validate DATABASE_URL format
- [ ] Prevent connection string exposure in logs
- [ ] Add database credential rotation support
- [ ] Review SQLx pool configuration
- [ ] Add connection health checks

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

### 8. Pagination Implementation

- [ ] Design consistent pagination pattern
- [ ] Add `limit` and `offset` parameters
- [ ] Add `cursor`-based pagination (optional)
- [ ] Return total count in responses
- [ ] Add pagination metadata (page, per_page, total)
- [ ] Implement in dashboard list endpoint
- [ ] Implement in query list endpoint
- [ ] Implement in run history endpoint
- [ ] Add default and max page sizes
- [ ] Document pagination in API docs

### 9. Filtering & Sorting

- [ ] Design query parameter schema
- [ ] Add filter by created_at, updated_at
- [ ] Add filter by status (for runs)
- [ ] Add filter by user/owner
- [ ] Add sort by multiple fields
- [ ] Add search/text filtering
- [ ] Validate filter parameters
- [ ] Prevent SQL injection in filters
- [ ] Add filter combination logic (AND/OR)
- [ ] Document filter syntax

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

### 21. Rate Limiting

- [ ] Add rate limiting middleware
- [ ] Rate limit by IP address
- [ ] Rate limit by user/API key
- [ ] Add different limits per endpoint
- [ ] Return proper 429 status codes
- [ ] Add Retry-After headers
- [ ] Implement token bucket algorithm
- [ ] Add rate limit bypass for internal services
- [ ] Document rate limits in API docs
- [ ] Monitor rate limit violations

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

- [ ] Add organization model
- [ ] Add team/user management
- [ ] Add role-based permissions
- [ ] Add invitation system
- [ ] Add organization settings
- [ ] Add usage tracking per org
- [ ] Add billing integration (future)

---

## Progress Tracking

**Started:** _______________
**Last Updated:** _______________

**Critical Security:** 0/5
**API Design:** 0/4
**Testing:** 0/4
**Database:** 0/4
**Performance:** 0/5
**Observability:** 0/5
**Code Organization:** 0/4
**Security Hardening:** 0/5
**Documentation:** 0/3
**DevOps:** 0/4
**Data Management:** 0/3

**Overall Progress:** 0/48 major tasks (0%)

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
