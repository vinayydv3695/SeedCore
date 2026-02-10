// Cryptographic operations for API key encryption

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::RngCore, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};

const NONCE_SIZE: usize = 12; // 96 bits for AES-GCM

/// Cryptographic manager for encrypting/decrypting API keys
pub struct CryptoManager {
    cipher: Aes256Gcm,
}

impl CryptoManager {
    /// Create a new CryptoManager from a master password
    ///
    /// # Arguments
    /// * `password` - The master password
    /// * `salt` - Salt for key derivation (must be 16+ bytes)
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self> {
        // Derive encryption key from password using Argon2
        let argon2 = Argon2::default();

        // Create a fixed-length salt for Argon2
        let salt_string =
            SaltString::encode_b64(salt).map_err(|e| anyhow!("Failed to encode salt: {}", e))?;

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?;

        // Extract 32-byte key from hash
        let hash_bytes = password_hash
            .hash
            .ok_or_else(|| anyhow!("No hash generated"))?;
        let key_bytes = hash_bytes.as_bytes();

        if key_bytes.len() < 32 {
            return Err(anyhow!("Derived key too short"));
        }

        // Create AES-256-GCM cipher
        let key = &key_bytes[..32];
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;

        Ok(Self { cipher })
    }

    /// Encrypt plaintext
    ///
    /// # Arguments
    /// * `plaintext` - The text to encrypt (e.g., API key)
    ///
    /// # Returns
    /// Tuple of (ciphertext, nonce)
    pub fn encrypt(&self, plaintext: &str) -> Result<(Vec<u8>, Vec<u8>)> {
        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        Ok((ciphertext, nonce_bytes.to_vec()))
    }

    /// Decrypt ciphertext
    ///
    /// # Arguments
    /// * `ciphertext` - The encrypted data
    /// * `nonce` - The nonce used during encryption
    ///
    /// # Returns
    /// Decrypted plaintext string
    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<String> {
        if nonce.len() != NONCE_SIZE {
            return Err(anyhow!("Invalid nonce size: {}", nonce.len()));
        }

        let nonce = Nonce::from_slice(nonce);

        // Decrypt
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        // Convert to string
        String::from_utf8(plaintext).map_err(|e| anyhow!("Invalid UTF-8: {}", e))
    }
}

/// Hash a master password for storage verification
pub fn hash_master_password(password: &str, salt: &[u8]) -> Result<Vec<u8>> {
    let argon2 = Argon2::default();

    let salt_string =
        SaltString::encode_b64(salt).map_err(|e| anyhow!("Failed to encode salt: {}", e))?;

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| anyhow!("Failed to hash password: {}", e))?;

    Ok(password_hash.to_string().into_bytes())
}

/// Verify a master password against a stored hash
pub fn verify_master_password(password: &str, hash: &[u8]) -> Result<bool> {
    let hash_str =
        String::from_utf8(hash.to_vec()).map_err(|e| anyhow!("Invalid hash UTF-8: {}", e))?;

    let password_hash =
        PasswordHash::new(&hash_str).map_err(|e| anyhow!("Failed to parse hash: {}", e))?;

    let argon2 = Argon2::default();

    Ok(argon2
        .verify_password(password.as_bytes(), &password_hash)
        .is_ok())
}

/// Generate a random salt for password hashing
pub fn generate_salt() -> Vec<u8> {
    let mut salt = vec![0u8; 32]; // 256 bits
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let password = "test_password_123";
        let salt = generate_salt();

        let crypto = CryptoManager::from_password(password, &salt).unwrap();

        let plaintext = "my_secret_api_key_12345";
        let (ciphertext, nonce) = crypto.encrypt(plaintext).unwrap();

        let decrypted = crypto.decrypt(&ciphertext, &nonce).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_wrong_password() {
        let password1 = "password1";
        let password2 = "password2";
        let salt = generate_salt();

        let crypto1 = CryptoManager::from_password(password1, &salt).unwrap();
        let crypto2 = CryptoManager::from_password(password2, &salt).unwrap();

        let plaintext = "secret_key";
        let (ciphertext, nonce) = crypto1.encrypt(plaintext).unwrap();

        // Trying to decrypt with wrong password should fail
        let result = crypto2.decrypt(&ciphertext, &nonce);
        assert!(result.is_err());
    }

    #[test]
    fn test_password_hashing() {
        let password = "my_master_password";
        let salt = generate_salt();

        let hash = hash_master_password(password, &salt).unwrap();

        // Correct password should verify
        assert!(verify_master_password(password, &hash).unwrap());

        // Wrong password should not verify
        assert!(!verify_master_password("wrong_password", &hash).unwrap());
    }
}
