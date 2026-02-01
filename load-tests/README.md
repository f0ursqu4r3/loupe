# Load Testing Suite

Comprehensive load and performance testing for the Loupe API using [k6](https://k6.io/).

## Overview

This test suite validates:
- **API Performance**: Response times under realistic load
- **Database Performance**: Connection pool behavior and query performance
- **Concurrency**: Handling multiple simultaneous users
- **Scalability**: System behavior under increasing load
- **Rate Limiting**: Proper enforcement of limits
- **Error Handling**: Graceful degradation under stress

## Prerequisites

### Install k6

**macOS:**
```bash
brew install k6
```

**Windows:**
```bash
choco install k6
# Or download from https://k6.io/docs/get-started/installation/
```

**Linux:**
```bash
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6
```

### Start the API Server

```bash
cd be
cargo run --release --bin api
```

The server should be running on `http://localhost:8080` (default).

## Test Scripts

### 1. Authentication Workflow (`auth-workflow.js`)

Tests user registration and login performance under load.

**What it tests:**
- User registration endpoint
- Login endpoint
- Password hashing performance (Argon2)
- JWT generation

**Load profile:**
- Ramp-up: 10 → 50 → 100 users
- Duration: 5 minutes
- Sustain: 50 users for 2 minutes
- Spike: 100 users for 1 minute

**Run:**
```bash
k6 run auth-workflow.js
```

**Custom parameters:**
```bash
k6 run --vus 50 --duration 30s auth-workflow.js
k6 run -e BASE_URL=http://prod-server:8080 auth-workflow.js
```

**Success criteria:**
- p95 latency < 2s
- p99 latency < 5s
- Error rate < 1%
- Registration p95 < 3s
- Login p95 < 1s

---

### 2. Dashboard API (`dashboard-api.js`)

Tests dashboard CRUD operations with realistic read/write ratios.

**What it tests:**
- List dashboards (with pagination)
- Get dashboard by ID
- Create dashboard
- Update dashboard
- Delete dashboard
- Cache hit rate

**Scenarios:**
- **Read-heavy workload**: 80% reads, 20% writes (100 VUs, 3 minutes)
- **Write-heavy workload**: Create → Update → Delete cycles (20 VUs, 3 minutes)

**Run:**
```bash
k6 run dashboard-api.js
```

**Success criteria:**
- p95 latency < 1.5s
- List dashboards p95 < 800ms
- Get dashboard p95 < 500ms
- Create dashboard p95 < 2s
- Cache hit rate > 50% (for reads)

---

### 3. Query Execution (`query-execution.js`)

Tests query creation and execution with concurrent run limits.

**What it tests:**
- Query creation
- Query run creation
- Concurrent query limiter (per-org and global)
- Query queue behavior
- Connection pool under query load

**Scenarios:**
- **Concurrent execution**: 50 queries/second for 2 minutes
- **Stress test**: Ramp 10 → 200 queries/second (tests limit enforcement)

**Run:**
```bash
k6 run query-execution.js
```

**Success criteria:**
- Query creation p95 < 2s
- Run creation p95 < 1s
- Proper 429 responses when limits exceeded
- Success rate > 80% under stress

---

### 4. Connection Pool Stress (`connection-pool-stress.js`)

Validates database connection pool behavior under extreme load.

**What it tests:**
- Pool scaling (0 → 200 VUs)
- Pool exhaustion handling
- Connection acquisition timeouts
- Connection reuse
- Long-running query handling

**Scenarios:**
- **Pool ramp**: Gradual increase to 2x pool max
- **Pool burst**: Sudden spike to 300 VUs
- **Long-running queries**: 10 VUs with 30s timeouts

**Run:**
```bash
k6 run connection-pool-stress.js
```

**Success criteria:**
- Pool acquisition errors < 100
- Timeouts < 50
- p95 latency < 10s (tolerate slower under stress)
- Graceful degradation (no crashes)

---

## Running All Tests

### Sequential execution:
```bash
k6 run auth-workflow.js
k6 run dashboard-api.js
k6 run query-execution.js
k6 run connection-pool-stress.js
```

### Parallel execution (separate terminals):
```bash
# Terminal 1
k6 run auth-workflow.js

# Terminal 2
k6 run dashboard-api.js

# Terminal 3
k6 run query-execution.js
```

## Interpreting Results

### Key Metrics

**HTTP Request Duration:**
- `http_req_duration` - Total request time (includes network, processing, response)
- `p(95)` - 95th percentile (95% of requests faster than this)
- `p(99)` - 99th percentile (99% of requests faster than this)
- `avg` - Average request time
- `max` - Maximum request time

**Request Rate:**
- `http_reqs` - Total requests made
- `iterations` - Number of complete VU iterations
- `vus` - Current virtual users

**Error Rate:**
- `http_req_failed` - Percentage of failed requests
- Should be < 1% under normal load
- < 5% acceptable under stress tests

**Custom Metrics:**
- `registration_duration` - User registration time
- `login_duration` - Login time
- `cache_hit_rate` - Percentage of requests served from cache
- `pool_acquisition_errors` - Database pool exhaustion count

### Example Output

```
     ✓ list dashboards status is 200
     ✓ list has items array
     ✓ create dashboard status is 201

     checks.........................: 98.50%  ✓ 2955  ✗ 45
     data_received..................: 12 MB   200 kB/s
     data_sent......................: 2.4 MB  40 kB/s
     http_req_duration..............: avg=245ms  min=12ms  med=198ms  max=3.2s   p(95)=567ms  p(99)=1.1s
     http_req_failed................: 1.50%   ✓ 45    ✗ 2955
     http_reqs......................: 3000    50/s
     iterations.....................: 1500    25/s
     vus............................: 50      min=0   max=100
     vus_max........................: 100     min=100 max=100

     cache_hit_rate.................: 65.23%  ✓ 1305  ✗ 695
     list_dashboards_duration.......: avg=180ms  p(95)=420ms  p(99)=780ms
```

**Interpretation:**
- ✅ 98.5% success rate (good)
- ✅ p95 latency 567ms (under 800ms threshold)
- ✅ p99 latency 1.1s (under 1.5s threshold)
- ✅ 65% cache hit rate (above 50% target)
- ⚠️ 1.5% error rate (slightly high, investigate)

## Performance Targets

### API Endpoints

| Endpoint | p50 | p95 | p99 | Throughput |
|----------|-----|-----|-----|------------|
| GET /dashboards | <100ms | <500ms | <1s | 500 req/s |
| POST /dashboards | <200ms | <1s | <2s | 100 req/s |
| POST /auth/register | <500ms | <2s | <5s | 50 req/s |
| POST /auth/login | <100ms | <500ms | <1s | 200 req/s |
| POST /queries | <200ms | <1s | <3s | 100 req/s |
| POST /runs | <300ms | <1.5s | <3s | 50 req/s |

### Database Operations

| Operation | p50 | p95 | p99 |
|-----------|-----|-----|-----|
| Simple SELECT | <10ms | <50ms | <100ms |
| Complex JOIN | <50ms | <200ms | <500ms |
| INSERT | <20ms | <100ms | <200ms |
| UPDATE | <20ms | <100ms | <200ms |

### Connection Pool

| Metric | Target | Max |
|--------|--------|-----|
| Active connections | 20-40 | 50 |
| Idle connections | 10-20 | 30 |
| Acquisition time | <10ms | <100ms |
| Acquisition timeouts | 0 | <10/min |

## Troubleshooting

### High Error Rates

**Symptoms:** `http_req_failed` > 5%

**Causes:**
- Rate limiting triggered (429 responses)
- Database connection pool exhausted
- Query timeout exceeded
- Server overload

**Solutions:**
- Reduce VU count or arrival rate
- Increase connection pool size
- Optimize slow queries
- Scale server resources

### Slow Response Times

**Symptoms:** p95 > threshold

**Causes:**
- Database query performance
- Connection pool queuing
- Network latency
- CPU/memory pressure

**Solutions:**
- Check Prometheus metrics: http://localhost:8080/metrics
- Review slow query logs
- Examine connection pool stats
- Profile with `cargo flamegraph`

### Connection Pool Errors

**Symptoms:** `pool_acquisition_errors` > 0

**Causes:**
- More concurrent requests than pool max
- Long-running queries holding connections
- Connection leaks

**Solutions:**
- Increase `DATABASE_POOL_MAX_CONNECTIONS`
- Reduce query timeout
- Check for connection leaks in code
- Monitor with: `db_pool_connections_active` metric

### Memory Issues

**Symptoms:** Test crashes or OOM errors

**Causes:**
- Too many concurrent VUs
- Response body accumulation
- Large result sets

**Solutions:**
- Reduce `maxVUs` in scenarios
- Use `discardResponseBodies` option
- Limit query result sizes

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Load Tests

on:
  schedule:
    - cron: '0 2 * * *' # Nightly at 2 AM
  workflow_dispatch:

jobs:
  load-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Start API server
        run: |
          cd be
          cargo build --release --bin api
          cargo run --release --bin api &
          sleep 10

      - name: Install k6
        run: |
          sudo gpg -k
          sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
          echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
          sudo apt-get update
          sudo apt-get install k6

      - name: Run auth workflow test
        run: k6 run --out json=auth-results.json load-tests/auth-workflow.js

      - name: Run dashboard API test
        run: k6 run --out json=dashboard-results.json load-tests/dashboard-api.js

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: load-test-results
          path: '*-results.json'
```

## Best Practices

1. **Start Small**: Begin with low VU counts and ramp up gradually
2. **Monitor Metrics**: Watch Prometheus metrics during tests
3. **Isolate Tests**: Run one test at a time for accurate results
4. **Realistic Data**: Use production-like data volumes
5. **Clean Up**: Delete test data after runs (implement teardown)
6. **Baseline First**: Establish baseline performance before optimization
7. **Compare Results**: Track performance over time
8. **Test Environments**: Use staging, not production

## Advanced Usage

### Custom Thresholds

```javascript
export const options = {
  thresholds: {
    'http_req_duration{endpoint:login}': ['p(95)<500'],
    'http_req_duration{endpoint:register}': ['p(95)<2000'],
    'http_req_failed{endpoint:dashboard}': ['rate<0.01'],
  },
};
```

### Environment Variables

```bash
export BASE_URL=https://staging.loupe.example.com
export TEST_DURATION=300s
export MAX_VUS=200

k6 run \
  -e BASE_URL=$BASE_URL \
  -e DURATION=$TEST_DURATION \
  --vus $MAX_VUS \
  auth-workflow.js
```

### Cloud Execution (k6 Cloud)

```bash
k6 cloud auth-workflow.js
```

## Resources

- [k6 Documentation](https://k6.io/docs/)
- [k6 Examples](https://k6.io/docs/examples/)
- [Performance Testing Best Practices](https://k6.io/docs/testing-guides/automated-performance-testing/)
- [Prometheus Metrics Guide](https://prometheus.io/docs/practices/naming/)

## Contributing

When adding new load tests:

1. Follow the existing test structure
2. Include clear comments and documentation
3. Set realistic thresholds
4. Add custom metrics for key operations
5. Update this README with usage instructions
6. Test locally before committing

---

**Last Updated:** 2026-02-01
**Test Suite Version:** 1.0.0
