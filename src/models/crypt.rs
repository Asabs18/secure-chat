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
    /// Create encryption engine from shared secret (ECDH result)
    /// This is the proper way to initialize encryption after key exchange
    /// 
    /// # Arguments
    /// * `shared_secret` - 32-byte shared secret from ECDH key exchange
    pub fn from_shared_secret(shared_secret: &[u8; 32]) -> Self {
        println!("🔍 DEBUG CryptEngine: Initializing with key: {}", hex::encode(shared_secret));
        let key = Key::<Aes256Gcm>::from_slice(shared_secret);
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
        println!("🔍 DEBUG Decrypt: Received {} bytes", encrypted_data.len());
        if encrypted_data.len() < 12 {
            return Err("Invalid encrypted data".to_string());
        }
        
        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        println!("🔍 DEBUG Decrypt: Nonce: {}", hex::encode(nonce_bytes));
        println!("🔍 DEBUG Decrypt: Ciphertext: {}", hex::encode(ciphertext));
        
        // Decrypt
        let plaintext_bytes = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| {
                println!("❌ DEBUG Decrypt FAILED: {}", e);
                format!("Decryption failed: {}", e)
            })?;
        
        String::from_utf8(plaintext_bytes)
            .map_err(|e| format!("Invalid UTF-8: {}", e))
    }
}