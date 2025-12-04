// Model: Cryptographic key exchange using X25519 (Elliptic Curve Diffie-Hellman)
// Provides secure key agreement without transmitting the shared secret
// Uses Ed25519 signatures for authentication to prevent MITM attacks

use serde::{Deserialize, Serialize};
use x25519_dalek::PublicKey as X25519PublicKey;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Sha256, Digest};

// X25519 base point for scalar multiplication
const X25519_BASEPOINT_BYTES: [u8; 32] = [
    9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// Key exchange message containing public keys and signatures
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyExchangeMessage {
    pub dh_public_key: [u8; 32],        // X25519 public key for ECDH
    pub identity_public_key: [u8; 32],  // Ed25519 public key for identity
    #[serde(with = "serde_arrays")]
    pub signature: [u8; 64],            // Ed25519 signature of DH public key
    pub username: String,                // User identifier
}

// Helper module for serializing large arrays
mod serde_arrays {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error> {
        bytes[..].serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<[u8; 64], D::Error> {
        let vec = Vec::<u8>::deserialize(deserializer)?;
        let mut array = [0u8; 64];
        if vec.len() != 64 {
            return Err(serde::de::Error::custom("Invalid signature length"));
        }
        array.copy_from_slice(&vec);
        Ok(array)
    }
}

/// Key exchange manager handling ECDH key agreement and authentication
pub struct KeyExchangeManager {
    // Long-term identity key pair (Ed25519)
    identity_signing_key: SigningKey,
    identity_verifying_key: VerifyingKey,
    
    // Ephemeral DH key pair (X25519) - stored as bytes to allow recreation
    dh_secret_bytes: [u8; 32],
    dh_public: X25519PublicKey,
    
    // Ephemeral DH shared secret - computed once during key exchange
    shared_secret: Option<[u8; 32]>,
    
    // Username for this instance
    username: String,
}

impl KeyExchangeManager {
    /// Create a new key exchange manager with generated keys
    /// 
    /// # Arguments
    /// * `username` - Username for this user
    /// 
    /// # Returns
    /// New KeyExchangeManager with fresh key pairs
    pub fn new(username: String) -> Self {
        // Generate long-term identity key pair (Ed25519)
        let identity_signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
        let identity_verifying_key = identity_signing_key.verifying_key();
        
        // Generate ephemeral DH key pair (X25519)
        // Secret: random 32 bytes, Public: x25519(secret, basepoint)
        let dh_secret_bytes: [u8; 32] = rand::random();
        let dh_public_bytes = x25519_dalek::x25519(dh_secret_bytes, X25519_BASEPOINT_BYTES);
        let dh_public = X25519PublicKey::from(dh_public_bytes);
        
        Self {
            identity_signing_key,
            identity_verifying_key,
            dh_secret_bytes,
            dh_public,
            shared_secret: None,
            username,
        }
    }
    
    /// Create a key exchange message to send to peer
    /// Signs the DH public key with identity key to prove ownership
    /// 
    /// # Returns
    /// KeyExchangeMessage ready for transmission
    pub fn create_exchange_message(&self) -> KeyExchangeMessage {
        let dh_public_bytes = self.dh_public.to_bytes();
        
        // Sign the DH public key with identity key to prove it's ours
        let signature = self.identity_signing_key.sign(&dh_public_bytes);
        
        KeyExchangeMessage {
            dh_public_key: dh_public_bytes,
            identity_public_key: self.identity_verifying_key.to_bytes(),
            signature: signature.to_bytes(),
            username: self.username.clone(),
        }
    }
    
    /// Process received key exchange message and derive shared secret
    /// Verifies signature to prevent man-in-the-middle attacks
    /// 
    /// # Arguments
    /// * `peer_message` - Key exchange message from peer
    /// 
    /// # Returns
    /// Ok([u8; 32]) - 32-byte shared secret for AES-256
    /// Err(String) - Error if signature verification fails
    /// 
    /// # Security
    /// - Verifies Ed25519 signature to authenticate peer
    /// - Uses X25519 ECDH to compute shared secret
    /// - Applies KDF (SHA-256) to derive encryption key
    pub fn process_exchange(&mut self, peer_message: &KeyExchangeMessage) -> Result<[u8; 32], String> {
        // Reconstruct peer's identity public key
        let peer_identity_key = VerifyingKey::from_bytes(&peer_message.identity_public_key)
            .map_err(|e| format!("Invalid identity key: {}", e))?;
        
        // Verify signature to authenticate peer (prevent MITM)
        let signature = Signature::from_bytes(&peer_message.signature);
        peer_identity_key
            .verify(&peer_message.dh_public_key, &signature)
            .map_err(|_| "Signature verification failed - possible MITM attack!".to_string())?;
        
        // Compute shared secret using x25519 scalar multiplication
        // x25519(our_secret, peer_public) = shared_secret
        let shared_secret_bytes = x25519_dalek::x25519(self.dh_secret_bytes, peer_message.dh_public_key);
        
        println!("🔍 DEBUG: Our secret (first 16 bytes): {}", hex::encode(&self.dh_secret_bytes[..16]));
        println!("🔍 DEBUG: Shared secret: {}", hex::encode(&shared_secret_bytes));
        println!("🔍 DEBUG: Our public: {}", hex::encode(self.dh_public.to_bytes()));
        println!("🔍 DEBUG: Peer public: {}", hex::encode(peer_message.dh_public_key));
        
        // Derive encryption key using KDF (SHA-256)
        // Use deterministic ordering: sort public keys to ensure both sides derive same key
        let mut hasher = Sha256::new();
        hasher.update(&shared_secret_bytes);
        
        // Add both public keys in sorted order for deterministic derivation
        let mut keys = [self.dh_public.to_bytes(), peer_message.dh_public_key];
        keys.sort();
        hasher.update(&keys[0]);
        hasher.update(&keys[1]);
        hasher.update(b"secure-chat-v1"); // Domain separation
        
        let key_material = hasher.finalize();
        
        println!("🔍 DEBUG: Derived key: {}", hex::encode(&key_material));
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&key_material);
        
        // Store for potential re-use
        self.shared_secret = Some(key);
        
        Ok(key)
    }
    
    /// Get the identity fingerprint for display/verification
    /// Users can compare fingerprints out-of-band to detect MITM
    /// 
    /// # Returns
    /// Hex-encoded fingerprint of identity public key
    pub fn get_fingerprint(&self) -> String {
        let identity_bytes = self.identity_verifying_key.to_bytes();
        let mut hasher = Sha256::new();
        hasher.update(identity_bytes);
        let hash = hasher.finalize();
        
        // Return first 16 bytes as hex (128-bit fingerprint)
        hex::encode(&hash[..16]).to_uppercase()
    }
}
