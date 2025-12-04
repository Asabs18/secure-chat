// View: Chat window user interface
// Renders the main chat interface with message history and input
// Pure UI layer - no business logic

use crate::controllers::app::ChatApp;
use egui::{Color32, RichText};

/// Chat window view (stateless)
pub struct ChatWindow;

impl ChatWindow {
    /// Render the main chat interface
    /// 
    /// # Arguments
    /// * `ctx` - egui context for rendering
    /// * `app` - Mutable reference to app state
    pub fn render(ctx: &egui::Context, app: &mut ChatApp) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Header with app title
            ui.heading("🔒 Secure Chat");
            
            // Connection information bar
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("👤 User: {}", app.username)).strong());
                ui.separator();
                ui.label(RichText::new(format!("📡 Listening on port: {}", app.local_port)).color(Color32::GREEN));
                ui.separator();
                
                // Key exchange status
                if app.key_established {
                    ui.label(RichText::new("🔐 Encrypted").color(Color32::GREEN).strong());
                    if let Some(peer) = &app.peer_username {
                        ui.label(RichText::new(format!("with {}", peer)).color(Color32::LIGHT_GREEN));
                    }
                } else {
                    ui.label(RichText::new("⚠️ Not Encrypted").color(Color32::YELLOW).strong());
                }
            });
            
            ui.separator();
            
            // Target address and key exchange controls
            ui.horizontal(|ui| {
                ui.label("Send to:");
                ui.text_edit_singleline(&mut app.target_address);
                ui.label("(format: IP:PORT)");
                
                ui.separator();
                
                // Key exchange button
                if !app.key_established {
                    if ui.button("🔑 Initiate Key Exchange").clicked() {
                        app.initiate_key_exchange();
                    }
                } else {
                    ui.label(RichText::new("✓ Key Established").color(Color32::GREEN));
                }
            });
            
            // Show fingerprints for verification
            if app.key_established {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("Your fingerprint: {}", app.fingerprint)).small().monospace());
                    if let Some(peer_fp) = &app.peer_fingerprint {
                        ui.separator();
                        ui.label(RichText::new(format!("Peer fingerprint: {}", peer_fp)).small().monospace());
                    }
                });
            }
            
            ui.separator();
            
            // Message history area
            ui.group(|ui| {
                ui.label(RichText::new("📜 Chat History").heading());
                
                // Scrollable message list
                egui::ScrollArea::vertical()
                    .id_salt("chat_history")
                    .max_height(400.0)
                    .stick_to_bottom(true)  // Auto-scroll to newest messages
                    .show(ui, |ui| {
                        for msg in &app.messages {
                            let is_own = msg.sender_id == app.username;
                            
                            // Message bubble layout (left for others, right for own)
                            ui.horizontal(|ui| {
                                if is_own {
                                    ui.add_space(100.0);  // Push own messages to the right
                                }
                                
                                ui.group(|ui| {
                                    ui.vertical(|ui| {
                                        // Sender name with color coding
                                        let sender_color = if is_own { Color32::LIGHT_GREEN } else { Color32::LIGHT_BLUE };
                                        ui.label(RichText::new(&msg.sender_id).color(sender_color).strong());
                                        
                                        // Display cached decrypted message
                                        ui.label(msg.decrypted.as_ref().unwrap_or(&"[Decryption Failed]".to_string()));
                                        
                                        // Timestamp
                                        let time = chrono::DateTime::from_timestamp(msg.timestamp, 0)
                                            .map(|dt| dt.format("%H:%M:%S").to_string())
                                            .unwrap_or_else(|| "??:??:??".to_string());
                                        ui.label(RichText::new(time).small().weak());
                                    });
                                });
                                
                                if !is_own {
                                    ui.add_space(100.0);  // Push other messages to the left
                                }
                            });
                            
                            ui.add_space(5.0);
                        }
                    });
            });
            
            ui.separator();
            
            // Message input area
            ui.horizontal(|ui| {
                // Multi-line text input
                let response = ui.add_sized(
                    [ui.available_width() - 80.0, 60.0],
                    egui::TextEdit::multiline(&mut app.input_text)
                        .hint_text("Type your message...")
                );
                
                // Send button
                let send_button = ui.add_sized([70.0, 60.0], egui::Button::new("🔐 Send"));
                
                // Send on button click or Enter key (Shift+Enter for new line)
                if send_button.clicked() || 
                   (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift)) {
                    app.send_message();
                }
            });
            
            ui.separator();
            
            // Security notice footer
            if app.key_established {
                ui.label(RichText::new("🔒 All messages encrypted with AES-256-GCM using ECDH-derived keys").small().weak());
            } else {
                ui.label(RichText::new("⚠️ Click 'Initiate Key Exchange' to establish secure channel before sending messages").small().color(Color32::YELLOW));
            }
        });
    }
}