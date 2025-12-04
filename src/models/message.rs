// Model: Message data structure
// Stores encrypted messages with metadata (sender, timestamp)
// Security: Only encrypted data is stored, no plaintext

use serde::{Deserialize, Serialize};

/// Message structure for secure chat
/// All messages are stored in encrypted form only
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub encrypted: Vec<u8>,    // Encrypted message data (AES-256-GCM)
    pub timestamp: i64,        // Unix timestamp when message was created
    pub sender_id: String,     // Username of the sender
}

impl Message {
    /// Create a new message with encrypted data
    /// 
    /// # Arguments
    /// * `encrypted` - Pre-encrypted message bytes
    /// * `sender_id` - Username of the sender
    /// 
    /// # Returns
    /// New Message instance with current timestamp
    pub fn new(encrypted: Vec<u8>, sender_id: String) -> Self {
        Self {
            encrypted,
            timestamp: chrono::Utc::now().timestamp(),
            sender_id,
        }
    }
}