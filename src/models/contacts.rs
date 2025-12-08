// Model: Contact list management
// Stores known contacts with their connection info and identity fingerprints
// Persisted to ~/.secure-chat/contacts.json

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Contact information for a peer
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contact {
    pub username: String,
    pub address: String,  // IP:PORT format (e.g., "127.0.0.1:3001")
    pub fingerprint: String,  // Identity fingerprint for verification
    pub last_connected: Option<i64>,  // Unix timestamp of last successful connection
    pub notes: String,  // User notes about this contact
}

/// Contact list manager
pub struct ContactList {
    contacts: HashMap<String, Contact>,  // Key: username
    contacts_path: PathBuf,
}

impl ContactList {
    /// Load or create contact list
    pub fn load_or_create() -> Result<Self, String> {
        let contacts_path = Self::get_contacts_path()?;
        
        if contacts_path.exists() {
            Self::load_from_file(&contacts_path)
        } else {
            Ok(Self {
                contacts: HashMap::new(),
                contacts_path,
            })
        }
    }
    
    /// Get path to contacts file
    fn get_contacts_path() -> Result<PathBuf, String> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| "Failed to find config directory".to_string())?;
        
        let app_dir = config_dir.join("secure-chat");
        
        if !app_dir.exists() {
            fs::create_dir_all(&app_dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        
        Ok(app_dir.join("contacts.json"))
    }
    
    /// Load contacts from file
    fn load_from_file(path: &PathBuf) -> Result<Self, String> {
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read contacts file: {}", e))?;
        
        let contacts: HashMap<String, Contact> = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse contacts file: {}", e))?;
        
        println!("📇 Loaded {} contact(s)", contacts.len());
        
        Ok(Self {
            contacts,
            contacts_path: path.clone(),
        })
    }
    
    /// Save contacts to file
    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.contacts)
            .map_err(|e| format!("Failed to serialize contacts: {}", e))?;
        
        fs::write(&self.contacts_path, json)
            .map_err(|e| format!("Failed to write contacts file: {}", e))?;
        
        Ok(())
    }
    
    /// Add or update a contact
    pub fn add_contact(&mut self, contact: Contact) {
        println!("📝 Saving contact: {}", contact.username);
        self.contacts.insert(contact.username.clone(), contact);
        let _ = self.save(); // Ignore save errors for now
    }
    
    /// Get a contact by username
    pub fn get_contact(&self, username: &str) -> Option<&Contact> {
        self.contacts.get(username)
    }
    
    /// Update last connected timestamp for a contact
    pub fn update_last_connected(&mut self, username: &str) {
        if let Some(contact) = self.contacts.get_mut(username) {
            contact.last_connected = Some(chrono::Utc::now().timestamp());
            let _ = self.save();
        }
    }
}
