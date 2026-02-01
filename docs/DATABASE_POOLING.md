# Database Connection Pooling

## Overview

Loupe uses [SQLx](https://github.com/launchbadge/sqlx) with PostgreSQL connection pooling to efficiently manage database connections. Proper pool configuration is critical for performance and stability.

## Configuration

Connection pool settings are configured via environment variables and the `DatabaseConfig` struct.

### Environment Variables

```bash
# Pool size
DB_MIN_CONNECTIONS=2          # Minimum connections to maintain (default: 2)
DB_MAX_CONNECTIONS=10         # Maximum connections allowed (default: 10)

# SSL/TLS
DB_SSL_MODE=prefer            # Options: disable, allow, prefer, require, verify-ca, verify-full

# Connection string
DATABASE_URL=postgresql://user:password@localhost:5432/loupe
```

### Code Configuration

```rust
use loupe::{Database, DatabaseConfig};

// Use default configuration
let db = Database::connect(&database_url).await?;

// Use production configuration (requires SSL)
let config = DatabaseConfig::production();
let db = Database::connect_with_config(&database_url, config).await?;

// Custom configuration
let config = DatabaseConfig {
    min_connections: 5,
    max_connections: 20,
    connect_timeout: Duration::from_secs(10),
    idle_timeout: Some(Duration::from_secs(600)),     // 10 minutes
    max_lifetime: Some(Duration::from_secs(1800)),    // 30 minutes
    acquire_timeout: Duration::from_secs(5),
    ssl_mode: PgSslMode::Require,
    test_before_acquire: true,
};
let db = Database::connect_with_config(&database_url, config).await?;
```

## Pool Parameters

### min_connections

**Default:** 2

Minimum number of connections to maintain in the pool. These connections are established at startup and kept alive.

**Recommendations:**
- Development: 2
- Staging: 2-5
- Production: 5-10

**Trade-offs:**
- Higher values reduce cold-start latency but consume more database resources
- Lower values save resources but may cause latency spikes under sudden load

### max_connections

**Default:** 10

Maximum number of connections the pool can create.

**Recommendations:**
- Development: 10
- Staging: 20-50
- Production: 50-100 (depends on your database server capacity and application load)

**Important Considerations:**
- PostgreSQL default max_connections is 100
- Each application instance creates its own pool
- Formula: `max_connections` < (postgres max_connections / number of app instances)
- Leave headroom for admin connections and other services

**Trade-offs:**
- Too low: Pool exhaustion, request timeouts, degraded performance
- Too high: Database overload, memory exhaustion, connection thrashing

### connect_timeout

**Default:** 10 seconds

Maximum time to wait when establishing a new database connection.

**Recommendations:**
- Development: 10s
- Production: 5-10s

**Trade-offs:**
- Higher values: More resilient to slow networks, but slower failure detection
- Lower values: Faster failure detection, but may fail on legitimate slow connections

### idle_timeout

**Default:** 10 minutes (600s)

Maximum time a connection can remain idle before being closed.

**Recommendations:**
- Development: 10-30 minutes
- Production: 5-10 minutes

**Trade-offs:**
- Higher values: Fewer reconnections, lower overhead
- Lower values: Frees up database resources faster

### max_lifetime

**Default:** 30 minutes (1800s)

Maximum lifetime of a connection before it's closed and replaced.

**Recommendations:**
- Development: 30 minutes - 1 hour
- Production: 15-30 minutes

**Purpose:**
- Prevents connection leaks
- Ensures connections pick up database configuration changes
- Distributes load across database servers in a cluster

### acquire_timeout

**Default:** 5 seconds

Maximum time to wait to acquire a connection from the pool when all connections are in use.

**Recommendations:**
- Development: 5s
- Production: 3-5s

**Trade-offs:**
- Higher values: More patient waiting, but slower request failures
- Lower values: Faster failures, but may timeout during legitimate high load

**What happens on timeout:**
- SQLx returns a `PoolTimedOut` error
- The API returns HTTP 503 Service Unavailable
- Pool metrics show acquisition timeouts

### ssl_mode

**Default:** `prefer` (development), `require` (production)

SSL/TLS connection mode.

**Options:**
- `disable`: No SSL (insecure, only for local development)
- `allow`: Try SSL, fall back to plain
- `prefer`: Try SSL first, fall back to plain (default)
- `require`: Require SSL connection
- `verify-ca`: Require SSL and verify CA certificate
- `verify-full`: Require SSL, verify CA and hostname

**Recommendations:**
- Development: `prefer` or `disable` (local only)
- Production: `require` or `verify-full`

### test_before_acquire

**Default:** `true`

Test connections before returning them from the pool.

**Recommendations:**
- Always keep this `true` in production

**Purpose:**
- Ensures connections are healthy before use
- Prevents errors from stale or broken connections
- Small performance cost (~1ms) but greatly improves reliability

## Monitoring

### Prometheus Metrics

The `/metrics` endpoint exposes pool statistics:

```prometheus
# Current active connections
loupe_db_pool_connections_active 3

# Current idle connections
loupe_db_pool_connections_idle 7

# Maximum allowed connections
loupe_db_pool_connections_max 10

# Connection acquisition duration histogram
loupe_db_pool_acquire_duration_seconds_bucket{operation="query",le="0.001"} 1234
loupe_db_pool_acquire_duration_seconds_bucket{operation="query",le="0.005"} 1240
# ... more buckets

# Total acquisition timeouts
loupe_db_pool_acquire_timeout_total 5
```

### Health Checks

The `/api/v1/health` endpoint includes database connectivity checks:

```bash
curl http://localhost:8080/api/v1/health
```

```json
{
  "status": "healthy",
  "checks": {
    "database": {
      "status": "healthy",
      "latency_ms": 2
    }
  }
}
```

### Distributed Tracing

Database operations are automatically traced when OpenTelemetry is enabled:

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317
```

Traces include:
- Connection acquisition time
- Query execution time
- Transaction boundaries

## Tuning for Different Workloads

### API Server (Read-Heavy)

```bash
DB_MIN_CONNECTIONS=5
DB_MAX_CONNECTIONS=50
```

**Rationale:** API servers handle many concurrent requests, need larger pool.

### Background Worker (Write-Heavy)

```bash
DB_MIN_CONNECTIONS=2
DB_MAX_CONNECTIONS=10
```

**Rationale:** Workers process jobs sequentially, don't need many connections.

### Scheduler (Low Frequency)

```bash
DB_MIN_CONNECTIONS=1
DB_MAX_CONNECTIONS=3
```

**Rationale:** Scheduler runs periodically, minimal connection needs.

## Troubleshooting

### Pool Exhaustion

**Symptoms:**
- HTTP 503 errors
- `PoolTimedOut` errors in logs
- `loupe_db_pool_acquire_timeout_total` metric increasing
- High `loupe_db_pool_connections_active` near `max_connections`

**Solutions:**
1. Increase `DB_MAX_CONNECTIONS` (if database can handle it)
2. Reduce request processing time (optimize queries)
3. Add horizontal scaling (more API instances)
4. Reduce `acquire_timeout` to fail faster
5. Investigate connection leaks (check for missing `.await` or dropped futures)

### Connection Leaks

**Symptoms:**
- `connections_active` never decreases
- Pool exhaustion over time
- Database shows many idle connections

**Solutions:**
1. Ensure all queries are `.await`ed
2. Check for panics or dropped futures
3. Review transaction handling (ensure commits/rollbacks)
4. Reduce `max_lifetime` to force connection recycling

### Slow Queries

**Symptoms:**
- High `loupe_db_pool_acquire_duration_seconds` p99
- Requests timing out
- High database CPU

**Solutions:**
1. Add database indexes (see schema review in BE_TODO.md)
2. Use `EXPLAIN ANALYZE` to identify slow queries
3. Add query result caching
4. Optimize N+1 query patterns

### Database Connection Limits

**Symptoms:**
- Database rejects new connections
- `FATAL: too many connections` errors
- Random connection failures

**Solutions:**
1. Check total connections across all services
2. Reduce `max_connections` per instance
3. Increase PostgreSQL `max_connections` setting
4. Use PgBouncer for connection pooling at database level

## Performance Testing

### Load Testing Example

```bash
# Install k6
brew install k6

# Run load test
k6 run - <<EOF
import http from 'k6/http';
import { check } from 'k6';

export const options = {
  stages: [
    { duration: '1m', target: 50 },   // Ramp up to 50 users
    { duration: '3m', target: 50 },   // Stay at 50 users
    { duration: '1m', target: 100 },  // Ramp up to 100 users
    { duration: '3m', target: 100 },  // Stay at 100 users
    { duration: '1m', target: 0 },    // Ramp down
  ],
};

export default function() {
  const res = http.get('http://localhost:8080/api/v1/dashboards');
  check(res, {
    'status is 200': (r) => r.status === 200,
    'response time < 500ms': (r) => r.timings.duration < 500,
  });
}
EOF
```

### Monitor During Testing

```bash
# Watch metrics
watch -n 1 'curl -s http://localhost:8080/metrics | grep loupe_db_pool'

# Watch database connections
watch -n 1 'psql -c "SELECT count(*) FROM pg_stat_activity;"'
```

## Best Practices

1. **Start Conservative**: Begin with smaller pools and increase based on metrics
2. **Monitor Continuously**: Track pool metrics in production
3. **Test Under Load**: Load test before deploying pool changes
4. **Leave Headroom**: Don't use 100% of database connection capacity
5. **Use Transactions Wisely**: Keep transactions short, commit/rollback promptly
6. **Handle Errors**: Gracefully handle pool exhaustion with retries or circuit breakers
7. **Document Changes**: Record pool tuning decisions for your specific workload

## References

- [SQLx Pool Documentation](https://docs.rs/sqlx/latest/sqlx/pool/index.html)
- [PostgreSQL Connection Management](https://www.postgresql.org/docs/current/runtime-config-connection.html)
- [Database Connection Pooling Best Practices](https://wiki.postgresql.org/wiki/Number_Of_Database_Connections)
