# 🔒 Secure Chat Application

A peer-to-peer encrypted messaging application built with Rust, featuring end-to-end encryption using AES-256-GCM, X25519 Elliptic Curve Diffie-Hellman key exchange, and real-time TCP networking.

## ✨ Features

- **End-to-End Encryption**: All messages encrypted with AES-256-GCM authenticated encryption
- **Secure Key Exchange**: X25519 ECDH for dynamic key agreement without transmitting secrets
- **Peer Authentication**: Ed25519 digital signatures prevent man-in-the-middle attacks
- **Peer-to-Peer Networking**: Direct TCP connections between users with multi-threaded server
- **Real-Time Messaging**: Instant message delivery with efficient message caching
- **Fingerprint Verification**: Display identity fingerprints for out-of-band verification
- **Clean MVC Architecture**: Professional separation of concerns
- **Modern UI**: Built with egui for a native desktop experience

## 🏗️ Architecture

### MVC Structure
```
src/
├── models/              # Data structures and business logic
│   ├── message.rs       # Message data structure (encrypted storage with caching)
│   ├── crypt.rs         # AES-256-GCM encryption engine
│   ├── keyexchange.rs   # X25519 ECDH + Ed25519 authentication
│   └── network.rs       # Multi-threaded TCP client/server networking
├── views/               # User interface
│   └── chat_window.rs   # Main chat UI with encryption status
└── controllers/         # Application logic
    └── app.rs           # Connects models and views, manages key exchange
```

## 🔐 Security Features

### Cryptographic Protocols
- **AES-256-GCM**: Industry-standard authenticated encryption with 256-bit keys
- **X25519 ECDH**: Elliptic Curve Diffie-Hellman for secure key agreement
- **Ed25519 Signatures**: Digital signatures for peer authentication
- **SHA-256 KDF**: Key derivation function with domain separation

### Security Properties
- **Forward Secrecy Ready**: Ephemeral key pairs for each session
- **MITM Prevention**: Ed25519 signatures authenticate key exchange messages
- **No Plaintext Storage**: Messages stored encrypted with cached decryption
- **Encrypted Transmission**: Only encrypted data and key exchange messages sent
- **Nonce Randomization**: Unique 96-bit nonce per message
- **Fingerprint Verification**: Display identity fingerprints for out-of-band verification

## 🚀 Getting Started

### Prerequisites
- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))

### Building
```bash
cargo build --release
```

### Running

**User 1 (Alice on port 3000):**
```bash
cargo run 3000 Alice
```

**User 2 (Bob on port 3001):**
```bash
cargo run 3001 Bob
```

### Establishing Secure Connection
1. In Bob's window, connect to Alice: Set "Send to:" to `127.0.0.1:3000`
2. In Alice's window, set "Send to:" to `127.0.0.1:3001`
3. Click "🔑 Initiate Key Exchange" button (either side can initiate)
4. Wait for "✅ Encrypted with [peer]" status to appear
5. Verify fingerprints match (compare displayed fingerprints out-of-band if needed)
6. Type messages and click "🔐 Send" - messages are now end-to-end encrypted!

## 📦 Dependencies

- **eframe/egui**: Modern GUI framework for native desktop UI
- **aes-gcm**: AES-256-GCM authenticated encryption
- **x25519-dalek**: X25519 Elliptic Curve Diffie-Hellman
- **ed25519-dalek**: Ed25519 digital signatures
- **sha2**: SHA-256 cryptographic hash function
- **serde/serde_json**: Message serialization for network transmission
- **chrono**: Timestamp handling
- **rand**: Cryptographic randomness
- **hex**: Hexadecimal encoding for fingerprints

## 🛠️ Usage

### Command Line Arguments
```bash
cargo run [PORT] [USERNAME]
```

- `PORT`: Local listening port (default: 3000)
- `USERNAME`: Display name (default: User_[PORT])

### Examples
```bash
# Default settings
cargo run

# Custom port
cargo run 5000

# Custom port and username
cargo run 5000 Charlie
```

## 🔒 How It Works

### Key Exchange Flow
1. **Initialization**: Each user generates two key pairs:
   - **Identity Key Pair** (Ed25519): Long-term identity for signing
   - **Ephemeral DH Key Pair** (X25519): Session-specific for key exchange

2. **Key Exchange Protocol**:
   - Alice sends: `{dh_public_key, identity_public_key, signature, username}`
   - Signature proves Alice controls the DH public key (prevents MITM)
   - Bob verifies signature, computes shared secret: `x25519(bob_secret, alice_public)`
   - Bob responds with his own signed key exchange message
   - Alice verifies Bob's signature, computes: `x25519(alice_secret, bob_public)`
   - Both derive same shared secret due to ECDH property

3. **Key Derivation**:
   - Shared secret → SHA-256 KDF with both public keys (deterministic ordering)
   - Produces 256-bit AES encryption key

4. **Encrypted Messaging**:
   - Messages encrypted with AES-256-GCM before sending
   - Each message has unique random nonce for security
   - Decryption cached to avoid repeated computation

### Network Architecture
- **Multi-threaded Server**: Accepts connections on local port
- **Per-Connection Threads**: Each peer connection handled independently
- **Non-blocking Message Queue**: Messages received via mpsc channel
- **Client Thread**: Sends outgoing messages to target address

## 🔍 Security Considerations

### ✅ What This Protects Against
- **Eavesdropping**: All messages encrypted end-to-end
- **Tampering**: GCM authentication tag prevents modification
- **MITM Attacks**: Ed25519 signatures authenticate key exchange
- **Replay Attacks**: Unique nonces prevent message replay

### ⚠️ Current Limitations
- **No Forward Secrecy**: Key pairs persist for entire session (can be improved)
- **No Identity Persistence**: Identity keys regenerated each run
- **Trust On First Use**: Fingerprints should be verified out-of-band
- **No TLS/Transport Security**: TCP connections unprotected (ECDH provides end-to-end)
- **Single Session**: No rekeying or key rotation implemented

## 🚀 Future Enhancements

### Security Improvements
- **Signal Protocol**: Implement Double Ratchet for forward secrecy
- **Persistent Identity**: Store identity keys across sessions
- **Certificate/PKI**: Integrate with certificate system
- **TLS Transport**: Add TLS layer for defense-in-depth

### Features
- **Message Persistence**: Save encrypted message history
- **File Transfer**: Encrypt and send files
- **Group Chat**: Multi-party encrypted conversations
- **Read Receipts**: Message delivery confirmation
- **Typing Indicators**: Real-time typing status

### Infrastructure
- **Relay Server**: Support NAT traversal
- **Contact Discovery**: Find peers by username
- **Auto-Reconnect**: Handle network interruptions
- **Mobile Support**: Cross-platform deployment

## 📚 Learning Resources

For detailed networking concepts and implementation details, see [LESSON.md](LESSON.md).

## 📄 License

MIT License - See LICENSE file for details