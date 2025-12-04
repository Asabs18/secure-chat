// Model: TCP networking layer for peer-to-peer communication
// Handles both server (receiving) and client (sending) functionality
// Uses multi-threading for concurrent operations

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use crate::models::keyexchange::KeyExchangeMessage;

/// Network message types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NetworkMessage {
    /// Key exchange initiation message
    KeyExchange(KeyExchangeMessage),
    /// Encrypted chat message
    EncryptedMessage {
        encrypted_data: Vec<u8>,
        sender_id: String,
        timestamp: i64,
    },
}

/// Network manager handling TCP client and server operations
/// Uses channels for thread-safe communication
pub struct NetworkManager {
    pub incoming_rx: Receiver<NetworkMessage>,  // Channel for received messages
    outgoing_tx: Sender<(String, NetworkMessage)>,  // Channel for messages to send
}

impl NetworkManager {
    /// Create a new network manager and start TCP server
    /// 
    /// # Arguments
    /// * `port` - Local port to listen on for incoming connections
    /// 
    /// # Returns
    /// NetworkManager instance with running server and client threads
    pub fn new(port: u16) -> Self {
        let (incoming_tx, incoming_rx) = channel();
        let (outgoing_tx, outgoing_rx) = channel::<(String, NetworkMessage)>();

        // Start server thread for receiving messages
        let incoming_tx_clone = incoming_tx.clone();
        thread::spawn(move || {
            Self::run_server(port, incoming_tx_clone);
        });

        // Start client sender thread for outgoing messages
        thread::spawn(move || {
            Self::run_client_sender(outgoing_rx);
        });

        Self {
            incoming_rx,
            outgoing_tx,
        }
    }

    /// Run TCP server to accept incoming connections
    /// Runs in separate thread, spawns handler thread per connection
    /// 
    /// # Arguments
    /// * `port` - Port to bind to
    /// * `tx` - Channel sender for incoming messages
    fn run_server(port: u16, tx: Sender<NetworkMessage>) {
        let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to bind to port {}: {}", port, e);
                return;
            }
        };

        println!("Server listening on port {}", port);

        // Accept connections in a loop
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let tx = tx.clone();
                    // Spawn handler thread for each connection
                    thread::spawn(move || {
                        Self::handle_client(&mut stream, tx);
                    });
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
    }

    /// Handle a single client connection
    /// Reads messages from the stream and forwards to main thread
    /// 
    /// # Arguments
    /// * `stream` - TCP stream for this connection
    /// * `tx` - Channel sender for incoming messages
    fn handle_client(stream: &mut TcpStream, tx: Sender<NetworkMessage>) {
        let mut buffer = vec![0u8; 8192];  // 8KB buffer for incoming data
        
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => break, // Connection closed
                Ok(n) => {
                    // Deserialize JSON message
                    if let Ok(msg) = serde_json::from_slice::<NetworkMessage>(&buffer[..n]) {
                        let _ = tx.send(msg);
                    }
                }
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    break;
                }
            }
        }
    }

    /// Run client sender thread
    /// Connects to remote addresses and sends messages
    /// 
    /// # Arguments
    /// * `rx` - Channel receiver for outgoing messages (address, message)
    fn run_client_sender(rx: Receiver<(String, NetworkMessage)>) {
        while let Ok((address, msg)) = rx.recv() {
            // Connect to target and send message
            if let Ok(mut stream) = TcpStream::connect(&address) {
                if let Ok(data) = serde_json::to_vec(&msg) {
                    let _ = stream.write_all(&data);
                }
            } else {
                eprintln!("Failed to connect to {}", address);
            }
        }
    }

    /// Send a message to a remote address
    /// Non-blocking: queues message for sending in background thread
    /// 
    /// # Arguments
    /// * `address` - Target address in format "IP:PORT"
    /// * `msg` - NetworkMessage to send
    pub fn send_message(&self, address: String, msg: NetworkMessage) {
        let _ = self.outgoing_tx.send((address, msg));
    }
}
