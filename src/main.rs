// Main entry point for the Secure Chat application
// Handles command-line argument parsing and GUI initialization

mod models;
mod views;
mod controllers;
mod utils;

use controllers::app::ChatApp;
use std::env;
use utils::port::find_available_port;

fn main() -> eframe::Result<()> {
    // Parse command line arguments for port and username configuration
    let args: Vec<String> = env::args().collect();
    
    // First argument: Local listening port (default: auto-select)
    let port = args.get(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or_else(|| find_available_port());
    
    // Second argument: Username for display (default: User_[PORT])
    let username = args.get(2)
        .cloned()
        .unwrap_or_else(|| format!("User_{}", port));
    
    // Display startup information
    println!("🚀 Starting Secure Chat on port {} as '{}'", port, username);
    println!("💡 Usage: secure-chat [PORT] [USERNAME]");
    println!("   (Both arguments optional - port auto-selected if not specified)");
    println!();
    
    // Configure the native window options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 600.0])  // Set window dimensions
            .with_title(format!("Secure Chat - {}", username)),  // Set window title
        ..Default::default()
    };
    
    // Launch the GUI application with the chat controller
    eframe::run_native(
        "Secure Chat",
        options,
        Box::new(move |_cc| Ok(Box::new(ChatApp::new(port, username.clone())))),
    )
}