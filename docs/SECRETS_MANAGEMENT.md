# Secrets Management

## Overview

Loupe implements secure secrets management with support for multiple secret sources, automatic rotation, and comprehensive security controls to prevent secret leakage.

## Supported Secret Sources

Secrets are loaded in priority order (highest to lowest):

### 1. Docker Secrets (Highest Priority)

Files in `/run/secrets/` directory (Docker Swarm, Kubernetes)

**Example:**
```bash
# Create a Docker secret
echo "my-super-secret-password" | docker secret create database_password -

# Use in docker-compose.yml
services:
  api:
    secrets:
      - database_password

secrets:
  database_password:
    external: true
```

**Secret naming:** File name is converted to uppercase for environment variable compatibility.
- File: `/run/secrets/database_password` → Variable: `DATABASE_PASSWORD`

### 2. File-Based Secrets (Medium Priority)

Environment variables ending in `_FILE` that point to secret files.

**Example:**
```bash
# Store secret in a file
echo "my-jwt-secret" > /var/secrets/jwt_secret.txt

# Point to it with environment variable
export JWT_SECRET_FILE=/var/secrets/jwt_secret.txt
```

**Benefits:**
- Works with any orchestration platform
- Compatible with Kubernetes ConfigMaps/Secrets mounted as files
- Easier to rotate than environment variables

### 3. Environment Variables (Lowest Priority)

Standard environment variables.

**Example:**
```bash
export JWT_SECRET=my-jwt-secret
export DATABASE_URL=postgresql://user:pass@localhost/db
```

## Required Secrets

The following secrets must be configured for the application to start:

| Secret | Description | Example |
|--------|-------------|---------|
| `JWT_SECRET` | JWT token signing key (min 32 chars) | Generate with: `openssl rand -base64 32` |
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://user:pass@localhost:5432/loupe` |

## Optional Secrets

| Secret | Description | Default |
|--------|-------------|---------|
| `REDIS_URL` | Redis connection string | `redis://localhost:6379` |
| `SENTRY_DSN` | Sentry error tracking DSN | None (disabled) |
| `ADMIN_PASSWORD` | Default admin password (first run only) | None |

## Secrets Rotation

### Automatic Rotation Support

The `SecretsManager` supports hot-reloading of secrets without application restart:

```rust
use loupe::SecretsManager;

let mut secrets = SecretsManager::new();

// Reload secrets from all sources
let changed = secrets.reload();
tracing::info!("{} secrets changed during rotation", changed);
```

### Manual Rotation Process

#### 1. Docker Secrets Rotation

```bash
# Create new secret version
echo "new-secret-value" | docker secret create database_password_v2 -

# Update service to use new secret
docker service update \
  --secret-rm database_password \
  --secret-add source=database_password_v2,target=database_password \
  loupe-api

# Remove old secret (after verification)
docker secret rm database_password
```

#### 2. File-Based Secrets Rotation

```bash
# Update secret file atomically
echo "new-secret-value" > /var/secrets/jwt_secret.txt.new
mv /var/secrets/jwt_secret.txt.new /var/secrets/jwt_secret.txt

# Send SIGHUP to trigger reload (if implemented)
pkill -HUP loupe-api
```

#### 3. Environment Variable Rotation

Requires application restart (not recommended for production).

### Rotation Best Practices

1. **Use Versioned Secrets**
   - Name secrets with versions: `db_password_v1`, `db_password_v2`
   - Makes rollback easier

2. **Test New Secrets**
   - Deploy to staging first
   - Verify connectivity before production deployment

3. **Gradual Rollout**
   - Use blue-green deployment
   - Keep old secrets active during transition

4. **Automated Rotation**
   - Schedule regular rotation (30-90 days)
   - Use HashiCorp Vault, AWS Secrets Manager, or similar

## Security Features

### 1. Secret Redaction

Secrets are automatically redacted in logs:

```rust
use loupe::redact_secret;

let secret = "my_secret_key_12345";
tracing::info!("Secret loaded: {}", redact_secret(secret));
// Output: "Secret loaded: my***45"
```

### 2. No Logging of Secret Values

The `SecretsManager` never logs actual secret values:
- ✅ Logs the source (file path, environment variable)
- ❌ Never logs the actual secret value

### 3. File Size Validation

Secret files are limited to 1MB to prevent memory exhaustion attacks.

### 4. Error Message Sanitization

Connection errors don't expose credentials:
```rust
// ✅ Good: Generic error message
Error::Database("Failed to connect. Check configuration.")

// ❌ Bad: Exposes credentials
Error::Database(format!("Failed to connect to {}", database_url))
```

## CI/CD Secrets Scanning

### GitHub Actions - Gitleaks

Add to `.github/workflows/security.yml`:

```yaml
name: Security Scan

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  secrets-scan:
    name: Scan for Secrets
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full history for gitleaks

      - name: Run Gitleaks
        uses: gitleaks/gitleaks-action@v2
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          GITLEAKS_LICENSE: ${{ secrets.GITLEAKS_LICENSE }}
```

### Pre-Commit Hook

Install locally for developers:

```bash
# Install gitleaks
brew install gitleaks  # macOS
# or
wget https://github.com/gitleaks/gitleaks/releases/download/v8.18.0/gitleaks_8.18.0_linux_x64.tar.gz
tar -xzf gitleaks_8.18.0_linux_x64.tar.gz
sudo mv gitleaks /usr/local/bin/

# Add pre-commit hook
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/sh
gitleaks protect --staged --verbose --redact
EOF

chmod +x .git/hooks/pre-commit
```

### GitLab CI

Add to `.gitlab-ci.yml`:

```yaml
secrets-scan:
  stage: test
  image: zricethezav/gitleaks:latest
  script:
    - gitleaks detect --verbose --redact --no-git
  allow_failure: false
```

## Troubleshooting

### Secret Not Found

```
Required secret 'JWT_SECRET' not found. Set via environment variable,
JWT_SECRET_FILE=/path/to/secret, or Docker secret at /run/secrets/jwt_secret
```

**Solutions:**
1. Check environment variable is set: `echo $JWT_SECRET`
2. Verify file exists: `ls -la /run/secrets/jwt_secret`
3. Check file permissions: Ensure the application can read the file
4. Verify Docker secret is created: `docker secret ls`

### Secret Value Incorrect

The `SecretsManager` trims whitespace from file-based secrets. If your secret contains intentional whitespace, use environment variables instead.

### Rotation Not Working

1. Verify new secret file is readable: `cat /run/secrets/database_password`
2. Check application logs for reload messages
3. Ensure service has access to new secret (Docker/Kubernetes)

## Migration Guide

### From Environment Variables to Docker Secrets

**Before:**
```yaml
environment:
  - JWT_SECRET=my-super-secret-key
  - DATABASE_URL=postgresql://user:pass@db:5432/loupe
```

**After:**
```yaml
secrets:
  - jwt_secret
  - database_url

services:
  api:
    secrets:
      - jwt_secret
      - database_url
```

Create secrets:
```bash
echo "my-super-secret-key" | docker secret create jwt_secret -
echo "postgresql://user:pass@db:5432/loupe" | docker secret create database_url -
```

### From Plaintext Files to Encrypted Vault

1. **Export secrets to Vault:**
   ```bash
   vault kv put secret/loupe/prod/jwt_secret value="$(cat /run/secrets/jwt_secret)"
   vault kv put secret/loupe/prod/database_url value="$(cat /run/secrets/database_url)"
   ```

2. **Update application to use Vault:**
   ```bash
   # Future enhancement: Vault integration
   export VAULT_ADDR=https://vault.example.com
   export VAULT_TOKEN=s.xxxxxxxxxxxxx
   ```

## Future Enhancements

- [ ] HashiCorp Vault integration
- [ ] AWS Secrets Manager support
- [ ] Azure Key Vault support
- [ ] Automatic rotation API endpoint (POST /admin/rotate-secrets)
- [ ] Secret versioning and rollback
- [ ] Audit logging for secret access

## References

- [Docker Secrets Documentation](https://docs.docker.com/engine/swarm/secrets/)
- [Kubernetes Secrets](https://kubernetes.io/docs/concepts/configuration/secret/)
- [OWASP Secrets Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html)
- [Gitleaks](https://github.com/gitleaks/gitleaks)
