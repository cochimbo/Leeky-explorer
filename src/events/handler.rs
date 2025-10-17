// Event handler
use crate::app::{AppState, ConfirmAction, DialogState};
use crate::events::keybindings::{map_key_to_action, map_key_to_input_action, Action};
use crate::models::operation::Operation;
use anyhow::Result;
use crossterm::event::KeyEvent;

pub fn handle_key(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    // T627: Special handling for preview mode
    if app.has_preview() {
        return handle_preview_mode(app, key);
    }
    
    // Special handling for search mode
    if app.search_mode {
        return handle_search_mode(app, key);
    }
    
    // Special handling for input dialogs
    if let Some(DialogState::Input { .. }) = &app.dialog_state {
        return handle_input_dialog(app, key);
    }
    
    // T843: Special handling for password input dialogs
    if let Some(DialogState::PasswordInput { .. }) = &app.dialog_state {
        return handle_password_input_dialog(app, key);
    }
    
    // T844: Special handling for collision prompts
    if let Some(DialogState::CollisionPrompt { .. }) = &app.dialog_state {
        return handle_collision_dialog(app, key);
    }
    
    let action = map_key_to_action(key);

    // Handle other dialog-specific actions
    if app.has_dialog() {
        return handle_dialog_action(app, action);
    }

    // Handle normal navigation actions
    match action {
        Action::MoveUp => {
            app.active_panel_mut().move_cursor_up();
        }
        Action::MoveDown => {
            app.active_panel_mut().move_cursor_down();
        }
        Action::QuickJump(c) => {
            // T128c-d: Quick jump to file starting with character
            app.active_panel_mut().quick_jump(c);
        }
        Action::PageDown => {
            // T128f: Move 5 positions down
            app.active_panel_mut().page_down();
        }
        Action::PageUp => {
            // T128g: Move 5 positions up
            app.active_panel_mut().page_up();
        }
        Action::JumpToStart => {
            // T128h: Jump to first entry
            app.active_panel_mut().move_cursor_to_top();
        }
        Action::JumpToEnd => {
            // T128i: Jump to last entry
            app.active_panel_mut().move_cursor_to_bottom();
        }
        Action::SwitchPanel => {
            // T579: Marks are preserved per panel (by design - separate HashSets)
            app.switch_panel();
        }
        Action::EnterDirectory => {
            app.active_panel_mut().enter_dir()?;
            refresh_and_store(app)?;
            // T578: Clear marks when navigating to different directory
            app.selection_state.clear(app.active_panel);
        }
        Action::GoUp => {
            app.active_panel_mut().go_up()?;
            // T112b: go_up() now refreshes internally, so we only need to store
            let entries = app.active_panel().entries.clone();
            app.store_all_entries(entries);
            // T578: Clear marks when navigating to parent directory
            app.selection_state.clear(app.active_panel);
        }
        Action::Refresh => {
            // Store current panel
            let current_panel = app.active_panel;
            
            // Refresh left panel
            app.active_panel = crate::app::PanelSide::Left;
            refresh_and_store(app)?;
            
            // Refresh right panel
            app.active_panel = crate::app::PanelSide::Right;
            refresh_and_store(app)?;
            
            // Restore original panel
            app.active_panel = current_panel;
        }
        Action::Copy => {
            handle_copy_request(app)?;
        }
        Action::Move => {
            handle_move_request(app)?;
        }
        Action::Delete => {
            handle_delete_request(app)?;
        }
        Action::CreateFolder => {
            handle_create_folder_request(app)?;
        }
        Action::Search => {
            // T411: Activate search mode
            app.activate_search();
            // T580: Clear marks when entering search mode to avoid confusion
            app.selection_state.clear(app.active_panel);
        }
        Action::OpenPreview => {
            // T625-T626: Open preview for current file
            // This needs to be async, so we'll handle it in main.rs
            return Ok(action);
        }
        Action::ExtractArchive => {
            // T838-T839: Extract archive
            // This needs to be async, so we'll handle it in main.rs
            return Ok(action);
        }
        Action::ToggleSelection => {
            // T568: Toggle mark on current item and advance cursor
            app.toggle_selection();
        }
        Action::SelectAll => {
            // T569: Toggle all items in active panel
            app.select_all();
        }
        Action::ClearSelection => {
            // T567: Clear selection only if there are marks
            if app.has_selection() {
                app.clear_selection();
            } else {
                // If no marks, Esc should not do anything (handled elsewhere)
                return Ok(Action::Cancel);
            }
        }
        Action::Cancel => {
            // T567: Check if we should clear selection first
            if app.has_selection() {
                app.clear_selection();
            }
        }
        Action::Quit | Action::None => {}
        _ => {}
    }

    Ok(action)
}

fn handle_copy_request(app: &mut AppState) -> Result<()> {
    // T570: Check if there are marked items first
    if app.has_selection() {
        let marked_count = app.selection_state.count(app.active_panel);
        let dest_path = app.inactive_panel().current_path.clone();
        let message = format!(
            "Copy {} items to '{}'?",
            marked_count,
            dest_path.display()
        );
        app.show_confirm_dialog(message, ConfirmAction::Copy);
    } else {
        // Single item copy
        let source_panel = app.active_panel();
        let dest_panel = app.inactive_panel();
        
        if let Some(entry) = source_panel.selected_entry() {
            let dest_path = dest_panel.current_path.clone();
            let message = format!(
                "Copy '{}' to '{}'?",
                entry.name,
                dest_path.display()
            );
            app.show_confirm_dialog(message, ConfirmAction::Copy);
        }
    }
    
    Ok(())
}

fn handle_move_request(app: &mut AppState) -> Result<()> {
    // T571: Check if there are marked items first
    if app.has_selection() {
        let marked_count = app.selection_state.count(app.active_panel);
        let dest_path = app.inactive_panel().current_path.clone();
        let message = format!(
            "Move {} items to '{}'?",
            marked_count,
            dest_path.display()
        );
        app.show_confirm_dialog(message, ConfirmAction::Move);
    } else {
        // Single item move
        let source_panel = app.active_panel();
        let dest_panel = app.inactive_panel();
        
        if let Some(entry) = source_panel.selected_entry() {
            let dest_path = dest_panel.current_path.clone();
            let message = format!(
                "Move '{}' to '{}'?",
                entry.name,
                dest_path.display()
            );
            app.show_confirm_dialog(message, ConfirmAction::Move);
        }
    }
    
    Ok(())
}

fn handle_dialog_action(app: &mut AppState, action: Action) -> Result<Action> {
    // Handle ExtractOptions dialog separately
    if let Some(DialogState::ExtractOptions { selected: _, .. }) = &app.dialog_state {
        match action {
            Action::MoveUp | Action::MoveDown => {
                // Toggle between options
                if let Some(DialogState::ExtractOptions { selected, .. }) = &mut app.dialog_state {
                    *selected = if *selected == 0 { 1 } else { 0 };
                }
                return Ok(Action::None);
            }
            Action::ConfirmYes | Action::EnterDirectory => {
                // Return to main.rs for async extraction
                return Ok(Action::ConfirmYes);
            }
            Action::ConfirmNo | Action::Cancel => {
                app.close_dialog();
                return Ok(Action::None);
            }
            Action::Quit => {
                return Ok(action);
            }
            _ => {
                return Ok(Action::None);
            }
        }
    }
    
    match action {
        Action::ConfirmYes => {
            if let Some(DialogState::Confirm { confirm_action, .. }) = &app.dialog_state {
                match confirm_action {
                    ConfirmAction::Copy => {
                        start_copy_operation(app)?;
                        // Don't close dialog - start_copy_operation sets progress dialog
                    }
                    ConfirmAction::Move => {
                        start_move_operation(app)?;
                        // Don't close dialog - start_move_operation sets progress dialog
                    }
                    ConfirmAction::Delete => {
                        start_delete_operation(app)?;
                        // Don't close dialog - start_delete_operation sets progress dialog
                    }
                    ConfirmAction::ExtractArchive { .. } => {
                        // Return action to main.rs for async extraction
                        return Ok(Action::ConfirmYes);
                    }
                }
            }
        }
        Action::ConfirmNo | Action::Cancel => {
            app.close_dialog();
        }
        Action::Quit => {
            // Allow quit even in dialog
            return Ok(action);
        }
        _ => {
            // Ignore other actions while dialog is open
        }
    }
    
    Ok(Action::None)
}

fn start_copy_operation(app: &mut AppState) -> Result<()> {
    let dest_panel_path = app.inactive_panel().current_path.clone();
    
    // T844: Check for collisions first
    let collision_path = if app.has_selection() {
        // Check first marked item for collision
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        marked_paths.iter().find_map(|path| {
            let file_name = path.file_name()?.to_string_lossy().to_string();
            let dest = dest_panel_path.join(&file_name);
            if dest.exists() {
                Some(dest.to_string_lossy().to_string())
            } else {
                None
            }
        })
    } else {
        // Check single item
        app.active_panel().selected_entry().and_then(|entry| {
            let dest = dest_panel_path.join(&entry.name);
            if dest.exists() {
                Some(dest.to_string_lossy().to_string())
            } else {
                None
            }
        })
    };
    
    // If collision detected, show collision dialog
    if let Some(collision_path) = collision_path {
        app.dialog_state = Some(DialogState::CollisionPrompt {
            file_path: collision_path,
            selected: 0,
            operation: crate::app::CollisionOperation::Copy,
        });
        return Ok(());
    }
    
    // No collision, proceed with operation
    // T574: Check if batch operation or single
    if app.has_selection() {
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        let count = marked_paths.len();
        
        // Calculate total size and create batch operation
        let mut total_bytes = 0u64;
        let mut operations = Vec::new();
        
        for path in &marked_paths {
            if let Ok(metadata) = std::fs::metadata(path) {
                total_bytes += metadata.len();
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let destination = dest_panel_path.join(&file_name);
                operations.push((path.clone(), destination, file_name));
            }
        }
        
        let operation = Operation::copy_batch(operations, total_bytes, count);
        app.current_operation = Some(operation);
        
        app.dialog_state = Some(DialogState::Progress {
            message: format!("Copying {} items...", count),
        });
    } else {
        // Single file operation
        let source_panel = app.active_panel();
        
        if let Some(entry) = source_panel.selected_entry() {
            let source = entry.path.clone();
            let destination = dest_panel_path.join(&entry.name);
            let entry_name = entry.name.clone();
            let entry_type = entry.entry_type.clone();
            let total_bytes = entry.size;
            
            let total_files = if entry_type == crate::models::file_entry::EntryType::Dir {
                1
            } else {
                1
            };
            
            let operation = Operation::copy(source, destination, total_bytes, total_files);
            app.current_operation = Some(operation);
            
            app.dialog_state = Some(DialogState::Progress {
                message: format!("Copying '{}'...", entry_name),
            });
        }
    }
    
    Ok(())
}

// T844: Start copy without collision check (for when user confirmed overwrite)
fn start_copy_operation_skip_check(app: &mut AppState) -> Result<()> {
    let dest_panel_path = app.inactive_panel().current_path.clone();
    
    // T574: Check if batch operation or single
    if app.has_selection() {
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        let count = marked_paths.len();
        
        // Calculate total size and create batch operation
        let mut total_bytes = 0u64;
        let mut operations = Vec::new();
        
        for path in &marked_paths {
            if let Ok(metadata) = std::fs::metadata(path) {
                total_bytes += metadata.len();
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let destination = dest_panel_path.join(&file_name);
                operations.push((path.clone(), destination, file_name));
            }
        }
        
        let operation = Operation::copy_batch(operations, total_bytes, count);
        app.current_operation = Some(operation);
        
        app.dialog_state = Some(DialogState::Progress {
            message: format!("Copying {} items...", count),
        });
    } else {
        // Single file operation
        let source_panel = app.active_panel();
        
        if let Some(entry) = source_panel.selected_entry() {
            let source = entry.path.clone();
            let destination = dest_panel_path.join(&entry.name);
            let entry_name = entry.name.clone();
            let entry_type = entry.entry_type.clone();
            let total_bytes = entry.size;
            
            let total_files = if entry_type == crate::models::file_entry::EntryType::Dir {
                1
            } else {
                1
            };
            
            let operation = Operation::copy(source, destination, total_bytes, total_files);
            app.current_operation = Some(operation);
            
            app.dialog_state = Some(DialogState::Progress {
                message: format!("Copying '{}'...", entry_name),
            });
        }
    }
    
    Ok(())
}

fn start_move_operation(app: &mut AppState) -> Result<()> {
    let dest_panel_path = app.inactive_panel().current_path.clone();
    
    // T844: Check for collisions first
    let collision_path = if app.has_selection() {
        // Check first marked item for collision
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        marked_paths.iter().find_map(|path| {
            let file_name = path.file_name()?.to_string_lossy().to_string();
            let dest = dest_panel_path.join(&file_name);
            if dest.exists() {
                Some(dest.to_string_lossy().to_string())
            } else {
                None
            }
        })
    } else {
        // Check single item
        app.active_panel().selected_entry().and_then(|entry| {
            let dest = dest_panel_path.join(&entry.name);
            if dest.exists() {
                Some(dest.to_string_lossy().to_string())
            } else {
                None
            }
        })
    };
    
    // If collision detected, show collision dialog
    if let Some(collision_path) = collision_path {
        app.dialog_state = Some(DialogState::CollisionPrompt {
            file_path: collision_path,
            selected: 0,
            operation: crate::app::CollisionOperation::Move,
        });
        return Ok(());
    }
    
    // No collision, proceed with operation
    // T574: Check if batch operation or single
    if app.has_selection() {
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        let count = marked_paths.len();
        
        let mut total_bytes = 0u64;
        let mut operations = Vec::new();
        
        for path in &marked_paths {
            if let Ok(metadata) = std::fs::metadata(path) {
                total_bytes += metadata.len();
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let destination = dest_panel_path.join(&file_name);
                operations.push((path.clone(), destination, file_name));
            }
        }
        
        let operation = Operation::move_batch(operations, total_bytes, count);
        app.current_operation = Some(operation);
        
        app.dialog_state = Some(DialogState::Progress {
            message: format!("Moving {} items...", count),
        });
    } else {
        // Single file operation
        let source_panel = app.active_panel();
        
        if let Some(entry) = source_panel.selected_entry() {
            let source = entry.path.clone();
            let destination = dest_panel_path.join(&entry.name);
            let entry_name = entry.name.clone();
            let entry_type = entry.entry_type.clone();
            let total_bytes = entry.size;
            
            let total_files = if entry_type == crate::models::file_entry::EntryType::Dir {
                1
            } else {
                1
            };
            
            let operation = Operation::move_op(source, destination, total_bytes, total_files);
            app.current_operation = Some(operation);
            
            app.dialog_state = Some(DialogState::Progress {
                message: format!("Moving '{}'...", entry_name),
            });
        }
    }
    
    Ok(())
}

// T844: Start move without collision check (for when user confirmed overwrite)
fn start_move_operation_skip_check(app: &mut AppState) -> Result<()> {
    let dest_panel_path = app.inactive_panel().current_path.clone();
    
    // T574: Check if batch operation or single
    if app.has_selection() {
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        let count = marked_paths.len();
        
        let mut total_bytes = 0u64;
        let mut operations = Vec::new();
        
        for path in &marked_paths {
            if let Ok(metadata) = std::fs::metadata(path) {
                total_bytes += metadata.len();
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let destination = dest_panel_path.join(&file_name);
                operations.push((path.clone(), destination, file_name));
            }
        }
        
        let operation = Operation::move_batch(operations, total_bytes, count);
        app.current_operation = Some(operation);
        
        app.dialog_state = Some(DialogState::Progress {
            message: format!("Moving {} items...", count),
        });
    } else {
        // Single file operation
        let source_panel = app.active_panel();
        
        if let Some(entry) = source_panel.selected_entry() {
            let source = entry.path.clone();
            let destination = dest_panel_path.join(&entry.name);
            let entry_name = entry.name.clone();
            let entry_type = entry.entry_type.clone();
            let total_bytes = entry.size;
            
            let total_files = if entry_type == crate::models::file_entry::EntryType::Dir {
                1
            } else {
                1
            };
            
            let operation = Operation::move_op(source, destination, total_bytes, total_files);
            app.current_operation = Some(operation);
            
            app.dialog_state = Some(DialogState::Progress {
                message: format!("Moving '{}'...", entry_name),
            });
        }
    }
    
    Ok(())
}

fn handle_delete_request(app: &mut AppState) -> Result<()> {
    // T572: Check if there are marked items first
    if app.has_selection() {
        let marked_count = app.selection_state.count(app.active_panel);
        let message = format!(
            "Delete {} selected items?",
            marked_count
        );
        app.show_confirm_dialog(message, ConfirmAction::Delete);
    } else {
        // Single item delete
        let panel = app.active_panel();
        
        if let Some(entry) = panel.selected_entry() {
            let message = format!(
                "Delete '{}'?",
                entry.name
            );
            app.show_confirm_dialog(message, ConfirmAction::Delete);
        }
    }
    
    Ok(())
}

fn handle_create_folder_request(app: &mut AppState) -> Result<()> {
    app.show_input_dialog("Enter new folder name:".to_string());
    Ok(())
}

fn handle_input_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    let action = map_key_to_input_action(key);
    
    match action {
        Action::ConfirmInput => {
            if let Some(value) = app.get_input_value() {
                if !value.is_empty() {
                    create_folder(app, &value)?;
                }
            }
            app.close_dialog();
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

// T843: Handle password input dialog
fn handle_password_input_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    use crossterm::event::{KeyCode, KeyModifiers};
    
    match (key.code, key.modifiers) {
        // Enter: confirm password
        (KeyCode::Enter, _) => {
            return Ok(Action::ConfirmYes);
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

// T844: Handle collision dialog
fn handle_collision_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    use crossterm::event::{KeyCode, KeyModifiers};
    
    match (key.code, key.modifiers) {
        // Enter: confirm selected option
        (KeyCode::Enter, _) => {
            if let Some(DialogState::CollisionPrompt { selected, operation, .. }) = &app.dialog_state {
                let selected_option = *selected;
                let operation_type = operation.clone();
                
                match selected_option {
                    0 => {
                        // Overwrite this file
                        app.close_dialog();
                        // Resume the operation (the actual operation was saved in app state)
                        match operation_type {
                            crate::app::CollisionOperation::Copy => start_copy_operation_skip_check(app)?,
                            crate::app::CollisionOperation::Move => start_move_operation_skip_check(app)?,
                            crate::app::CollisionOperation::Extract => {}, // TODO: implement
                        }
                        return Ok(Action::None);
                    }
                    1 => {
                        // Overwrite All (TODO: set global flag)
                        app.close_dialog();
                        match operation_type {
                            crate::app::CollisionOperation::Copy => start_copy_operation_skip_check(app)?,
                            crate::app::CollisionOperation::Move => start_move_operation_skip_check(app)?,
                            crate::app::CollisionOperation::Extract => {},
                        }
                        return Ok(Action::None);
                    }
                    2 => {
                        // Rename (TODO: implement rename logic)
                        app.close_dialog();
                        return Ok(Action::None);
                    }
                    3 => {
                        // Skip
                        app.close_dialog();
                        return Ok(Action::None);
                    }
                    4 => {
                        // Cancel
                        app.close_dialog();
                        return Ok(Action::None);
                    }
                    _ => {}
                }
            }
        }
        // Up arrow: move selection up
        (KeyCode::Up, _) => {
            if let Some(DialogState::CollisionPrompt { selected, .. }) = &mut app.dialog_state {
                if *selected > 0 {
                    *selected -= 1;
                }
            }
        }
        // Down arrow: move selection down
        (KeyCode::Down, _) => {
            if let Some(DialogState::CollisionPrompt { selected, .. }) = &mut app.dialog_state {
                if *selected < 4 {
                    *selected += 1;
                }
            }
        }
        // Letter shortcuts
        (KeyCode::Char('s'), KeyModifiers::NONE) | (KeyCode::Char('S'), _) => {
            if let Some(DialogState::CollisionPrompt { selected, .. }) = &mut app.dialog_state {
                *selected = 0;
            }
            return Ok(Action::ConfirmYes);
        }
        (KeyCode::Char('t'), KeyModifiers::NONE) | (KeyCode::Char('T'), _) => {
            if let Some(DialogState::CollisionPrompt { selected, .. }) = &mut app.dialog_state {
                *selected = 1;
            }
            return Ok(Action::ConfirmYes);
        }
        (KeyCode::Char('r'), KeyModifiers::NONE) | (KeyCode::Char('R'), _) => {
            if let Some(DialogState::CollisionPrompt { selected, .. }) = &mut app.dialog_state {
                *selected = 2;
            }
            return Ok(Action::ConfirmYes);
        }
        (KeyCode::Char('o'), KeyModifiers::NONE) | (KeyCode::Char('O'), _) => {
            if let Some(DialogState::CollisionPrompt { selected, .. }) = &mut app.dialog_state {
                *selected = 3;
            }
            return Ok(Action::ConfirmNo);
        }
        (KeyCode::Char('c'), KeyModifiers::NONE) | (KeyCode::Char('C'), _) => {
            app.close_dialog();
        }
        // Escape: cancel (same as 'C')
        (KeyCode::Esc, _) => {
            app.close_dialog();
        }
        _ => {}
    }
    
    Ok(Action::None)
}

fn create_folder(app: &mut AppState, folder_name: &str) -> Result<()> {
    let panel = app.active_panel();
    let new_path = panel.current_path.join(folder_name);
    
    // Use blocking task for simplicity (create_dir is fast)
    std::fs::create_dir(&new_path)?;
    
    // Refresh panel and store entries
    refresh_and_store(app)?;
    
    Ok(())
}

fn start_delete_operation(app: &mut AppState) -> Result<()> {
    // T574: Check if we have marked items for batch delete
    if app.has_selection() {
        let panel = app.active_panel();
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        
        if !marked_paths.is_empty() {
            // Calculate total size and prepare batch items
            let mut total_bytes = 0u64;
            let mut batch_items = Vec::new();
            
            for marked_path in marked_paths {
                if let Some(entry) = panel.entries.iter().find(|e| e.path == *marked_path) {
                    total_bytes += entry.size;
                    batch_items.push((
                        entry.path.clone(),
                        entry.path.clone(), // For delete, destination is same as source
                        entry.name.clone()
                    ));
                }
            }
            
            let total_files = batch_items.len();
            let operation = Operation::delete_batch(batch_items, total_bytes, total_files);
            app.current_operation = Some(operation);
            
            app.dialog_state = Some(DialogState::Progress {
                message: format!("Deleting {} items...", total_files),
            });
            
            return Ok(());
        }
    }
    
    // Single file delete
    let panel = app.active_panel();
    
    if let Some(entry) = panel.selected_entry() {
        let source = entry.path.clone();
        let entry_name = entry.name.clone();
        let entry_type = entry.entry_type.clone();
        let total_bytes = entry.size;
        
        let total_files = if entry_type == crate::models::file_entry::EntryType::Dir {
            1 // Estimate
        } else {
            1
        };
        
        let operation = Operation::delete(source, total_bytes, total_files);
        app.current_operation = Some(operation);
        
        // Show progress dialog
        app.dialog_state = Some(DialogState::Progress {
            message: format!("Deleting '{}'...", entry_name),
        });
    }
    
    Ok(())
}

// Helper to refresh panel entries and store unfiltered list
fn refresh_and_store(app: &mut AppState) -> Result<()> {
    app.active_panel_mut().refresh_entries()?;
    let entries = app.active_panel().entries.clone();
    app.store_all_entries(entries);
    Ok(())
}

// T411-T415: Handle search mode key events
fn handle_search_mode(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    if key.kind != crossterm::event::KeyEventKind::Press {
        return Ok(Action::None);
    }

    match key.code {
        crossterm::event::KeyCode::Esc => {
            // T414: Deactivate search and clear filter
            app.deactivate_search();
            Ok(Action::Cancel)
        }
        crossterm::event::KeyCode::Enter => {
            // T415: Finalize filter and return to navigation
            app.search_mode = false;
            Ok(Action::None)
        }
        crossterm::event::KeyCode::Backspace => {
            // T412: Remove last character
            app.search_backspace();
            Ok(Action::None)
        }
        crossterm::event::KeyCode::Char(c) => {
            // T412-T413: Append character and apply filter in real-time
            app.search_append(c);
            Ok(Action::None)
        }
        _ => Ok(Action::None),
    }
}

// T627-T630: Handle preview mode key events
fn handle_preview_mode(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    use crate::events::keybindings::map_key_to_preview_action;
    
    if key.kind != crossterm::event::KeyEventKind::Press {
        return Ok(Action::None);
    }

    let action = map_key_to_preview_action(key);

    match action {
        Action::ClosePreview => {
            // T628: Close preview with Esc or Q
            app.close_preview();
            Ok(Action::None)
        }
        Action::ScrollPreviewUp => {
            // Scroll up by 1 line
            app.scroll_preview(-1);
            Ok(Action::None)
        }
        Action::ScrollPreviewDown => {
            // Scroll down by 1 line
            app.scroll_preview(1);
            Ok(Action::None)
        }
        Action::PagePreviewUp => {
            // T630: Scroll up by page (20 lines)
            app.scroll_preview(-20);
            Ok(Action::None)
        }
        Action::PagePreviewDown => {
            // T630: Scroll down by page (20 lines)
            app.scroll_preview(20);
            Ok(Action::None)
        }
        Action::JumpPreviewStart => {
            // T629: Jump to start of file
            use crate::app::JumpTarget;
            app.jump_preview(JumpTarget::Start);
            Ok(Action::None)
        }
        Action::JumpPreviewEnd => {
            // T629: Jump to end of file
            use crate::app::JumpTarget;
            app.jump_preview(JumpTarget::End);
            Ok(Action::None)
        }
        _ => Ok(Action::None),
    }
}

