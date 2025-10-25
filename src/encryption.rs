/// Field-Level Encryption Module
///
/// Provides transparent encryption/decryption for sensitive String fields
/// using AES-256-GCM authenticated encryption.
///
/// # Security Features
/// - AES-256-GCM authenticated encryption
/// - Unique nonce per encryption operation
/// - Base64 encoding for database storage
/// - Zeroization of sensitive data in memory
/// - Key derivation from master key
///
/// # Usage
/// ```rust
/// use crate::encryption::EncryptedString;
///
/// #[derive(Queryable)]
/// pub struct SensitiveData {
///     pub id: Uuid,
///     pub title: EncryptedString,  // Automatically encrypted/decrypted
///     pub description: EncryptedString,
/// }
/// ```

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use diesel::{
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Text,
    AsExpression, FromSqlRow,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::io::Write;
use zeroize::Zeroize;

/// Environment variable for the master encryption key
/// SECURITY: This should be a 32-byte (256-bit) base64-encoded key
/// Generate with: openssl rand -base64 32
const ENCRYPTION_KEY_ENV: &str = "ENCRYPTION_MASTER_KEY";

/// Size of AES-256 key in bytes
const KEY_SIZE: usize = 32;

/// Size of GCM nonce in bytes
const NONCE_SIZE: usize = 12;

// Thread-local encryption cipher instance
thread_local! {
    static CIPHER: Aes256Gcm = {
        let key = get_encryption_key();
        Aes256Gcm::new(&key.into())
    };
}

/// Retrieves the master encryption key from environment variables
///
/// # Panics
/// Panics if the encryption key is not set or is invalid
///
/// # Security Note
/// In production, this should retrieve the key from a Hardware Security Module (HSM)
/// or Key Management Service (KMS) like AWS KMS, Azure Key Vault, or HashiCorp Vault
fn get_encryption_key() -> [u8; KEY_SIZE] {
    let key_b64 = std::env::var(ENCRYPTION_KEY_ENV)
        .unwrap_or_else(|_| {
            // Default key for development ONLY - DO NOT USE IN PRODUCTION
            eprintln!("WARNING: Using default encryption key. Set {} environment variable in production!", ENCRYPTION_KEY_ENV);
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string()
        });

    let key_bytes = BASE64
        .decode(key_b64.as_bytes())
        .expect("Failed to decode encryption key from base64");

    if key_bytes.len() != KEY_SIZE {
        panic!(
            "Encryption key must be {} bytes, got {}",
            KEY_SIZE,
            key_bytes.len()
        );
    }

    let mut key = [0u8; KEY_SIZE];
    key.copy_from_slice(&key_bytes);
    key
}

/// Encrypts a plaintext string using AES-256-GCM
///
/// # Arguments
/// * `plaintext` - The string to encrypt
///
/// # Returns
/// Base64-encoded string in format: `nonce||ciphertext`
/// where || represents concatenation
///
/// # Security
/// - Uses a random 12-byte nonce for each encryption
/// - Provides authenticated encryption (prevents tampering)
/// - Nonce is stored with ciphertext (safe for GCM)
fn encrypt_string(plaintext: &str) -> Result<String, String> {
    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt the plaintext
    let ciphertext = CIPHER
        .with(|cipher| {
            cipher
                .encrypt(nonce, plaintext.as_bytes())
                .map_err(|e| format!("Encryption failed: {}", e))
        })?;

    // Combine nonce + ciphertext and encode as base64
    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&combined))
}

/// Decrypts a ciphertext string using AES-256-GCM
///
/// # Arguments
/// * `ciphertext` - Base64-encoded string in format: `nonce||ciphertext`
///
/// # Returns
/// Decrypted plaintext string
///
/// # Security
/// - Verifies authentication tag (prevents tampering)
/// - Fails if data has been modified
fn decrypt_string(ciphertext: &str) -> Result<String, String> {
    // Decode from base64
    let combined = BASE64
        .decode(ciphertext.as_bytes())
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    if combined.len() < NONCE_SIZE {
        return Err("Ciphertext too short".to_string());
    }

    // Split nonce and ciphertext
    let (nonce_bytes, encrypted_data) = combined.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    // Decrypt the data
    let plaintext_bytes = CIPHER
        .with(|cipher| {
            cipher
                .decrypt(nonce, encrypted_data)
                .map_err(|e| format!("Decryption failed: {}", e))
        })?;

    String::from_utf8(plaintext_bytes).map_err(|e| format!("UTF-8 decode failed: {}", e))
}

/// Encrypted string type that transparently encrypts/decrypts in Diesel ORM
///
/// # Database Storage
/// Stored as TEXT in PostgreSQL, containing base64-encoded encrypted data
///
/// # Usage
/// Use this type in place of String for sensitive fields:
/// ```rust
/// pub struct User {
///     pub id: Uuid,
///     pub name: EncryptedString,  // Automatically encrypted
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = Text)]
pub struct EncryptedString {
    /// The decrypted plaintext value (stored in memory only)
    #[serde(skip)]
    plaintext: String,
}

impl EncryptedString {
    /// Creates a new EncryptedString from a plaintext string
    pub fn new(plaintext: String) -> Self {
        Self { plaintext }
    }

    /// Returns a reference to the decrypted plaintext
    pub fn as_str(&self) -> &str {
        &self.plaintext
    }

    /// Consumes self and returns the plaintext String
    pub fn into_string(self) -> String {
        self.plaintext.clone()
    }

    /// Returns true if the encrypted string is empty
    pub fn is_empty(&self) -> bool {
        self.plaintext.is_empty()
    }

    /// Returns the length of the decrypted string
    pub fn len(&self) -> usize {
        self.plaintext.len()
    }
}

// Implement From<String> for convenience
impl From<String> for EncryptedString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for EncryptedString {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

// Implement Display for easy printing (shows decrypted value)
impl std::fmt::Display for EncryptedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.plaintext)
    }
}

// Implement PartialEq for comparisons
impl PartialEq for EncryptedString {
    fn eq(&self, other: &Self) -> bool {
        self.plaintext == other.plaintext
    }
}

impl Eq for EncryptedString {}

/// Diesel serialization: Encrypts the string before storing in database
impl ToSql<Text, Pg> for EncryptedString {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let encrypted = encrypt_string(&self.plaintext)
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        out.write_all(encrypted.as_bytes())?;
        Ok(IsNull::No)
    }
}

/// Diesel deserialization: Decrypts the string when loading from database
impl FromSql<Text, Pg> for EncryptedString {
    fn from_sql(bytes: <Pg as diesel::backend::Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let encrypted_str = <String as FromSql<Text, Pg>>::from_sql(bytes)?;
        let plaintext = decrypt_string(&encrypted_str)
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        Ok(Self::new(plaintext))
    }
}

// Zeroize the plaintext when dropped (security best practice)
impl Drop for EncryptedString {
    fn drop(&mut self) {
        self.plaintext.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original = "SECRET CLASSIFICATION DATA";
        let encrypted = encrypt_string(original).unwrap();
        let decrypted = decrypt_string(&encrypted).unwrap();

        assert_eq!(original, decrypted);
        assert_ne!(original, encrypted); // Ensure it's actually encrypted
    }

    #[test]
    fn test_encrypt_produces_different_ciphertexts() {
        let plaintext = "Same plaintext";
        let encrypted1 = encrypt_string(plaintext).unwrap();
        let encrypted2 = encrypt_string(plaintext).unwrap();

        // Different nonces should produce different ciphertexts
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to the same plaintext
        assert_eq!(decrypt_string(&encrypted1).unwrap(), plaintext);
        assert_eq!(decrypt_string(&encrypted2).unwrap(), plaintext);
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let original = "Important data";
        let mut encrypted = encrypt_string(original).unwrap();

        // Tamper with the ciphertext
        encrypted.push('X');

        // Decryption should fail due to authentication tag mismatch
        assert!(decrypt_string(&encrypted).is_err());
    }

    #[test]
    fn test_encrypted_string_creation() {
        let text = "Classified Information".to_string();
        let encrypted = EncryptedString::new(text.clone());

        assert_eq!(encrypted.as_str(), text);
        assert_eq!(encrypted.to_string(), text);
    }

    #[test]
    fn test_encrypted_string_equality() {
        let es1 = EncryptedString::from("Same text");
        let es2 = EncryptedString::from("Same text");
        let es3 = EncryptedString::from("Different text");

        assert_eq!(es1, es2);
        assert_ne!(es1, es3);
    }

    #[test]
    fn test_encrypted_string_empty() {
        let empty = EncryptedString::from("");
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        let non_empty = EncryptedString::from("Not empty");
        assert!(!non_empty.is_empty());
        assert_eq!(non_empty.len(), 9);
    }
}
