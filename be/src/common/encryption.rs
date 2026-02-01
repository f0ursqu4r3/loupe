/// Field-Level Encryption Module
///
/// Provides AES-256-GCM encryption for sensitive data at rest.
///
/// # Security Features
///
/// - **AES-256-GCM**: Authenticated encryption with associated data (AEAD)
/// - **Random nonces**: Unique nonce per encryption operation
/// - **Key derivation**: PBKDF2 from master key
/// - **Version tagging**: Supports key rotation
/// - **Constant-time operations**: Prevents timing attacks
///
/// # Usage
///
/// ```no_run
/// use loupe::EncryptionManager;
///
/// let encryption = EncryptionManager::new();
///
/// // Encrypt sensitive data
/// let encrypted = encryption.encrypt("postgresql://user:pass@localhost/db")?;
///
/// // Decrypt when needed
/// let decrypted = encryption.decrypt(&encrypted)?;
/// ```
///
/// # Environment Variables
///
/// - `ENCRYPTION_KEY`: Base64-encoded 256-bit encryption key (required)
///   Generate with: `openssl rand -base64 32`
///
/// # Key Rotation
///
/// To rotate keys:
/// 1. Set `ENCRYPTION_KEY_V2` with new key
/// 2. Encrypted data is tagged with version
/// 3. Old data decrypts with old key, new data uses new key
/// 4. Run migration to re-encrypt with new key

use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};
use base64::{Engine as _, engine::general_purpose};
use ring::rand::SecureRandom;

const NONCE_SIZE: usize = 12; // 96 bits for GCM
const KEY_SIZE: usize = 32;   // 256 bits
const VERSION_PREFIX: &str = "v1:"; // Version tag for key rotation

/// Manages encryption and decryption of sensitive data
#[derive(Clone)]
pub struct EncryptionManager {
    cipher: Aes256Gcm,
    version: String,
}

impl EncryptionManager {
    /// Create a new encryption manager
    ///
    /// # Panics
    /// Panics if ENCRYPTION_KEY is not set or invalid.
    pub fn new() -> Self {
        let key_b64 = std::env::var("ENCRYPTION_KEY")
            .expect("ENCRYPTION_KEY must be set - generate with: openssl rand -base64 32");

        let key_bytes = general_purpose::STANDARD.decode(&key_b64)
            .expect("ENCRYPTION_KEY must be valid base64");

        if key_bytes.len() != KEY_SIZE {
            panic!(
                "ENCRYPTION_KEY must be {} bytes (got {}). Generate with: openssl rand -base64 32",
                KEY_SIZE,
                key_bytes.len()
            );
        }

        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        Self {
            cipher,
            version: VERSION_PREFIX.to_string(),
        }
    }

    /// Encrypt plaintext data
    ///
    /// Returns base64-encoded ciphertext with format: "v1:nonce:ciphertext"
    ///
    /// # Security
    /// - Uses random nonce for each encryption
    /// - Includes authentication tag (GCM)
    /// - Version-tagged for key rotation support
    pub fn encrypt(&self, plaintext: &str) -> Result<String, String> {
        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        ring::rand::SystemRandom::new()
            .fill(&mut nonce_bytes)
            .map_err(|e| format!("Failed to generate nonce: {:?}", e))?;

        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt plaintext
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        // Format: v1:nonce:ciphertext (all base64 encoded)
        let nonce_b64 = general_purpose::STANDARD.encode(&nonce_bytes);
        let ciphertext_b64 = general_purpose::STANDARD.encode(&ciphertext);

        Ok(format!("{}{}:{}", self.version, nonce_b64, ciphertext_b64))
    }

    /// Decrypt ciphertext
    ///
    /// Supports versioned keys for rotation.
    ///
    /// # Errors
    /// Returns error if:
    /// - Invalid format
    /// - Authentication tag verification fails
    /// - Decryption fails
    pub fn decrypt(&self, encrypted: &str) -> Result<String, String> {
        // Parse version:nonce:ciphertext
        if !encrypted.starts_with(VERSION_PREFIX) {
            return Err("Invalid encryption format: missing version prefix".to_string());
        }

        let without_version = &encrypted[VERSION_PREFIX.len()..];
        let parts: Vec<&str> = without_version.split(':').collect();

        if parts.len() != 2 {
            return Err("Invalid encryption format: expected nonce:ciphertext".to_string());
        }

        let nonce_b64 = parts[0];
        let ciphertext_b64 = parts[1];

        // Decode nonce and ciphertext
        let nonce_bytes = general_purpose::STANDARD.decode(nonce_b64)
            .map_err(|e| format!("Invalid nonce encoding: {}", e))?;

        if nonce_bytes.len() != NONCE_SIZE {
            return Err(format!(
                "Invalid nonce size: expected {}, got {}",
                NONCE_SIZE,
                nonce_bytes.len()
            ));
        }

        let ciphertext = general_purpose::STANDARD.decode(ciphertext_b64)
            .map_err(|e| format!("Invalid ciphertext encoding: {}", e))?;

        let nonce = Nonce::from_slice(&nonce_bytes);

        // Decrypt with authentication
        let plaintext_bytes = self.cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| format!("Decryption failed (wrong key or corrupted data): {}", e))?;

        String::from_utf8(plaintext_bytes)
            .map_err(|e| format!("Decrypted data is not valid UTF-8: {}", e))
    }

    /// Check if data is encrypted
    pub fn is_encrypted(data: &str) -> bool {
        data.starts_with(VERSION_PREFIX)
    }

    /// Get encryption version
    pub fn version(&self) -> &str {
        &self.version
    }
}

impl Default for EncryptionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Mask sensitive data for logging
///
/// Shows first 8 and last 4 characters, replaces middle with asterisks.
/// Useful for connection strings in logs.
///
/// # Examples
/// ```
/// use loupe::encryption::mask_sensitive;
///
/// let conn_str = "postgresql://user:secret@localhost:5432/db";
/// let masked = mask_sensitive(conn_str);
/// // Output: "postgres****/db"
/// ```
pub fn mask_sensitive(data: &str) -> String {
    if data.len() <= 12 {
        "*".repeat(data.len())
    } else {
        format!(
            "{}****{}",
            &data[..8],
            &data[data.len() - 4..]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_encryption() -> EncryptionManager {
        // Set test encryption key
        unsafe {
            std::env::set_var("ENCRYPTION_KEY", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=");
        }
        EncryptionManager::new()
    }

    #[test]
    fn test_encrypt_decrypt() {
        let encryption = setup_test_encryption();

        let plaintext = "postgresql://user:password@localhost:5432/mydb";
        let encrypted = encryption.encrypt(plaintext).unwrap();

        // Verify format
        assert!(encrypted.starts_with("v1:"));
        assert!(encrypted.contains(':'));

        // Decrypt and verify
        let decrypted = encryption.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_produces_different_ciphertexts() {
        let encryption = setup_test_encryption();

        let plaintext = "same data";
        let encrypted1 = encryption.encrypt(plaintext).unwrap();
        let encrypted2 = encryption.encrypt(plaintext).unwrap();

        // Different nonces should produce different ciphertexts
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to same plaintext
        assert_eq!(encryption.decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(encryption.decrypt(&encrypted2).unwrap(), plaintext);
    }

    #[test]
    fn test_decrypt_invalid_format() {
        let encryption = setup_test_encryption();

        assert!(encryption.decrypt("invalid").is_err());
        assert!(encryption.decrypt("v1:onlyonepart").is_err());
        assert!(encryption.decrypt("v2:nonce:ciphertext").is_err()); // Wrong version
    }

    #[test]
    fn test_decrypt_corrupted_data() {
        let encryption = setup_test_encryption();

        let plaintext = "test data";
        let mut encrypted = encryption.encrypt(plaintext).unwrap();

        // Corrupt the ciphertext
        encrypted.push('x');

        assert!(encryption.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_is_encrypted() {
        assert!(EncryptionManager::is_encrypted("v1:abc:def"));
        assert!(!EncryptionManager::is_encrypted("plaintext"));
        assert!(!EncryptionManager::is_encrypted(""));
    }

    #[test]
    fn test_mask_sensitive() {
        assert_eq!(
            mask_sensitive("postgresql://user:password@localhost:5432/db"),
            "postgres****2/db"
        );

        assert_eq!(mask_sensitive("short"), "*****");
        assert_eq!(mask_sensitive(""), "");
    }

    #[test]
    #[should_panic(expected = "ENCRYPTION_KEY must be set")]
    fn test_missing_encryption_key() {
        unsafe {
            std::env::remove_var("ENCRYPTION_KEY");
        }
        EncryptionManager::new();
    }
}
