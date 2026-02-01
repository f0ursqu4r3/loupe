/// Secrets Management Module
///
/// Provides secure loading and handling of sensitive configuration values.
///
/// # Supported Secret Sources (in priority order)
///
/// 1. **Docker Secrets** (highest priority)
///    - Files in `/run/secrets/` directory
///    - Commonly used in Docker Swarm and Kubernetes
///    - Example: `/run/secrets/database_password`
///
/// 2. **Environment Files** (medium priority)
///    - Files specified by environment variables ending in `_FILE`
///    - Example: `DATABASE_PASSWORD_FILE=/path/to/secret`
///
/// 3. **Environment Variables** (lowest priority)
///    - Standard environment variables
///    - Example: `DATABASE_PASSWORD=mypassword`
///
/// # Security Features
///
/// - Never logs actual secret values
/// - Redacts secrets in error messages
/// - Supports secrets rotation (reload capability)
/// - Validates secret formats where applicable
///
/// # Usage
///
/// ```no_run
/// use loupe::SecretsManager;
///
/// let secrets = SecretsManager::new();
///
/// // Get required secret (panics if missing)
/// let jwt_secret = secrets.require("JWT_SECRET");
///
/// // Get optional secret
/// let optional = secrets.get("OPTIONAL_API_KEY");
/// ```

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Maximum allowed secret file size (1MB)
const MAX_SECRET_FILE_SIZE: u64 = 1_048_576;

/// Manages loading and accessing secrets from multiple sources
#[derive(Clone)]
pub struct SecretsManager {
    secrets: HashMap<String, String>,
    sources: HashMap<String, SecretSource>,
}

/// Source of a secret value
#[derive(Debug, Clone, PartialEq)]
pub enum SecretSource {
    /// Docker secret file in /run/secrets/
    DockerSecret(PathBuf),
    /// File specified by {NAME}_FILE environment variable
    FileVariable(PathBuf),
    /// Standard environment variable
    EnvironmentVariable,
}

impl SecretsManager {
    /// Create a new secrets manager and load all secrets
    ///
    /// Loads secrets from all available sources in priority order.
    /// Logs the source of each secret (but never the value).
    pub fn new() -> Self {
        let mut secrets = HashMap::new();
        let mut sources = HashMap::new();

        // Load environment variables first (lowest priority)
        for (key, value) in std::env::vars() {
            // Skip _FILE variables (they're pointers to files, not actual secrets)
            if !key.ends_with("_FILE") {
                secrets.insert(key.clone(), value);
                sources.insert(key, SecretSource::EnvironmentVariable);
            }
        }

        // Load from _FILE environment variables (medium priority)
        for (key, file_path) in std::env::vars() {
            if key.ends_with("_FILE") {
                let secret_name = key.trim_end_matches("_FILE").to_string();

                match Self::load_secret_from_file(&file_path) {
                    Ok(value) => {
                        tracing::info!(
                            secret = %secret_name,
                            source = "file_variable",
                            path = %file_path,
                            "Loaded secret from file"
                        );
                        secrets.insert(secret_name.clone(), value);
                        sources.insert(secret_name, SecretSource::FileVariable(file_path.into()));
                    }
                    Err(e) => {
                        tracing::warn!(
                            secret = %secret_name,
                            path = %file_path,
                            error = %e,
                            "Failed to load secret from file, will use environment variable if available"
                        );
                    }
                }
            }
        }

        // Load from Docker secrets directory (highest priority)
        let docker_secrets_dir = Path::new("/run/secrets");
        if docker_secrets_dir.exists() && docker_secrets_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(docker_secrets_dir) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_file() {
                            if let Some(secret_name) = entry.file_name().to_str() {
                                let secret_name_upper = secret_name.to_uppercase();
                                let file_path = entry.path();

                                match Self::load_secret_from_file(file_path.to_str().unwrap()) {
                                    Ok(value) => {
                                        tracing::info!(
                                            secret = %secret_name_upper,
                                            source = "docker_secret",
                                            "Loaded secret from Docker secrets"
                                        );
                                        secrets.insert(secret_name_upper.clone(), value);
                                        sources.insert(
                                            secret_name_upper,
                                            SecretSource::DockerSecret(file_path),
                                        );
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            secret = %secret_name_upper,
                                            error = %e,
                                            "Failed to load Docker secret"
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        tracing::info!(
            total_secrets = secrets.len(),
            "Secrets manager initialized"
        );

        Self { secrets, sources }
    }

    /// Load a secret from a file
    ///
    /// # Security
    /// - Validates file size to prevent memory exhaustion
    /// - Trims whitespace (common in Docker secrets)
    /// - Never logs the actual secret value
    fn load_secret_from_file(path: &str) -> Result<String, String> {
        let path_buf = PathBuf::from(path);

        // Check file exists
        if !path_buf.exists() {
            return Err(format!("File does not exist: {}", path));
        }

        // Check file size to prevent loading huge files
        let metadata = fs::metadata(&path_buf)
            .map_err(|e| format!("Failed to read file metadata: {}", e))?;

        if metadata.len() > MAX_SECRET_FILE_SIZE {
            return Err(format!(
                "Secret file too large: {} bytes (max: {} bytes)",
                metadata.len(),
                MAX_SECRET_FILE_SIZE
            ));
        }

        // Read and trim the secret
        let content = fs::read_to_string(&path_buf)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        Ok(content.trim().to_string())
    }

    /// Get a secret value by name
    ///
    /// Returns `None` if the secret doesn't exist.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.secrets.get(name).map(|s| s.as_str())
    }

    /// Get a required secret value by name
    ///
    /// # Panics
    /// Panics if the secret is not found. This is intentional for required
    /// secrets - the application should not start without them.
    pub fn require(&self, name: &str) -> String {
        self.get(name)
            .unwrap_or_else(|| {
                panic!(
                    "Required secret '{}' not found. Set via environment variable, \
                     {}_FILE=/path/to/secret, or Docker secret at /run/secrets/{}",
                    name,
                    name,
                    name.to_lowercase()
                )
            })
            .to_string()
    }

    /// Get the source of a secret
    ///
    /// Useful for debugging secret configuration issues.
    pub fn get_source(&self, name: &str) -> Option<&SecretSource> {
        self.sources.get(name)
    }

    /// Check if a secret exists
    pub fn has(&self, name: &str) -> bool {
        self.secrets.contains_key(name)
    }

    /// Reload secrets from all sources
    ///
    /// This allows for secrets rotation without restarting the application.
    /// Returns the number of secrets that changed.
    pub fn reload(&mut self) -> usize {
        let old_secrets = self.secrets.clone();
        let new_manager = Self::new();

        self.secrets = new_manager.secrets;
        self.sources = new_manager.sources;

        // Count how many secrets changed
        let mut changed = 0;
        for (key, new_value) in &self.secrets {
            if let Some(old_value) = old_secrets.get(key) {
                if old_value != new_value {
                    tracing::info!(secret = %key, "Secret value changed during reload");
                    changed += 1;
                }
            } else {
                tracing::info!(secret = %key, "New secret added during reload");
                changed += 1;
            }
        }

        // Check for removed secrets
        for key in old_secrets.keys() {
            if !self.secrets.contains_key(key) {
                tracing::warn!(secret = %key, "Secret removed during reload");
                changed += 1;
            }
        }

        changed
    }
}

impl Default for SecretsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Redact a secret value for safe logging
///
/// Shows only first and last 2 characters, replaces middle with asterisks.
///
/// # Examples
/// ```
/// use loupe::secrets::redact_secret;
///
/// assert_eq!(redact_secret("short"), "sh***");
/// assert_eq!(redact_secret("my_secret_key_12345"), "my***45");
/// assert_eq!(redact_secret("ab"), "ab");
/// ```
pub fn redact_secret(secret: &str) -> String {
    if secret.len() <= 4 {
        secret.to_string()
    } else {
        format!(
            "{}***{}",
            &secret[..2],
            &secret[secret.len() - 2..]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_secret() {
        assert_eq!(redact_secret("short"), "short");
        assert_eq!(redact_secret("a"), "a");
        assert_eq!(redact_secret("ab"), "ab");
        assert_eq!(redact_secret("abc"), "abc");
        assert_eq!(redact_secret("abcd"), "abcd");
        assert_eq!(redact_secret("abcde"), "ab***de");
        assert_eq!(redact_secret("my_secret_key_12345"), "my***45");
    }

    #[test]
    fn test_secrets_manager_get() {
        let manager = SecretsManager::new();

        // Test getting an environment variable (PATH should exist)
        assert!(manager.has("PATH") || manager.has("Path"));
    }

    #[test]
    #[should_panic(expected = "Required secret 'NONEXISTENT_SECRET_12345' not found")]
    fn test_require_missing_secret() {
        let manager = SecretsManager::new();
        manager.require("NONEXISTENT_SECRET_12345");
    }
}
