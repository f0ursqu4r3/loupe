# Security Audit Report - January 2026

**Date:** January 31, 2026
**Auditor:** Security Review
**Scope:** Critical Security & Validation (BE_TODO items 1-5)

---

## Executive Summary

This audit identified **CRITICAL security vulnerabilities** that must be addressed before production deployment:

- 🔴 **CRITICAL:** Insecure authentication (forgeable tokens)
- 🔴 **CRITICAL:** Arbitrary SQL execution without validation
- 🟠 **HIGH:** Information disclosure in error messages
- 🟠 **HIGH:** Missing input validation
- 🟡 **MEDIUM:** No rate limiting or brute force protection

**Production Readiness:** ❌ **NOT READY** - Critical issues must be fixed first.

---

## Critical Vulnerabilities

### 🔴 CRITICAL #1: Insecure Authentication System

**File:** [auth.rs:22-44](../be/src/api/routes/auth.rs#L22-L44)

**Issue:** The authentication system uses simple base64 encoding instead of cryptographically signed tokens. Any user can forge tokens to impersonate other users.

**Current Implementation:**

```rust
fn create_token(user_id: Uuid, org_id: Uuid) -> String {
    let payload = format!("{}:{}", user_id, org_id);
    URL_SAFE_NO_PAD.encode(payload.as_bytes())
}
```

**Attack Scenario:**

1. Attacker registers a normal account and gets a token
2. Attacker decodes the token to understand the format: `user_id:org_id`
3. Attacker creates a new token: `base64("admin-user-id:target-org-id")`
4. Attacker uses forged token to access any account/organization

**Impact:** Complete authentication bypass, full system compromise

**Remediation Required:**

- [ ] Implement JWT with cryptographic signing (use `jsonwebtoken` crate)
- [ ] Add token expiration (e.g., 24 hours)
- [ ] Add token refresh mechanism
- [ ] Add secret key from environment variable
- [ ] Store tokens in HttpOnly cookies or require Bearer auth
- [ ] Add token revocation capability

**Example Fix:**

```rust
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,      // user_id
    org: String,      // org_id
    exp: usize,       // expiration
    iat: usize,       // issued at
}

fn create_token(user_id: Uuid, org_id: Uuid, secret: &str) -> Result<String, Error> {
    let exp = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        org: org_id.to_string(),
        exp,
        iat: Utc::now().timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes())
    )
    .map_err(|e| Error::Internal(format!("Token creation failed: {}", e)))
}
```

---

### 🔴 CRITICAL #2: Arbitrary SQL Execution

**Files:**

- [runs.rs:121-164](../be/src/api/routes/runs.rs#L121-L164) (execute_adhoc)
- [connectors/postgres.rs:38-99](../be/src/common/connectors/postgres.rs#L38-L99)

**Issue:** Users can submit arbitrary SQL through the `/runs/execute` endpoint with no validation or sanitization. While the connector wraps queries in a subquery with LIMIT, this provides minimal protection.

**Current Flow:**

```
User → POST /runs/execute → create_query(user_sql) → create_run(user_sql) → connector.execute(user_sql)
```

**Vulnerable Code:**

```rust
// runs.rs:156 - User SQL stored directly
&body.sql,  // No validation!

// postgres.rs:47-50 - Wraps but doesn't validate
let limited_sql = format!(
    "SELECT * FROM ({}) AS _q LIMIT {}",
    sql.trim().trim_end_matches(';'),  // User SQL inserted here!
    max_rows
);
```

**Attack Scenarios:**

1. **Information Disclosure:**

```sql
-- Read from other tables
SELECT * FROM users; -- View all user data
SELECT * FROM organizations; -- View all org data
```

1. **Privilege Escalation:**

```sql
-- Use database functions
SELECT pg_read_file('/etc/passwd');
SELECT pg_ls_dir('/');
```

1. **Denial of Service:**

```sql
-- Expensive operations
SELECT * FROM pg_stat_activity, pg_stat_activity, pg_stat_activity;
SELECT COUNT(*) FROM generate_series(1, 1000000000);
```

1. **Time-Based Attacks:**

```sql
-- Exfiltrate data via timing
SELECT CASE WHEN (SELECT password FROM users LIMIT 1) LIKE 'a%'
       THEN pg_sleep(5) ELSE 0 END;
```

**Current Protections (Insufficient):**

- ✅ LIMIT clause prevents unbounded result sets
- ✅ Timeout prevents indefinite execution
- ❌ No SQL validation or AST parsing
- ❌ No database permission restrictions
- ❌ No query allowlist/denylist
- ❌ No detection of dangerous functions

**Impact:**

- Data breach (access to all org data)
- Lateral movement (access other organizations)
- DoS attacks
- Potential database server compromise

**Remediation Required:**

**Option A: SQL Parser & Validator (Recommended)**

- [ ] Add SQL parser (use `sqlparser-rs` crate)
- [ ] Parse and validate SQL AST before execution
- [ ] Block dangerous statements (DROP, ALTER, CREATE, etc.)
- [ ] Block system functions (pg_read_file, pg_ls_dir, etc.)
- [ ] Restrict to SELECT statements only
- [ ] Validate table access against schema allowlist
- [ ] Add query complexity analysis

```rust
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

fn validate_sql(sql: &str) -> Result<(), Error> {
    let dialect = PostgreSqlDialect {};
    let ast = Parser::parse_sql(&dialect, sql)
        .map_err(|e| Error::BadRequest(format!("Invalid SQL: {}", e)))?;

    for statement in ast {
        match statement {
            Statement::Query(query) => {
                // Validate query is safe
                validate_query(&query)?;
            },
            _ => {
                return Err(Error::BadRequest(
                    "Only SELECT queries are allowed".to_string()
                ));
            }
        }
    }
    Ok(())
}
```

**Option B: Database User Restrictions**

- [ ] Create read-only database user per datasource
- [ ] Grant SELECT permission only on specific schemas
- [ ] Revoke all dangerous privileges
- [ ] Use separate connection pool for user queries
- [ ] Implement row-level security in PostgreSQL

```sql
-- Create restricted user
CREATE ROLE loupe_query_user WITH LOGIN PASSWORD 'secure_password';
GRANT CONNECT ON DATABASE analytics TO loupe_query_user;
GRANT USAGE ON SCHEMA public TO loupe_query_user;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO loupe_query_user;
REVOKE ALL ON pg_catalog.pg_proc FROM loupe_query_user;
```

**Option C: Hybrid Approach (Best)**

- Implement both SQL validation AND database restrictions
- Defense in depth

---

### 🟠 HIGH #3: Information Disclosure in Errors

**File:** [error.rs:23-69](../be/src/common/error.rs#L23-L69)

**Issue:** Detailed error messages are returned to clients, potentially leaking sensitive information about the database, file paths, and internal implementation.

**Vulnerable Code:**

```rust
fn error_response(&self) -> HttpResponse {
    // ...
    HttpResponse::build(status).json(serde_json::json!({
        "error": {
            "type": error_type,
            "message": self.to_string()  // ❌ Leaks internal details
        }
    }))
}

// error.rs:76 - Database errors leak SQL details
Error::Database(e.to_string())  // ❌ Exposes database internals
```

**Examples of Information Leakage:**

```json
// Bad: Reveals database schema
{
  "error": {
    "type": "database_error",
    "message": "Database error: column 'password_hash' of relation 'users' does not exist"
  }
}

// Bad: Reveals file paths
{
  "error": {
    "type": "internal_error",
    "message": "Internal error: No such file or directory: /var/app/config/secrets.toml"
  }
}
```

**Impact:**

- Reveals database schema and table names
- Exposes internal file structure
- Aids in crafting targeted attacks
- Violates security through obscurity principle

**Remediation Required:**

- [ ] Separate client-facing and server-side error messages
- [ ] Log detailed errors server-side only
- [ ] Return generic messages to clients for server errors
- [ ] Add error correlation IDs for debugging
- [ ] Implement structured logging with context

**Example Fix:**

```rust
impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        let (status, error_type, client_message) = match self {
            Error::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                "not_found",
                msg.clone(), // OK to show
            ),
            Error::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "bad_request",
                msg.clone(), // OK to show
            ),
            Error::Database(msg) => {
                // Log detailed error server-side
                tracing::error!("Database error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database_error",
                    "A database error occurred".to_string(), // Generic message
                )
            },
            // ... similar for other server errors
        };

        let error_id = Uuid::new_v4();
        tracing::error!(error_id = %error_id, error = %self, "Request failed");

        HttpResponse::build(status).json(serde_json::json!({
            "error": {
                "type": error_type,
                "message": client_message,
                "error_id": error_id, // For support/debugging
            }
        }))
    }
}
```

---

### 🟠 HIGH #4: Missing Input Validation

**Files:** All route handlers

**Issue:** Request payloads lack validation for length, format, and business rules. This can lead to:

- Database errors from malformed input
- Storage exhaustion from large payloads
- Logic errors from invalid data

**Examples of Missing Validation:**

**dashboards.rs:**

```rust
// ❌ No validation on name length
&body.name,  // Could be 1MB of text

// ❌ No validation on tags array
&body.tags,  // Could be 100,000 tags

// ❌ No validation on JSON schema
&body.parameters,  // Could be deeply nested, huge payload
```

**queries.rs:**

```rust
// ❌ No validation on SQL length
&body.sql,  // Could be megabytes of SQL

// ❌ No validation on timeout values
body.timeout_seconds,  // Could be 999999 seconds

// ❌ No validation on max_rows
body.max_rows,  // Could be usize::MAX
```

**auth.rs:**

```rust
// ❌ No email format validation
&req.email,  // Could be "not-an-email"

// ❌ No password strength requirements
&req.password,  // Could be "a"

// ❌ No name length validation
&req.name,  // Could be empty or very long
```

**Impact:**

- Poor user experience (unclear errors)
- Database performance issues
- Storage exhaustion
- Business logic bypasses

**Remediation Required:**

**Step 1: Add validation middleware**

```rust
use validator::{Validate, ValidationError};

#[derive(Deserialize, Validate)]
pub struct CreateDashboardRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    #[validate(length(max = 2000))]
    pub description: Option<String>,

    #[validate(length(max = 100))]
    pub tags: Vec<String>,

    // Custom validator for parameters
    #[validate(custom = "validate_parameters")]
    pub parameters: serde_json::Value,
}

fn validate_parameters(params: &serde_json::Value) -> Result<(), ValidationError> {
    // Check depth, size, etc.
    if params.to_string().len() > 10_000 {
        return Err(ValidationError::new("parameters_too_large"));
    }
    Ok(())
}
```

**Step 2: Add validation to routes**

```rust
async fn create_dashboard(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    body: web::Json<CreateDashboardRequest>,
) -> Result<HttpResponse, Error> {
    // Validate input
    body.validate()
        .map_err(|e| Error::BadRequest(format!("Validation failed: {}", e)))?;

    // ... rest of handler
}
```

**Step 3: Add domain-specific validators**

```rust
#[derive(Deserialize, Validate)]
pub struct CreateQueryRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    #[validate(length(max = 100_000))]
    pub sql: String,

    #[validate(range(min = 1, max = 300))]
    pub timeout_seconds: i32,

    #[validate(range(min = 1, max = 10_000))]
    pub max_rows: usize,

    #[validate(email)]
    pub owner_email: Option<String>,
}
```

**Validation Checklist:**

- [ ] Email format validation
- [ ] Password strength (min 8 chars, complexity)
- [ ] Name/description length limits
- [ ] SQL length limits
- [ ] Timeout value ranges (1-300 seconds)
- [ ] max_rows ranges (1-10,000)
- [ ] Array size limits (tags, parameters)
- [ ] JSON depth/size limits
- [ ] UUID format validation
- [ ] Enum value validation
- [ ] Cron expression validation (schedules)
- [ ] URL format validation (webhooks)

---

### 🟡 MEDIUM #5: Missing Rate Limiting & Brute Force Protection

**Issue:** No protection against automated attacks on authentication endpoints or API abuse.

**Missing Protections:**

- ❌ No rate limiting on `/auth/login`
- ❌ No rate limiting on `/auth/register`
- ❌ No account lockout after failed logins
- ❌ No CAPTCHA or similar bot protection
- ❌ No rate limiting on expensive endpoints (queries, runs)
- ❌ No IP-based throttling
- ❌ No user-based quotas

**Attack Scenarios:**

1. **Brute Force Login:**

```bash
# Attacker tries passwords rapidly
for pwd in $(cat passwords.txt); do
  curl -X POST /auth/login -d "{\"email\":\"admin@example.com\",\"password\":\"$pwd\"}"
done
```

1. **API Abuse:**

```bash
# Flood system with expensive queries
while true; do
  curl -X POST /runs/execute -d '{"sql":"SELECT COUNT(*) FROM generate_series(1,1000000)"}'
done
```

1. **Account Enumeration:**

```bash
# Determine which emails are registered
for email in $(cat emails.txt); do
  curl -X POST /auth/login -d "{\"email\":\"$email\",\"password\":\"test\"}"
  # Different response time/message = account exists
done
```

**Remediation Required:**

**Option A: Use actix-governor**

```rust
use actix_governor::{Governor, GovernorConfigBuilder};

// In main.rs
let governor_conf = GovernorConfigBuilder::default()
    .per_second(2)
    .burst_size(5)
    .finish()
    .unwrap();

App::new()
    .wrap(Governor::new(&governor_conf))
    // ... routes
```

**Option B: Custom rate limiting middleware**

```rust
use std::collections::HashMap;
use std::sync::Mutex;

struct RateLimiter {
    // IP -> (count, window_start)
    limits: Mutex<HashMap<String, (u32, Instant)>>,
}

impl RateLimiter {
    fn check(&self, ip: &str, limit: u32, window: Duration) -> bool {
        let mut limits = self.limits.lock().unwrap();
        let now = Instant::now();

        match limits.get_mut(ip) {
            Some((count, window_start)) => {
                if now.duration_since(*window_start) > window {
                    *count = 1;
                    *window_start = now;
                    true
                } else if *count < limit {
                    *count += 1;
                    true
                } else {
                    false
                }
            }
            None => {
                limits.insert(ip.to_string(), (1, now));
                true
            }
        }
    }
}
```

**Required Limits:**

- [ ] `/auth/login`: 5 attempts per 15 minutes per IP
- [ ] `/auth/register`: 3 registrations per hour per IP
- [ ] `/runs/execute`: 100 requests per minute per user
- [ ] `/queries`: 50 requests per minute per user
- [ ] Add account lockout after 5 failed logins (15 min)
- [ ] Add exponential backoff
- [ ] Return 429 status with Retry-After header

---

## Additional Security Issues

### Database Connection Security

**File:** [connectors/postgres.rs:15-24](../be/src/common/connectors/postgres.rs#L15-L24)

**Issues:**

- [ ] No SSL/TLS enforcement for database connections
- [ ] Connection string validation missing
- [ ] No verification of connection string format

**Remediation:**

```rust
pub async fn new(connection_string: &str) -> Result<Self> {
    // Validate connection string format
    if !connection_string.starts_with("postgres://") &&
       !connection_string.starts_with("postgresql://") {
        return Err(Error::BadRequest("Invalid connection string format".into()));
    }

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .ssl_mode(sqlx::postgres::PgSslMode::Require) // ✅ Force SSL
        .connect(connection_string)
        .await?;

    Ok(Self { pool })
}
```

### Session Management

**Missing Features:**

- [ ] Session storage (Redis)
- [ ] Session expiration
- [ ] Logout functionality
- [ ] Session invalidation on password change
- [ ] Concurrent session limits

### Authorization Granularity

**Current State:** Organization-level only

**Needed:**

- [ ] Resource-level permissions (who can edit queries?)
- [ ] Role-based access control (Admin, Editor, Viewer)
- [ ] Permission inheritance
- [ ] Audit logging of permission changes

---

## Compliance & Best Practices

### OWASP Top 10 Coverage

| OWASP Risk                         | Status    | Notes                                |
| ---------------------------------- | --------- | ------------------------------------ |
| A01:2021 Broken Access Control     | 🔴 FAIL    | Forgeable tokens allow full bypass   |
| A02:2021 Cryptographic Failures    | 🔴 FAIL    | No token signing, weak auth          |
| A03:2021 Injection                 | 🔴 FAIL    | SQL injection via execute_adhoc      |
| A04:2021 Insecure Design           | 🟡 PARTIAL | Missing security requirements        |
| A05:2021 Security Misconfiguration | 🟡 PARTIAL | No SSL enforcement, detailed errors  |
| A06:2021 Vulnerable Components     | 🟢 PASS    | Dependencies appear current          |
| A07:2021 Auth Failures             | 🔴 FAIL    | No rate limiting, weak tokens        |
| A08:2021 Data Integrity Failures   | 🟡 PARTIAL | Missing input validation             |
| A09:2021 Logging Failures          | 🟡 PARTIAL | Logging exists but incomplete        |
| A10:2021 SSRF                      | 🟢 PASS    | No external requests from user input |

### Security Headers

**Missing Headers:**

- [ ] Content-Security-Policy
- [ ] X-Frame-Options: DENY
- [ ] X-Content-Type-Options: nosniff
- [ ] Strict-Transport-Security
- [ ] Referrer-Policy

---

## Prioritized Remediation Plan

### Phase 1: CRITICAL (Week 1) 🔴

**Must fix before ANY production deployment:**

1. **Fix Authentication (2-3 days)**
   - Implement JWT with signing
   - Add token expiration
   - Add refresh tokens
   - Store JWT secret in environment

2. **Fix SQL Injection (2-3 days)**
   - Add SQL parser and validator
   - Create read-only database users
   - Implement query allowlist
   - Add query logging

3. **Fix Error Disclosure (1 day)**
   - Implement separate client/server error messages
   - Add error correlation IDs
   - Update all error handlers

### Phase 2: HIGH (Week 2) 🟠

1. **Add Input Validation (2 days)**
   - Add validator crate
   - Create validation rules for all DTOs
   - Add middleware validation
   - Update error messages

2. **Add Rate Limiting (1-2 days)**
   - Implement rate limiting middleware
   - Add per-endpoint limits
   - Add account lockout
   - Return proper 429 responses

### Phase 3: MEDIUM (Week 3) 🟡

1. **Database Security**
   - Enforce SSL connections
   - Create restricted users
   - Add connection string validation

2. **Security Headers**
   - Add security header middleware
   - Configure CSP
   - Test header configuration

3. **Session Management**
   - Add Redis for sessions
   - Implement logout
   - Add session limits

### Phase 4: ONGOING

1. **Monitoring & Testing**
   - Add security event logging
   - Set up intrusion detection
   - Regular penetration testing
   - Dependency vulnerability scanning

---

## Testing Requirements

### Security Tests Required

**Authentication Tests:**

- [ ] Test token forgery attempts fail
- [ ] Test expired tokens are rejected
- [ ] Test invalid signatures are rejected
- [ ] Test password hashing is secure
- [ ] Test login rate limiting

**SQL Injection Tests:**

- [ ] Test dangerous SQL is blocked
- [ ] Test system functions are blocked
- [ ] Test SQL parser handles edge cases
- [ ] Test query timeout works
- [ ] Test result limits work

**Authorization Tests:**

- [ ] Test cross-org access is blocked
- [ ] Test resource ownership is verified
- [ ] Test permission checks work

**Input Validation Tests:**

- [ ] Test overly long inputs are rejected
- [ ] Test invalid formats are rejected
- [ ] Test boundary values
- [ ] Test Unicode/special characters

**Error Handling Tests:**

- [ ] Test errors don't leak sensitive info
- [ ] Test error IDs are logged
- [ ] Test proper status codes

---

## Conclusion

The application has **CRITICAL security vulnerabilities** that make it unsafe for production use:

1. ❌ Authentication can be completely bypassed
2. ❌ Users can execute arbitrary SQL
3. ❌ Sensitive information leaks in errors

**Recommendation:** **DO NOT DEPLOY TO PRODUCTION** until Phase 1 critical fixes are completed and tested.

**Timeline:** Minimum 2-3 weeks of security hardening required before production deployment.

**Next Steps:**

1. Review and approve this audit
2. Begin Phase 1 critical fixes
3. Conduct security testing
4. Re-audit after fixes
5. Consider professional penetration testing

---

## References

- [OWASP Top 10 2021](https://owasp.org/Top10/)
- [OWASP Authentication Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)
- [OWASP SQL Injection Prevention](https://cheatsheetseries.owasp.org/cheatsheets/SQL_Injection_Prevention_Cheat_Sheet.html)
- [Rust Security Best Practices](https://anssi-fr.github.io/rust-guide/)
- [JWT Best Practices](https://tools.ietf.org/html/rfc8725)
