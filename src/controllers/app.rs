// Controller: Main application logic connecting models and views
// Manages the chat state, encryption, networking, and message flow

use crate::models::{crypt::CryptEngine, message::Message, network::{NetworkManager, NetworkMessage}};
use crate::views::chat_window::ChatWindow;

/// Main application state structure
/// Contains all necessary components for secure messaging
pub struct ChatApp {
    pub crypto: CryptEngine,        // Encryption/decryption engine
    pub messages: Vec<Message>,     // Message history (stored encrypted)
    pub input_text: String,         // Current text being typed by user
    pub network: NetworkManager,    // TCP networking layer
    pub target_address: String,     // Destination IP:PORT for outgoing messages
    pub local_port: u16,           // Local listening port
    pub username: String,          // Display name for this user
}

impl ChatApp {
    /// Create a new ChatApp instance
    /// 
    /// # Arguments
    /// * `port` - Local port to listen on for incoming messages
    /// * `username` - Display name for this user
    pub fn new(port: u16, username: String) -> Self {
        Self {
            crypto: CryptEngine::new(),
            messages: Vec::new(),
            input_text: String::new(),
            network: NetworkManager::new(port),  // Start TCP server
            target_address: "127.0.0.1:3001".to_string(),  // Default target
            local_port: port,
            username,
        }
    }
    
    /// Encrypt and send the current input message
    /// Called when user clicks Send or presses Enter
    pub fn send_message(&mut self) {
        // Ignore empty messages
        if self.input_text.trim().is_empty() {
            return;
        }
        
        // Encrypt the plaintext message
        match self.crypto.encrypt(&self.input_text) {
            Ok(encrypted) => {
                // Store message locally (encrypted only, no plaintext)
                let message = Message::new(encrypted.clone(), self.username.clone());
                self.messages.push(message);
                
                // Prepare network message with metadata
                let network_msg = NetworkMessage {
                    encrypted_data: encrypted,
                    sender_id: self.username.clone(),
                    timestamp: chrono::Utc::now().timestamp(),
                };
                
                // Send over TCP to target address
                self.network.send_message(self.target_address.clone(), network_msg);
                
                // Clear input field
                self.input_text.clear();
            }
            Err(e) => {
                eprintln!("Encryption error: {}", e);
            }
        }
    }
    
    /// Poll for incoming messages from the network
    /// Called on every frame update
    pub fn check_incoming_messages(&mut self) {
        // Non-blocking check for new messages
        while let Ok(network_msg) = self.network.incoming_rx.try_recv() {
            // Store incoming encrypted message
            let message = Message::new(network_msg.encrypted_data, network_msg.sender_id);
            self.messages.push(message);
        }
    }
    
    /// Decrypt a message for display
    /// 
    /// # Arguments
    /// * `encrypted` - The encrypted message bytes
    /// 
    /// # Returns
    /// Decrypted plaintext string or "[Decryption Failed]" on error
    pub fn decrypt_message(&self, encrypted: &[u8]) -> String {
        self.crypto
            .decrypt(encrypted)
            .unwrap_or_else(|_| "[Decryption Failed]".to_string())
    }
}

/// Implement the eframe App trait for GUI rendering
impl eframe::App for ChatApp {
    /// Called every frame to update and render the UI
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for new incoming messages
        self.check_incoming_messages();
        
        // Render the chat window UI
        ChatWindow::render(ctx, self);
        
        // Request continuous repainting to check for new messages
        ctx.request_repaint();
    }
}