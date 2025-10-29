//! Connection dialog handlers
//!
//! Handles dialogs for remote connections and search:
//! - Recursive search dialog
//! - Remote connection dialog (SFTP/SMB)

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{AppState, DialogState};
use crate::ui::connection_dialog::ConnectionDialogState;
use crate::events::keybindings::Action;

/// Handle recursive search dialog
pub fn handle_search_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    if let Some(dialog) = app.search_dialog.as_mut() {
        match dialog.handle_key(key) {
            crate::ui::search_dialog::DialogAction::Close => {
                app.close_search_dialog();
                Ok(Action::None)
            }
            crate::ui::search_dialog::DialogAction::Navigate(result) => {
                app.navigate_to_search_result(&result);
                Ok(Action::None)
            }
            crate::ui::search_dialog::DialogAction::Continue => {
                Ok(Action::None)
            }
        }
    } else {
        Ok(Action::None)
    }
}

/// Handle remote connection dialog (SFTP/SMB)
pub fn handle_connection_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    let should_close = matches!(key.code, KeyCode::Esc);
    
    if let Some(DialogState::RemoteConnection { ref mut state }) = app.dialog_state {
        match key.code {
            KeyCode::Esc => {
                // Will close below
            }
            _ => {
                // Handle based on state type
                match state {
                    ConnectionDialogState::SavedConnections { selected, connections } => {
                        match key.code {
                            KeyCode::Up => {
                                if *selected > 0 {
                                    *selected -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if *selected < connections.len().saturating_sub(1) {
                                    *selected += 1;
                                }
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                // New connection
                                *state = ConnectionDialogState::TypeSelection { selected: 0 };
                            }
                            KeyCode::Delete => {
                                // Delete selected connection
                                if let Ok(mut manager) = crate::remote::ConnectionManager::load() {
                                    if let Err(e) = manager.remove(*selected) {
                                        app.show_error(format!("Failed to delete connection: {}", e));
                                    } else {
                                        connections.remove(*selected);
                                        if *selected >= connections.len() && *selected > 0 {
                                            *selected -= 1;
                                        }
                                        if connections.is_empty() {
                                            *state = ConnectionDialogState::TypeSelection { selected: 0 };
                                        }
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                // Connect to selected
                                if let Some(mut config) = connections.get(*selected).cloned() {
                                    // Try to resolve password from keychain
                                    let password_resolved = match config.resolve_password() {
                                        Ok(Some(_pwd)) => {
                                            log::info!("Successfully retrieved password from keychain");
                                            true
                                        }
                                        Ok(None) => {
                                            log::warn!("No password in keychain for this connection");
                                            false
                                        }
                                        Err(e) => {
                                            log::error!("Failed to resolve password: {}", e);
                                            false
                                        }
                                    };
                                    
                                    if !password_resolved {
                                        // Password not available, show error and ask user to enter it
                                        app.show_error(format!(
                                            "Contraseña no disponible para '{}'. Por favor, crea una nueva conexión con tus credenciales.",
                                            config.name
                                        ));
                                        return Ok(Action::None);
                                    }
                                    
                                    // Try to connect
                                    use crate::remote::sftp::SftpFileSystem;
                                    use std::sync::Arc;
                                    
                                    log::info!("Attempting to connect to saved connection: {}", config.name);
                                    match SftpFileSystem::connect(config.clone()) {
                                        Ok(sftp_fs) => {
                                            log::info!("Connection successful!");
                                            let vfs: Arc<dyn crate::remote::VirtualFileSystem> = Arc::new(sftp_fs);
                                            let conn_info = format!("{}@{}", config.username, config.host);
                                            let success_msg = format!("Connected to {}!", conn_info);
                                            let initial_path = config.initial_path.clone().unwrap_or_else(|| std::path::PathBuf::from("/"));
                                            
                                            // Connect the active panel
                                            app.active_panel_mut().connect_remote(vfs, conn_info, initial_path);
                                            
                                            // Refresh directory
                                            if let Err(e) = app.active_panel_mut().refresh_entries() {
                                                log::error!("Failed to refresh remote directory: {}", e);
                                                app.error_message = Some(format!("Connected but failed to list directory: {}", e));
                                            } else {
                                                app.error_message = Some(success_msg);
                                            }
                                            
                                            // Close dialog
                                            app.dialog_state = None;
                                        }
                                        Err(e) => {
                                            log::error!("Connection failed: {}", e);
                                            app.show_error(format!("Connection failed: {}", e));
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    ConnectionDialogState::TypeSelection { selected } => {
                        match key.code {
                            KeyCode::Up => {
                                if *selected > 0 {
                                    *selected -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if *selected < 1 {  // 0=SFTP, 1=SMB
                                    *selected += 1;
                                }
                            }
                            KeyCode::Enter => {
                                let conn_type = *selected;
                                state.select_type(conn_type);
                            }
                            _ => {}
                        }
                    }
                    ConnectionDialogState::SftpForm {
                        selected_field,
                        name,
                        host,
                        port,
                        username,
                        auth_type,
                        password,
                        key_path,
                        save_credentials,
                        error,
                    } => {
                        match key.code {
                            KeyCode::Up => {
                                if *selected_field > 0 {
                                    *selected_field -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if *selected_field < 6 {  // Updated max field
                                    *selected_field += 1;
                                }
                            }
                            KeyCode::Tab if *selected_field == 4 => {
                                // Toggle auth type
                                *auth_type = 1 - *auth_type;
                            }
                            KeyCode::Char(' ') if *selected_field == 6 => {
                                // Toggle save_credentials checkbox
                                *save_credentials = !*save_credentials;
                            }
                            KeyCode::Char(c) => {
                                let field_value = match *selected_field {
                                    0 => name,
                                    1 => host,
                                    2 => port,
                                    3 => username,
                                    5 if *auth_type == 0 => password,
                                    5 => key_path,
                                    _ => return Ok(Action::None),
                                };
                                field_value.push(c);
                            }
                            KeyCode::Backspace => {
                                let field_value = match *selected_field {
                                    0 => name,
                                    1 => host,
                                    2 => port,
                                    3 => username,
                                    5 if *auth_type == 0 => password,
                                    5 => key_path,
                                    _ => return Ok(Action::None),
                                };
                                field_value.pop();
                            }
                            KeyCode::Enter => {
                                // Check if all required fields are filled
                                let has_all_fields = !name.is_empty() && !host.is_empty() && !username.is_empty() &&
                                    ((*auth_type == 0 && !password.is_empty()) || (*auth_type == 1 && !key_path.is_empty()));
                                
                                if has_all_fields {
                                    // Try to connect to SFTP
                                    use crate::remote::sftp::SftpFileSystem;
                                    use crate::remote::{AuthMethod, ConnectionConfig, ConnectionType};
                                    use std::sync::Arc;
                                    use std::path::PathBuf;
                                    
                                    log::info!("Attempting SFTP connection to {}@{}:{}", username, host, port);
                                    
                                    let auth = if *auth_type == 0 {
                                        AuthMethod::Password {
                                            password: Some(password.clone()),
                                            stored: *save_credentials,
                                        }
                                    } else {
                                        AuthMethod::PublicKey {
                                            key_path: PathBuf::from(key_path.clone()),
                                            passphrase: if password.is_empty() { None } else { Some(password.clone()) },
                                            stored: *save_credentials,
                                        }
                                    };
                                    
                                    let port_num: u16 = port.parse().unwrap_or(22);
                                    
                                    let config = ConnectionConfig {
                                        name: name.clone(),
                                        connection_type: ConnectionType::Sftp,
                                        host: host.clone(),
                                        port: port_num,
                                        username: username.clone(),
                                        auth,
                                        initial_path: Some(PathBuf::from("/")),
                                    };
                                    
                                    // Save password to keychain if requested
                                    if *save_credentials {
                                        if let Err(e) = config.store_password(password) {
                                            log::warn!("Failed to store password in keychain: {}", e);
                                        }
                                    }
                                    
                                    // Try to connect (blocking operation - will freeze UI)
                                    log::info!("Calling SftpFileSystem::connect...");
                                    match SftpFileSystem::connect(config.clone()) {
                                        Ok(sftp_fs) => {
                                            log::info!("SFTP connection successful!");
                                            
                                            // Save connection config
                                            if *save_credentials {
                                                if let Ok(mut manager) = crate::remote::ConnectionManager::load() {
                                                    if let Err(e) = manager.add(config) {
                                                        log::warn!("Failed to save connection: {}", e);
                                                    }
                                                }
                                            }
                                            
                                            let vfs: Arc<dyn crate::remote::VirtualFileSystem> = Arc::new(sftp_fs);
                                            let conn_info = format!("{}@{}", username, host);
                                            let success_msg = format!("Connected to {}!", conn_info);
                                            let initial_path = PathBuf::from("/");
                                            
                                            // Drop the borrow on state before modifying app
                                            let _ = state;
                                            
                                            // Connect the active panel
                                            app.active_panel_mut().connect_remote(vfs, conn_info, initial_path);
                                            
                                            // Immediately refresh to load directory contents
                                            if let Err(e) = app.active_panel_mut().refresh_entries() {
                                                log::error!("Failed to refresh remote directory after connect: {}", e);
                                                app.error_message = Some(format!("Connected but failed to list directory: {}", e));
                                            } else {
                                                app.error_message = Some(success_msg);
                                            }
                                            
                                            app.close_dialog();
                                            return Ok(Action::Refresh);
                                        }
                                        Err(e) => {
                                            log::error!("SFTP connection failed: {}", e);
                                            *error = Some(format!("Connection failed: {}", e));
                                        }
                                    }
                                } else {
                                    *error = Some("Please fill all required fields".to_string());
                                }
                            }
                            _ => {}
                        }
                    }
                    ConnectionDialogState::SmbForm {
                        selected_field,
                        name,
                        host,
                        share,
                        username,
                        password,
                        domain,
                        guest_mode,
                        save_credentials,
                        error,
                    } => {
                        match key.code {
                            KeyCode::Up => {
                                if *selected_field > 0 {
                                    *selected_field -= 1;
                                }
                            }
                            KeyCode::Down => {
                                let max_field = if *guest_mode { 6 } else { 7 };  // Less fields in guest mode
                                if *selected_field < max_field {
                                    *selected_field += 1;
                                }
                            }
                            KeyCode::Char(' ') if *selected_field == 6 => {
                                // Toggle guest_mode checkbox
                                *guest_mode = !*guest_mode;
                                // Clear username and password when entering guest mode
                                if *guest_mode {
                                    username.clear();
                                    password.clear();
                                }
                            }
                            KeyCode::Char(' ') if *selected_field == 7 && !*guest_mode => {
                                // Toggle save_credentials checkbox (only when not in guest mode)
                                *save_credentials = !*save_credentials;
                            }
                            KeyCode::Char(c) if !*guest_mode || *selected_field < 3 => {
                                // Allow input for name, host, share always
                                // Allow input for username, password, domain only when not in guest mode
                                let field_value = match *selected_field {
                                    0 => name,
                                    1 => host,
                                    2 => share,
                                    3 if !*guest_mode => username,
                                    4 if !*guest_mode => password,
                                    5 => domain,  // Domain always editable
                                    _ => return Ok(Action::None),
                                };
                                field_value.push(c);
                            }
                            KeyCode::Backspace if !*guest_mode || *selected_field < 3 => {
                                let field_value = match *selected_field {
                                    0 => name,
                                    1 => host,
                                    2 => share,
                                    3 if !*guest_mode => username,
                                    4 if !*guest_mode => password,
                                    5 => domain,
                                    _ => return Ok(Action::None),
                                };
                                field_value.pop();
                            }
                            KeyCode::Enter => {
                                // Validate required fields
                                let has_required_fields = !name.is_empty() && !host.is_empty() && !share.is_empty();
                                let has_auth = *guest_mode || (!username.is_empty() && !password.is_empty());
                                
                                if has_required_fields && has_auth {
                                    use crate::remote::ConnectionType;
                                    use crate::remote::AuthMethod;
                                    use std::path::PathBuf;
                                    
                                    let config = crate::remote::ConnectionConfig {
                                        name: name.clone(),
                                        connection_type: ConnectionType::Smb,
                                        host: host.clone(),
                                        port: 445,
                                        username: if *guest_mode { "guest".to_string() } else { username.clone() },
                                        auth: AuthMethod::Password {
                                            password: if *guest_mode { None } else { Some(password.clone()) },
                                            stored: *save_credentials,
                                        },
                                        initial_path: Some(PathBuf::from(format!("/{}", share))),
                                    };
                                    
                                    // Save password to keychain if requested
                                    if *save_credentials && !*guest_mode {
                                        if let Err(e) = config.store_password(password) {
                                            log::warn!("Failed to store password in keychain: {}", e);
                                        }
                                    }
                                    
                                    // Try to connect
                                    log::info!("Calling SmbFileSystem::connect...");
                                    match crate::remote::smb::SmbFileSystem::connect(config.clone()) {
                                        Ok(smb_fs) => {
                                            log::info!("SMB connection successful!");
                                            
                                            // Save connection config
                                            if *save_credentials && !*guest_mode {
                                                if let Ok(mut manager) = crate::remote::ConnectionManager::load() {
                                                    if let Err(e) = manager.add(config) {
                                                        log::warn!("Failed to save connection: {}", e);
                                                    }
                                                }
                                            }
                                            
                                            let vfs: std::sync::Arc<dyn crate::remote::VirtualFileSystem> = std::sync::Arc::new(smb_fs);
                                            let conn_info = format!("\\\\{}\\{}", host, share);
                                            let success_msg = format!("Connected to {}!", conn_info);
                                            let initial_path = PathBuf::from("/");
                                            
                                            // Drop the borrow on state before modifying app
                                            let _ = state;
                                            
                                            // Connect the active panel
                                            app.active_panel_mut().connect_remote(vfs, conn_info, initial_path);
                                            
                                            // Refresh to load directory contents
                                            if let Err(e) = app.active_panel_mut().refresh_entries() {
                                                log::error!("Failed to refresh remote directory after connect: {}", e);
                                                app.error_message = Some(format!("Connected but failed to list directory: {}", e));
                                            } else {
                                                app.error_message = Some(success_msg);
                                            }
                                            
                                            app.close_dialog();
                                            return Ok(Action::Refresh);
                                        }
                                        Err(e) => {
                                            log::error!("SMB connection failed: {}", e);
                                            *error = Some(format!("Connection failed: {}", e));
                                        }
                                    }
                                } else {
                                    *error = Some("Please fill all required fields".to_string());
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    
    if should_close {
        app.close_dialog();
    }
    
    Ok(Action::None)
}
