// Utility: Auto port selection
// Finds available TCP port for server binding

use std::net::TcpListener;

/// Find an available TCP port
/// 
/// # Returns
/// Available port number, or default 3000 if search fails
pub fn find_available_port() -> u16 {
    // Try ports in the ephemeral range (49152-65535)
    for port in 49152..=65535 {
        if is_port_available(port) {
            return port;
        }
    }
    
    // Fallback: Try common user ports (3000-3100)
    for port in 3000..=3100 {
        if is_port_available(port) {
            return port;
        }
    }
    
    // Last resort: Use OS-assigned port
    match TcpListener::bind("0.0.0.0:0") {
        Ok(listener) => {
            if let Ok(addr) = listener.local_addr() {
                return addr.port();
            }
        }
        Err(_) => {}
    }
    
    // Ultimate fallback
    3000
}

/// Check if a port is available for binding
fn is_port_available(port: u16) -> bool {
    TcpListener::bind(format!("0.0.0.0:{}", port)).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_find_available_port() {
        let port = find_available_port();
        assert!(port >= 3000);
        assert!(port <= 65535);
    }
    
    #[test]
    fn test_port_availability() {
        // Bind to a port
        let listener = TcpListener::bind("0.0.0.0:0").unwrap();
        let bound_port = listener.local_addr().unwrap().port();
        
        // Port should not be available while bound
        assert!(!is_port_available(bound_port));
        
        drop(listener);
        
        // Port should be available after release
        assert!(is_port_available(bound_port));
    }
}
