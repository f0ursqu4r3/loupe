# Data Encryption Strategy

## Overview

Loupe implements field-level encryption for sensitive data at rest using **AES-256-GCM** (Galois/Counter Mode), an authenticated encryption algorithm that provides both confidentiality and integrity.

## Encrypted Fields

The following sensitive fields are encrypted in the database:

| Table | Field | Description |
|-------|-------|-------------|
| `datasources` | `connection_string_encrypted` | Database connection strings containing credentials |

Future candidates for encryption:
- API keys (when implemented)
- OAuth tokens (when implemented)
- Webhook secrets (when implemented)

## Encryption Algorithm

### AES-256-GCM

**Algorithm:** Advanced Encryption Standard with 256-bit keys
**Mode:** Galois/Counter Mode (AEAD - Authenticated Encryption with Associated Data)
**Key Size:** 256 bits (32 bytes)
**Nonce Size:** 96 bits (12 bytes)

**Security Properties:**
- ✅ **Confidentiality**: Data cannot be read without the key
- ✅ **Integrity**: Tampering is detected via authentication tag
- ✅ **Non-deterministic**: Each encryption produces different ciphertext (random nonce)
- ✅ **Authenticated**: Prevents modification attacks
- ✅ **Industry standard**: NIST-approved, used by TLS 1.3

### Encryption Format

Encrypted data is stored with the following format:

```
v1:nonce:ciphertext
```

- **v1**: Version tag for key rotation support
- **nonce**: Base64-encoded 96-bit random nonce
- **ciphertext**: Base64-encoded encrypted data + authentication tag

**Example:**
```
v1:SGVsbG8gV29ybGQ=:U29tZUVuY3J5cHRlZERhdGFIZXJl...
```

## Key Management

### Master Encryption Key

**Environment Variable:** `ENCRYPTION_KEY`
**Format:** Base64-encoded 256-bit key
**Generation:**
```bash
openssl rand -base64 32
```

**Example:**
```bash
export ENCRYPTION_KEY="AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
```

### Security Requirements

1. **Never commit keys to version control**
   - Use `.gitignore` for `.env` files
   - Use Docker secrets or vault in production

2. **Unique keys per environment**
   - Development: Local `.env` file
   - Staging: Docker secrets or vault
   - Production: HashiCorp Vault, AWS KMS, or Azure Key Vault

3. **Key rotation**
   - Rotate keys annually or after suspected compromise
   - Use versioned encryption to support gradual migration

### Storage Options

#### Development
```bash
# .env file (never commit!)
ENCRYPTION_KEY=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
```

#### Docker Secrets
```bash
# Generate key
openssl rand -base64 32 | docker secret create encryption_key -

# Use in docker-compose.yml
services:
  api:
    secrets:
      - encryption_key
    environment:
      ENCRYPTION_KEY_FILE: /run/secrets/encryption_key
```

#### HashiCorp Vault
```bash
# Store key in Vault
vault kv put secret/loupe/prod/encryption key="$(openssl rand -base64 32)"

# Retrieve at runtime
export ENCRYPTION_KEY=$(vault kv get -field=key secret/loupe/prod/encryption)
```

#### AWS Secrets Manager
```bash
# Store key
aws secretsmanager create-secret \
  --name loupe/prod/encryption-key \
  --secret-string "$(openssl rand -base64 32)"

# Retrieve at runtime (using AWS SDK)
```

## Key Rotation

### Strategy

The encryption module supports versioned keys for zero-downtime rotation:

1. **Add new key:**
   ```bash
   export ENCRYPTION_KEY_V2="NEW_KEY_HERE"
   ```

2. **Update code to support both keys:**
   ```rust
   // Decrypt with old key (v1) or new key (v2)
   match version {
       "v1" => decrypt_with_v1_key(data),
       "v2" => decrypt_with_v2_key(data),
       _ => Err("Unknown version")
   }
   ```

3. **Re-encrypt data:**
   ```sql
   -- Migration to re-encrypt with new key
   UPDATE datasources
   SET connection_string_encrypted = encrypt_with_v2(
       decrypt_with_v1(connection_string_encrypted)
   );
   ```

4. **Remove old key** after all data is migrated

### Rotation Schedule

- **Normal rotation:** Annually
- **Emergency rotation:** Immediately upon suspected compromise
- **Compliance rotation:** Per your organization's security policy

### Rotation Checklist

- [ ] Generate new encryption key
- [ ] Store new key in secrets manager
- [ ] Deploy code with dual-key support
- [ ] Run migration to re-encrypt data
- [ ] Verify all data decrypts with new key
- [ ] Remove old key from secrets manager
- [ ] Update documentation

## Usage

### Encrypting Data

```rust
use loupe::EncryptionManager;

let encryption = EncryptionManager::new();

// Encrypt a connection string
let plaintext = "postgresql://user:password@localhost:5432/db";
let encrypted = encryption.encrypt(plaintext)?;

// Store encrypted value in database
datasource.connection_string_encrypted = encrypted;
```

### Decrypting Data

```rust
// Retrieve from database
let encrypted = datasource.connection_string_encrypted;

// Decrypt when needed
let decrypted = encryption.decrypt(&encrypted)?;

// Use the connection string
let pool = sqlx::postgres::PgPool::connect(&decrypted).await?;
```

### Checking if Data is Encrypted

```rust
if EncryptionManager::is_encrypted(&value) {
    // Decrypt before use
    let plaintext = encryption.decrypt(&value)?;
} else {
    // Already plaintext (migration in progress)
    let plaintext = value;
}
```

## Data Masking in Logs

To prevent accidental leakage of sensitive data in logs:

### Connection Strings

```rust
use loupe::encryption::mask_sensitive;

let conn_str = "postgresql://user:secret@localhost:5432/db";
tracing::info!("Connecting to: {}", mask_sensitive(&conn_str));
// Output: "Connecting to: postgres****/db"
```

### Secrets

```rust
use loupe::redact_secret;

let api_key = "sk_live_abc123def456";
tracing::info!("API key: {}", redact_secret(api_key));
// Output: "API key: sk***56"
```

### Error Messages

Never log encrypted data or decrypted secrets in error messages:

```rust
// ❌ BAD: Leaks connection string
tracing::error!("Failed to connect to {}: {}", conn_str, e);

// ✅ GOOD: Generic error message
tracing::error!("Database connection failed: {}", e);
tracing::debug!("Connection: {}", mask_sensitive(&conn_str));
```

## Security Considerations

### Encryption Key Protection

1. **Never log the encryption key**
   ```rust
   // ❌ NEVER DO THIS
   tracing::debug!("Encryption key: {}", encryption_key);

   // ✅ DO THIS
   tracing::info!("Encryption manager initialized");
   ```

2. **Restrict access**
   - Limit who can read secrets in production
   - Use IAM roles/policies for cloud secrets
   - Audit access to encryption keys

3. **Monitor for leaks**
   - Run Gitleaks in CI/CD (already configured)
   - Scan logs for Base64-encoded keys
   - Rotate immediately if leaked

### Nonce Uniqueness

Each encryption operation generates a random nonce. This ensures:
- Same plaintext encrypts to different ciphertext
- Prevents pattern analysis
- Required for GCM security

**DO NOT:**
- Reuse nonces with the same key
- Use predictable nonces (counters, timestamps)

The `EncryptionManager` handles this automatically using a cryptographically secure random number generator.

### Authentication Tag

GCM includes a 128-bit authentication tag that:
- Detects tampering
- Prevents bit-flipping attacks
- Ensures data integrity

If the tag verification fails, decryption returns an error.

## Backup and Recovery

### Encrypted Backups

Database backups contain encrypted data:
- ✅ **Secure at rest**: Data is encrypted even in backups
- ⚠️ **Requires key**: Must have encryption key to restore

### Key Backup

**Critical:** Back up encryption keys securely

```bash
# Backup to encrypted file
vault kv get -field=key secret/loupe/prod/encryption | \
  gpg --encrypt --recipient admin@example.com > encryption-key.gpg

# Store in secure location (offline, hardware security module)
```

### Disaster Recovery

1. **Restore database** from backup
2. **Retrieve encryption key** from secure backup
3. **Set ENCRYPTION_KEY** environment variable
4. **Start application** - data decrypts transparently

## Compliance

### Standards

- **GDPR**: Encryption satisfies "appropriate technical measures"
- **HIPAA**: AES-256 is approved for PHI protection
- **PCI-DSS**: Meets encryption requirements for cardholder data
- **SOC 2**: Demonstrates security controls

### Audit Trail

Log encryption-related events:
```rust
tracing::info!(
    datasource_id = %id,
    "Connection string encrypted for datasource"
);

tracing::warn!(
    datasource_id = %id,
    "Connection string decryption failed - possible key mismatch"
);
```

## Troubleshooting

### Decryption Fails

**Symptom:** `Decryption failed (wrong key or corrupted data)`

**Causes:**
1. Wrong encryption key
2. Data corrupted in database
3. Key rotation in progress

**Resolution:**
```bash
# Verify encryption key is set
echo $ENCRYPTION_KEY

# Check if key rotation occurred
# Try decrypting with old key

# Verify database value format
SELECT id, LEFT(connection_string_encrypted, 20)
FROM datasources
LIMIT 1;
-- Should start with "v1:"
```

### Key Not Found

**Symptom:** `ENCRYPTION_KEY must be set`

**Resolution:**
```bash
# Generate new key
openssl rand -base64 32

# Set environment variable
export ENCRYPTION_KEY="..."

# Or use Docker secrets
ENCRYPTION_KEY_FILE=/run/secrets/encryption_key
```

### Migration Issues

**Symptom:** Some records encrypted, others not

**Cause:** Gradual migration in progress

**Resolution:**
```rust
// Handle both encrypted and plaintext during migration
let connection_string = if EncryptionManager::is_encrypted(&raw_value) {
    encryption.decrypt(&raw_value)?
} else {
    raw_value  // Still plaintext
};
```

## Future Enhancements

- [ ] Support for AWS KMS envelope encryption
- [ ] Automatic key rotation with zero downtime
- [ ] Field-level encryption for query results cache
- [ ] Encryption for API keys and OAuth tokens
- [ ] Hardware Security Module (HSM) integration
- [ ] Key derivation from multiple sources (split-key)

## References

- [NIST SP 800-38D: GCM Mode](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38d.pdf)
- [RFC 5116: AEAD Cipher Suites](https://tools.ietf.org/html/rfc5116)
- [OWASP Cryptographic Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html)
- [Rust Crypto: AES-GCM](https://github.com/RustCrypto/AEADs/tree/master/aes-gcm)

## Contact

**Security questions:** security@example.com
**Key rotation requests:** #security on Slack
**Encryption issues:** Open GitHub issue with `[security]` tag
