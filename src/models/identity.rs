// Model: Persistent identity management
// Handles loading and saving long-term Ed25519 identity keys
// Stored in user's config directory (~/.secure-chat/identity.key)

use ed25519_dalek::{SigningKey, VerifyingKey};
use sha2::{Sha256, Digest};
use std::fs;
use std::path::PathBuf;

/// Manages persistent identity keys for the user
pub struct Identity {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
    pub fingerprint: String,
}

impl Identity {
    /// Load or create identity from config directory
    /// 
    /// # Returns
    /// Identity with persistent keys (same across app restarts)
    pub fn load_or_create() -> Result<Self, String> {
        let identity_path = Self::get_identity_path()?;
        
        // Try to load existing identity
        if identity_path.exists() {
            println!("📂 Loading existing identity from {}", identity_path.display());
            Self::load_from_file(&identity_path)
        } else {
            println!("🆕 Creating new identity at {}", identity_path.display());
            Self::create_and_save(&identity_path)
        }
    }
    
    /// Get the path to identity file in config directory
    fn get_identity_path() -> Result<PathBuf, String> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| "Failed to find config directory".to_string())?;
        
        let app_dir = config_dir.join("secure-chat");
        
        // Create directory if it doesn't exist
        if !app_dir.exists() {
            fs::create_dir_all(&app_dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        
        Ok(app_dir.join("identity.key"))
    }
    
    /// Load identity from file
    fn load_from_file(path: &PathBuf) -> Result<Self, String> {
        let key_bytes = fs::read(path)
            .map_err(|e| format!("Failed to read identity file: {}", e))?;
        
        if key_bytes.len() != 32 {
            return Err("Invalid identity file (expected 32 bytes)".to_string());
        }
        
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&key_bytes);
        
        let signing_key = SigningKey::from_bytes(&key_array);
        let verifying_key = signing_key.verifying_key();
        let fingerprint = Self::compute_fingerprint(&verifying_key);
        
        println!("✅ Identity loaded. Fingerprint: {}", fingerprint);
        
        Ok(Self {
            signing_key,
            verifying_key,
            fingerprint,
        })
    }
    
    /// Create new identity and save to file
    fn create_and_save(path: &PathBuf) -> Result<Self, String> {
        // Generate new Ed25519 identity key pair
        let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
        let verifying_key = signing_key.verifying_key();
        let fingerprint = Self::compute_fingerprint(&verifying_key);
        
        // Save private key to file
        fs::write(path, signing_key.to_bytes())
            .map_err(|e| format!("Failed to save identity file: {}", e))?;
        
        println!("✅ New identity created. Fingerprint: {}", fingerprint);
        println!("⚠️  Identity saved to: {}", path.display());
        println!("⚠️  Keep this file secure - it's your cryptographic identity!");
        
        Ok(Self {
            signing_key,
            verifying_key,
            fingerprint,
        })
    }
    
    /// Compute fingerprint from verifying key
    fn compute_fingerprint(verifying_key: &VerifyingKey) -> String {
        let mut hasher = Sha256::new();
        hasher.update(verifying_key.to_bytes());
        let hash = hasher.finalize();
        hex::encode(&hash[..16]).to_uppercase()
    }
}
