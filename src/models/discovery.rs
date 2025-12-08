// Model: Peer discovery using mDNS (multicast DNS)
// Automatically discovers other Secure Chat instances on the local network
// Uses Bonjour/Zeroconf protocol for service advertisement and discovery

use mdns_sd::{ServiceDaemon, ServiceInfo, ServiceEvent};
use std::sync::mpsc::{Receiver, channel};
use std::thread;

/// Discovered peer information
#[derive(Clone, Debug)]
pub struct DiscoveredPeer {
    pub username: String,
    pub address: String,  // IP:PORT format
    pub port: u16,
}

/// mDNS service discovery manager
pub struct PeerDiscovery {
    pub peer_rx: Receiver<DiscoveredPeer>,  // Receives discovered peers
    _service_daemon: ServiceDaemon,          // Keep daemon alive
    #[allow(dead_code)]  // Used in thread closure for filtering
    local_port: u16,                         // Our own port (to filter ourselves)
}

impl PeerDiscovery {
    /// Start peer discovery service
    /// 
    /// # Arguments
    /// * `port` - Local port this instance is listening on
    /// * `username` - Username to advertise
    /// 
    /// # Returns
    /// PeerDiscovery instance with receiver for discovered peers
    pub fn new(port: u16, username: String) -> Result<Self, String> {
        // Create mDNS service daemon
        let mdns = ServiceDaemon::new()
            .map_err(|e| format!("Failed to create mDNS daemon: {}", e))?;
        
        // Service type for Secure Chat
        let service_type = "_securechat._tcp.local.";
        
        // Create service info for advertising this instance
        let service_name = format!("{}_{}", username, port);
        let host_name = format!("{}.local.", hostname::get()
            .unwrap_or_else(|_| "unknown".into())
            .to_string_lossy());
        
        let service_info = ServiceInfo::new(
            service_type,
            &service_name,
            &host_name,
            "",  // No specific IP (use default)
            port,
            None,  // No TXT records for now
        ).map_err(|e| format!("Failed to create service info: {}", e))?
         .enable_addr_auto();  // Auto-detect local IP
        
        // Register our service (advertise ourselves)
        mdns.register(service_info)
            .map_err(|e| format!("Failed to register service: {}", e))?;
        
        println!("🔍 Broadcasting as '{}' on port {}", username, port);
        
        // Create channel for discovered peers
        let (peer_tx, peer_rx) = channel();
        
        // Browse for other Secure Chat instances
        let browser = mdns.browse(service_type)
            .map_err(|e| format!("Failed to browse for services: {}", e))?;
        
        // Clone port for thread
        let local_port = port;
        
        // Spawn thread to listen for discovered services
        thread::spawn(move || {
            loop {
                match browser.recv() {
                    Ok(event) => {
                        match event {
                            ServiceEvent::ServiceResolved(info) => {
                                // Extract peer information
                                let _hostname = info.get_hostname();
                                // Get IP address - prefer IPv4 over IPv6
                                let addresses = info.get_addresses();
                                
                                // Find first IPv4 address, fall back to IPv6 if none
                                let addr = addresses.iter()
                                    .find(|a| a.is_ipv4())
                                    .or_else(|| addresses.iter().next());
                                
                                if let Some(addr) = addr {
                                    let port = info.get_port();
                                    
                                    // Skip our own service
                                    if port == local_port {
                                        println!("🔍 Ignoring own service on port {}", port);
                                        continue;
                                    }
                                    
                                    let address = format!("{}:{}", addr, port);
                                    
                                    // Extract username from service name (format: username_port)
                                    let full_name = info.get_fullname();
                                    let username = full_name
                                        .split('.')
                                        .next()
                                        .unwrap_or("Unknown")
                                        .split('_')
                                        .next()
                                        .unwrap_or("Unknown")
                                        .to_string();
                                    
                                    println!("🔎 Discovered peer: {} at {}", username, address);
                                    
                                    let peer = DiscoveredPeer {
                                        username,
                                        address,
                                        port,
                                    };
                                    
                                    // Send to main app (ignore errors if receiver dropped)
                                    let _ = peer_tx.send(peer);
                                }
                            },
                            ServiceEvent::ServiceRemoved(_, full_name) => {
                                println!("🔌 Peer left: {}", full_name);
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        eprintln!("Discovery error: {}", e);
                        break;
                    }
                }
            }
        });
        
        Ok(Self {
            peer_rx,
            _service_daemon: mdns,
            local_port: port,
        })
    }
    
    /// Check for newly discovered peers (non-blocking)
    pub fn check_for_peers(&self) -> Option<DiscoveredPeer> {
        self.peer_rx.try_recv().ok()
    }
}
