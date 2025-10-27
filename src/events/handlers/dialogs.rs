//! Dialog event handlers
//! 
//! This module contains handlers for all dialog types in the application.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::{AppState, DialogState};
use crate::events::keybindings::{Action, map_key_to_input_action};
use crate::archive::formats::ArchiveFormat;
use crate::archive::compressor::CompressionLevel;
use crate::ui::connection_dialog::ConnectionDialogState;

// Re-export collision handlers
pub use super::collision::{continue_batch_operation, process_batch_without_collision_check, process_single_file_operation};

/// Handle collision prompt dialog
pub fn handle_collision(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    
    // Filter out key release events
    if key.kind != KeyEventKind::Press {
        return Ok(Action::None);
    }
    
    match (key.code, key.modifiers) {
        // Enter: confirm selected option
        (KeyCode::Enter, _) => {
            if let Some(DialogState::CollisionPrompt { 
                file_path,
                selected, 
                operation, 
                remaining_files,
                dest_path,
                source_vfs,
                dest_vfs,
            }) = &app.dialog_state {
                let selected_option = *selected;
                let operation_type = operation.clone();
                let collision_file = std::path::PathBuf::from(file_path);
                let remaining = remaining_files.clone();
                let dest = dest_path.clone();
                let src_vfs = source_vfs.clone();
                let dst_vfs = dest_vfs.clone();
                
                app.close_dialog();
                
                match selected_option {
                    0 => {
                        // Overwrite this file - process just this one, then continue with remaining
                        process_single_file_operation(&collision_file, &dest, &src_vfs, &dst_vfs, operation_type.clone(), true, app)?;
                        
                        // If there are remaining files, continue processing them
                        if !remaining.is_empty() {
                            continue_batch_operation(remaining, dest, src_vfs, dst_vfs, operation_type, app)?;
                        }
                        return Ok(Action::None);
                    }
                    1 => {
                        // Overwrite All - process this one and all remaining without checking
                        process_single_file_operation(&collision_file, &dest, &src_vfs, &dst_vfs, operation_type.clone(), true, app)?;
                        
                        // Process all remaining files with overwrite enabled
                        if !remaining.is_empty() {
                            process_batch_without_collision_check(remaining, dest, src_vfs, dst_vfs, operation_type, app)?;
                        }
                        return Ok(Action::None);
                    }
                    2 => {
                        // Rename - process this file with a new name, then continue with remaining
                        process_single_file_operation(&collision_file, &dest, &src_vfs, &dst_vfs, operation_type.clone(), false, app)?;
                        
                        // Continue with remaining files
                        if !remaining.is_empty() {
                            continue_batch_operation(remaining, dest, src_vfs, dst_vfs, operation_type, app)?;
                        }
                        return Ok(Action::None);
                    }
                    3 => {
                        // Skip this file - just continue with remaining
                        if !remaining.is_empty() {
                            continue_batch_operation(remaining, dest, src_vfs, dst_vfs, operation_type, app)?;
                        }
                        return Ok(Action::None);
                    }
                    4 => {
                        // Cancel - don't process anything
                        return Ok(Action::None);
                    }
                    _ => {}
                }
            }
        }
        // Up arrow: move selection up
        (KeyCode::Up, _) => {
            if let Some(DialogState::CollisionPrompt { selected, .. }) = &mut app.dialog_state
                && *selected > 0 {
                    *selected -= 1;
                }
        }
        // Down arrow: move selection down
        (KeyCode::Down, _) => {
            if let Some(DialogState::CollisionPrompt { selected, .. }) = &mut app.dialog_state
                && *selected < 4 {
                    *selected += 1;
                }
        }
        // Number keys: direct selection
        (KeyCode::Char('1'), _) => {
            if let Some(DialogState::CollisionPrompt { selected, .. }) = &mut app.dialog_state {
                *selected = 0;
            }
        }
        (KeyCode::Char('2'), _) => {
            if let Some(DialogState::CollisionPrompt { selected, .. }) = &mut app.dialog_state {
                *selected = 1;
            }
        }
        (KeyCode::Char('3'), _) => {
            if let Some(DialogState::CollisionPrompt { selected, .. }) = &mut app.dialog_state {
                *selected = 2;
            }
        }
        (KeyCode::Char('4'), _) => {
            if let Some(DialogState::CollisionPrompt { selected, .. }) = &mut app.dialog_state {
                *selected = 3;
            }
        }
        (KeyCode::Char('5'), _) => {
            if let Some(DialogState::CollisionPrompt { selected, .. }) = &mut app.dialog_state {
                *selected = 4;
            }
        }
        // Esc: cancel
        (KeyCode::Esc, _) => {
            app.close_dialog();
        }
        _ => {}
    }
    
    Ok(Action::None)
}

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
                    } else if !context.starts_with("Error") {
                        // Add bookmark (context contains the path)
                        // TASK-009: Sanitize bookmark name before adding
                        let sanitized_name = crate::config::bookmarks::sanitize_bookmark_name(&value);
                        
                        if sanitized_name.is_empty() {
                            app.error_message = Some("Bookmark name cannot be empty".to_string());
                            app.dialog_state = Some(DialogState::Error {
                                message: "Bookmark name cannot be empty".to_string(),
                            });
                            return Ok(Action::None);
                        }
                        
                        let path = std::path::PathBuf::from(context.clone());
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

/// Handle rename dialog
pub fn handle_rename_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    // Filter out key release events to prevent double processing
    if key.kind != KeyEventKind::Press {
        return Ok(Action::None);
    }
    
    match key.code {
        KeyCode::Enter => {
            if let Some(DialogState::Rename { value, old_path, include_extension, .. }) = &app.dialog_state {
                let new_name = value.trim();
                
                if new_name.is_empty() {
                    app.show_error("El nombre no puede estar vacío".to_string());
                    return Ok(Action::None);
                }
                
                // Build new path
                let parent = old_path.parent().unwrap_or(old_path.as_path());
                
                // If not including extension and this is a file, add back the original extension
                let final_name = if !include_extension && old_path.is_file() {
                    if let Some(ext) = old_path.extension() {
                        format!("{}.{}", new_name, ext.to_string_lossy())
                    } else {
                        new_name.to_string()
                    }
                } else {
                    new_name.to_string()
                };
                
                let new_path = parent.join(&final_name);
                
                // Check if name is the same
                if new_path == *old_path {
                    app.close_dialog();
                    return Ok(Action::None);
                }
                
                // Get VFS if we're on a remote filesystem
                let vfs = app.active_panel().vfs.clone();
                
                // Check if target already exists (use VFS if available)
                let exists = if let Some(vfs) = &vfs {
                    vfs.exists(&new_path).unwrap_or(false)
                } else {
                    new_path.exists()
                };
                
                if exists {
                    app.show_error(format!("Ya existe un archivo o directorio con el nombre '{}'", final_name));
                    return Ok(Action::None);
                }
                
                // Perform rename (use VFS if available)
                let result = if let Some(vfs) = vfs {
                    vfs.rename(old_path, &new_path)
                } else {
                    std::fs::rename(old_path, &new_path)
                        .map_err(|e| anyhow::anyhow!("Failed to rename: {}", e))
                };
                
                match result {
                    Ok(()) => {
                        log::info!("Renamed successfully: {:?} -> {:?}", old_path, new_path);
                        app.close_dialog();
                        
                        // Refresh panels
                        let _ = app.left_panel.refresh_entries();
                        app.left_all_entries = app.left_panel.entries.clone();
                        let _ = app.right_panel.refresh_entries();
                        app.right_all_entries = app.right_panel.entries.clone();
                    }
                    Err(e) => {
                        log::error!("Failed to rename: {}", e);
                        app.show_error(format!("Error al renombrar: {}", e));
                    }
                }
            }
        }
        KeyCode::Char(c) => {
            if let Some(DialogState::Rename { value, .. }) = &mut app.dialog_state {
                value.push(c);
            }
        }
        KeyCode::Backspace => {
            if let Some(DialogState::Rename { value, .. }) = &mut app.dialog_state {
                value.pop();
            }
        }
        KeyCode::Esc => {
            app.close_dialog();
        }
        _ => {}
    }
    
    Ok(Action::None)
}

/// Handle password input dialog (T843)
pub fn handle_password_input_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    // Filter out key release events to prevent double processing
    if key.kind != KeyEventKind::Press {
        return Ok(Action::None);
    }
    
    match (key.code, key.modifiers) {
        // Enter: confirm password (only if not empty)
        (KeyCode::Enter, _) => {
            if let Some(DialogState::PasswordInput { value, .. }) = &app.dialog_state
                && !value.is_empty() {
                    return Ok(Action::ConfirmYes);
                }
                // If password is empty, do nothing (user must enter password or press Esc to cancel)
            return Ok(Action::None);
        }
        // Tab: toggle password visibility
        (KeyCode::Tab, _) | (KeyCode::BackTab, _) => {
            if let Some(DialogState::PasswordInput { show_password, .. }) = &mut app.dialog_state {
                *show_password = !*show_password;
            }
        }
        // Backspace: delete character
        (KeyCode::Backspace, _) => {
            if let Some(DialogState::PasswordInput { value, .. }) = &mut app.dialog_state {
                value.pop();
            }
        }
        // Char: append to password
        (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
            if let Some(DialogState::PasswordInput { value, .. }) = &mut app.dialog_state {
                value.push(c);
            }
        }
        // Escape: cancel
        (KeyCode::Esc, _) => {
            app.close_dialog();
        }
        _ => {}
    }
    
    Ok(Action::None)
}

/// Handle recursive search dialog (TASK-040)
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

/// Handle compress options dialog
pub fn handle_compress_options_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    // Filter out key release events to prevent double navigation (same as BUG-001 fix)
    if key.kind != KeyEventKind::Press {
        return Ok(Action::None);
    }
    
    match (key.code, key.modifiers) {
        // Enter: confirm and start compression
        (KeyCode::Enter, _) => {
            if let Some(DialogState::CompressOptions {
                sources: _,
                output_name,
                format: _,
                level: _,
                use_password,
                password,
                confirm_password,
                ..
            }) = &app.dialog_state {
                // Validate output name
                if output_name.trim().is_empty() {
                    app.show_error("El nombre no puede estar vacío".to_string());
                    return Ok(Action::None);
                }
                
                // Validate passwords if enabled
                if *use_password {
                    if password.is_empty() {
                        app.show_error("La contraseña no puede estar vacía".to_string());
                        return Ok(Action::None);
                    }
                    if password != confirm_password {
                        app.show_error("Las contraseñas no coinciden".to_string());
                        return Ok(Action::None);
                    }
                }
                
                // Start compression (will be handled in event_loop.rs)
                return Ok(Action::ConfirmYes);
            }
        }
        // Tab or Down: move to next field
        (KeyCode::Tab, _) | (KeyCode::Down, _) => {
            if let Some(DialogState::CompressOptions { selected_field, use_password, .. }) = &mut app.dialog_state {
                let max_field = if *use_password { 5 } else { 3 };
                *selected_field = (*selected_field + 1).min(max_field);
            }
        }
        // Shift+Tab or Up: move to previous field
        (KeyCode::BackTab, _) | (KeyCode::Up, _) => {
            if let Some(DialogState::CompressOptions { selected_field, .. }) = &mut app.dialog_state
                && *selected_field > 0 {
                    *selected_field -= 1;
                }
        }
        // Left arrow: cycle format/level backwards
        (KeyCode::Left, _) => {
            if let Some(DialogState::CompressOptions { selected_field, format, level, .. }) = &mut app.dialog_state {
                match *selected_field {
                    1 => {
                        // Cycle format backwards
                        *format = match format {
                            ArchiveFormat::ZIP => ArchiveFormat::TarXz,
                            ArchiveFormat::TarGz => ArchiveFormat::ZIP,
                            ArchiveFormat::TarBz2 => ArchiveFormat::TarGz,
                            ArchiveFormat::TarXz => ArchiveFormat::TarBz2,
                            _ => ArchiveFormat::ZIP,
                        };
                    }
                    2 => {
                        // Cycle level backwards
                        *level = match level {
                            CompressionLevel::Fast => CompressionLevel::Maximum,
                            CompressionLevel::Normal => CompressionLevel::Fast,
                            CompressionLevel::Maximum => CompressionLevel::Normal,
                        };
                    }
                    _ => {}
                }
            }
        }
        // Right arrow: cycle format/level forwards
        (KeyCode::Right, _) => {
            if let Some(DialogState::CompressOptions { selected_field, format, level, .. }) = &mut app.dialog_state {
                match *selected_field {
                    1 => {
                        // Cycle format forwards
                        *format = match format {
                            ArchiveFormat::ZIP => ArchiveFormat::TarGz,
                            ArchiveFormat::TarGz => ArchiveFormat::TarBz2,
                            ArchiveFormat::TarBz2 => ArchiveFormat::TarXz,
                            ArchiveFormat::TarXz => ArchiveFormat::ZIP,
                            _ => ArchiveFormat::ZIP,
                        };
                    }
                    2 => {
                        // Cycle level forwards
                        *level = match level {
                            CompressionLevel::Fast => CompressionLevel::Normal,
                            CompressionLevel::Normal => CompressionLevel::Maximum,
                            CompressionLevel::Maximum => CompressionLevel::Fast,
                        };
                    }
                    _ => {}
                }
            }
        }
        // Space: toggle password checkbox
        (KeyCode::Char(' '), _) => {
            if let Some(DialogState::CompressOptions { selected_field, use_password, .. }) = &mut app.dialog_state
                && *selected_field == 3 {
                    *use_password = !*use_password;
                }
        }
        // Backspace: delete character from name or password fields
        (KeyCode::Backspace, _) => {
            if let Some(DialogState::CompressOptions { selected_field, output_name, password, confirm_password, .. }) = &mut app.dialog_state {
                match *selected_field {
                    0 => { output_name.pop(); }
                    4 => { password.pop(); }
                    5 => { confirm_password.pop(); }
                    _ => {}
                }
            }
        }
        // Char: append to name or password fields
        (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
            if (c != ' ' || matches!(key.code, KeyCode::Char(' ')))
                && let Some(DialogState::CompressOptions { selected_field, output_name, password, confirm_password, .. }) = &mut app.dialog_state {
                    match *selected_field {
                        0 => { output_name.push(c); }
                        4 => { password.push(c); }
                        5 => { confirm_password.push(c); }
                        _ => {}
                    }
                }
        }
        // Escape: cancel
        (KeyCode::Esc, _) => {
            app.close_dialog();
        }
        _ => {}
    }
    
    Ok(Action::None)
}

/// Handle drive selector dialog (US4)
pub fn handle_drive_selector_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    match (key.code, key.modifiers) {
        // Up: move selection up
        (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
            if let Some(DialogState::DriveSelector { selected, .. }) = &mut app.dialog_state
                && *selected > 0
            {
                *selected -= 1;
            }
        }
        // Down: move selection down
        (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => {
            if let Some(DialogState::DriveSelector { selected, drives }) = &mut app.dialog_state
                && *selected < drives.len().saturating_sub(1)
            {
                *selected += 1;
            }
        }
        // Enter: select drive and navigate to it
        (KeyCode::Enter, _) => {
            if let Some(DialogState::DriveSelector { drives, selected }) = &app.dialog_state
                && let Some((drive_path, _)) = drives.get(*selected)
            {
                let new_path = std::path::PathBuf::from(drive_path);
                // Change the active panel to this drive
                let panel = app.active_panel_mut();
                panel.current_path = new_path.clone();
                
                // Add to navigation history
                panel.history.push(new_path);
                
                panel.refresh_entries()?;
                panel.cursor = 0; // Reset cursor to top
            }
            app.close_dialog();
        }
        // Escape: cancel
        (KeyCode::Esc, _) => {
            app.close_dialog();
        }
        _ => {}
    }
    
    Ok(Action::None)
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

/// Handle bookmark manager dialog (TASK-008)
pub fn handle_bookmark_manager_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    match (key.code, key.modifiers) {
        // Up: move selection up
        (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
            let bookmark_count = app.bookmarks.count();
            if let Some(DialogState::BookmarkManager { state }) = &mut app.dialog_state {
                state.move_up(bookmark_count);
            }
        }
        // Down: move selection down
        (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => {
            let bookmark_count = app.bookmarks.count();
            if let Some(DialogState::BookmarkManager { state }) = &mut app.dialog_state {
                state.move_down(bookmark_count);
            }
        }
        // Enter: navigate to selected bookmark
        (KeyCode::Enter, _) => {
            // Clone bookmark info before mutating app
            let bookmark_info = if let Some(DialogState::BookmarkManager { state }) = &app.dialog_state {
                app.bookmarks.get_all()
                    .get(state.selected)
                    .map(|b| (b.name.clone(), b.path.clone(), b.path_exists()))
            } else {
                None
            };
            
            if let Some((name, path, exists)) = bookmark_info {
                if exists {
                    // Navigate active panel to bookmarked path
                    let panel = app.active_panel_mut();
                    panel.current_path = path.clone();
                    
                    // Add to navigation history
                    panel.history.push(path);
                    
                    // Refresh panel entries to load the new directory
                    panel.refresh_entries()?;
                    let entries = app.active_panel().entries.clone();
                    app.store_all_entries(entries);
                    
                    // Clear selection marks when navigating
                    app.selection_state.clear(app.active_panel);
                    
                    // Exit search mode if active
                    if app.search_mode {
                        app.search_mode = false;
                        app.search_pattern.clear();
                    }
                    
                    // Update last accessed timestamp
                    let _ = app.bookmarks.access(&name);
                    
                    app.close_dialog();
                    return Ok(Action::None);
                } else {
                    // Show error for invalid bookmark
                    app.error_message = Some(format!("Bookmark path does not exist: {}", path.display()));
                    app.close_dialog();
                }
            }
        }
        // 'a': add current directory as bookmark
        (KeyCode::Char('a'), KeyModifiers::NONE) | (KeyCode::Char('A'), KeyModifiers::NONE) => {
            let current_path = app.active_panel().current_path.clone();
            let default_name = current_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("bookmark")
                .to_string();
            
            // Close bookmark manager and open input dialog
            app.close_dialog();
            app.dialog_state = Some(DialogState::Input {
                prompt: format!("Add bookmark for: {}", current_path.display()),
                value: default_name,
            });
            // Store the path in error_message temporarily (we'll use it when input is confirmed)
            app.error_message = Some(current_path.display().to_string());
        }
        // 'r': rename selected bookmark
        (KeyCode::Char('r'), KeyModifiers::NONE) | (KeyCode::Char('R'), KeyModifiers::NONE) => {
            let bookmark_info = if let Some(DialogState::BookmarkManager { state }) = &app.dialog_state {
                app.bookmarks.get_all()
                    .get(state.selected)
                    .map(|b| b.name.clone())
            } else {
                None
            };
            
            if let Some(old_name) = bookmark_info {
                // Close bookmark manager and open input dialog
                app.close_dialog();
                app.dialog_state = Some(DialogState::Input {
                    prompt: format!("Rename bookmark: {}", old_name),
                    value: old_name.clone(),
                });
                // Store the old name in error_message temporarily
                app.error_message = Some(format!("RENAME:{}", old_name));
            }
        }
        // 'd': delete selected bookmark
        (KeyCode::Char('d'), KeyModifiers::NONE) | (KeyCode::Char('D'), KeyModifiers::NONE) => {
            let bookmark_name = if let Some(DialogState::BookmarkManager { state }) = &app.dialog_state {
                app.bookmarks.get_all()
                    .get(state.selected)
                    .map(|b| b.name.clone())
            } else {
                None
            };
            
            if let Some(name) = bookmark_name {
                if let Err(e) = app.bookmarks.remove(&name) {
                    app.error_message = Some(format!("Failed to delete bookmark: {}", e));
                }
                
                // Update selection if needed
                if let Some(DialogState::BookmarkManager { state }) = &mut app.dialog_state {
                    let new_count = app.bookmarks.count();
                    if state.selected >= new_count && new_count > 0 {
                        state.selected = new_count - 1;
                    }
                    if new_count == 0 {
                        state.reset_selection();
                    }
                }
            }
        }
        // 'c': clean invalid bookmarks
        (KeyCode::Char('c'), KeyModifiers::NONE) | (KeyCode::Char('C'), KeyModifiers::NONE) => {
            match app.bookmarks.clean_invalid() {
                Ok(removed) => {
                    if removed > 0 {
                        app.error_message = Some(format!("Removed {} invalid bookmark(s)", removed));
                        
                        // Reset selection
                        if let Some(DialogState::BookmarkManager { state }) = &mut app.dialog_state {
                            state.reset_selection();
                        }
                    } else {
                        app.error_message = Some("No invalid bookmarks found".to_string());
                    }
                }
                Err(e) => {
                    app.error_message = Some(format!("Failed to clean bookmarks: {}", e));
                }
            }
        }
        // Escape: close dialog
        (KeyCode::Esc, _) | (KeyCode::Char('b'), KeyModifiers::CONTROL) | (KeyCode::Char('B'), KeyModifiers::CONTROL) => {
            app.close_dialog();
        }
        _ => {}
    }
    
    Ok(Action::None)
}

/// Handle navigation history viewer dialog (TASK-018)
pub fn handle_history_viewer_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    // First, collect the data we need from the immutable borrow
    let history_entries = if let Some(DialogState::HistoryViewer { .. }) = &app.dialog_state {
        let panel = app.active_panel();
        panel.history.get_all().to_vec()
    } else {
        return Ok(Action::None);
    };
    
    let history_count = history_entries.len();
    
    // Now handle the event with mutable borrow
    if let Some(DialogState::HistoryViewer { state }) = &mut app.dialog_state {
        match (key.code, key.modifiers) {
            // Navigation
            (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
                state.move_up(history_count);
            }
            (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => {
                state.move_down(history_count);
            }
            (KeyCode::Home, _) | (KeyCode::Char('g'), KeyModifiers::NONE) => {
                state.selected = 0;
            }
            (KeyCode::End, _) | (KeyCode::Char('G'), KeyModifiers::SHIFT) => {
                if history_count > 0 {
                    state.selected = history_count - 1;
                }
            }
            // Enter: Navigate to selected directory
            (KeyCode::Enter, _) => {
                if history_count > 0 && state.selected < history_count {
                    // Reverse index since UI shows most recent first
                    let reversed_index = history_count - 1 - state.selected;
                    let selected_path = history_entries[reversed_index].clone();
                    
                    // Check if path still exists
                    if selected_path.exists() {
                        // Close dialog first
                        app.close_dialog();
                        
                        // Navigate to the selected path
                        let panel = app.active_panel_mut();
                        panel.current_path = selected_path.clone();
                        let _ = panel.refresh_entries();
                        panel.cursor = 0;
                        panel.scroll_offset = 0;
                    } else {
                        app.error_message = Some(format!(
                            "Directory no longer exists: {}",
                            selected_path.display()
                        ));
                    }
                }
            }
            // 'c': Clean invalid paths from history
            (KeyCode::Char('c'), KeyModifiers::NONE) | (KeyCode::Char('C'), KeyModifiers::NONE) => {
                // Clean invalid paths
                let removed_count = {
                    let panel = app.active_panel_mut();
                    panel.history.clean_invalid()
                };
                
                // Get new count and update selection
                let new_count = {
                    let panel = app.active_panel();
                    panel.history.count()
                };
                
                if let Some(DialogState::HistoryViewer { state }) = &mut app.dialog_state {
                    if state.selected >= new_count && new_count > 0 {
                        state.selected = new_count - 1;
                    } else if new_count == 0 {
                        state.selected = 0;
                    }
                }
                
                // Set message
                if removed_count > 0 {
                    app.error_message = Some(format!("Removed {} invalid path(s) from history", removed_count));
                } else {
                    app.error_message = Some("All paths in history are valid".to_string());
                }
            }
            // Escape: Close dialog
            (KeyCode::Esc, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) | (KeyCode::Char('H'), KeyModifiers::CONTROL) => {
                app.close_dialog();
            }
            _ => {}
        }
    }
    
    Ok(Action::None)
}

/// Handle Go To Path dialog (TASK-021)
pub fn handle_goto_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    // Get current directory before mutable borrow (needed for Enter handler)
    let current_dir = app.active_panel().current_path.clone();
    
    if let Some(DialogState::GoToPath { input, error_message, suggestions, selected_suggestion }) = &mut app.dialog_state {
        match (key.code, key.modifiers) {
            // Text input: add character to input
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                input.push(c);
                *error_message = None; // Clear error when typing
                
                // Update suggestions based on new input
                *suggestions = get_suggestions_for_input(input, &current_dir);
                *selected_suggestion = 0;
            }
            // Backspace: remove last character
            (KeyCode::Backspace, _) => {
                input.pop();
                *error_message = None; // Clear error when editing
                
                // Update suggestions based on new input
                *suggestions = get_suggestions_for_input(input, &current_dir);
                *selected_suggestion = 0;
            }
            // Tab: Autocomplete
            (KeyCode::Tab, _) => {
                if !suggestions.is_empty() {
                    let completed = autocomplete_path(input, suggestions);
                    *input = completed;
                    
                    // Update suggestions again after completion
                    *suggestions = get_suggestions_for_input(input, &current_dir);
                    *selected_suggestion = 0;
                }
            }
            // Up/Down: Navigate suggestions
            (KeyCode::Up, _) => {
                if !suggestions.is_empty() && *selected_suggestion > 0 {
                    *selected_suggestion -= 1;
                } else if !suggestions.is_empty() {
                    // Wrap around to last item
                    *selected_suggestion = suggestions.len() - 1;
                }
            }
            (KeyCode::Down, _) => {
                if !suggestions.is_empty() && *selected_suggestion + 1 < suggestions.len() {
                    *selected_suggestion += 1;
                } else if !suggestions.is_empty() {
                    // Wrap around to first item
                    *selected_suggestion = 0;
                }
            }
            // Ctrl+V: Paste from clipboard (placeholder - clipboard not implemented yet)
            (KeyCode::Char('v'), KeyModifiers::CONTROL) | (KeyCode::Char('V'), KeyModifiers::CONTROL) => {
                // TODO: Implement clipboard paste if crossterm supports it
                // For now, just clear error
                *error_message = None;
            }
            // Enter: Select highlighted suggestion OR validate and navigate
            (KeyCode::Enter, _) => {
                // If there are suggestions, use Enter to select the highlighted one
                if !suggestions.is_empty() {
                    let selected_path = &suggestions[*selected_suggestion];
                    if let Some(path_str) = selected_path.to_str() {
                        // Complete to the selected path with trailing separator
                        *input = format!("{}{}", path_str, std::path::MAIN_SEPARATOR);
                        
                        // Update suggestions for the new path
                        *suggestions = get_suggestions_for_input(input, &current_dir);
                        *selected_suggestion = 0;
                        *error_message = None;
                        return Ok(Action::None);
                    }
                }
                
                // No suggestions or couldn't convert path, try to navigate
                let input_path = input.trim().to_string();
                
                if input_path.is_empty() {
                    *error_message = Some("Path cannot be empty".to_string());
                    return Ok(Action::None);
                }
                
                // Expand and validate path
                match expand_and_validate_path(&input_path, &current_dir) {
                    Ok(validated_path) => {
                        // Close dialog
                        app.close_dialog();
                        
                        // Navigate to the path
                        let panel = app.active_panel_mut();
                        panel.current_path = validated_path.clone();
                        
                        // Add to navigation history
                        panel.history.push(validated_path);
                        
                        // Refresh panel
                        if let Err(e) = panel.refresh_entries() {
                            app.error_message = Some(format!("Failed to read directory: {}", e));
                        } else {
                            panel.cursor = 0;
                            panel.scroll_offset = 0;
                        }
                    }
                    Err(e) => {
                        *error_message = Some(e);
                    }
                }
            }
            // Escape: Close dialog
            (KeyCode::Esc, _) | (KeyCode::Char('g'), KeyModifiers::CONTROL) | (KeyCode::Char('G'), KeyModifiers::CONTROL) => {
                app.close_dialog();
            }
            _ => {}
        }
    }
    
    Ok(Action::None)
}

/// Handle remote connection dialog
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
                                if *selected < 2 {  // 0=SFTP, 1=FTP, 2=FTPS
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
                    ConnectionDialogState::FtpForm {
                        selected_field,
                        name,
                        host,
                        port,
                        username,
                        password,
                        use_tls,
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
                            KeyCode::Tab if *selected_field == 5 => {
                                // Toggle TLS
                                *use_tls = !*use_tls;
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
                                    4 => password,
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
                                    4 => password,
                                    _ => return Ok(Action::None),
                                };
                                field_value.pop();
                            }
                            KeyCode::Enter => {
                                // Check if all required fields are filled
                                let has_all_fields = !name.is_empty() && !host.is_empty() && !username.is_empty() && !password.is_empty();
                                
                                if has_all_fields {
                                    // Close dialog and show success
                                    let msg = format!("FTP connection '{}' configured!", name);
                                    app.error_message = Some(msg);
                                    app.close_dialog();
                                    return Ok(Action::None);
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

// Helper functions for GoTo dialog

/// Expand environment variables and validate path
fn expand_and_validate_path(input: &str, current_dir: &std::path::Path) -> Result<std::path::PathBuf, String> {
    use std::path::PathBuf;
    
    let input = input.trim();
    
    // Expand ~ to home directory
    let expanded = if let Some(rest) = input.strip_prefix('~') {
        if let Some(home) = dirs::home_dir() {
            if rest.is_empty() {
                home.to_string_lossy().to_string()
            } else {
                format!("{}{}", home.display(), rest)
            }
        } else {
            return Err("Could not determine home directory".to_string());
        }
    } else {
        input.to_string()
    };
    
    // Expand environment variables (%VAR% on Windows, $VAR on Unix)
    let expanded = expand_env_vars(&expanded);
    
    // Create PathBuf
    let mut path = PathBuf::from(&expanded);
    
    // If relative path, resolve from current directory
    if path.is_relative() {
        path = current_dir.join(path);
    }
    
    // Canonicalize to resolve .. and . components
    let path = match path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            return Err(format!("Invalid path: {}", e));
        }
    };
    
    // Check if path exists
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    
    // Check if it's a directory
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path.display()));
    }
    
    // Check if we have read permissions
    if let Err(e) = std::fs::read_dir(&path) {
        return Err(format!("Cannot access directory: {}", e));
    }
    
    // Clean up Windows UNC prefix (\\?\) for display
    let cleaned_path = clean_windows_path(path);
    
    Ok(cleaned_path)
}

/// Remove Windows UNC prefix (\\?\) from canonicalized paths
#[cfg(target_os = "windows")]
fn clean_windows_path(path: std::path::PathBuf) -> std::path::PathBuf {
    use std::path::PathBuf;
    
    let path_str = path.to_string_lossy();
    if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
        // Remove the \\?\ prefix
        PathBuf::from(stripped)
    } else {
        path
    }
}

#[cfg(not(target_os = "windows"))]
fn clean_windows_path(path: std::path::PathBuf) -> std::path::PathBuf {
    path
}

/// Expand environment variables in the path string
fn expand_env_vars(input: &str) -> String {
    let mut result = input.to_string();
    
    // Windows style: %VARIABLE%
    #[cfg(target_os = "windows")]
    {
        while let Some(start) = result.find('%') {
            if let Some(end) = result[start + 1..].find('%') {
                let var_name = &result[start + 1..start + 1 + end];
                if let Ok(value) = std::env::var(var_name) {
                    result = format!("{}{}{}", &result[..start], value, &result[start + 2 + end..]);
                } else {
                    break; // Variable not found, stop processing
                }
            } else {
                break; // No closing %, stop processing
            }
        }
    }
    
    // Unix style: $VARIABLE or ${VARIABLE}
    #[cfg(not(target_os = "windows"))]
    {
        // Handle ${VAR} style
        while let Some(start) = result.find("${") {
            if let Some(end) = result[start..].find('}') {
                let var_name = &result[start + 2..start + end];
                if let Ok(value) = std::env::var(var_name) {
                    result = format!("{}{}{}", &result[..start], value, &result[start + end + 1..]);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        // Handle $VAR style (simple case)
        while let Some(start) = result.find('$') {
            let rest = &result[start + 1..];
            let end = rest.chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .count();
            
            if end > 0 {
                let var_name = &rest[..end];
                if let Ok(value) = std::env::var(var_name) {
                    result = format!("{}{}{}", &result[..start], value, &result[start + 1 + end..]);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
    
    result
}

/// Get all subdirectories of the given path
pub fn get_directory_children(path: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut children = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata()
                && metadata.is_dir() {
                    children.push(entry.path());
                }
        }
    }
    
    // Sort alphabetically for consistent display
    children.sort();
    children
}

/// Expand path variables (~, %VAR%, $VAR) without validation
fn expand_path_variables_only(input: &str) -> String {
    let input = input.trim();
    
    // Expand ~ to home directory
    let expanded = if let Some(rest) = input.strip_prefix('~') {
        if let Some(home) = dirs::home_dir() {
            if rest.is_empty() {
                home.to_string_lossy().to_string()
            } else {
                format!("{}{}", home.display(), rest)
            }
        } else {
            input.to_string()
        }
    } else {
        input.to_string()
    };
    
    // Expand environment variables
    expand_env_vars(&expanded)
}

/// Get suggestions based on current input and working directory
fn get_suggestions_for_input(input: &str, current_dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    use std::path::Path;
    
    if input.is_empty() {
        // Show children of current directory when input is empty
        return get_directory_children(current_dir);
    }
    
    // Expand the input path
    let expanded = expand_path_variables_only(input);
    let input_path = Path::new(&expanded);
    
    // Check if input ends with a path separator (means we want to list that directory)
    let ends_with_separator = input.ends_with(std::path::MAIN_SEPARATOR) 
        || input.ends_with('/') 
        || input.ends_with('\\');
    
    // Determine the directory to list and the prefix to filter by
    let (dir_to_list, filter_prefix) = if input_path.is_absolute() {
        // If ends with separator or path exists as a directory, list its children
        if ends_with_separator && input_path.exists() && input_path.is_dir() {
            (input_path.to_path_buf(), String::new())
        } else if !ends_with_separator && input_path.exists() && input_path.is_dir() {
            // Path exists as complete directory, list its children
            (input_path.to_path_buf(), String::new())
        } else if let Some(parent) = input_path.parent() {
            // Path is partial, list parent and filter
            if parent.exists() && parent.is_dir() {
                let prefix = input_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                (parent.to_path_buf(), prefix)
            } else {
                // Parent doesn't exist, fallback to current dir
                (current_dir.to_path_buf(), String::new())
            }
        } else {
            // No parent (root), list the root itself
            (input_path.to_path_buf(), String::new())
        }
    } else {
        // Relative path: resolve against current_dir
        let absolute = current_dir.join(input_path);
        
        if ends_with_separator && absolute.exists() && absolute.is_dir() {
            (absolute, String::new())
        } else if !ends_with_separator && absolute.exists() && absolute.is_dir() {
            // Complete directory exists, list its children
            (absolute, String::new())
        } else if let Some(parent) = absolute.parent() {
            if parent.exists() && parent.is_dir() {
                let prefix = absolute.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                (parent.to_path_buf(), prefix)
            } else {
                (current_dir.to_path_buf(), String::new())
            }
        } else {
            (current_dir.to_path_buf(), String::new())
        }
    };
    
    // Get children of the directory
    let children = get_directory_children(&dir_to_list);
    
    // Filter by prefix if any
    if filter_prefix.is_empty() {
        children
    } else {
        let prefix_lower = filter_prefix.to_lowercase();
        children
            .into_iter()
            .filter(|path| {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    name.to_lowercase().starts_with(&prefix_lower)
                } else {
                    false
                }
            })
            .collect()
    }
}

/// Autocomplete path based on suggestions (bash-like behavior)
fn autocomplete_path(input: &str, suggestions: &[std::path::PathBuf]) -> String {
    if suggestions.is_empty() {
        return input.to_string();
    }
    
    // If only one suggestion, complete to that path
    if suggestions.len() == 1 {
        if let Some(path_str) = suggestions[0].to_str() {
            // Add trailing separator to indicate it's a directory
            return format!("{}{}", path_str, std::path::MAIN_SEPARATOR);
        }
        return input.to_string();
    }
    
    // Multiple suggestions: find common prefix
    let names: Vec<String> = suggestions
        .iter()
        .filter_map(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .collect();
    
    if names.is_empty() {
        return input.to_string();
    }
    
    // Find common prefix among all names
    let first = &names[0];
    let mut common_len = first.len();
    
    for name in &names[1..] {
        let mut len = 0;
        for (c1, c2) in first.chars().zip(name.chars()) {
            if c1.to_lowercase().next() == c2.to_lowercase().next() {
                len += c1.len_utf8();
            } else {
                break;
            }
        }
        common_len = common_len.min(len);
    }
    
    if common_len > 0 {
        let common_prefix = &first[..common_len];
        
        // Build the completed path
        if let Some(parent) = suggestions[0].parent() {
            let completed = parent.join(common_prefix);
            if let Some(completed_str) = completed.to_str() {
                return completed_str.to_string();
            }
        }
    }
    
    // If no common prefix, return input unchanged
    input.to_string()
}
