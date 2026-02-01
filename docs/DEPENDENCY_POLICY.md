# Dependency Management Policy

## Overview

This document defines the dependency management strategy for the Loupe project, including update policies, security practices, and license compliance.

## Automated Tools

### 1. Dependabot

Automated dependency updates via GitHub Dependabot.

**Configuration:** [.github/dependabot.yml](../.github/dependabot.yml)

**Schedule:**
- **Weekly updates** every Monday at 9:00 AM ET
- Separate PRs for backend (Cargo), frontend (npm), GitHub Actions, and Docker
- Groups minor and patch updates together to reduce PR noise

**Review Process:**
1. Dependabot creates PR with dependency update
2. CI runs automated tests and security scans
3. Maintainer reviews changelog and breaking changes
4. Merge if tests pass and no breaking changes

### 2. cargo-audit

Security vulnerability scanner for Rust dependencies.

**CI Integration:** Runs on every push and PR

**Command:**
```bash
cd be && cargo audit
```

**Failure Policy:** Build fails on any security vulnerability

### 3. cargo-deny

Comprehensive dependency analysis tool.

**Configuration:** [be/deny.toml](../be/deny.toml)

**Checks:**
- **Security advisories** (RustSec database)
- **License compliance** (MIT, Apache-2.0, BSD allowed)
- **Banned crates** (none currently)
- **Duplicate versions** (warns on multiple versions)
- **Source validation** (crates.io only)

**CI Integration:**
```bash
cd be && cargo deny check
```

## Dependency Update Policy

### Semantic Versioning

Follow semantic versioning (SemVer) for all updates:

- **Patch updates (0.0.X)**: Bug fixes, security patches
  - **Policy:** Auto-merge if CI passes
  - **Risk:** Low
  - **Review:** Optional

- **Minor updates (0.X.0)**: New features, backward-compatible
  - **Policy:** Merge weekly batch
  - **Risk:** Medium
  - **Review:** Required (changelog review)

- **Major updates (X.0.0)**: Breaking changes
  - **Policy:** Manual review and testing
  - **Risk:** High
  - **Review:** Required (full testing)

### Update Cadence

| Type | Frequency | Batching |
|------|-----------|----------|
| Security patches | Immediate | No |
| Patch updates | Weekly | Yes (grouped) |
| Minor updates | Weekly | Yes (grouped) |
| Major updates | As needed | No (individual PRs) |

### Security Updates

**Critical vulnerabilities (CVSS >= 7.0):**
- Update within 24 hours
- Deploy hotfix if actively exploited
- Notify team via Slack/email

**High vulnerabilities (CVSS 4.0-6.9):**
- Update within 1 week
- Include in next release

**Medium/Low vulnerabilities:**
- Update in regular cycle
- Document in release notes

## License Compliance

### Allowed Licenses

✅ **Permissive licenses:**
- MIT
- Apache-2.0
- BSD-2-Clause / BSD-3-Clause
- ISC
- Unicode-DFS-2016
- OpenSSL (for crypto libraries)

❌ **Prohibited licenses:**
- GPL-2.0 / GPL-3.0 (copyleft)
- AGPL-3.0 (network copyleft)
- Any proprietary/commercial licenses

### License Review Process

1. **Automated check:** `cargo deny check licenses`
2. **Manual review:** For new dependencies with uncommon licenses
3. **Documentation:** Add license clarifications to `deny.toml`
4. **Legal approval:** Consult legal team for edge cases

## Pinning Strategy

### When to Pin Versions

Pin exact versions for:

1. **Security-critical dependencies:**
   - JWT libraries (jsonwebtoken)
   - Crypto libraries (argon2, ring)
   - Database drivers (sqlx)

2. **Unstable dependencies:**
   - Pre-1.0 crates with frequent breaking changes
   - Deprecated crates awaiting replacement

3. **Build reproducibility:**
   - CI/CD tooling
   - Docker base images

### Cargo.toml Version Syntax

```toml
# Allow patch updates (recommended for most deps)
actix-web = "4.5"         # >=4.5.0, <4.6.0

# Allow minor updates (for stable libraries)
serde = "1"               # >=1.0.0, <2.0.0

# Pin exact version (security-critical only)
jsonwebtoken = "=9.2.0"   # Exactly 9.2.0

# Range specification
uuid = ">=1.0, <2.0"      # More explicit range
```

### Cargo.lock

**Policy:** Always commit `Cargo.lock` to version control

**Rationale:**
- Ensures reproducible builds
- Locks transitive dependencies
- Prevents supply chain attacks

## Dependency Evaluation Criteria

Before adding a new dependency, evaluate:

### 1. Necessity
- [ ] Is this functionality required?
- [ ] Can it be implemented in-house reasonably?
- [ ] Does it add significant value vs. code bloat?

### 2. Maintenance
- [ ] Active development (commits in last 6 months)?
- [ ] Responsive maintainers (issue response time)?
- [ ] Regular releases?
- [ ] Good documentation?

### 3. Security
- [ ] No known vulnerabilities?
- [ ] Security audit history?
- [ ] Trusted author/organization?
- [ ] Minimal attack surface?

### 4. Quality
- [ ] High code quality?
- [ ] Comprehensive tests?
- [ ] Good API design?
- [ ] Minimal dependencies?

### 5. Compatibility
- [ ] Compatible license?
- [ ] Works with our Rust version?
- [ ] Cross-platform (if needed)?
- [ ] Performance acceptable?

### 6. Popularity
- [ ] Used by other projects?
- [ ] Download count (crates.io)?
- [ ] GitHub stars/forks?
- [ ] Community support?

**Scoring:** 5 or more "yes" answers required to add dependency.

## Removing Dependencies

### When to Remove

- Dependency becomes unmaintained (>12 months no commits)
- Security vulnerabilities with no fix
- Better alternative available
- Functionality no longer needed
- License incompatibility

### Removal Process

1. **Assess impact:** Search codebase for usage
2. **Plan replacement:** Implement alternative or in-house solution
3. **Test thoroughly:** Ensure no regressions
4. **Document:** Add to release notes

## Transitive Dependencies

### Monitoring

Use `cargo tree` to audit dependency tree:

```bash
# Show all dependencies
cargo tree

# Find specific dependency
cargo tree -i tokio

# Show duplicates
cargo tree -d
```

### Duplicate Version Management

**Policy:** Warn on duplicate versions, but don't fail builds

**Resolution strategies:**
1. Update parent dependency to use same version
2. Wait for ecosystem to converge
3. Document in `deny.toml` skip list if unavoidable

## Supply Chain Security

### Trust Model

1. **crates.io only:** Don't use git dependencies in production
2. **Verify checksums:** Cargo.lock ensures integrity
3. **Audit changes:** Review dependency diffs in PRs
4. **Scan for malware:** CI runs security checks

### Security Advisories

**Monitoring:**
- RustSec Advisory Database (automated via cargo-audit)
- GitHub Security Advisories
- Dependabot alerts

**Response SLA:**
- Critical: 24 hours
- High: 1 week
- Medium: 2 weeks
- Low: Next release cycle

## Documentation Requirements

### For Each Dependency

Document in code comments why dependency was added:

```rust
// Using actix-web for HTTP server
// - High performance async framework
// - Battle-tested in production
// - Active maintenance
// - MIT license
use actix_web::{App, HttpServer};
```

### Cargo.toml Organization

Group dependencies by category:

```toml
[dependencies]
# Web framework
actix-web = "4.5"
actix-cors = "0.7"

# Database
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Security
argon2 = "0.5"
jsonwebtoken = "9.2"
```

## CI/CD Integration

### Required Checks (Fail Build)
- ✅ `cargo audit` - Security vulnerabilities
- ✅ `cargo deny check advisories` - RustSec advisories
- ✅ `cargo deny check bans` - Banned crates

### Optional Checks (Warn Only)
- ⚠️ `cargo deny check licenses` - License compliance
- ⚠️ `cargo outdated` - Outdated dependencies
- ⚠️ `cargo tree -d` - Duplicate versions

## Quarterly Dependency Review

Every quarter, review all dependencies:

1. **Check for updates:** Run `cargo outdated`
2. **Review security:** Check RustSec advisories
3. **Assess usage:** Remove unused dependencies
4. **Evaluate alternatives:** Better libraries available?
5. **Update policy:** Refine based on learnings

**Schedule:** First Monday of Jan/Apr/Jul/Oct

## Tools Reference

### Cargo Commands

```bash
# Check for security vulnerabilities
cargo audit

# Check licenses and advisories
cargo deny check

# List outdated dependencies
cargo outdated

# Show dependency tree
cargo tree

# Update dependencies
cargo update

# Update specific dependency
cargo update -p serde

# Remove unused dependencies
cargo machete  # Install: cargo install cargo-machete
```

### CI Commands

```bash
# Full security scan
cargo audit && cargo deny check

# License check only
cargo deny check licenses

# Advisory check only
cargo deny check advisories
```

## Exceptions and Overrides

### Documenting Exceptions

When ignoring a security advisory or allowing a banned license:

```toml
# In deny.toml
[advisories]
ignore = [
    "RUSTSEC-2024-0001",  # False positive: not used in our context
]

[licenses.clarify]
[[licenses.clarify]]
name = "special-crate"
version = "1.0"
expression = "Custom-License"
# Justification: Legal approved on 2024-01-15
```

### Approval Process

1. Document justification in deny.toml
2. Get approval from tech lead
3. Add expiration date for review
4. Track in quarterly review

## Contact

**Security issues:** security@example.com
**Dependency questions:** #engineering on Slack
**Policy updates:** Submit PR to this document

## Revision History

- **2026-02-01:** Initial policy created
- **TBD:** Future updates...
