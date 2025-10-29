//! UI dialog handlers
//!
//! Handles general UI dialogs:
//! - Generic input dialog (for creating folders, adding bookmarks)
//! - Theme selector

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{AppState, DialogState};
use crate::events::keybindings::{Action, map_key_to_input_action};

/// Handle input dialog for folder creation or bookmark naming
pub fn handle_input_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    let action = map_key_to_input_action(key);
    
    match action {
        Action::ConfirmInput => {
            if let Some(value) = app.get_input_value()
                && !value.is_empty()
            {
                // TASK-008: Check if this is a bookmark operation
                if let Some(ref context) = app.error_message {
                    if context.starts_with("RENAME:") {
                        // Rename bookmark
                        // TASK-009: Sanitize bookmark name before renaming
                        let sanitized_name = crate::config::bookmarks::sanitize_bookmark_name(&value);
                        
                        if sanitized_name.is_empty() {
                            app.error_message = Some("Bookmark name cannot be empty".to_string());
                            app.dialog_state = Some(DialogState::Error {
                                message: "Bookmark name cannot be empty".to_string(),
                            });
                            return Ok(Action::None);
                        }
                        
                        let old_name = context.strip_prefix("RENAME:").unwrap();
                        if let Err(e) = app.bookmarks.rename(old_name, sanitized_name) {
                            app.error_message = Some(format!("Failed to rename bookmark: {}", e));
                            app.dialog_state = Some(DialogState::Error {
                                message: format!("Failed to rename bookmark: {}", e),
                            });
                        } else {
                            app.error_message = None;
                            app.close_dialog();
                            // Reopen bookmark manager
                            let state = crate::ui::bookmark_manager::BookmarkManagerState::new();
                            app.dialog_state = Some(DialogState::BookmarkManager { state });
                        }
                        return Ok(Action::None);
                    } else if context.starts_with("BOOKMARK:") {
                        // Add bookmark (context contains the path with BOOKMARK: prefix)
                        // TASK-009: Sanitize bookmark name before adding
                        let sanitized_name = crate::config::bookmarks::sanitize_bookmark_name(&value);
                        
                        if sanitized_name.is_empty() {
                            app.error_message = Some("Bookmark name cannot be empty".to_string());
                            app.dialog_state = Some(DialogState::Error {
                                message: "Bookmark name cannot be empty".to_string(),
                            });
                            return Ok(Action::None);
                        }
                        
                        let path_str = context.strip_prefix("BOOKMARK:").unwrap();
                        let path = std::path::PathBuf::from(path_str);
                        if let Err(e) = app.bookmarks.add(sanitized_name, path.clone()) {
                            app.error_message = Some(format!("Failed to add bookmark: {}", e));
                            app.dialog_state = Some(DialogState::Error {
                                message: format!("Failed to add bookmark: {}", e),
                            });
                        } else {
                            app.error_message = None;
                            app.close_dialog();
                            // Reopen bookmark manager
                            let state = crate::ui::bookmark_manager::BookmarkManagerState::new();
                            app.dialog_state = Some(DialogState::BookmarkManager { state });
                        }
                        return Ok(Action::None);
                    }
                }
                
                // Original behavior: create folder
                // BUG-001 FIX: Only close dialog if create_folder succeeds
                if create_folder(app, &value).is_ok() && app.error_message.is_none() {
                    app.close_dialog();
                } else {
                    // Keep dialog open to show error, but clear input to allow retry
                    // Actually, convert to error dialog
                    if let Some(err_msg) = app.error_message.take() {
                        app.dialog_state = Some(DialogState::Error {
                            message: err_msg,
                        });
                    }
                }
            } else {
                app.close_dialog();
            }
        }
        Action::InputChar(c) => {
            app.input_dialog_append(c);
        }
        Action::InputBackspace => {
            app.input_dialog_backspace();
        }
        Action::Cancel => {
            app.close_dialog();
        }
        Action::Quit => {
            return Ok(Action::Quit);
        }
        _ => {}
    }
    
    Ok(Action::None)
}

/// Create a new folder in the current directory
fn create_folder(app: &mut AppState, folder_name: &str) -> Result<()> {
    let panel = app.active_panel();
    let new_path = panel.current_path.join(folder_name);
    let vfs = panel.vfs.clone();
    
    // BUG-001 FIX: Check if directory already exists (use VFS if available)
    let exists = if let Some(vfs) = &vfs {
        vfs.exists(&new_path).unwrap_or(false)
    } else {
        new_path.exists()
    };
    
    if exists {
        // Check if it's a directory using VFS
        let is_dir = if let Some(vfs) = &vfs {
            vfs.metadata(&new_path)
                .map(|m| m.entry_type == crate::remote::VfsEntryType::Directory)
                .unwrap_or(false)
        } else {
            new_path.is_dir()
        };
        
        if is_dir {
            log::warn!("Folder already exists: {:?}", new_path);
            app.error_message = Some(format!("El directorio '{}' ya existe", folder_name));
            return Ok(()); // Don't crash, just show error
        } else {
            log::warn!("File with same name exists: {:?}", new_path);
            app.error_message = Some(format!("Ya existe un archivo con el nombre '{}'", folder_name));
            return Ok(());
        }
    }
    
    // T851d: Log folder creation
    log::info!("Creating folder: {:?}", new_path);
    
    // Create directory (use VFS if available)
    let result = if let Some(vfs) = vfs {
        vfs.create_dir(&new_path)
    } else {
        std::fs::create_dir(&new_path)
            .map_err(|e| anyhow::anyhow!("Failed to create directory: {}", e))
    };
    
    match result {
        Ok(_) => {
            log::info!("Folder created successfully: {:?}", new_path);
            // Refresh panel and store entries
            refresh_and_store(app)?;
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to create folder {:?}: {}", new_path, e);
            app.error_message = Some(format!("No se pudo crear el directorio: {}", e));
            Ok(()) // Don't crash, show error message instead
        }
    }
}

/// Helper to refresh panel entries and store unfiltered list
fn refresh_and_store(app: &mut AppState) -> Result<()> {
    app.active_panel_mut().refresh_entries()?;
    let entries = app.active_panel().entries.clone();
    app.store_all_entries(entries);
    Ok(())
}

/// Handle theme selector dialog (US5)
pub fn handle_theme_selector_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    match (key.code, key.modifiers) {
        // Up: move selection up
        (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
            if let Some(DialogState::ThemeSelector { selected, .. }) = &mut app.dialog_state
                && *selected > 0
            {
                *selected -= 1;
            }
        }
        // Down: move selection down
        (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => {
            if let Some(DialogState::ThemeSelector { selected, themes }) = &mut app.dialog_state
                && *selected < themes.len().saturating_sub(1)
            {
                *selected += 1;
            }
        }
        // Enter: apply selected theme
        (KeyCode::Enter, _) => {
            if let Some(DialogState::ThemeSelector { themes, selected }) = &app.dialog_state
                && let Some(theme) = themes.get(*selected).cloned()
            {
                // Apply theme immediately
                app.theme = theme;
                // Save state to persist theme selection
                let _ = app.save_state();
            }
            app.close_dialog();
        }
        // Escape: cancel without applying
        (KeyCode::Esc, _) => {
            app.close_dialog();
        }
        _ => {}
    }
    
    Ok(Action::None)
}
