// Controller: Main application logic connecting models and views
// Manages the chat state, encryption, networking, and message flow

use crate::models::{
    crypt::CryptEngine, 
    message::Message, 
    network::{NetworkManager, NetworkMessage},
    keyexchange::{KeyExchangeManager, KeyExchangeMessage},
    identity::Identity,
    contacts::{ContactList, Contact},
    discovery::PeerDiscovery,
};
use crate::views::chat_window::ChatWindow;
use sha2::Digest;

/// Main application state structure
/// Contains all necessary components for secure messaging
pub struct ChatApp {
    pub crypto: Option<CryptEngine>,    // Encryption engine (Some after key exchange)
    pub messages: Vec<Message>,         // Message history (stored encrypted)
    pub input_text: String,             // Current text being typed by user
    pub network: NetworkManager,        // TCP networking layer
    pub target_address: String,         // Destination IP:PORT for outgoing messages
    pub local_port: u16,               // Local listening port
    pub username: String,              // Display name for this user
    pub key_exchange: KeyExchangeManager,  // Key exchange manager
    pub key_established: bool,         // Whether secure channel is established
    pub peer_username: Option<String>, // Connected peer's username
    pub fingerprint: String,           // This user's identity fingerprint
    pub peer_fingerprint: Option<String>, // Peer's identity fingerprint
    #[allow(dead_code)]  // Used during initialization for fingerprint
    pub identity: Identity,            // Persistent identity (loaded from disk)
    pub contacts: ContactList,         // Contact list
    pub discovery: Option<PeerDiscovery>, // Peer discovery service
    pub auto_connect_attempted: bool,  // Track if we already tried auto-connect
}

impl ChatApp {
    /// Create a new ChatApp instance
    /// 
    /// # Arguments
    /// * `port` - Local port to listen on for incoming messages
    /// * `username` - Display name for this user
    pub fn new(port: u16, username: String) -> Self {
        // Load or create persistent identity
        let identity = Identity::load_or_create()
            .expect("Failed to load/create identity");
        
        // Load or create contact list
        let contacts = ContactList::load_or_create()
            .expect("Failed to load/create contacts");
        
        let key_exchange = KeyExchangeManager::new(username.clone(), &identity);
        let fingerprint = identity.fingerprint.clone();
        
        // Start peer discovery service
        let discovery = match PeerDiscovery::new(port, username.clone()) {
            Ok(d) => {
                println!("✅ Peer discovery enabled");
                Some(d)
            }
            Err(e) => {
                eprintln!("⚠️  Peer discovery disabled: {}", e);
                None
            }
        };
        
        Self {
            crypto: None,  // Will be set after key exchange
            messages: Vec::new(),
            input_text: String::new(),
            network: NetworkManager::new(port),  // Start TCP server
            target_address: "127.0.0.1:3001".to_string(),  // Default target
            local_port: port,
            username,
            key_exchange,
            key_established: false,
            peer_username: None,
            fingerprint,
            peer_fingerprint: None,
            identity,
            contacts,
            discovery,
            auto_connect_attempted: false,
        }
    }
    
    /// Initiate key exchange with peer
    /// Sends our public key and identity to establish secure channel
    pub fn initiate_key_exchange(&mut self) {
        let exchange_msg = self.key_exchange.create_exchange_message(self.local_port);
        let network_msg = NetworkMessage::KeyExchange {
            message: exchange_msg,
            sender_address: None,  // Will be filled by receiver
        };
        self.network.send_message(self.target_address.clone(), network_msg);
    }
    
    /// Encrypt and send the current input message
    /// Called when user clicks Send or presses Enter
    pub fn send_message(&mut self) {
        // Check if encryption is available
        if self.crypto.is_none() {
            eprintln!("Cannot send message: encryption not established. Initiate key exchange first.");
            return;
        }
        
        // Ignore empty messages
        if self.input_text.trim().is_empty() {
            return;
        }
        
        // Encrypt the plaintext message
        match self.crypto.as_ref().unwrap().encrypt(&self.input_text) {
            Ok(encrypted) => {
                // Store message locally with cached plaintext
                let mut message = Message::new(encrypted.clone(), self.username.clone());
                message.decrypted = Some(self.input_text.clone()); // Cache our own plaintext
                self.messages.push(message);
                
                // Prepare network message with metadata
                let network_msg = NetworkMessage::EncryptedMessage {
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
            match network_msg {
                NetworkMessage::KeyExchange { message: key_msg, sender_address } => {
                    // Set target_address from the peer's listening port (not the ephemeral connection port)
                    if self.target_address == "127.0.0.1:3001" {
                        if let Some(addr) = sender_address {
                            // Extract IP from the socket address and combine with peer's listening port
                            if let Some(ip) = addr.split(':').next() {
                                self.target_address = format!("{}:{}", ip, key_msg.listening_port);
                                println!("📍 Set target address to: {}", self.target_address);
                            }
                        }
                    }
                    // Process key exchange message
                    self.handle_key_exchange(key_msg);
                }
                NetworkMessage::EncryptedMessage { encrypted_data, sender_id, timestamp: _ } => {
                    // Store incoming encrypted message and decrypt immediately
                    let mut message = Message::new(encrypted_data, sender_id);
                    
                    // Decrypt and cache plaintext immediately
                    message.decrypted = Some(self.decrypt_message(&message.encrypted));
                    
                    self.messages.push(message);
                }
            }
        }
    }
    
    /// Handle incoming key exchange message
    /// Derives shared secret and initializes encryption
    fn handle_key_exchange(&mut self, peer_message: KeyExchangeMessage) {
        // Always process incoming key exchange (idempotent operation)
        match self.key_exchange.process_exchange(&peer_message) {
            Ok(shared_secret) => {
                // Initialize encryption with derived shared secret
                self.crypto = Some(CryptEngine::from_shared_secret(&shared_secret));
                self.peer_username = Some(peer_message.username.clone());
                
                // Store peer fingerprint
                let mut hasher = sha2::Sha256::new();
                hasher.update(peer_message.identity_public_key);
                let hash = hasher.finalize();
                let peer_fp = hex::encode(&hash[..8]).to_uppercase();
                self.peer_fingerprint = Some(peer_fp.clone());
                
                println!("✅ Secure channel established with {}", peer_message.username);
                println!("🔑 Your fingerprint: {}", self.fingerprint);
                println!("🔑 Peer fingerprint: {}", peer_fp);
                
                // Check if this is a known contact
                if let Some(existing_contact) = self.contacts.get_contact(&peer_message.username) {
                    // Verify fingerprint matches
                    if existing_contact.fingerprint != peer_fp {
                        println!("⚠️  WARNING: Fingerprint mismatch for {}!", peer_message.username);
                        println!("   Expected: {}", existing_contact.fingerprint);
                        println!("   Received: {}", peer_fp);
                        println!("   Possible MITM attack or identity changed!");
                    } else {
                        println!("✅ Contact verified: {} (fingerprint matches)", peer_message.username);
                        self.contacts.update_last_connected(&peer_message.username);
                    }
                } else {
                    // New contact - save it
                    println!("📇 New contact: {} - saving to contact list", peer_message.username);
                    let contact = Contact {
                        username: peer_message.username.clone(),
                        address: self.target_address.clone(),
                        fingerprint: peer_fp,
                        last_connected: Some(chrono::Utc::now().timestamp()),
                        notes: String::new(),
                    };
                    self.contacts.add_contact(contact);
                }
                
                // If we haven't established our side yet, send our key exchange
                if !self.key_established {
                    println!("📤 Sending key exchange response...");
                    let exchange_msg = self.key_exchange.create_exchange_message(self.local_port);
                    let network_msg = NetworkMessage::KeyExchange {
                        message: exchange_msg,
                        sender_address: None,
                    };
                    self.network.send_message(self.target_address.clone(), network_msg);
                }
                
                // Mark as established after processing
                self.key_established = true;
            }
            Err(e) => {
                eprintln!("❌ Key exchange failed: {}", e);
            }
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
        match &self.crypto {
            Some(crypto) => crypto
                .decrypt(encrypted)
                .unwrap_or_else(|_| "[Decryption Failed]".to_string()),
            None => "[No Encryption Key]".to_string(),
        }
    }
    
    /// Check for discovered peers and auto-connect
    pub fn check_peer_discovery(&mut self) {
        // Only auto-connect once and only if not already connected
        if self.key_established || self.auto_connect_attempted {
            return;
        }
        
        // Check for discovered peers
        if let Some(discovery) = &self.discovery {
            if let Some(peer) = discovery.check_for_peers() {
                // Don't connect to ourselves
                if peer.port != self.local_port {
                    println!("🤝 Auto-connecting to {} at {}...", peer.username, peer.address);
                    
                    // Set target address
                    self.target_address = peer.address.clone();
                    
                    // Initiate key exchange
                    self.initiate_key_exchange();
                    
                    // Mark that we attempted auto-connect
                    self.auto_connect_attempted = true;
                } else {
                    println!("🔍 Ignoring own service (port {})", peer.port);
                }
            }
        }
    }
}

/// Implement the eframe App trait for GUI rendering
impl eframe::App for ChatApp {
    /// Called every frame to update and render the UI
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for new incoming messages
        self.check_incoming_messages();
        
        // Check for discovered peers and auto-connect
        self.check_peer_discovery();
        
        // Render the chat window UI
        ChatWindow::render(ctx, self);
        
        // Request continuous repainting to check for new messages
        ctx.request_repaint();
    }
}