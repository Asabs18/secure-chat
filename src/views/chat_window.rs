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
            // Set custom style
            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 8.0);
            
            // Header with gradient background
            ui.vertical(|ui| {
                ui.visuals_mut().widgets.noninteractive.bg_fill = Color32::from_rgb(45, 55, 72);
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    ui.label(RichText::new("🔒 Secure Chat")
                        .heading()
                        .size(28.0)
                        .color(Color32::from_rgb(99, 230, 190)));
                });
                ui.add_space(10.0);
            });
            
            ui.add_space(5.0);
            
            // Connection information bar with colored background
            egui::Frame::none()
                .fill(Color32::from_rgb(30, 41, 59))
                .inner_margin(8.0)
                .rounding(5.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("👤 {}", app.username))
                            .size(14.0)
                            .color(Color32::from_rgb(147, 197, 253)));
                        ui.separator();
                        ui.label(RichText::new(format!("📡 Port {}", app.local_port))
                            .size(14.0)
                            .color(Color32::from_rgb(134, 239, 172)));
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if app.key_established {
                                ui.label(RichText::new("🔐 ENCRYPTED")
                                    .size(14.0)
                                    .strong()
                                    .color(Color32::from_rgb(74, 222, 128)));
                                if let Some(peer) = &app.peer_username {
                                    ui.label(RichText::new(format!("with {}", peer))
                                        .size(14.0)
                                        .color(Color32::from_rgb(167, 243, 208)));
                                }
                            } else {
                                ui.label(RichText::new("🔍 SEARCHING...")
                                    .size(14.0)
                                    .strong()
                                    .color(Color32::from_rgb(251, 191, 36)));
                            }
                        });
                    });
                });
            
            ui.add_space(5.0);
            
            // Show fingerprints for verification in colored frame
            if app.key_established {
                egui::Frame::none()
                    .fill(Color32::from_rgb(22, 30, 46))
                    .inner_margin(6.0)
                    .rounding(3.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("🔑 Your: {}", app.fingerprint))
                                .size(11.0)
                                .monospace()
                                .color(Color32::from_rgb(196, 181, 253)));
                            if let Some(peer_fp) = &app.peer_fingerprint {
                                ui.separator();
                                ui.label(RichText::new(format!("Peer: {}", peer_fp))
                                    .size(11.0)
                                    .monospace()
                                    .color(Color32::from_rgb(251, 207, 232)));
                            }
                        });
                    });
            }
            
            ui.add_space(10.0);
            
            // Message history area with custom styling
            egui::Frame::none()
                .fill(Color32::from_rgb(15, 23, 42))
                .inner_margin(12.0)
                .rounding(8.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("💬 Messages")
                        .size(18.0)
                        .strong()
                        .color(Color32::from_rgb(203, 213, 225)));
                    
                    ui.add_space(8.0);
                    
                    // Scrollable message list
                    egui::ScrollArea::vertical()
                        .id_salt("chat_history")
                        .max_height(450.0)
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            if app.messages.is_empty() {
                                ui.centered_and_justified(|ui| {
                                    ui.label(RichText::new("No messages yet. Start chatting!")
                                        .size(16.0)
                                        .color(Color32::from_rgb(100, 116, 139)));
                                });
                            }
                            
                            for msg in &app.messages {
                                let is_own = msg.sender_id == app.username;
                                
                                // Message bubble with alignment
                                ui.horizontal(|ui| {
                                    if is_own {
                                        ui.add_space(ui.available_width() * 0.2);
                                    }
                                    
                                    let bubble_color = if is_own {
                                        Color32::from_rgb(37, 99, 235)  // Blue for own messages
                                    } else {
                                        Color32::from_rgb(75, 85, 99)   // Gray for peer messages
                                    };
                                    
                                    egui::Frame::none()
                                        .fill(bubble_color)
                                        .inner_margin(12.0)
                                        .rounding(12.0)
                                        .show(ui, |ui| {
                                            ui.vertical(|ui| {
                                                // Sender name
                                                let name_color = if is_own {
                                                    Color32::from_rgb(191, 219, 254)
                                                } else {
                                                    Color32::from_rgb(156, 163, 175)
                                                };
                                                ui.label(RichText::new(&msg.sender_id)
                                                    .size(13.0)
                                                    .strong()
                                                    .color(name_color));
                                                
                                                ui.add_space(4.0);
                                                
                                                // Message text - LARGER FONT
                                                ui.label(RichText::new(msg.decrypted.as_ref()
                                                    .unwrap_or(&"[Decryption Failed]".to_string()))
                                                    .size(16.0)
                                                    .color(Color32::WHITE));
                                                
                                                ui.add_space(4.0);
                                                
                                                // Timestamp
                                                let time = chrono::DateTime::from_timestamp(msg.timestamp, 0)
                                                    .map(|dt| dt.format("%H:%M:%S").to_string())
                                                    .unwrap_or_else(|| "??:??:??".to_string());
                                                ui.label(RichText::new(time)
                                                    .size(11.0)
                                                    .color(if is_own {
                                                        Color32::from_rgb(147, 197, 253)
                                                    } else {
                                                        Color32::from_rgb(156, 163, 175)
                                                    }));
                                            });
                                        });
                                    
                                    if !is_own {
                                        ui.add_space(ui.available_width());
                                    }
                                });
                                
                                ui.add_space(8.0);
                            }
                        });
                });
            
            ui.add_space(10.0);
            ui.add_space(10.0);
            
            // Message input area with modern styling
            egui::Frame::none()
                .fill(Color32::from_rgb(30, 41, 59))
                .inner_margin(10.0)
                .rounding(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Multi-line text input with custom style
                        let text_edit = egui::TextEdit::multiline(&mut app.input_text)
                            .hint_text("Type your message here...")
                            .desired_width(ui.available_width() - 90.0)
                            .desired_rows(2);
                        
                        ui.add(text_edit);
                        
                        // Send button with gradient-like effect
                        let send_enabled = !app.input_text.trim().is_empty() && app.key_established;
                        let button_color = if send_enabled {
                            Color32::from_rgb(34, 197, 94)  // Green
                        } else {
                            Color32::from_rgb(75, 85, 99)   // Gray
                        };
                        
                        let send_button = ui.add_enabled(
                            send_enabled,
                            egui::Button::new(RichText::new("📤 Send")
                                .size(16.0)
                                .strong()
                                .color(Color32::WHITE))
                                .fill(button_color)
                                .min_size(egui::vec2(75.0, 50.0))
                        );
                        
                        // Send on button click or Ctrl+Enter
                        if send_button.clicked() || 
                           (ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl)) {
                            app.send_message();
                        }
                    });
                });
            
            ui.add_space(8.0);
            
            // Security notice footer
            egui::Frame::none()
                .fill(Color32::from_rgb(22, 30, 46))
                .inner_margin(6.0)
                .rounding(4.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if app.key_established {
                            ui.label(RichText::new("🔒 End-to-end encrypted with AES-256-GCM • X25519 ECDH key exchange • Ed25519 signatures")
                                .size(11.0)
                                .color(Color32::from_rgb(134, 239, 172)));
                        } else {
                            ui.label(RichText::new("⏳ Waiting for secure connection... Messages will be encrypted once connected")
                                .size(11.0)
                                .color(Color32::from_rgb(251, 191, 36)));
                        }
                    });
                });
        });
    }
}