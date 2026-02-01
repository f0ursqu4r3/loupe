# Backend TODO - Loupe

## Overview

Track backend improvements for security, performance, and maintainability.

**Started:** 2026-01-11
**Last Updated:** 2026-02-01

---

## ✅ Completed (21/48 tasks - 43.8%)

### Critical Security (5/5 - 100%)

1. ✅ **Input Validation & Sanitization** - Comprehensive validation with validator crate, 22 unit tests
2. ✅ **SQL Injection Prevention** - Parameterized queries, SQL parser validation, dangerous function blocking
3. ✅ **Authentication & Authorization** - JWT auth, Argon2 password hashing, full RBAC implementation
4. ✅ **Error Handling & Information Disclosure** - Structured errors, sanitized messages, proper status codes
5. ✅ **Database Connection Security** - SSL/TLS, connection pooling, health checks, timeout configuration

### API Design (4/4 - 100%)

1. ✅ **REST API Standards** - Versioned endpoints (/api/v1/), standardized DTOs, HTTP cache headers
2. ✅ **Request/Response Validation** - Field-level validation, custom validators, date range checks
3. ✅ **Pagination Implementation** - Consistent across all 7 endpoints, metadata (has_next/prev, total_pages)
4. ✅ **Filtering & Sorting** - Text search, tag filtering, multi-column sorting, whitelist validation

### Database (1/4 - 25%)

1. ✅ **Schema Review** - 47 indexes/constraints added: sorting (27), filtering (4), composite (3), partial (2), CHECK (11)

### Performance (5/5 - 100%)

1. ✅ **Rate Limiting** - Global 100 req/min per IP via actix-governor
2. ✅ **Connection Pooling** - SQLx pool monitoring with Prometheus metrics (active/idle/max connections, acquisition duration/timeouts), comprehensive documentation in [DATABASE_POOLING.md](DATABASE_POOLING.md)
3. ✅ **Query Execution Safety** - Prometheus metrics for query execution (duration, status, rows returned), concurrent query limiter with per-org (5) and global (50) limits configurable via environment, slow query logging (1s threshold), query cancellation API endpoint (POST /api/v1/runs/{id}/cancel)
4. ✅ **Background Job Processing** - Graceful shutdown with SIGTERM/SIGINT handlers (30s timeout), proper task tracking with JoinSet, retry logic with exponential backoff (30s base, 3 max retries, retryable error detection), dead letter queue for permanently failed jobs, job processing metrics (claims, queue depths, processing duration)
5. ✅ **Caching Strategy** - Redis-based distributed caching with async support, dashboard GET endpoint caching with automatic invalidation on updates/deletes, configurable TTL (default 5min), standardized cache key management, cache hit/miss metrics in Prometheus, graceful degradation when Redis unavailable

### Observability (5/5 - 100%)

1. ✅ **Structured Logging** - JSON format support, correlation IDs, request duration tracking, auth event logging
2. ✅ **Metrics & Monitoring** - Prometheus metrics, request count/duration histograms, /metrics endpoint
3. ✅ **Health Checks** - Liveness/readiness probes, database connectivity, migration status checks
4. ✅ **Error Tracking** - Sentry integration, automatic error capture, correlation IDs, environment tagging
5. ✅ **Distributed Tracing** - OpenTelemetry/OTLP integration, HTTP request tracing, database query spans, Jaeger/Zipkin compatible

---

## 📋 Remaining Tasks (28/48)

### Testing & Quality (0/4)

#### 10. Unit Test Coverage

- [ ] Audit existing tests in [tests/](../be/tests/)
- [ ] Add tests for all model methods
- [ ] Add tests for auth logic and connectors
- [ ] Target 80%+ code coverage with tarpaulin
- [ ] Run tests in CI/CD

**Current test files:** api_tests.rs, connector_tests.rs, db_tests.rs

#### 11. Integration Tests

- [ ] Test complete API workflows (auth → dashboard → query → visualization)
- [ ] Test schedule → run workflow
- [ ] Test error scenarios and concurrent requests
- [ ] Use testcontainers for isolation
- [ ] Test database migrations and rollback

#### 12. Load & Performance Testing

- [ ] Set up load testing framework (k6, wrk)
- [ ] Test API endpoint performance and database query performance
- [ ] Test connection pool under load
- [ ] Identify bottlenecks and document performance benchmarks

#### 13. Property-Based Testing

- [ ] Add proptest for models
- [ ] Test invariants hold (serialization/deserialization, validation rules)
- [ ] Find edge cases automatically
- [ ] Add fuzzing for critical paths

---

### Database & Migrations (3/4)

#### 15. Migration Best Practices

- [ ] Add migration testing in CI
- [ ] Document breaking changes
- [ ] Test migrations on production-size data
- [ ] Add migration versioning strategy
- [ ] Document rollback procedures

#### 16. Query Optimization

- [ ] Identify N+1 query problems
- [ ] Add eager loading where needed
- [ ] Review slow query logs
- [ ] Use EXPLAIN ANALYZE for complex queries
- [ ] Add query result caching
- [ ] Monitor query performance in production

#### 17. Data Integrity

- [ ] Add unique constraints where missing
- [ ] Handle orphaned records
- [ ] Implement soft deletes (optional)
- [ ] Add data consistency checks
- [ ] Test referential integrity

---

### Performance & Scalability (0/5 - 0%)

All tasks in this category are now complete!

---

### Observability & Operations (5/5 - 100%)

All tasks in this category are now complete!

---

### Code Organization & Architecture (0/4)

#### 28. Module Structure

- [ ] Separate domain logic from API
- [ ] Create service layer and repository layer
- [ ] Implement dependency injection
- [ ] Add trait-based abstractions
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

#### 29. Error Handling Patterns

- [ ] Review [common/error.rs](../be/src/common/error.rs)
- [ ] Use thiserror consistently
- [ ] Add error context with anyhow
- [ ] Create domain-specific error types
- [ ] Document error handling patterns
- [ ] Distinguish retriable vs non-retriable errors

#### 30. Configuration Management

- [ ] Review [common/config.rs](../be/src/common/config.rs)
- [ ] Use typed configuration with validation on startup
- [ ] Support multiple environments
- [ ] Add config file support (YAML)
- [ ] Document all config options
- [ ] Support config hot-reload (where safe)

#### 31. Async Patterns

- [ ] Review tokio runtime configuration
- [ ] Use async-trait consistently
- [ ] Avoid blocking in async contexts
- [ ] Handle cancellation properly
- [ ] Add timeout handling
- [ ] Document async patterns

---

### Security Hardening (1/5)

#### 32. CORS Configuration ✅

- [x] Review actix-cors settings
- [x] Restrict allowed origins
- [x] Add environment-based CORS config
- [x] Test CORS preflight requests
- [x] Document CORS policy

#### 33. Security Headers

- [ ] Add Content-Security-Policy
- [ ] Add X-Frame-Options
- [ ] Add X-Content-Type-Options
- [ ] Add Strict-Transport-Security (HSTS)
- [ ] Add X-XSS-Protection
- [ ] Add Referrer-Policy and Permissions-Policy

#### 34. Secrets Management

- [ ] Support secrets from files (Docker secrets)
- [ ] Support secrets from vault
- [ ] Never log secrets (audit logs)
- [ ] Add secrets rotation support
- [ ] Add secrets scanning in CI

#### 35. Dependency Security

- [ ] Run cargo audit regularly
- [ ] Add dependabot/renovate
- [ ] Review security advisories
- [ ] Pin dependency versions
- [ ] Document dependency policy

#### 36. Data Encryption

- [ ] Encrypt sensitive data at rest
- [ ] Add field-level encryption for sensitive fields
- [ ] Add encryption key management
- [ ] Document encryption strategy
- [ ] Add data masking in logs

---

### Documentation (0/3)

#### 37. OpenAPI/Swagger Spec

- [ ] Generate OpenAPI specification
- [ ] Document all endpoints with request/response schemas
- [ ] Add authentication and error response documentation
- [ ] Host interactive API docs (Swagger UI)
- [ ] Version API documentation

#### 38. Code Documentation

- [ ] Add rustdoc comments to public APIs
- [ ] Document modules with //!
- [ ] Add usage examples
- [ ] Document error conditions and panics
- [ ] Generate and publish docs

#### 39. Developer Onboarding

- [ ] Create comprehensive README
- [ ] Add setup and troubleshooting instructions
- [ ] Document testing procedures
- [ ] Add contribution guidelines
- [ ] Create development runbook
- [ ] Add architecture diagrams

---

### DevOps & Deployment (0/4)

#### 40. Containerization

- [ ] Create optimized Dockerfile with multi-stage builds
- [ ] Add docker-compose for local dev
- [ ] Minimize image size
- [ ] Add health checks in container
- [ ] Add container security scanning

#### 41. CI/CD Pipeline

- [ ] Set up GitHub Actions / GitLab CI
- [ ] Run tests, lints (clippy), security audit on every commit
- [ ] Run code formatting check (rustfmt)
- [ ] Build and push Docker images
- [ ] Add deployment automation and rollback procedures

#### 42. Environment Configuration

- [ ] Define dev/staging/prod environments
- [ ] Add environment-specific configs
- [ ] Support 12-factor app principles
- [ ] Add configuration templates
- [ ] Document deployment process

#### 43. Database Migrations in Production

- [ ] Test migrations on production-like data
- [ ] Add migration rollback plan
- [ ] Implement zero-downtime migrations
- [ ] Document migration procedures
- [ ] Add backup before migration

---

### Data Management (0/3)

#### 44. Backup & Recovery

- [ ] Implement database backup strategy
- [ ] Add automated backups
- [ ] Test backup restoration
- [ ] Document recovery procedures
- [ ] Document RTO/RPO

#### 45. Data Retention & Cleanup

- [ ] Define data retention policies
- [ ] Add old run cleanup job
- [ ] Add query result archival
- [ ] Implement soft delete
- [ ] Monitor database growth

#### 46. Query Result Storage

- [ ] Design result storage strategy
- [ ] Add result compression and expiration
- [ ] Handle large result sets
- [ ] Add result export formats
- [ ] Monitor storage usage

---

### Multi-Tenancy (Partial)

#### 47. Tenant Isolation

- [ ] Design tenant architecture
- [ ] Add tenant_id to all tables (already present as org_id)
- [ ] Add row-level security
- [ ] Add tenant creation workflow
- [ ] Add tenant limits/quotas
- [ ] Test cross-tenant access prevention

#### 48. Organization Management

- [x] Add organization model (exists in database)
- [x] Add role-based permissions (RBAC implemented)
- [ ] Add organization management API (create, update, settings)
- [x] Add team/user management API (list users, update roles, remove users)
- [ ] Add invitation system
- [ ] Add usage tracking per org

**Status:** Partially complete - RBAC enforced, user management implemented, org settings pending

---

## Progress Summary

**Critical Security:** 5/5 (100%) ✅
**API Design:** 4/4 (100%) ✅
**Testing:** 0/4 (0%)
**Database:** 1/4 (25%)
**Performance:** 5/5 (100%) ✅
**Observability:** 5/5 (100%) ✅
**Code Organization:** 0/4 (0%)
**Security Hardening:** 1/5 (20%)
**Documentation:** 0/3 (0%)
**DevOps:** 0/4 (0%)
**Data Management:** 0/3 (0%)

**Overall Progress:** 21/48 major tasks (43.8%)

---

## Notes

- Consider adding API versioning strategy documentation
- GraphQL for complex queries (future consideration)
- Database credential rotation (future enhancement)

---

## Related Documents

- [Frontend TODO](./FE_TODO.md)
- [Product Brief](./BRIEF.md)
- [RBAC Implementation](./RBAC_IMPLEMENTATION.md)
- [API Documentation](./API.md) (to be created)
- [Deployment Guide](./DEPLOYMENT.md) (to be created)
