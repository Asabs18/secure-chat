# 🔒 Secure Chat Application

A peer-to-peer encrypted messaging application built with Rust, featuring **automatic peer discovery**, end-to-end encryption using AES-256-GCM, X25519 ECDH key exchange with Ed25519 authentication, and real-time TCP networking.

## ✨ Features

- **🔍 Automatic Peer Discovery**: mDNS/Bonjour service discovery - zero configuration needed
- **🤝 Auto-Connect**: Automatically discovers and connects to peers on your local network
- **🔐 End-to-End Encryption**: All messages encrypted with AES-256-GCM authenticated encryption
- **🔑 Secure Key Exchange**: X25519 ECDH for dynamic key agreement without transmitting secrets
- **✍️ Peer Authentication**: Ed25519 digital signatures prevent man-in-the-middle attacks
- **💾 Persistent Identity**: Ed25519 identity stored securely on disk, consistent fingerprint across sessions
- **📇 Contact Management**: Automatically saves contacts with fingerprint verification
- **🌐 Peer-to-Peer Networking**: Direct TCP connections with multi-threaded server
- **⚡ Real-Time Messaging**: Instant message delivery with efficient message caching
- **🎨 Modern UI**: Beautiful dark-themed interface with color-coded messages
- **📏 Clean Architecture**: Professional MVC structure with separation of concerns

## 🏗️ Architecture

### MVC Structure
```
src/
├── models/              # Data structures and business logic
│   ├── message.rs       # Message data with encrypted storage and caching
│   ├── crypt.rs         # AES-256-GCM encryption engine
│   ├── keyexchange.rs   # X25519 ECDH + Ed25519 authentication
│   ├── identity.rs      # Persistent Ed25519 identity management
│   ├── contacts.rs      # Contact list with fingerprint verification
│   ├── discovery.rs     # mDNS peer discovery service
│   └── network.rs       # Multi-threaded TCP client/server
├── views/               # User interface
│   └── chat_window.rs   # Modern chat UI with message bubbles
├── controllers/         # Application logic
│   └── app.rs           # Main controller coordinating all components
└── utils/
    └── port.rs          # Automatic port selection utility
```

## 🔐 Security Features

### Cryptographic Protocols
- **AES-256-GCM**: Industry-standard authenticated encryption with 256-bit keys
- **X25519 ECDH**: Elliptic Curve Diffie-Hellman for secure key agreement
- **Ed25519 Signatures**: Digital signatures for peer authentication and identity
- **SHA-256 KDF**: Key derivation function with sorted public key concatenation

### Security Properties
- **Forward Secrecy Ready**: Ephemeral X25519 key pairs generated per session
- **MITM Prevention**: Ed25519 signatures authenticate all key exchange messages
- **Persistent Identity**: Ed25519 identity persists across sessions for consistent verification
- **Fingerprint Verification**: 128-bit identity fingerprints displayed for out-of-band verification
- **No Plaintext Storage**: Messages stored encrypted with cached decryption
- **Encrypted Transmission**: Only encrypted data and signed key exchanges transmitted
- **Nonce Randomization**: Unique 96-bit random nonce per message
- **Contact Verification**: Warns on fingerprint mismatch to detect identity changes

## 🚀 Getting Started

### Prerequisites
- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- Windows/macOS/Linux with network access

### Building
```bash
cargo build --release
```

### Running

**Simply start the application - it will auto-discover peers:**

```bash
cargo run
```

Or specify a custom username:

```bash
cargo run -- 0 Alice
```

**That's it!** The application will:
1. ✅ Auto-select an available port (49152-65535 range)
2. ✅ Load or create your persistent Ed25519 identity
3. ✅ Broadcast your presence via mDNS
4. ✅ Auto-discover other instances on your network
5. ✅ Auto-connect and establish encrypted channel
6. ✅ Ready to send encrypted messages!

### Manual Port Selection (Optional)

```bash
# User 1 (Alice on port 5000)
cargo run 5000 Alice

# User 2 (Bob on port 5001)  
cargo run 5001 Bob
```

## 📦 Dependencies

### Core Cryptography
- **aes-gcm**: AES-256-GCM authenticated encryption
- **x25519-dalek**: X25519 Elliptic Curve Diffie-Hellman
- **ed25519-dalek**: Ed25519 digital signatures
- **sha2**: SHA-256 cryptographic hash function
- **rand**: Cryptographic randomness

### Networking & Discovery
- **mdns-sd**: mDNS/Bonjour service discovery protocol
- **hostname**: System hostname detection

### UI & Data
- **eframe/egui**: Modern native desktop GUI framework
- **serde/serde_json**: Message serialization
- **chrono**: Timestamp handling
- **hex**: Hexadecimal encoding for fingerprints
- **dirs**: Cross-platform config directory paths

## 🛠️ Usage

### Command Line Arguments
```bash
cargo run [PORT] [USERNAME]
```

- `PORT`: Local listening port (optional - auto-selected if omitted)
- `USERNAME`: Display name (optional - defaults to User_[PORT])

### Examples
```bash
# Auto-discovery mode (recommended)
cargo run                    # Auto-select port, auto-discover peers

# Custom username
cargo run -- 0 Alice         # Port 0 = auto-select

# Manual port selection
cargo run 5000 Charlie       # Listen on port 5000
```

### User Interface

**Status Indicators:**
- 🔍 **SEARCHING...** - Broadcasting presence and looking for peers
- 🔐 **ENCRYPTED** - Secure connection established, ready to chat
- ✓ **Connected to [peer]** - Shows connected peer's username

**Message Display:**
- **Blue bubbles (right)**: Your messages
- **Gray bubbles (left)**: Peer's messages  
- **Large readable text**: 16px font for comfortable reading
- **Timestamps**: Displayed on each message

**Sending Messages:**
- Type in the input box at the bottom
- Click **📤 Send** or press **Ctrl+Enter**
- Send button only enabled when connected and text entered

**Fingerprint Verification:**
- Identity fingerprints displayed at top when connected
- Compare with peer out-of-band (phone call, in person) to verify identity
- Warns if fingerprint changes (potential MITM attack)

## 🔒 How It Works

### Automatic Peer Discovery (mDNS)
1. **Service Broadcasting**: Application advertises itself as `_securechat._tcp.local.`
2. **Service Discovery**: Listens for other Secure Chat instances on the network
3. **IPv4 Preference**: Prioritizes IPv4 addresses for better compatibility
4. **Self-Filtering**: Automatically ignores own service announcements
5. **Auto-Connect**: First peer discovered triggers automatic connection

### Identity & Contact Management
1. **Persistent Identity**: Ed25519 key pair stored in `~/.secure-chat/identity.key`
   - Windows: `C:\Users\[user]\AppData\Roaming\secure-chat\`
   - macOS: `~/Library/Application Support/secure-chat/`
   - Linux: `~/.config/secure-chat/`
2. **Contact List**: Automatically saved to `contacts.json` in same directory
3. **Fingerprint Verification**: 128-bit SHA-256 hash of identity public key
4. **Automatic Updates**: Last connected timestamp updated on each session

### Key Exchange Protocol
1. **Initialization**: Each user has:
   - **Identity Key Pair** (Ed25519): Persistent identity for signing
   - **Ephemeral DH Key Pair** (X25519): Session-specific, regenerated each run

2. **Exchange Flow**:
   - Peer A sends: `{dh_public_key, identity_public_key, signature, username, listening_port}`
   - Signature proves Peer A controls the DH public key (prevents MITM)
   - Peer B verifies signature, computes shared secret: `x25519(b_secret, a_public)`
   - Peer B extracts listening_port and sets target address for responses
   - Peer B responds with signed key exchange message
   - Peer A verifies signature, computes: `x25519(a_secret, b_public)`
   - Both derive identical shared secret due to ECDH property

3. **Key Derivation**:
   - Shared secret → SHA-256(sorted concatenation of both public keys)
   - Produces deterministic 256-bit AES-GCM encryption key

4. **Encrypted Communication**:
   - All messages encrypted with AES-256-GCM before transmission
   - Unique random 96-bit nonce per message
   - Authentication tag prevents tampering
   - Decrypted text cached in memory to avoid repeated decryption

### Network Architecture
- **Multi-threaded TCP Server**: Accepts incoming connections
- **Per-Connection Handlers**: Each peer handled in separate thread
- **Non-blocking Channels**: mpsc channels for async message passing
- **Client Sender Thread**: Dedicated thread for outgoing messages
- **Automatic Port Selection**: Tries ephemeral port range (49152-65535)

## 🎨 UI Features

- **Modern Dark Theme**: Professional color scheme with gradient-style backgrounds
- **Message Bubbles**: Color-coded bubbles (blue for you, gray for peer)
- **Large Readable Text**: 16px font size for comfortable reading  
- **Connection Status**: Real-time status indicators with color coding
- **Fingerprint Display**: Always visible for easy verification
- **Smart Send Button**: Only enabled when connected and ready
- **Keyboard Shortcuts**: Ctrl+Enter to send messages quickly
- **Empty State**: Helpful message when no conversation yet