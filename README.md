# Secure Chat Application

A peer-to-peer encrypted messaging application built with Rust, featuring automatic peer discovery, end-to-end encryption using AES-256-GCM, X25519 ECDH key exchange with Ed25519 authentication, and real-time TCP networking.


## MVC Structure

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

## Security Features

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

## Usage
**Rust 1.70+**

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

### Then the application will:
- Auto-select an available port (49152-65535 range)
- Load or create your persistent Ed25519 identity
- Broadcast your presence via mDNS
- Auto-discover other instances on your network
- Auto-connect and establish encrypted channel
- Ready to send encrypted messages!

### Manual Port Selection (Optional)

```bash
# User 1 (Alice on port 5000)
cargo run 5000 Alice

# User 2 (Bob on port 5001)  
cargo run 5001 Bob
```

## Dependencies

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

## Usage

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
- **SEARCHING...** - Broadcasting presence and looking for peers
- **ENCRYPTED** - Secure connection established, ready to chat
- **Connected to [peer]** - Shows connected peer's username

**Message Display:**
- **Blue bubbles (right)**: Your messages
- **Gray bubbles (left)**: Peer's messages  
- **Large readable text**: 16px font for comfortable reading
- **Timestamps**: Displayed on each message

**Sending Messages:**
- Type in the input box at the bottom
- Click **Send** or press **Ctrl+Enter**
- Send button only enabled when connected and text entered

**Fingerprint Verification:**
- Identity fingerprints displayed at top when connected
- Compare with peer out-of-band (phone call, in person) to verify identity
- Warns if fingerprint changes (potential MITM attack)