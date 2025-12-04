// Models: Data structures and business logic
// Contains message structure, encryption engine, and networking layer

pub mod message;   // Message data structure with encrypted storage
pub mod crypt;     // AES-256-GCM encryption/decryption engine
pub mod network;   // TCP client/server networking