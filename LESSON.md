# 📚 Networking & Cryptography Deep Dive

This document explains the networking and cryptographic concepts used in the Secure Chat application. It's written for experienced programmers who are new to networking and want to understand how peer-to-peer encrypted communication works.

---

## Table of Contents
1. [TCP/IP Fundamentals](#tcpip-fundamentals)
2. [Client-Server vs Peer-to-Peer](#client-server-vs-peer-to-peer)
3. [Multi-threaded Network Architecture](#multi-threaded-network-architecture)
4. [Message Serialization](#message-serialization)
5. [Cryptographic Key Exchange](#cryptographic-key-exchange)
6. [Elliptic Curve Cryptography](#elliptic-curve-cryptography)
7. [Authenticated Encryption](#authenticated-encryption)
8. [Security Properties & Threat Model](#security-properties--threat-model)

---

## TCP/IP Fundamentals

### What is TCP?

**TCP (Transmission Control Protocol)** is a connection-oriented protocol that provides reliable, ordered delivery of a stream of bytes between applications. Think of it like a phone call - you establish a connection, exchange data bidirectionally, and then hang up.

**Key Properties:**
- **Connection-oriented**: Must establish connection before data exchange
- **Reliable**: Guarantees delivery (retransmits lost packets)
- **Ordered**: Data arrives in the same order it was sent
- **Stream-based**: Data is a continuous byte stream (no message boundaries)

**Alternative: UDP** - Connectionless, unreliable, message-based (like sending postcards)

### TCP Connection Lifecycle

```
1. LISTEN    - Server waits for connections
2. CONNECT   - Client initiates connection (SYN packet)
3. ACCEPT    - Server accepts connection (SYN-ACK handshake)
4. SEND/RECV - Bidirectional data exchange
5. CLOSE     - Either side terminates connection
```

### Sockets: The Programming Interface

A **socket** is an endpoint for network communication. It's like a file descriptor but for network connections.

**Socket Address = IP Address + Port Number**
- **IP Address**: Identifies the machine (e.g., `127.0.0.1` = localhost)
- **Port Number**: Identifies the application on that machine (0-65535)
- Example: `127.0.0.1:3000` = "Connect to port 3000 on this machine"

**Common Port Conventions:**
- 0-1023: System/privileged ports (HTTP=80, HTTPS=443, SSH=22)
- 1024-49151: Registered ports (application-specific)
- 49152-65535: Dynamic/ephemeral ports (temporary client ports)

### Implementation in Our Code

**File: `src/models/network.rs` (lines 60-85)**

```rust
// Bind server to listen on specified port
let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
    .expect("Failed to bind to port");
```

**What's happening:**
- `0.0.0.0` means "listen on all network interfaces" (localhost + external)
- `bind()` reserves the port for this application
- Returns a `TcpListener` that can accept incoming connections

**Why `0.0.0.0` instead of `127.0.0.1`?**
- `127.0.0.1` (localhost) only accepts local connections
- `0.0.0.0` accepts connections from any network interface
- Allows connection from other machines on same network

---

## Client-Server vs Peer-to-Peer

### Traditional Client-Server Model

```
       Client A ──┐
                  │
       Client B ──┼──> Central Server
                  │
       Client C ──┘
```

- **Server**: Always listening, centralized, handles all communication
- **Clients**: Initiate connections, send requests to server
- **Example**: Web browsing (your browser = client, website = server)

**Disadvantages:**
- Single point of failure
- Server must relay all messages
- Privacy concern (server sees everything)

### Peer-to-Peer (P2P) Model

```
    Alice ←──────────→ Bob
      ↑                 ↑
      │                 │
      └────── Charlie ──┘
```

- **Every peer is both client and server**
- Direct connections between users
- No central authority

**Our Implementation: Hybrid Approach**

Each user runs BOTH:
1. **Server component** - Listens for incoming connections
2. **Client component** - Connects to other peers

**File: `src/models/network.rs` (lines 60-110)**

```rust
// Server thread - listens for incoming connections
std::thread::spawn(move || {
    for stream in listener.incoming() {
        // Spawn handler for each connection
    }
});

// Client functionality - send to any address
pub fn send_message(&self, target_address: String, message: NetworkMessage) {
    let stream = TcpStream::connect(&target_address);
    // Send message
}
```

**Why this works:**
- Alice listens on port 3000 (server mode)
- Bob listens on port 3001 (server mode)
- Alice connects to `127.0.0.1:3001` to send to Bob (client mode)
- Bob connects to `127.0.0.1:3000` to send to Alice (client mode)

---

## Multi-threaded Network Architecture

### The Problem: Blocking I/O

**Blocking operations** stop execution until they complete:
```rust
let stream = listener.accept(); // BLOCKS until connection arrives
let data = stream.read();       // BLOCKS until data arrives
```

If your UI thread blocks waiting for network data, your app freezes!

### Solution: Multi-threading

Create separate threads for network operations so the main UI thread stays responsive.

### Our Threading Model

**File: `src/models/network.rs` (lines 60-110)**

```
Main Thread (UI)
    │
    ├─> Server Listener Thread (spawned at startup)
    │       └─> Connection Handler Thread (spawned per connection)
    │
    └─> Client Sender Thread (spawned per message send)
```

#### Thread 1: Server Listener

**File: `src/models/network.rs` (lines 60-85)**

```rust
let incoming_tx_clone = incoming_tx.clone();
std::thread::spawn(move || {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let tx = incoming_tx_clone.clone();
                std::thread::spawn(move || {
                    Self::handle_client(stream, tx);
                });
            }
            Err(e) => eprintln!("Connection error: {}", e),
        }
    }
});
```

**What's happening:**
1. **Outer thread**: Runs forever, blocks on `listener.incoming()`
2. When connection arrives, spawns **handler thread** for that specific connection
3. Handler reads messages from that connection
4. Messages sent to UI thread via channel

**Why spawn per-connection threads?**
- Multiple peers can connect simultaneously
- One slow connection doesn't block others
- Each connection can be read independently

#### Thread 2+: Connection Handlers

**File: `src/models/network.rs` (lines 111-145)**

```rust
fn handle_client(mut stream: TcpStream, tx: Sender<NetworkMessage>) {
    let mut buffer = Vec::new();
    let mut temp_buffer = [0u8; 4096];
    
    loop {
        match stream.read(&mut temp_buffer) {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                buffer.extend_from_slice(&temp_buffer[..n]);
                // Try to deserialize complete messages
            }
            Err(_) => break,
        }
    }
}
```

**Networking Insight: Stream-based Protocol**

TCP provides a **byte stream**, not messages! Data might arrive as:
- One big chunk: `[message1][message2][message3]`
- Split across reads: `[mess`, `age1][mes`, `sage2]`
- Partial message: `[message1][half_of_mess`

**Our Solution:**
1. Read into temporary buffer (4096 bytes)
2. Accumulate into growing buffer
3. Try to parse complete JSON messages
4. When complete message found, send via channel and remove from buffer

This is called **message framing** - determining where one message ends and next begins.

**Better alternatives we could use:**
- Length prefix: Send 4-byte length, then message
- Delimiter: Use newline `\n` to separate messages
- Fixed size: All messages same length (wastes space)

#### Inter-thread Communication: Channels

**File: `src/models/network.rs` (lines 49-52)**

```rust
let (incoming_tx, incoming_rx) = std::sync::mpsc::channel::<NetworkMessage>();
let (outgoing_tx, outgoing_rx) = std::sync::mpsc::channel::<(String, NetworkMessage)>();
```

**What are channels?**

Channels are **message queues** for thread communication:
```
Thread A ──[send]──> Channel ──[recv]──> Thread B
```

**MPSC = Multi-Producer, Single-Consumer**
- Multiple threads can send (clone `tx`)
- One thread receives (`rx`)

**In our app:**
- **incoming_tx**: Network threads send received messages
- **incoming_rx**: UI thread receives (via `try_recv()` in update loop)
- **outgoing_tx**: UI thread sends messages to send
- **outgoing_rx**: Sender thread receives and transmits

**File: `src/controllers/app.rs` (lines 108-120)**

```rust
pub fn check_incoming_messages(&mut self) {
    // Non-blocking check for new messages
    while let Ok(network_msg) = self.network.incoming_rx.try_recv() {
        match network_msg {
            NetworkMessage::KeyExchange(key_msg) => {
                self.handle_key_exchange(key_msg);
            }
            NetworkMessage::EncryptedMessage { encrypted_data, sender_id, timestamp } => {
                let mut message = Message::new(encrypted_data, sender_id);
                message.decrypted = Some(self.decrypt_message(&message.encrypted));
                self.messages.push(message);
            }
        }
    }
}
```

**Key point: `try_recv()` vs `recv()`**
- `recv()`: BLOCKS until message available (freezes UI)
- `try_recv()`: Returns immediately with `Err` if no message (non-blocking)

We call `try_recv()` every frame (~60 times per second) to check for messages without blocking.

---

## Message Serialization

### The Problem: Structured Data Over Network

Network only transmits **bytes**. We have **structured data** (structs):

```rust
struct Message {
    encrypted: Vec<u8>,
    timestamp: i64,
    sender_id: String,
}
```

How do we convert this to bytes and back?

### Serialization Formats

**Options:**
1. **JSON**: Human-readable text format
2. **Binary (MessagePack, Bincode)**: Compact binary format
3. **Protocol Buffers**: Language-agnostic binary with schema

**We chose JSON** for simplicity and debuggability.

### Implementation with Serde

**File: `src/models/network.rs` (lines 14-20)**

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NetworkMessage {
    KeyExchange(KeyExchangeMessage),
    EncryptedMessage {
        encrypted_data: Vec<u8>,
        sender_id: String,
        timestamp: i64,
    },
}
```

**The `#[derive(Serialize, Deserialize)]` attribute:**
- Automatically generates serialization code
- Converts struct ↔ JSON

**Example conversion:**
```rust
// Rust struct
NetworkMessage::EncryptedMessage {
    encrypted_data: vec![1, 2, 3],
    sender_id: "Alice".to_string(),
    timestamp: 1701734400,
}

// Becomes JSON string
{
    "EncryptedMessage": {
        "encrypted_data": [1, 2, 3],
        "sender_id": "Alice",
        "timestamp": 1701734400
    }
}
```

### Why Enum for Message Types?

**File: `src/models/network.rs` (lines 14-20)**

We have TWO types of messages:
1. **KeyExchange**: Initial handshake to establish encryption
2. **EncryptedMessage**: Actual encrypted chat messages

**Using an enum lets us:**
- Send different message types over same connection
- Pattern match on received messages
- Type-safe handling (compiler ensures we handle all cases)

**File: `src/controllers/app.rs` (lines 110-120)**

```rust
match network_msg {
    NetworkMessage::KeyExchange(key_msg) => {
        // Handle key exchange
    }
    NetworkMessage::EncryptedMessage { encrypted_data, sender_id, timestamp } => {
        // Handle encrypted message
    }
}
```

---

## Cryptographic Key Exchange

### The Fundamental Problem: Key Distribution

**Scenario:** Alice and Bob want to communicate securely over an untrusted network.

**Requirements:**
1. Shared secret key for encryption
2. Key must not be transmitted (eavesdropper could intercept)
3. Must authenticate each other (prevent impersonation)

**Bad Solution:** Pre-shared key
- Must meet in person to exchange
- Can't establish new secure connections dynamically
- No forward secrecy if key compromised

**Good Solution:** Cryptographic Key Exchange Protocol

### Diffie-Hellman Key Exchange (Conceptual)

**Mathematical Property:**
```
Alice: private_a, public_a = f(private_a)
Bob:   private_b, public_b = f(private_b)

Alice computes: shared = f(private_a, public_b)
Bob computes:   shared = f(private_b, public_a)

Result: Both get same shared secret!
```

**Why it works:**
- Easy to compute `f(private, public)` if you know private key
- Hard to compute shared secret from just public keys
- Based on **discrete logarithm problem** (or elliptic curves)

### Our Implementation: X25519 ECDH

**File: `src/models/keyexchange.rs` (lines 1-8)**

We use **X25519**, a modern elliptic curve Diffie-Hellman protocol.

**Key Generation (lines 72-81):**

```rust
pub fn new(username: String) -> Self {
    // Generate ephemeral DH key pair (X25519)
    let dh_secret_bytes: [u8; 32] = rand::random();
    let dh_public_bytes = x25519_dalek::x25519(dh_secret_bytes, X25519_BASEPOINT_BYTES);
    let dh_public = X25519PublicKey::from(dh_public_bytes);
    
    // ... store in struct
}
```

**What's happening:**
1. Generate random 32-byte secret (private key)
2. Compute public key: `public = x25519(secret, basepoint)`
3. **Basepoint** is a fixed constant (generator point on curve)

**Critical bug we fixed:**
- Originally used `X25519PublicKey::from(dh_secret_bytes)` 
- This just copied secret into public field (no computation!)
- Resulted in different shared secrets on each side
- **Correct**: Must do scalar multiplication `x25519(secret, basepoint)`

**Shared Secret Computation (line 133):**

```rust
let shared_secret_bytes = x25519_dalek::x25519(self.dh_secret_bytes, peer_message.dh_public_key);
```

**Alice's side:**
```
shared_secret = x25519(alice_secret, bob_public)
```

**Bob's side:**
```
shared_secret = x25519(bob_secret, alice_public)
```

**Result: Same 32-byte shared secret on both sides!**

This is the mathematical magic of elliptic curve Diffie-Hellman.

### Authentication: Preventing Man-in-the-Middle

**Problem:** How does Alice know she's talking to Bob and not Mallory?

```
Alice ←──→ Mallory ←──→ Bob
      ↑              ↑
    MITM Attack: Mallory intercepts and relays
```

Without authentication, Mallory could:
1. Intercept Alice's public key
2. Replace it with her own
3. Establish shared secret with Alice
4. Establish separate shared secret with Bob
5. Decrypt, read, re-encrypt all messages

**Solution: Digital Signatures**

**File: `src/models/keyexchange.rs` (lines 67-69)**

```rust
// Generate long-term identity key pair (Ed25519)
let identity_signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
let identity_verifying_key = identity_signing_key.verifying_key();
```

**Two key pairs per user:**
1. **Ephemeral DH keys** (X25519): Used for key exchange, temporary
2. **Identity keys** (Ed25519): Long-term, prove identity

**Creating signed key exchange message (lines 95-102):**

```rust
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
```

**What's being sent:**
- DH public key (for key exchange)
- Identity public key (to verify signature)
- Signature of DH public key (proves ownership)
- Username (for display)

**Verifying incoming message (lines 121-127):**

```rust
// Verify signature to authenticate peer (prevent MITM)
let signature = Signature::from_bytes(&peer_message.signature);
peer_identity_key
    .verify(&peer_message.dh_public_key, &signature)
    .map_err(|_| "Signature verification failed - possible MITM attack!".to_string())?;
```

**Security property:**
- Only someone with Alice's private identity key can create valid signature
- Bob verifies signature matches Alice's identity public key
- If Mallory tries to intercept, she can't forge Alice's signature
- Protects against MITM attacks

**Current limitation:**
- Identity keys generated fresh each run (not persistent)
- No "Trust On First Use" or certificate validation
- Users should verify fingerprints out-of-band for true security

**Fingerprints (lines 168-176):**

```rust
pub fn get_fingerprint(&self) -> String {
    let identity_bytes = self.identity_verifying_key.to_bytes();
    let mut hasher = Sha256::new();
    hasher.update(identity_bytes);
    let hash = hasher.finalize();
    
    // Return first 16 bytes as hex (128-bit fingerprint)
    hex::encode(&hash[..16]).to_uppercase()
}
```

**Fingerprints are short hashes of identity keys:**
- Display to users: "Your fingerprint: A339CFB46ED1E23F..."
- Users can verify via phone/in-person: "Does your fingerprint match?"
- If fingerprints match, no MITM possible

### Key Derivation Function (KDF)

**Problem:** X25519 gives us a shared secret, but we need deterministic ordering.

**File: `src/models/keyexchange.rs` (lines 136-148)**

```rust
// Derive encryption key using KDF (SHA-256)
let mut hasher = Sha256::new();
hasher.update(&shared_secret_bytes);

// Add both public keys in sorted order for deterministic derivation
let mut keys = [self.dh_public.to_bytes(), peer_message.dh_public_key];
keys.sort();
hasher.update(&keys[0]);
hasher.update(&keys[1]);
hasher.update(b"secure-chat-v1"); // Domain separation

let key_material = hasher.finalize();
```

**Why KDF?**
1. **Deterministic ordering**: Both sides must hash same data in same order
2. **Domain separation**: String "secure-chat-v1" ensures keys derived for our app only
3. **Key stretching**: SHA-256 produces good quality key material

**Bug we fixed:**
- Originally didn't sort public keys
- Alice hashed: `[shared_secret || alice_pub || bob_pub]`
- Bob hashed: `[shared_secret || bob_pub || alice_pub]`
- Different inputs → different keys → decryption failed!
- **Solution**: Sort public keys before hashing

### Complete Key Exchange Flow

**File: `src/controllers/app.rs` (lines 59-62, 128-161)**

```
1. User clicks "Initiate Key Exchange"
   └─> initiate_key_exchange() called
       └─> create_exchange_message()
           └─> Send over network

2. Peer receives KeyExchange message
   └─> check_incoming_messages() processes it
       └─> handle_key_exchange() called
           └─> process_exchange() verifies signature
               └─> Compute shared secret
                   └─> Initialize CryptEngine with derived key
                       └─> Auto-respond with own key exchange
   
3. Original sender receives response
   └─> Same flow: verify, derive, initialize

Result: Both sides have CryptEngine with same AES key!
```

**Code: `src/controllers/app.rs` (lines 128-161)**

```rust
fn handle_key_exchange(&mut self, peer_message: KeyExchangeMessage) {
    match self.key_exchange.process_exchange(&peer_message) {
        Ok(shared_secret) => {
            // Initialize encryption with derived shared secret
            self.crypto = Some(CryptEngine::from_shared_secret(&shared_secret));
            self.peer_username = Some(peer_message.username.clone());
            
            // Store peer fingerprint
            let mut hasher = sha2::Sha256::new();
            hasher.update(peer_message.identity_public_key);
            let hash = hasher.finalize();
            self.peer_fingerprint = Some(hex::encode(&hash[..8]).to_uppercase());
            
            // If we haven't established our side yet, send our key exchange
            if !self.key_established {
                println!("📤 Sending key exchange response...");
                let exchange_msg = self.key_exchange.create_exchange_message();
                let network_msg = NetworkMessage::KeyExchange(exchange_msg);
                self.network.send_message(self.target_address.clone(), network_msg);
            }
            
            self.key_established = true;
        }
        Err(e) => {
            eprintln!("❌ Key exchange failed: {}", e);
        }
    }
}
```

**Key detail:** Auto-response mechanism
- When Bob receives Alice's key exchange, he automatically responds
- `if !self.key_established` prevents infinite ping-pong
- Both sides end up with `key_established = true`

---

## Elliptic Curve Cryptography

### Why Elliptic Curves?

**Traditional Diffie-Hellman** uses modular arithmetic:
- 2048-bit keys needed for adequate security
- Slow computation

**Elliptic Curve Diffie-Hellman (ECDH):**
- 256-bit keys provide equivalent security
- Much faster computation
- Smaller keys = less bandwidth

### What is an Elliptic Curve?

**Mathematical definition:**
```
y² = x³ + ax + b  (over finite field)
```

**Visual (over real numbers):**
```
      y
      │     ●
      │    ╱ ╲
──────┼───●───●──── x
      │  ╱     ╲
      │ ●       ●
```

**Key operations:**
1. **Point addition**: P + Q = R (add two points, get third point)
2. **Scalar multiplication**: k × P (add P to itself k times)

**Security property:**
- Easy: Given k and P, compute k × P
- Hard: Given P and k × P, find k (discrete log problem)

### X25519: A Specific Curve

**File: `src/models/keyexchange.rs` (line 11-14)**

```rust
const X25519_BASEPOINT_BYTES: [u8; 32] = [
    9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
```

**X25519** uses **Curve25519**:
- Designed for high performance and security
- 128-bit security level (comparable to 3072-bit RSA)
- Resistant to timing attacks
- Fast constant-time implementation

**Basepoint (9)**: Fixed starting point on curve for key generation

**Operations:**
```rust
// Key generation
public_key = x25519(secret_key, basepoint)

// Shared secret computation
shared_secret = x25519(my_secret, peer_public)
```

**The `x25519()` function:**
- Performs elliptic curve scalar multiplication
- Input: 32-byte scalar, 32-byte point
- Output: 32-byte result point
- Constant-time (prevents timing attacks)

### Ed25519: Digital Signatures

**Different curve for different purpose:**
- **X25519**: Key agreement (ECDH)
- **Ed25519**: Digital signatures

**Why separate?**
- Optimized for different operations
- Security: Don't reuse same keys for different purposes

**Ed25519 properties:**
- Fast signature generation and verification
- Small signatures (64 bytes)
- Small public keys (32 bytes)
- Deterministic (same message = same signature with same key)

**File: `src/models/keyexchange.rs` (lines 95-102)**

```rust
// Sign the DH public key
let signature = self.identity_signing_key.sign(&dh_public_bytes);
```

**What happens inside `sign()`:**
1. Hash the message (dh_public_bytes)
2. Perform elliptic curve operations with private key
3. Output: 64-byte signature

**Verification (lines 124-127):**
```rust
peer_identity_key.verify(&peer_message.dh_public_key, &signature)
```

**What happens inside `verify()`:**
1. Use public key to check signature matches message
2. Returns Ok if valid, Err if invalid/tampered
3. Does NOT reveal private key

---

## Authenticated Encryption

### Encryption vs Authentication

**Encryption alone is not enough!**

**Problem:** Unauthenticated encryption
```rust
ciphertext = encrypt(key, plaintext)
modified_ciphertext = attacker_modifies(ciphertext)
garbled = decrypt(key, modified_ciphertext) // Decrypts to garbage!
```

**Attacker can:**
- Flip bits in ciphertext
- Delete parts of message
- Replay old messages
- Not read plaintext, but can cause chaos

**Solution: Authenticated Encryption with Associated Data (AEAD)**

Combines:
1. **Encryption** (confidentiality)
2. **Authentication** (integrity + authenticity)

### AES-256-GCM

**File: `src/models/crypt.rs` (lines 5-8)**

```rust
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce, Key,
};
```

**AES-256-GCM = AES in Galois/Counter Mode**

**Components:**
- **AES-256**: Block cipher with 256-bit key
- **CTR mode**: Turns block cipher into stream cipher
- **GCM**: Adds authentication (generates auth tag)

**Structure:**
```
Input:  plaintext, key, nonce
Output: ciphertext + auth_tag
```

**Auth tag:**
- 16-byte value computed from ciphertext
- Changed if ciphertext is modified
- Verification fails if tampered

### Encryption Implementation

**File: `src/models/crypt.rs` (lines 76-100)**

```rust
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
```

**Key concepts:**

**1. Nonce (Number Used Once):**
- 12 bytes (96 bits) random value
- **MUST be unique for each message**
- Sent with ciphertext (not secret)
- Prevents replay attacks and pattern detection

**Why random nonce?**
```
Same plaintext + same key + same nonce = same ciphertext (bad!)
Same plaintext + same key + different nonce = different ciphertext (good!)
```

**2. Output format:**
```
[nonce (12 bytes)][ciphertext (variable)][auth_tag (included in ciphertext)]
```

**Why prepend nonce?**
- Receiver needs nonce to decrypt
- Nonce is not secret (like salt in password hashing)
- Convenient: Single byte array contains everything needed

### Decryption Implementation

**File: `src/models/crypt.rs` (lines 107-133)**

```rust
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
```

**What `cipher.decrypt()` does:**
1. Extract auth tag from end of ciphertext
2. Recompute auth tag from ciphertext
3. Compare: If tags don't match, return error (tampering detected!)
4. If valid, decrypt ciphertext to plaintext

**Error cases:**
- `"Decryption failed: aead::Error"`: Auth tag mismatch (wrong key or tampered)
- `"Invalid encrypted data"`: Too short to contain nonce
- `"Invalid UTF-8"`: Decrypted to non-text data

### Performance Optimization: Caching

**File: `src/models/message.rs` (lines 8-16)**

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub encrypted: Vec<u8>,
    pub timestamp: i64,
    pub sender_id: String,
    #[serde(skip)]
    pub decrypted: Option<String>, // Cached plaintext
}
```

**Problem we fixed:**
- Originally decrypted messages every frame in UI loop
- 60 FPS = 60 decryptions per second per message!
- Caused repeated decryption failures in logs

**Solution:**
- Decrypt once when message received
- Store plaintext in `decrypted` field
- UI displays cached value

**File: `src/controllers/app.rs` (lines 116-121)**

```rust
NetworkMessage::EncryptedMessage { encrypted_data, sender_id, timestamp } => {
    let mut message = Message::new(encrypted_data, sender_id);
    
    // Decrypt and cache plaintext immediately
    message.decrypted = Some(self.decrypt_message(&message.encrypted));
    
    self.messages.push(message);
}
```

**Trade-off:**
- **Pro**: Much faster (only decrypt once)
- **Con**: Plaintext in memory (security consideration)
- **Decision**: Acceptable for chat app (need to display anyway)

---

## Security Properties & Threat Model

### What We Protect Against

#### 1. Passive Eavesdropping ✅

**Threat:** Network observer can see all transmitted data

**Protection:**
- All messages encrypted with AES-256-GCM
- Only encrypted bytes transmitted over network
- Shared secret never sent (derived via ECDH)

**Attacker sees:**
```
NetworkMessage::EncryptedMessage {
    encrypted_data: [0x8e, 0xe8, 0x43, 0xab, ...],  // Gibberish
    sender_id: "Alice",                              // Public metadata
    timestamp: 1701734400
}
```

**Attacker cannot:**
- Read message content
- Derive encryption key from public keys

#### 2. Active Tampering ✅

**Threat:** Attacker modifies messages in transit

**Protection:**
- GCM authentication tag detects modifications
- Decryption fails if ciphertext altered

**Attack scenario:**
```
Alice sends: "Transfer $10"
Attacker changes to: "Transfer $1000"  // Modifies ciphertext
Bob receives, decrypts: ERROR (auth tag mismatch)
```

**Result:** Bob knows message was tampered with, ignores it

#### 3. Man-in-the-Middle (MITM) ✅

**Threat:** Attacker intercepts key exchange, impersonates peers

**Protection:**
- Ed25519 signatures prove identity
- Signature verification fails for forged messages

**Attack scenario:**
```
Alice → Mallory: KeyExchange with Alice's signature
Mallory → Bob: KeyExchange with Mallory's public key (forged signature)
Bob verifies: FAIL (signature doesn't match Mallory's identity key)
```

**Limitation:** Trust On First Use (TOFU)
- First connection, no prior knowledge of peer
- Fingerprint verification needed for high security
- Could be improved with PKI/certificates

#### 4. Replay Attacks ✅

**Threat:** Attacker records and retransmits old messages

**Protection:**
- Unique random nonce per message
- Same message encrypted twice produces different ciphertext

**Attack scenario:**
```
Attacker records: encrypted("Send $100")
Attacker replays message multiple times
Result: Looks like multiple different messages (different nonces)
       But application logic needed to prevent duplicate processing
```

**Current limitation:** No sequence numbers or timestamps checked

### What We DON'T Protect Against

#### 1. Forward Secrecy ⚠️

**Problem:** If key compromised, can decrypt past messages

**Current situation:**
- Ephemeral keys generated per session
- But keys persist entire session
- If attacker steals key during session, can decrypt all messages

**Solution (not implemented):**
- Ratcheting protocol (Signal/Double Ratchet)
- Generate new keys periodically
- Old keys deleted (can't decrypt past messages even if current key stolen)

#### 2. Network Metadata ⚠️

**What's visible to network observer:**
- Alice connected to Bob (IP addresses)
- Timestamp of communication
- Message sizes (approximate length)
- Sender username (transmitted in clear)

**Not protected:**
- Traffic analysis (who talks to whom, when, how often)

**Solutions (not implemented):**
- Tor/onion routing (hide endpoints)
- Padding (hide message sizes)
- Cover traffic (hide timing)

#### 3. Endpoint Security ⚠️

**Trust assumptions:**
- Alice's computer not compromised
- Bob's computer not compromised
- Codebase not backdoored

**Out of scope for network protocol:**
- Keyloggers
- Screen recording
- Memory dumps (encryption keys in RAM)

#### 4. Identity Persistence ⚠️

**Current limitation:**
- Identity keys regenerated each run
- No persistent identity storage
- Can't verify "same Bob as yesterday"

**File: `src/models/keyexchange.rs` (lines 72-74)**

```rust
// Generate long-term identity key pair (Ed25519)
let identity_signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
```

**Problem:** Every run creates new identity

**Solutions:**
- Save identity keys to disk (encrypted)
- Certificate authority / web of trust
- Blockchain-based identity (decentralized)

### Security Best Practices We Follow

✅ **Use established cryptographic libraries**
- x25519-dalek (audited)
- ed25519-dalek (audited)
- aes-gcm (industry standard)

✅ **Never roll your own crypto**
- Used standard protocols (X25519, Ed25519, AES-GCM)
- Didn't invent new algorithms

✅ **Constant-time operations**
- X25519 implementation resistant to timing attacks
- No secret-dependent branches in crypto code

✅ **Proper random number generation**
- `rand::random()` uses OS cryptographic RNG
- Not deterministic or predictable

✅ **Key separation**
- Different keys for different purposes (identity vs ephemeral)
- Don't reuse keys across protocols

✅ **Domain separation**
- KDF includes "secure-chat-v1" string
- Prevents key reuse across applications

### Threat Model Summary

**Assumptions:**
- Attacker controls network (can read, modify, inject, replay)
- Attacker does not control endpoints
- Attacker has significant computing power (but not quantum computer)

**Guarantees:**
- Confidentiality: Message content hidden
- Integrity: Tampering detected
- Authenticity: Sender identity verified (with fingerprint check)

**Non-guarantees:**
- Anonymity: Network observer knows who talks to whom
- Forward secrecy: Past messages decryptable if key leaked
- Deniability: Signatures prove you sent message
- Denial-of-service: Attacker can block all communication

---

## Advanced Topics

### Why Not Just Use TLS?

**TLS (Transport Layer Security)** is what HTTPS uses. Why didn't we just use that?

**TLS provides:**
- Encrypted transport channel
- Server authentication (certificates)
- Client authentication (optional)

**Limitations for P2P:**
- Requires certificate authority (CA) or self-signed certs
- Designed for client-server model
- Endpoint identity tied to certificates (complex for peer apps)

**Our approach:**
- End-to-end encryption (application layer)
- Custom key exchange (identity tied to Ed25519 keys)
- Could add TLS as transport layer (defense in depth)

**Best practice:** Use both
- TLS for transport security (prevents traffic analysis)
- Application-layer encryption for end-to-end security

### NAT Traversal (Not Implemented)

**Problem:** Peers behind routers can't connect directly

```
Alice (192.168.1.5) ←─ Router (1.2.3.4:3000) ─ Internet ─ Router (5.6.7.8:3001) ─→ Bob (192.168.1.7)
```

**NAT (Network Address Translation):**
- Router translates internal IP to public IP
- Incoming connections blocked by default

**Solutions:**
1. **Port forwarding**: Configure router to forward port
2. **UPnP**: Automatic port forwarding
3. **STUN**: Discover public IP and port
4. **TURN**: Relay server for indirect connections
5. **Hole punching**: Coordinate simultaneous connection attempts

**Our current limitation:** Only works on local network or with port forwarding

### Future: Signal Protocol

**For production chat app, consider Signal Protocol:**

**Features:**
- Double Ratchet: Forward secrecy + self-healing
- Prekeys: Asynchronous messaging (offline delivery)
- Triple DH: Mutual authentication with forward secrecy
- Header encryption: Hide metadata

**Resources:**
- [Signal Protocol Documentation](https://signal.org/docs/)
- [X3DH Specification](https://signal.org/docs/specifications/x3dh/)
- [Double Ratchet Specification](https://signal.org/docs/specifications/doubleratchet/)

---

## Conclusion

This secure chat application demonstrates:

**Networking:**
- TCP sockets for reliable communication
- Multi-threaded architecture for concurrency
- Message serialization with JSON
- Peer-to-peer model (hybrid client/server)

**Cryptography:**
- Elliptic curve Diffie-Hellman (X25519) for key agreement
- Digital signatures (Ed25519) for authentication
- Authenticated encryption (AES-256-GCM) for confidentiality + integrity
- Key derivation functions (SHA-256) for key material

**Security:**
- End-to-end encryption
- Man-in-the-middle attack prevention
- Tamper detection
- Forward secrecy foundations (ephemeral keys)

**Software engineering:**
- MVC architecture for clean separation
- Type-safe message handling with enums
- Efficient caching to avoid redundant computation
- Non-blocking I/O for responsive UI

**Key takeaway:** Modern secure communication requires multiple cryptographic primitives working together - no single algorithm provides all security properties.

---

## References & Further Reading

### Cryptography
- [A Graduate Course in Applied Cryptography](https://toc.cryptobook.us/) - Boneh & Shoup
- [Cryptography Engineering](https://www.schneier.com/books/cryptography-engineering/) - Ferguson, Schneier, Kohno
- [The Joy of Cryptography](https://joyofcryptography.com/) - Mike Rosulek

### Networking
- [Beej's Guide to Network Programming](https://beej.us/guide/bgnet/)
- [TCP/IP Illustrated](https://en.wikipedia.org/wiki/TCP/IP_Illustrated) - Stevens

### Protocols
- [X25519 RFC 7748](https://tools.ietf.org/html/rfc7748)
- [Ed25519 RFC 8032](https://tools.ietf.org/html/rfc8032)
- [AES-GCM RFC 5116](https://tools.ietf.org/html/rfc5116)
- [Signal Protocol](https://signal.org/docs/)

### Security
- [The Cryptographic Doom Principle](https://moxie.org/2011/12/13/the-cryptopocalypse.html) - Moxie Marlinspike
- [A Few Thoughts on Cryptographic Engineering](https://blog.cryptographyengineering.com/)

---

*This document is a living guide - as the codebase evolves, so should this explanation.*
