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
    ├── api_tests.rs          # API integration tests
    ├── connector_tests.rs    # Database connector tests
    ├── db_tests.rs           # Database layer tests
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

**Run:**
```bash
cargo test --test api_tests -- --ignored
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

```rust
use testcontainers::clients::Cli;
use testcontainers_modules::postgres::Postgres;

#[tokio::test]
async fn test_with_real_database() {
    let docker = Cli::default();
    let postgres = docker.run(Postgres::default());

    let connection_string = format!(
        "postgres://postgres:postgres@localhost:{}/postgres",
        postgres.get_host_port_ipv4(5432)
    );

    let db = Database::connect(&connection_string).await.unwrap();
    // Run test with real database
}
```

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

## Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo-tarpaulin Documentation](https://github.com/xd009642/tarpaulin)
- [testcontainers-rs](https://github.com/testcontainers/testcontainers-rs)
- [wiremock-rs](https://github.com/LukeMathWalker/wiremock-rs)

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
- **Multiple test types** (unit, integration, E2E)
- **CI/CD integration** ready
- **Well-documented** testing patterns and practices

Testing is a first-class citizen in Loupe's development process!
