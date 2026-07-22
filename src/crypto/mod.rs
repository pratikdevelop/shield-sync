use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use std::path::Path;
use crate::error::{Result, ShieldSyncError};

const KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 24;

/// Master key derived from user password
#[derive(Clone)]
pub struct MasterKey {
    key: [u8; KEY_SIZE],
}

impl MasterKey {
    /// Derive master key from password using Argon2id
    pub fn derive_from_password(password: &str, salt: &[u8]) -> Result<Self> {
        let argon2 = Argon2::default();
        
        // Hash the password
        let salt_string = SaltString::encode_b64(salt)
            .map_err(|e| ShieldSyncError::Encryption(e.to_string()))?;
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| ShieldSyncError::Encryption(e.to_string()))?;
        
        // Extract the hash as our key
        let hash_bytes = password_hash.hash
            .ok_or_else(|| ShieldSyncError::Encryption("Failed to generate hash".to_string()))?;
        
        let mut key = [0u8; KEY_SIZE];
        key.copy_from_slice(&hash_bytes.as_bytes()[..KEY_SIZE]);
        
        Ok(Self { key })
    }

    /// Generate a random salt for key derivation
    pub fn generate_salt() -> [u8; 16] {
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);
        salt
    }

    /// Get the raw key bytes
    pub fn as_bytes(&self) -> &[u8; KEY_SIZE] {
        &self.key
    }
}

/// Handles all encryption/decryption operations
pub struct CryptoEngine {
    master_key: MasterKey,
}

impl CryptoEngine {
    pub fn new(master_key: MasterKey) -> Self {
        Self { master_key }
    }

    /// Encrypt data using ChaCha20-Poly1305
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new(self.master_key.as_bytes().into());
        
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12]; // ChaCha20Poly1305 uses 12-byte nonce
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| ShieldSyncError::Encryption(e.to_string()))?;
        
        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    /// Decrypt data
    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        if encrypted_data.len() < 12 {
            return Err(ShieldSyncError::Decryption(
                "Encrypted data too short".to_string(),
            ));
        }

        let cipher = ChaCha20Poly1305::new(self.master_key.as_bytes().into());
        
        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        // Decrypt
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| ShieldSyncError::Decryption(e.to_string()))?;
        
        Ok(plaintext)
    }

    /// Encrypt a file in chunks
    pub async fn encrypt_file(&self, path: &Path) -> Result<Vec<u8>> {
        let data = tokio::fs::read(path).await?;
        self.encrypt(&data)
    }

    /// Decrypt a file
    pub async fn decrypt_file(&self, encrypted_data: &[u8], output_path: &Path) -> Result<()> {
        let plaintext = self.decrypt(encrypted_data)?;
        tokio::fs::write(output_path, plaintext).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_key_derivation() {
        let password = "test_password_123";
        let salt = MasterKey::generate_salt();
        
        let key1 = MasterKey::derive_from_password(password, &salt).unwrap();
        let key2 = MasterKey::derive_from_password(password, &salt).unwrap();
        
        // Same password + salt should produce same key
        assert_eq!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password";
        let salt = MasterKey::generate_salt();
        let master_key = MasterKey::derive_from_password(&password, &salt).unwrap();
        
        let engine = CryptoEngine::new(master_key);
        let plaintext = b"Hello, Shield-Sync!";
        
        let encrypted = engine.encrypt(plaintext).unwrap();
        let decrypted = engine.decrypt(&encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[tokio::test]
    async fn test_file_encryption() {
        let password = "file_password";
        let salt = MasterKey::generate_salt();
        let master_key = MasterKey::derive_from_password(&password, &salt).unwrap();
        
        let engine = CryptoEngine::new(master_key);
        
        // Create temp file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Secret file content").unwrap();
        let temp_path = temp_file.path();
        
        // Encrypt
        let encrypted = engine.encrypt_file(temp_path).await.unwrap();
        
        // Decrypt to new file
        let output_file = NamedTempFile::new().unwrap();
        let output_path = output_file.path();
        engine.decrypt_file(&encrypted, output_path).await.unwrap();
        
        // Verify content
        let decrypted_content = tokio::fs::read(output_path).await.unwrap();
        assert_eq!(b"Secret file content", decrypted_content.as_slice());
    }
}