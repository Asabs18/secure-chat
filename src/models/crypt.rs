// Model: Cryptographic engine for message encryption/decryption
// Implements AES-256-GCM authenticated encryption
// Security: Private key never exposed, only encrypt/decrypt methods are public

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce, Key,
};
use rand::Rng;

/// Encryption engine using AES-256-GCM
/// Provides authenticated encryption with additional data (AEAD)
pub struct CryptEngine {
    cipher: Aes256Gcm,  // AES-256-GCM cipher instance (key stored internally)
}

impl CryptEngine {
    /// Create a new encryption engine with shared key
    /// 
    /// Note: Currently uses a hardcoded shared key for simplicity
    /// Production systems should implement proper key exchange (Diffie-Hellman, etc.)
    pub fn new() -> Self {
        // Use a shared hardcoded key so all instances can decrypt each other's messages
        // In production, implement proper key exchange (Diffie-Hellman, etc.)
        let key_bytes = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
            0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
            0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
        ];
        
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        
        Self { cipher }
    }

    /// Create encryption engine from existing key bytes
    /// Used for key exchange scenarios
    /// 
    /// # Arguments
    /// * `key_bytes` - 32-byte AES-256 key
    #[allow(dead_code)]
    pub fn from_key(key_bytes: &[u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);
        Self { cipher }
    }

    /// Encrypt plaintext message using AES-256-GCM
    /// 
    /// # Arguments
    /// * `plaintext` - The message to encrypt
    /// 
    /// # Returns
    /// Ok(Vec<u8>) - Encrypted data with nonce prepended (12 bytes nonce + ciphertext)
    /// Err(String) - Error message if encryption fails
    /// 
    /// # Security
    /// - Uses random nonce for each encryption
    /// - Nonce is prepended to ciphertext for storage/transmission
    /// - GCM mode provides authentication (tamper detection)
    pub fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>, String> {
        // Generate random nonce (96 bits for GCM)
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        // Prepend nonce to ciphertext for storage
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
    
    /// Decrypt ciphertext using AES-256-GCM
    /// 
    /// # Arguments
    /// * `encrypted_data` - Encrypted data (12 bytes nonce + ciphertext)
    /// 
    /// # Returns
    /// Ok(String) - Decrypted plaintext message
    /// Err(String) - Error message if decryption fails or data is tampered
    /// 
    /// # Security
    /// - Extracts nonce from first 12 bytes
    /// - GCM authentication will fail if data was tampered with
    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<String, String> {
        if encrypted_data.len() < 12 {
            return Err("Invalid encrypted data".to_string());
        }
        
        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        // Decrypt
        let plaintext_bytes = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        String::from_utf8(plaintext_bytes)
            .map_err(|e| format!("Invalid UTF-8: {}", e))
    }
}