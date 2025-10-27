//! Dialog event handlers
//! 
//! This module contains handlers for all dialog types in the application.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::path::PathBuf;

use crate::app::{AppState, DialogState};
use crate::events::keybindings::{Action, map_key_to_input_action};

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

// TODO: Add other dialog handlers here:
// - handle_input
// - handle_rename
// - handle_password_input  
// - handle_search
// - handle_compress_options
// - handle_drive_selector
// - handle_theme_selector
// - handle_bookmark_manager
// - handle_history_viewer
// - handle_goto
// - handle_connection

// These will be moved from handler.rs as we continue refactoring

