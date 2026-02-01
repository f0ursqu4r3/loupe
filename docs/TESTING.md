# Testing Strategy

## Overview

Loupe implements a comprehensive testing strategy with **110+ tests** covering unit, integration, and end-to-end scenarios. This document outlines our testing approach, best practices, and guidelines for writing new tests.

**Current Test Coverage:** 107/110 tests passing (97% pass rate)

## Test Organization

### Directory Structure

```
be/
├── src/
│   └── common/
│       ├── cache.rs          # Unit tests inline (#[cfg(test)])
│       ├── encryption.rs     # 7 unit tests
│       ├── secrets.rs        # 8 unit tests
│       ├── validation.rs     # Tests inline
│       └── models/
│           └── tests.rs      # Model tests
└── tests/
    ├── api_tests.rs          # HTTP API endpoint integration tests
    ├── connector_tests.rs    # Database connector tests (Postgres)
    ├── db_tests.rs           # Database layer CRUD tests
    ├── workflow_tests.rs     # End-to-end workflow tests (NEW)
    └── common/
        ├── db.rs             # Test database setup
        ├── fixtures.rs       # Test data fixtures
        └── helpers.rs        # Test helper functions
```

### Test Types

#### 1. Unit Tests (Inline)

Located in `#[cfg(test)]` modules within source files.

**Examples:**
- `src/common/encryption.rs` - Encryption/decryption logic
- `src/common/secrets.rs` - Secret redaction and management
- `src/common/validation.rs` - Input validation
- `src/common/filtering.rs` - Query filtering logic

**Run:**
```bash
cargo test --lib
```

#### 2. Integration Tests

Located in `tests/` directory, test multiple components together.

**Examples:**
- `tests/api_tests.rs` - HTTP API endpoints
- `tests/connector_tests.rs` - Database connectors
- `tests/db_tests.rs` - Database operations

**Run:**
```bash
cargo test --test api_tests
cargo test --test db_tests
```

#### 3. End-to-End Tests

Full system tests using testcontainers for real database instances.

**Examples:**
- `tests/workflow_tests.rs` - Complete user workflows from registration to dashboard creation
- `tests/api_tests.rs` - HTTP API endpoint testing with actix-web

**Workflows tested:**
- Complete analytics pipeline (user → datasource → queries → visualizations → dashboards)
- Scheduled query execution (create → schedule → execute → update)
- Concurrent operations (parallel query runs, concurrent tile additions)
- Error scenarios (query failures, org isolation, cascade deletion)
- Pagination (large result sets, page boundaries)

**Run:**
```bash
cargo test --test workflow_tests
cargo test --test api_tests
```

## Testing Best Practices

### 1. Test Naming Convention

```rust
#[test]
fn test_<functionality>_<scenario>_<expected_result>() {
    // Good examples:
    // test_encrypt_decrypt_roundtrip_succeeds
    // test_invalid_jwt_returns_unauthorized
    // test_pagination_with_empty_results
}
```

### 2. Arrange-Act-Assert Pattern

```rust
#[test]
fn test_user_creation_with_valid_data() {
    // Arrange
    let email = "test@example.com";
    let password = "SecurePassword123!";

    // Act
    let result = create_user(email, password);

    // Assert
    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.email, email);
}
```

### 3. Use Test Fixtures

```rust
// tests/common/fixtures.rs
pub fn create_test_user() -> User {
    User {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        // ... other fields
    }
}

// In your test
#[test]
fn test_something() {
    let user = fixtures::create_test_user();
    // Use the user
}
```

### 4. Isolate Tests

```rust
use serial_test::serial;

#[test]
#[serial]  // Run serially to avoid conflicts
fn test_database_migration() {
    // Test that modifies shared state
}
```

### 5. Test Error Cases

```rust
#[test]
fn test_invalid_email_returns_error() {
    let result = validate_email("not-an-email");

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.field, "email");
    assert!(err.message.contains("invalid"));
}
```

### 6. Use Descriptive Assertions

```rust
// ❌ Not helpful
assert!(result);

// ✅ Descriptive
assert!(
    result,
    "User creation should succeed with valid email and password"
);
```

## Test Coverage Goals

### Current Coverage by Module

| Module | Tests | Coverage | Status |
|--------|-------|----------|--------|
| Encryption | 7 | ~95% | ✅ Excellent |
| Secrets | 8 | ~90% | ✅ Excellent |
| Validation | 15+ | ~85% | ✅ Good |
| Models | 20+ | ~80% | ✅ Good |
| API Routes | 30+ | ~70% | ⚠️ Needs improvement |
| Database | 25+ | ~75% | ✅ Good |
| Connectors | 10+ | ~65% | ⚠️ Needs improvement |

### Target Coverage

- **Critical modules** (auth, encryption, security): 90%+
- **Business logic** (models, validation): 80%+
- **Infrastructure** (DB, connectors): 70%+
- **Overall project**: 75%+

## Running Tests

### All Tests

```bash
cd be && cargo test
```

### Specific Module

```bash
cargo test encryption
cargo test validation
cargo test api_tests
```

### With Output

```bash
cargo test -- --nocapture
```

### Quiet Mode

```bash
cargo test --quiet
```

### Single Test

```bash
cargo test test_encrypt_decrypt
```

### Ignored Tests Only

```bash
cargo test -- --ignored
```

## Code Coverage

### Install tarpaulin

```bash
cargo install cargo-tarpaulin
```

### Generate Coverage Report

```bash
cd be
cargo tarpaulin --out Html --output-dir coverage
```

### View Report

```bash
open coverage/index.html
```

### CI Integration

```yaml
# .github/workflows/test.yml
- name: Run tests with coverage
  run: |
    cargo install cargo-tarpaulin
    cargo tarpaulin --out Xml

- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v3
```

## Test Database Setup

### Using Testcontainers

All integration tests use testcontainers to spin up isolated PostgreSQL instances.

**Example pattern:**
```rust
use testcontainers::{ContainerAsync, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;

struct TestDb {
    db: Database,
    #[allow(dead_code)]
    container: ContainerAsync<Postgres>,
}

impl TestDb {
    async fn new() -> Self {
        let container = Postgres::default()
            .start()
            .await
            .expect("Failed to start postgres container");

        let host = container.get_host().await.expect("get host");
        let port = container.get_host_port_ipv4(5432).await.expect("get port");
        let database_url = format!(
            "postgres://postgres:postgres@{}:{}/postgres",
            host, port
        );

        let db = Database::connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        db.run_migrations().await.expect("Failed to run migrations");

        Self { db, container }
    }
}

#[tokio::test]
async fn test_with_real_database() {
    let test_db = TestDb::new().await;
    let db = test_db.database();

    // Run test with real database
}
```

**Benefits:**
- ✅ Isolated test environment per test
- ✅ Real PostgreSQL behavior (not mocked)
- ✅ Automatic cleanup after tests
- ✅ Parallel test execution safe

### Test Database Helpers

```rust
// tests/common/db.rs
pub async fn setup_test_db() -> Database {
    let db = Database::connect(test_database_url()).await.unwrap();
    db.run_migrations().await.unwrap();
    db
}

pub async fn cleanup_test_db(db: &Database) {
    // Clean up test data
}
```

## Testing Async Code

### Using tokio::test

```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_operation().await;
    assert!(result.is_ok());
}
```

### Testing Timeouts

```rust
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_operation_completes_within_timeout() {
    let result = timeout(
        Duration::from_secs(5),
        slow_operation()
    ).await;

    assert!(result.is_ok(), "Operation should complete within 5 seconds");
}
```

## Testing HTTP APIs

### Using actix-web test utilities

```rust
use actix_web::{test, App};

#[actix_web::test]
async fn test_health_endpoint() {
    let app = test::init_service(
        App::new().configure(routes::configure)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
```

### Testing with Authentication

```rust
#[actix_web::test]
async fn test_protected_endpoint() {
    let jwt_token = generate_test_jwt();

    let req = test::TestRequest::get()
        .uri("/api/v1/dashboards")
        .insert_header(("Authorization", format!("Bearer {}", jwt_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}
```

## Mocking and Stubbing

### Using wiremock for HTTP mocks

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_external_api_call() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/data"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({"status": "ok"})))
        .mount(&mock_server)
        .await;

    let result = call_external_api(&mock_server.uri()).await;
    assert!(result.is_ok());
}
```

## Testing Security Features

### Testing Authentication

```rust
#[test]
fn test_jwt_generation_and_validation() {
    let jwt_manager = JwtManager::new("secret", 24);
    let claims = Claims::new(user_id, org_id, role);

    let token = jwt_manager.encode(&claims).unwrap();
    let decoded = jwt_manager.decode(&token).unwrap();

    assert_eq!(decoded.sub, user_id);
}
```

### Testing Password Hashing

```rust
#[test]
fn test_password_hashing_and_verification() {
    let password = "SecurePassword123!";
    let hash = hash_password(password).unwrap();

    assert_ne!(hash, password);
    assert!(verify_password(password, &hash).unwrap());
    assert!(!verify_password("wrong", &hash).unwrap());
}
```

### Testing Encryption

```rust
#[test]
fn test_encryption_decryption() {
    let encryption = EncryptionManager::new();
    let plaintext = "sensitive data";

    let encrypted = encryption.encrypt(plaintext).unwrap();
    assert_ne!(encrypted, plaintext);
    assert!(encrypted.starts_with("v1:"));

    let decrypted = encryption.decrypt(&encrypted).unwrap();
    assert_eq!(decrypted, plaintext);
}
```

## Performance Testing

### Benchmarking

```rust
#[cfg(test)]
mod benches {
    use super::*;
    use std::time::Instant;

    #[test]
    fn bench_encryption() {
        let encryption = EncryptionManager::new();
        let data = "test data".repeat(1000);

        let start = Instant::now();
        for _ in 0..1000 {
            let _ = encryption.encrypt(&data);
        }
        let duration = start.elapsed();

        println!("1000 encryptions took: {:?}", duration);
        assert!(duration.as_millis() < 5000); // Should be under 5s
    }
}
```

## CI/CD Integration

### GitHub Actions Workflow

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run tests
        run: cd be && cargo test

      - name: Run clippy
        run: cd be && cargo clippy -- -D warnings

      - name: Check formatting
        run: cd be && cargo fmt -- --check
```

## Debugging Tests

### Print Debug Output

```rust
#[test]
fn test_with_debug_output() {
    let value = calculate_something();
    println!("Calculated value: {:?}", value);  // Visible with --nocapture
    assert_eq!(value, expected);
}
```

### Use dbg! Macro

```rust
#[test]
fn test_with_dbg() {
    let result = dbg!(complex_operation());  // Prints value and location
    assert!(result.is_ok());
}
```

### Run with Backtrace

```bash
RUST_BACKTRACE=1 cargo test
```

## Common Testing Patterns

### Testing Validation

```rust
#[test]
fn test_email_validation() {
    assert!(validate_email("valid@example.com").is_ok());
    assert!(validate_email("invalid").is_err());
    assert!(validate_email("").is_err());
    assert!(validate_email("@example.com").is_err());
}
```

### Testing Pagination

```rust
#[tokio::test]
async fn test_pagination() {
    let page1 = get_items(PaginationParams {
        limit: 10,
        offset: 0,
    }).await.unwrap();

    assert_eq!(page1.items.len(), 10);
    assert_eq!(page1.page, 1);
    assert!(page1.has_next);

    let page2 = get_items(PaginationParams {
        limit: 10,
        offset: 10,
    }).await.unwrap();

    assert_eq!(page2.page, 2);
}
```

### Testing Error Handling

```rust
#[tokio::test]
async fn test_error_propagation() {
    let result = operation_that_fails().await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::NotFound(msg) => assert!(msg.contains("not found")),
        _ => panic!("Expected NotFound error"),
    }
}
```

### Testing End-to-End Workflows

Complete user workflows ensure all components work together:

```rust
#[tokio::test]
async fn test_complete_analytics_pipeline() {
    let test = TestWorkflow::new().await;
    let db = &test.db;

    // 1. Create organization and user
    let org = db.create_organization("Analytics Corp").await.unwrap();
    let user = db.create_user(
        org.id,
        "analyst@example.com",
        "hash",
        "Analyst",
        OrgRole::Admin
    ).await.unwrap();

    // 2. Create datasource
    let datasource = db.create_datasource(
        org.id,
        "Production DB",
        DatasourceType::Postgres,
        "connection_string",
        user.id
    ).await.unwrap();

    // 3. Create query
    let query = db.create_query(
        org.id,
        datasource.id,
        "Daily Active Users",
        Some("Count DAU"),
        "SELECT date, COUNT(DISTINCT user_id) FROM logins GROUP BY date",
        &json!([]),
        &json!([]),
        30,
        10000,
        user.id
    ).await.unwrap();

    // 4. Create visualization
    let viz = db.create_visualization(
        org.id,
        query.id,
        "DAU Chart",
        ChartType::Line,
        &json!({"x": "date", "y": "count"}),
        &json!([]),
        user.id
    ).await.unwrap();

    // 5. Create dashboard with tile
    let dashboard = db.create_dashboard(
        org.id,
        "Executive Dashboard",
        Some("Key metrics"),
        &json!({}),
        &json!([]),
        user.id
    ).await.unwrap();

    let tile = db.create_tile(
        dashboard.id,
        viz.id,
        Some("Daily Active Users"),
        0, 0, 12, 6,
        &json!({})
    ).await.unwrap();

    // 6. Execute query
    let run = db.create_run(
        org.id,
        query.id,
        datasource.id,
        &query.sql,
        &json!({}),
        30,
        10000,
        user.id
    ).await.unwrap();

    let claimed = db.claim_run("runner-1").await.unwrap().unwrap();

    let result = db.create_run_result(
        run.id,
        &json!([{"name": "date", "data_type": "DATE"}, {"name": "count", "data_type": "INT8"}]),
        &json!([["2024-01-01", 1523], ["2024-01-02", 1678]]),
        2,
        256,
        125
    ).await.unwrap();

    let completed = db.complete_run(run.id, result.id).await.unwrap();

    // 7. Verify complete workflow
    assert_eq!(completed.status, RunStatus::Completed);
    let tiles = db.list_tiles(dashboard.id).await.unwrap();
    assert_eq!(tiles.len(), 1);
}
```

### Testing Concurrent Operations

Test race conditions and parallel execution:

```rust
#[tokio::test]
async fn test_concurrent_query_runs() {
    let test = TestWorkflow::new().await;
    let db = &test.db;

    // Setup: create 10 queued runs
    let mut run_ids = vec![];
    for _ in 0..10 {
        let run = db.create_run(/* ... */).await.unwrap();
        run_ids.push(run.id);
    }

    // Spawn 10 runners concurrently
    let mut set = JoinSet::new();
    for i in 0..10 {
        let runner_id = format!("runner-{}", i);
        let db_clone = db.clone();
        set.spawn(async move {
            db_clone.claim_run(&runner_id).await
        });
    }

    // Collect claimed runs
    let mut claimed = vec![];
    while let Some(res) = set.join_next().await {
        if let Ok(Ok(Some(run))) = res {
            claimed.push(run.id);
        }
    }

    // All runs should be claimed exactly once
    assert_eq!(claimed.len(), 10);
    claimed.sort();
    claimed.dedup();
    assert_eq!(claimed.len(), 10); // No duplicates
}
```

### Testing Organization Isolation

Verify multi-tenant data isolation:

```rust
#[tokio::test]
async fn test_organization_isolation() {
    let test = TestWorkflow::new().await;
    let db = &test.db;

    let org1 = db.create_organization("Org 1").await.unwrap();
    let org2 = db.create_organization("Org 2").await.unwrap();

    let user1 = db.create_user(
        org1.id,
        "user1@org1.com",
        "hash",
        "User 1",
        OrgRole::Admin
    ).await.unwrap();

    // Create datasource in org1
    let ds1 = db.create_datasource(
        org1.id,
        "Org1 DS",
        DatasourceType::Postgres,
        "conn",
        user1.id
    ).await.unwrap();

    // Attempt to access from org2 should fail
    let wrong_org = db.get_datasource(ds1.id, org2.id).await;
    assert!(wrong_org.is_err());

    // List operations should be isolated
    let org1_datasources = db.list_datasources(org1.id).await.unwrap();
    let org2_datasources = db.list_datasources(org2.id).await.unwrap();

    assert_eq!(org1_datasources.len(), 1);
    assert_eq!(org2_datasources.len(), 0);
}
```

## Load & Performance Testing

### Using k6

Load and performance tests validate system behavior under realistic load.

**Location:** `load-tests/`

**Tests available:**
- `auth-workflow.js` - Authentication endpoints (50-100 concurrent users)
- `dashboard-api.js` - Dashboard CRUD with read/write scenarios
- `query-execution.js` - Query creation and concurrent execution
- `connection-pool-stress.js` - Database pool behavior under extreme load

**Install k6:**
```bash
# macOS
brew install k6

# Linux (Ubuntu/Debian)
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6
```

**Run tests:**
```bash
# Start API server first
cd be && cargo run --release --bin api

# In separate terminal
cd load-tests
k6 run auth-workflow.js
k6 run dashboard-api.js
k6 run query-execution.js
k6 run connection-pool-stress.js
```

**Performance targets:**
- API p95 latency: < 1.5s
- API p99 latency: < 3s
- Error rate: < 1% under normal load
- Throughput: 400-500 req/s
- Database pool acquisition: p95 < 100ms

**Key metrics tracked:**
- HTTP request duration (p50/p95/p99)
- Request success/failure rates
- Custom operation durations (registration, login, etc.)
- Cache hit rates
- Connection pool statistics
- Concurrent query limiter behavior

See [PERFORMANCE_BENCHMARKS.md](PERFORMANCE_BENCHMARKS.md) for baseline metrics and [load-tests/README.md](../load-tests/README.md) for detailed usage.

## Test Maintenance

### Keeping Tests Fast

- Use in-memory databases when possible
- Mock external services
- Run integration tests selectively
- Use `#[ignore]` for slow tests

### Avoiding Flaky Tests

- Don't rely on timing
- Properly clean up resources
- Use deterministic test data
- Avoid shared mutable state

### Test Documentation

```rust
/// Tests that user creation succeeds with valid input
///
/// # Test Scenario
/// 1. Create user with valid email and password
/// 2. Verify user is returned with correct data
/// 3. Verify password is hashed
#[test]
fn test_user_creation_success() {
    // Test implementation
}
```

## Property-Based Testing

Property-based tests use the [proptest](https://github.com/proptest-rs/proptest) framework to automatically generate test cases and verify invariants hold across a wide range of inputs.

### When to Use Property-Based Testing

Property tests are ideal for:
- **Serialization invariants** - Roundtrip encode/decode should preserve data
- **Security validation** - Input sanitization, SQL injection prevention
- **Boundary conditions** - Edge cases with min/max values
- **Invariant checking** - Properties that must always hold true

### Test Files

- `be/tests/proptest_models.rs` - Model serialization and validation invariants
- `be/tests/proptest_security.rs` - Security-critical path fuzzing

**Run all property tests:**
```bash
cd be
cargo test --test proptest_models
cargo test --test proptest_security
```

### Example: Enum Serialization Invariant

```rust
use proptest::prelude::*;

fn arb_org_role() -> impl Strategy<Value = OrgRole> {
    prop_oneof![
        Just(OrgRole::Admin),
        Just(OrgRole::Editor),
        Just(OrgRole::Viewer),
    ]
}

proptest! {
    #[test]
    fn test_org_role_roundtrip(role in arb_org_role()) {
        // Property: Any OrgRole value should serialize and deserialize correctly
        let json = serde_json::to_string(&role).unwrap();
        let deserialized: OrgRole = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(role, deserialized);
    }
}
```

### Example: Input Validation Fuzzing

```rust
proptest! {
    #[test]
    fn test_sql_validation_rejects_too_long(
        prefix in "[A-Z ]{10,100}"
    ) {
        // Property: SQL exceeding 100,000 chars should always be rejected
        let too_long = prefix + &"A".repeat(100_001);
        assert!(validate_sql_length(&too_long).is_err());
    }

    #[test]
    fn test_connection_string_rejects_sql_injection(
        prefix in "postgresql://[a-z]{3,10}",
        injection in prop_oneof![
            Just("';DROP TABLE users--"),
            Just("/**/;exec('"),
            Just("xp_cmdshell"),
        ],
    ) {
        // Property: SQL injection patterns should always be rejected
        let malicious = format!("{}{}", prefix, injection);
        assert!(validate_connection_string(&malicious).is_err());
    }
}
```

### Example: Security Invariant Testing

```rust
proptest! {
    #[test]
    fn test_user_response_excludes_password(
        email in arb_email(),
        name in "[a-zA-Z ]{3,50}",
        role in arb_org_role(),
    ) {
        let user = User {
            id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            email,
            password_hash: "super_secret_hash_12345".to_string(),
            name,
            role,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response = UserResponse::from(user);
        let json = serde_json::to_string(&response).unwrap();

        // CRITICAL PROPERTY: password_hash must NEVER appear in response
        prop_assert!(!json.contains("super_secret_hash"));
        prop_assert!(!json.contains("password_hash"));
    }
}
```

### Property Test Best Practices

1. **Test Invariants, Not Examples**
   - ❌ Bad: "User 'alice@example.com' should serialize correctly"
   - ✅ Good: "Any valid email should serialize/deserialize correctly"

2. **Use Generators Strategically**
   ```rust
   // Generate valid emails
   fn arb_email() -> impl Strategy<Value = String> {
       "[a-z]{3,10}@[a-z]{3,10}\\.(com|org|net)"
   }

   // Generate boundary values
   fn arb_timeout() -> impl Strategy<Value = i32> {
       1..=300i32  // Valid range for timeout_seconds
   }
   ```

3. **Focus on Critical Paths**
   - Encryption/decryption roundtrips
   - Authentication token generation/validation
   - SQL validation and sanitization
   - Permission checks and authorization

4. **Test Boundary Conditions**
   ```rust
   proptest! {
       #[test]
       fn test_pagination_boundaries(
           limit in 1i64..=100i64,
           offset in 0i64..10000i64,
       ) {
           // Property: All values in valid range should be accepted
           assert!(validate_pagination(limit, offset).is_ok());
       }

       #[test]
       fn test_pagination_invalid_limits(
           limit in prop_oneof![
               -1000i64..0i64,  // Negative
               101i64..1000i64,  // Too large
           ],
       ) {
           // Property: All values outside range should be rejected
           assert!(validate_pagination(limit, 0).is_err());
       }
   }
   ```

5. **Document Properties Clearly**
   - Every property test should have a comment explaining the invariant being tested
   - Use descriptive test names that indicate the property
   - Add context for non-obvious properties

### Coverage

Property tests currently cover:
- ✅ Enum serialization roundtrips (OrgRole, RunStatus, ChartType, etc.)
- ✅ Request model validation (timeout, max_rows, field constraints)
- ✅ Security response filtering (password_hash, connection_string exclusion)
- ✅ SQL validation (length limits, Unicode handling)
- ✅ Connection string validation (scheme requirements, injection prevention)
- ✅ Name validation (character whitelist, length limits)
- ✅ Pagination validation (limit ranges, offset constraints)
- ✅ Date range validation (ordering, duration limits)
- ✅ Data integrity invariants (row_count consistency, tile dimensions)

**Total property tests:** 17 tests across 2 test files

## Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo-tarpaulin Documentation](https://github.com/xd009642/tarpaulin)
- [testcontainers-rs](https://github.com/testcontainers/testcontainers-rs)
- [wiremock-rs](https://github.com/LukeMathWalker/wiremock-rs)
- [proptest Documentation](https://github.com/proptest-rs/proptest)

## Contributing

When adding new features:

1. ✅ Write tests first (TDD)
2. ✅ Aim for 80%+ coverage on new code
3. ✅ Add both happy path and error cases
4. ✅ Update this document if introducing new patterns
5. ✅ Run `cargo test` before submitting PR

## Summary

- **110+ tests** with 97% pass rate
- **Comprehensive coverage** of critical modules (encryption, auth, validation)
- **Multiple test types** (unit, integration, E2E, load/performance)
- **3,222 lines** of integration test code
- **4 load test scenarios** with k6
- **CI/CD integration** ready
- **Performance benchmarks** documented
- **Well-documented** testing patterns and practices

Testing is a first-class citizen in Loupe's development process!
