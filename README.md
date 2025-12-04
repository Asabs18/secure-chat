# 🔒 Secure Chat Application

A peer-to-peer encrypted messaging application built with Rust, featuring end-to-end encryption using AES-256-GCM and real-time TCP networking.

## ✨ Features

- **End-to-End Encryption**: All messages encrypted with AES-256-GCM before transmission
- **Peer-to-Peer Networking**: Direct TCP connections between users
- **Real-Time Messaging**: Instant message delivery with auto-refresh
- **Multi-User Support**: Connect to different ports/IP addresses
- **Clean MVC Architecture**: Professional separation of concerns
- **Modern UI**: Built with egui for a native desktop experience

## 🏗️ Architecture

### MVC Structure
```
src/
├── models/          # Data structures and business logic
│   ├── message.rs   # Message data structure (encrypted storage)
│   ├── crypt.rs     # AES-256-GCM encryption engine
│   └── network.rs   # TCP client/server networking
├── views/           # User interface
│   └── chat_window.rs # Main chat UI with message history
└── controllers/     # Application logic
    └── app.rs       # Connects models and views
```

## 🔐 Security Features

- **AES-256-GCM Encryption**: Industry-standard authenticated encryption
- **No Plaintext Storage**: Messages stored encrypted only
- **Encrypted Transmission**: Only encrypted data sent over network
- **Nonce Randomization**: Unique nonce per message for security

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

### Connecting Users
1. In Alice's window, set "Send to:" field to `127.0.0.1:3001`
2. In Bob's window, set "Send to:" field to `127.0.0.1:3000`
3. Type messages and click "🔐 Send" or press Enter

## 📦 Dependencies

- **eframe/egui**: Modern GUI framework
- **aes-gcm**: AES-256-GCM encryption
- **serde/serde_json**: Message serialization
- **chrono**: Timestamp handling
- **rand**: Cryptographic randomness

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