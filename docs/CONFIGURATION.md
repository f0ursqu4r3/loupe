# Configuration Guide

This document describes all configuration options for Loupe backend services.

## Environment Variables

All configuration is loaded from environment variables. You can set these in a `.env` file in the `be/` directory.

### Database

| Variable       | Required | Default | Description                                                                                                                                       |
| -------------- | -------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `DATABASE_URL` | ✅        | -       | PostgreSQL connection string<br/>Format: `postgres://user:password@host:port/database`<br/>Example: `postgres://loupe:loupe@localhost:5432/loupe` |

### API Server

| Variable               | Required | Default     | Description                                                                                                                                                                   |
| ---------------------- | -------- | ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `API_HOST`             | ❌        | `127.0.0.1` | Host address for the API server to bind to                                                                                                                                    |
| `API_PORT`             | ❌        | `8080`      | Port for the API server to listen on                                                                                                                                          |
| `CORS_ALLOWED_ORIGINS` | ❌        | -           | Comma-separated list of allowed origins for CORS<br/>If not set, allows all origins (development mode only!)<br/>Example: `https://app.example.com,https://admin.example.com` |

### Authentication

| Variable               | Required | Default | Description                                                                                                           |
| ---------------------- | -------- | ------- | --------------------------------------------------------------------------------------------------------------------- |
| `JWT_SECRET`           | ✅        | -       | Secret key for signing JWT tokens<br/>**Must be at least 32 characters**<br/>Generate with: `openssl rand -base64 32` |
| `JWT_EXPIRATION_HOURS` | ❌        | `24`    | Number of hours until JWT tokens expire<br/>Valid range: 1-720 (30 days)                                              |

### Cache (Redis)

| Variable            | Required | Default                  | Description                                                          |
| ------------------- | -------- | ------------------------ | -------------------------------------------------------------------- |
| `CACHE_ENABLED`     | ❌        | `true`                   | Enable/disable Redis caching<br/>Set to `false` to run without Redis |
| `REDIS_URL`         | ❌        | `redis://localhost:6379` | Redis connection string<br/>Only used if `CACHE_ENABLED=true`        |
| `CACHE_DEFAULT_TTL` | ❌        | `300`                    | Default cache TTL in seconds (5 minutes)                             |

### Observability

| Variable                      | Required | Default                 | Description                                                                                                                                                                           |
| ----------------------------- | -------- | ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `RUST_LOG`                    | ❌        | `info,sqlx=warn`        | Log level configuration<br/>Examples:<br/>- `debug` - Debug level for all modules<br/>- `info,sqlx=warn` - Info level, but warn for sqlx<br/>- `trace,sqlx=trace` - Maximum verbosity |
| `LOG_FORMAT`                  | ❌        | `text`                  | Log output format<br/>Options: `text` (human-readable) or `json` (structured)                                                                                                         |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | ❌        | `http://localhost:4317` | OpenTelemetry OTLP endpoint for distributed tracing                                                                                                                                   |
| `SENTRY_DSN`                  | ❌        | -                       | Sentry DSN for error tracking<br/>If not set, Sentry is disabled                                                                                                                      |

### Admin User Seeding

| Variable         | Required | Default | Description                                                                                                                                       |
| ---------------- | -------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ADMIN_USERNAME` | ❌        | -       | Email address for default admin user<br/>Only used if both username and password are set<br/>User is created on first startup if it doesn't exist |
| `ADMIN_PASSWORD` | ❌        | -       | Password for default admin user<br/>Only used if both username and password are set                                                               |

### Application Environment

| Variable  | Required | Default | Description                                                                                                                         |
| --------- | -------- | ------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| `APP_ENV` | ❌        | `local` | Application environment<br/>Options: `local`, `dev`, `staging`, `production`<br/>Affects Sentry sampling rates and logging behavior |

## Configuration Examples

### Development (.env)

```bash
# Database
DATABASE_URL=postgres://loupe:loupe@localhost:5432/loupe

# API Server
API_HOST=127.0.0.1
API_PORT=8080

# Authentication
JWT_SECRET=WfpDKGEdtWKFoGwmA1na6f5LmmpdY8Y41hK1WEHEJk4=
JWT_EXPIRATION_HOURS=24

# Cache (disabled for local development without Redis)
CACHE_ENABLED=false

# Logging
RUST_LOG=debug,sqlx=info
LOG_FORMAT=text

# Admin user (optional - creates admin@example.com on first run)
ADMIN_USERNAME=admin@example.com
ADMIN_PASSWORD=adminpassword

# App environment
APP_ENV=local
```

### Production (.env.prod)

```bash
# Database (use production credentials)
DATABASE_URL=postgres://loupe_prod:${DB_PASSWORD}@db.example.com:5432/loupe_prod

# API Server
API_HOST=0.0.0.0
API_PORT=8080

# CORS - Restrict to production frontend
CORS_ALLOWED_ORIGINS=https://loupe.example.com

# Authentication
JWT_SECRET=${JWT_SECRET}  # Load from secrets manager
JWT_EXPIRATION_HOURS=24

# Cache (enabled in production)
CACHE_ENABLED=true
REDIS_URL=redis://redis.example.com:6379
CACHE_DEFAULT_TTL=300

# Logging (JSON format for log aggregation)
RUST_LOG=info,sqlx=warn
LOG_FORMAT=json

# Observability
OTEL_EXPORTER_OTLP_ENDPOINT=https://otel.example.com:4317
SENTRY_DSN=https://abc123@o123.ingest.sentry.io/456

# NO admin seeding in production - create users via API
# ADMIN_USERNAME and ADMIN_PASSWORD should NOT be set

# App environment
APP_ENV=production
```

## Validation

The configuration is validated on startup. If any required variables are missing or invalid, the application will exit with a descriptive error message.

### Common Validation Errors

**JWT_SECRET too short:**

```
Configuration error: JWT_SECRET must be at least 32 characters long for security.
Current length: 16. Generate with: openssl rand -base64 32
```

**Invalid DATABASE_URL:**

```
Configuration error: DATABASE_URL must be a valid connection string
(e.g., postgres://user:pass@host:port/db), got: localhost:5432
```

**Invalid PORT:**

```
API_PORT must be a valid port number (0-65535)
```

## Security Best Practices

1. **Never commit .env files to version control**
   - Add `.env` to `.gitignore`
   - Use `.env.example` as a template without real secrets

2. **Use strong JWT secrets**
   - Generate with: `openssl rand -base64 32`
   - Rotate regularly in production
   - Never reuse across environments

3. **Restrict CORS in production**
   - Always set `CORS_ALLOWED_ORIGINS` in production
   - Never use wildcard (`*`) in production

4. **Use JSON logging in production**
   - Set `LOG_FORMAT=json` for structured logs
   - Easier to parse and aggregate

5. **Disable admin seeding in production**
   - Only use `ADMIN_USERNAME`/`ADMIN_PASSWORD` for local/dev
   - In production, create users through the API with proper onboarding

## Troubleshooting

### Application won't start

**Check configuration validation:**

```bash
cd be
cargo run --bin loupe-api
# Look for "Configuration error:" messages
```

**Common issues:**

- Missing required env vars (DATABASE_URL, JWT_SECRET)
- Invalid port numbers
- Malformed connection strings
- JWT secret too short

### Cache errors

**If you see "Cache manager initialization failed":**

```bash
# Option 1: Disable caching
CACHE_ENABLED=false

# Option 2: Start Redis
docker run -d -p 6379:6379 redis:7-alpine

# Option 3: Update REDIS_URL if Redis is elsewhere
REDIS_URL=redis://your-redis-host:6379
```

## Related Documentation

- [Deployment Guide](./DEPLOYMENT.md) (to be created)
- [Security Guide](./SECRETS_MANAGEMENT.md)
- [Database Configuration](./DATABASE_POOLING.md)
