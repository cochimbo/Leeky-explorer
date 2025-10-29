//! File operation dialog handlers
//!
//! Handles dialogs related to file operations:
//! - Collision resolution (overwrite/rename/skip)
//! - Rename files/folders
//! - Password input for archives
//! - Compression options

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::{AppState, DialogState};
use crate::events::keybindings::Action;
use crate::archive::formats::ArchiveFormat;
use crate::archive::compressor::CompressionLevel;

// Re-export collision handlers from parent module
pub use super::super::collision::{
    continue_batch_operation,
    process_batch_without_collision_check,
    process_single_file_operation,
};

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
                source_path,
                file_path: _,
                selected, 
                operation, 
                remaining_files,
                dest_path,
                source_vfs,
                dest_vfs,
            }) = &app.dialog_state {
                let selected_option = *selected;
                let operation_type = operation.clone();
                let collision_source = source_path.clone();
                let remaining = remaining_files.clone();
                let dest = dest_path.clone();
                let src_vfs = source_vfs.clone();
                let dst_vfs = dest_vfs.clone();
                
                app.close_dialog();
                
                match selected_option {
                    0 => {
                        // Overwrite this file - process just this one, then continue with remaining
                        log::info!("[COLLISION] Overwrite selected, processing file and saving {} remaining files to pending_batch", remaining.len());
                        process_single_file_operation(&collision_source, dest.as_path(), &src_vfs, &dst_vfs, operation_type.clone(), true, app)?;
                        
                        // Save remaining files to process after this operation completes
                        if !remaining.is_empty() {
                            app.pending_batch = Some(crate::app::PendingBatch {
                                remaining_files: remaining.clone(),
                                dest_path: dest.clone(),
                                source_vfs: src_vfs.clone(),
                                dest_vfs: dst_vfs.clone(),
                                operation: operation_type.clone(),
                            });
                            log::info!("[COLLISION] Saved pending_batch with {} files", remaining.len());
                        }
                        return Ok(Action::None);
                    }
                    1 => {
                        // Overwrite All - process this one and all remaining without checking
                        process_single_file_operation(&collision_source, dest.as_path(), &src_vfs, &dst_vfs, operation_type.clone(), true, app)?;
                        
                        // Process all remaining files with overwrite enabled
                        if !remaining.is_empty() {
                            process_batch_without_collision_check(remaining, dest, src_vfs, dst_vfs, operation_type, app)?;
                        }
                        return Ok(Action::None);
                    }
                    2 => {
                        // Rename - process this file with a new name, then continue with remaining
                        log::info!("[COLLISION] Rename selected, processing file and saving {} remaining files to pending_batch", remaining.len());
                        process_single_file_operation(&collision_source, dest.as_path(), &src_vfs, &dst_vfs, operation_type.clone(), false, app)?;
                        
                        // Save remaining files to process after this operation completes
                        if !remaining.is_empty() {
                            app.pending_batch = Some(crate::app::PendingBatch {
                                remaining_files: remaining.clone(),
                                dest_path: dest.clone(),
                                source_vfs: src_vfs.clone(),
                                dest_vfs: dst_vfs.clone(),
                                operation: operation_type.clone(),
                            });
                            log::info!("[COLLISION] Saved pending_batch with {} files", remaining.len());
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
                    app.show_error("Name cannot be empty".to_string());
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
                    app.show_error("Name cannot be empty".to_string());
                    return Ok(Action::None);
                }
                
                // Validate passwords if enabled
                if *use_password {
                    if password.is_empty() {
                        app.show_error("Password cannot be empty".to_string());
                        return Ok(Action::None);
                    }
                    if password != confirm_password {
                        app.show_error("Passwords do not match".to_string());
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
