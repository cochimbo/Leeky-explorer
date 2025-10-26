// Event handler
use anyhow::Result;
use crossterm::event::{KeyEvent, KeyCode, KeyEventKind, KeyEventState};

use crate::app::{AppState, ConfirmAction, DialogState};
use crate::events::keybindings::{map_key_to_action, map_key_to_input_action, Action};
use crate::models::operation::Operation;

pub fn handle_key(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    // Handle welcome screen - only Enter key dismisses it
    if app.show_welcome {
        if key.code == KeyCode::Enter {
            app.show_welcome = false;
        }
        // Ignore all other keys during welcome screen
        return Ok(Action::None);
    }

    // Special handling for dialogs (highest priority after welcome)
    // T561-T585: Handle dialogs with special input handling
    if let Some(DialogState::Input { .. }) = &app.dialog_state {
        return handle_input_dialog(app, key);
    }

    if let Some(DialogState::PasswordInput { .. }) = &app.dialog_state {
        return handle_password_input_dialog(app, key);
    }

    if let Some(DialogState::DriveSelector { .. }) = &app.dialog_state {
        return handle_drive_selector_dialog(app, key);
    }

    if let Some(DialogState::ThemeSelector { .. }) = &app.dialog_state {
        return handle_theme_selector_dialog(app, key);
    }

    if let Some(DialogState::BookmarkManager { .. }) = &app.dialog_state {
        return handle_bookmark_manager_dialog(app, key);
    }

    if let Some(DialogState::HistoryViewer { .. }) = &app.dialog_state {
        return handle_history_viewer_dialog(app, key);
    }

    if let Some(DialogState::GoToPath { .. }) = &app.dialog_state {
        return handle_goto_dialog(app, key);
    }

    if let Some(DialogState::CompressOptions { .. }) = &app.dialog_state {
        return handle_compress_options_dialog(app, key);
    }
    
    let action = map_key_to_action(key);

    // Handle other dialog-specific actions (Confirm dialogs, Progress, Error)
    if app.has_dialog() {
        return handle_dialog_action(app, action);
    }

    // T627: Special handling for preview mode (after dialogs)
    if app.has_preview() {
        return handle_preview_mode(app, key);
    }
    
    // TASK-030: Special handling for editor mode (after dialogs and preview)
    if app.has_editor() {
        return handle_editor_mode(app, key);
    }
    
    // Special handling for search mode
    if app.search_mode {
        return handle_search_mode(app, key);
    }
    
    // Special handling for rename dialog
    if let Some(DialogState::Rename { .. }) = &app.dialog_state {
        return handle_rename_dialog(app, key);
    }
    
    // T844: Special handling for collision prompts
    if let Some(DialogState::CollisionPrompt { .. }) = &app.dialog_state {
        return handle_collision_dialog(app, key);
    }
    
    let action = map_key_to_action(key);

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
            // Exit search mode when entering a directory (panel clears its own filter)
            if app.search_mode {
                app.search_mode = false;
                app.search_pattern.clear();
            }
        }
        Action::GoUp => {
            app.active_panel_mut().go_up()?;
            // T112b: go_up() now refreshes internally, so we only need to store
            let entries = app.active_panel().entries.clone();
            app.store_all_entries(entries);
            // T578: Clear marks when navigating to parent directory
            app.selection_state.clear(app.active_panel);
            // Exit search mode when going up to parent (panel clears its own filter)
            if app.search_mode {
                app.search_mode = false;
                app.search_pattern.clear();
            }
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
        Action::Rename => {
            handle_rename_request(app, false)?; // false = name only
        }
        Action::RenameWithExtension => {
            handle_rename_request(app, true)?; // true = name with extension
        }
        Action::Search => {
            // T411: Activate search mode or clear if already active
            if app.search_mode {
                // If already in search mode, pressing F3 again clears and exits
                app.deactivate_search();
            } else {
                // Activate search mode
                app.activate_search();
                // T580: Clear marks when entering search mode to avoid confusion
                app.selection_state.clear(app.active_panel);
            }
        }
        Action::ClearSearch => {
            // Shift+F3: Clear search pattern and filter
            app.deactivate_search();
        }
        Action::OpenPreview => {
            // T625-T626: Open preview for current file
            // This needs to be async, so we'll handle it in main.rs
            return Ok(action);
        }
        Action::OpenEditor => {
            // TASK-028: Open text editor for current file
            let panel = app.active_panel();
            
            if let Some(selected_entry) = panel.selected_entry() {
                if selected_entry.is_file() {
                    match app.open_editor(selected_entry.path.clone()) {
                        Ok(_) => {
                            // Editor opened successfully
                        }
                        Err(e) => {
                            app.error_message = Some(format!("Cannot open editor: {}", e));
                        }
                    }
                }
            }
        }
        Action::OpenDriveSelector => {
            // US4: Open drive selector dialog
            let drives = crate::fs::disk_info::get_available_drives();
            if !drives.is_empty() {
                app.dialog_state = Some(DialogState::DriveSelector {
                    drives,
                    selected: 0,
                });
            }
        }
        Action::OpenThemeSelector => {
            // US5: Open theme selector dialog
            let themes = crate::ui::theme::Theme::all_themes();
            
            // Find the index of the currently active theme
            let current_theme_name = &app.theme.name;
            let selected = themes
                .iter()
                .position(|t| &t.name == current_theme_name)
                .unwrap_or(0);
            
            app.dialog_state = Some(DialogState::ThemeSelector {
                themes,
                selected,
            });
        }
        Action::ToggleBookmarkManager => {
            // TASK-008: Open bookmark manager dialog
            let state = crate::ui::bookmark_manager::BookmarkManagerState::new();
            app.dialog_state = Some(DialogState::BookmarkManager { state });
        }
        Action::ToggleHistoryViewer => {
            // TASK-018: Open navigation history dialog
            let state = crate::ui::history_dialog::HistoryDialogState::new();
            app.dialog_state = Some(DialogState::HistoryViewer { state });
        }
        Action::ToggleGoToPath => {
            // TASK-021: Open Go To Path dialog
            // Get initial suggestions from current directory
            let current_path = app.active_panel().current_path.clone();
            let initial_suggestions = get_directory_children(&current_path);
            
            // Initialize input with current path
            let initial_input = current_path.to_string_lossy().to_string();
            
            app.dialog_state = Some(DialogState::GoToPath {
                input: initial_input,
                error_message: None,
                suggestions: initial_suggestions,
                selected_suggestion: 0,
            });
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
                    ConfirmAction::CloseEditor => {
                        app.close_editor();
                        app.close_dialog();
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
    
    // T951: Validate source files exist before starting operation
    if app.has_selection() {
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        for path in &marked_paths {
            if !path.exists() {
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                app.show_error(format!("Archivo no encontrado: {}", file_name));
                return Ok(());
            }
        }
    } else {
        // Check single file exists
        if let Some(entry) = app.active_panel().selected_entry() {
            let source_path = app.active_panel().current_path.join(&entry.name);
            if !source_path.exists() {
                app.show_error(format!("Archivo no encontrado: {}", entry.name));
                return Ok(());
            }
        }
    }
    
    // BUG-003/BUG-005 FIX: Check if copying to same directory
    // If so, skip collision check and generate suffix automatically
    let copying_to_same_dir = if app.has_selection() {
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        marked_paths.iter().any(|path| {
            if let Some(src_parent) = path.parent() {
                src_parent == dest_panel_path
            } else {
                false
            }
        })
    } else {
        app.active_panel().selected_entry()
            .map(|entry| {
                entry.path.parent()
                    .map(|p| p == dest_panel_path)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    };
    
    // T844: Check for collisions first (but skip if copying to same directory)
    if !copying_to_same_dir {
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
                if let Some(file_name) = path.file_name() {
                    let file_name = file_name.to_string_lossy().to_string();
                    let destination = dest_panel_path.join(&file_name);
                    
                    // BUG-003/BUG-005 FIX: Generate new name if copying to same directory
                    let final_destination = if copying_to_same_dir {
                        crate::fs::operations::generate_collision_free_name(&destination)
                    } else {
                        destination
                    };
                    
                    operations.push((path.clone(), final_destination, file_name));
                }
            }
        }
        
        // T953: Check available disk space before copying
        if let Ok(available_space) = fs2::available_space(&dest_panel_path)
            && available_space < total_bytes {
                let size_mb = total_bytes / (1024 * 1024);
                let avail_mb = available_space / (1024 * 1024);
                app.show_error(format!(
                    "Espacio insuficiente. Se necesitan {} MB, disponibles {} MB",
                    size_mb, avail_mb
                ));
                return Ok(());
            }
        
        // T956: Warn about large operations
        let size_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        if size_gb > 1.0 || count > 1000 {
            let warning_msg = if size_gb > 1.0 && count > 1000 {
                format!("Operación grande: {:.1} GB y {} archivos. ¿Continuar?", size_gb, count)
            } else if size_gb > 1.0 {
                format!("Operación grande: {:.1} GB. ¿Continuar?", size_gb)
            } else {
                format!("Operación grande: {} archivos. ¿Continuar?", count)
            };
            
            app.show_error(warning_msg);
            // TODO: En el futuro, mostrar diálogo de confirmación en lugar de error
            // Por ahora, mostramos advertencia pero continuamos
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
            let mut destination = dest_panel_path.join(&entry.name);
            let entry_name = entry.name.clone();
            let total_bytes = entry.size;
            
            // BUG-003 FIX: Check if copying to same directory
            if let (Some(src_parent), Some(dst_parent)) = (source.parent(), destination.parent())
                && src_parent == dst_parent {
                    // Copying to same directory - generate new name with suffix
                    destination = crate::fs::operations::generate_collision_free_name(&destination);
                }
            
            // T953: Check available disk space before copying
            if let Ok(available_space) = fs2::available_space(&dest_panel_path)
                && available_space < total_bytes {
                    let size_mb = total_bytes / (1024 * 1024);
                    let avail_mb = available_space / (1024 * 1024);
                    app.show_error(format!(
                        "Espacio insuficiente. Se necesitan {} MB, disponibles {} MB",
                        size_mb, avail_mb
                    ));
                    return Ok(());
                }
            
            let total_files = 1; // Single file or directory
            
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
                if let Some(file_name) = path.file_name() {
                    let file_name = file_name.to_string_lossy().to_string();
                    let mut destination = dest_panel_path.join(&file_name);
                    
                    // BUG-003 FIX: Check if copying to same directory
                    if let (Some(src_parent), Some(dst_parent)) = (path.parent(), destination.parent())
                        && src_parent == dst_parent {
                            // Copying to same directory - generate new name with suffix
                            destination = crate::fs::operations::generate_collision_free_name(&destination);
                        }
                    
                    operations.push((path.clone(), destination, file_name));
                }
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
            let mut destination = dest_panel_path.join(&entry.name);
            let entry_name = entry.name.clone();
            let total_bytes = entry.size;
            
            // BUG-003 FIX: Check if copying to same directory
            if let (Some(src_parent), Some(dst_parent)) = (source.parent(), destination.parent())
                && src_parent == dst_parent {
                    // Copying to same directory - generate new name with suffix
                    destination = crate::fs::operations::generate_collision_free_name(&destination);
                }
            
            let total_files = 1; // Single file or directory
            
            let operation = Operation::copy(source, destination, total_bytes, total_files);
            app.current_operation = Some(operation);
            
            app.dialog_state = Some(DialogState::Progress {
                message: format!("Copying '{}'...", entry_name),
            });
        }
    }
    
    Ok(())
}

// BUG-003/BUG-005 FIX: Start copy with automatic rename (generate suffix)
fn start_copy_operation_with_rename(app: &mut AppState) -> Result<()> {
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
                if let Some(file_name) = path.file_name() {
                    let file_name = file_name.to_string_lossy().to_string();
                    let destination = dest_panel_path.join(&file_name);
                    
                    // Always generate collision-free name
                    let final_destination = crate::fs::operations::generate_collision_free_name(&destination);
                    
                    operations.push((path.clone(), final_destination, file_name));
                }
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
            let total_bytes = entry.size;
            
            // Always generate collision-free name
            let final_destination = crate::fs::operations::generate_collision_free_name(&destination);
            
            let total_files = 1; // Single file or directory
            
            let operation = Operation::copy(source, final_destination, total_bytes, total_files);
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
    
    // T951: Validate source files exist before starting operation
    if app.has_selection() {
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        for path in &marked_paths {
            if !path.exists() {
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                app.show_error(format!("Archivo no encontrado: {}", file_name));
                return Ok(());
            }
        }
    } else {
        // Check single file exists
        if let Some(entry) = app.active_panel().selected_entry() {
            let source_path = app.active_panel().current_path.join(&entry.name);
            if !source_path.exists() {
                app.show_error(format!("Archivo no encontrado: {}", entry.name));
                return Ok(());
            }
        }
    }
    
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
                if let Some(file_name) = path.file_name() {
                    let file_name = file_name.to_string_lossy().to_string();
                    let destination = dest_panel_path.join(&file_name);
                    operations.push((path.clone(), destination, file_name));
                }
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
            let total_bytes = entry.size;
            
            let total_files = 1; // Single file or directory
            
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
                if let Some(file_name) = path.file_name() {
                    let file_name = file_name.to_string_lossy().to_string();
                    let destination = dest_panel_path.join(&file_name);
                    operations.push((path.clone(), destination, file_name));
                }
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
            let total_bytes = entry.size;
            
            let total_files = 1; // Single file or directory
            
            let operation = Operation::move_op(source, destination, total_bytes, total_files);
            app.current_operation = Some(operation);
            
            app.dialog_state = Some(DialogState::Progress {
                message: format!("Moving '{}'...", entry_name),
            });
        }
    }
    
    Ok(())
}

// BUG-003/BUG-005 FIX: Start move with automatic rename (generate suffix)
fn start_move_operation_with_rename(app: &mut AppState) -> Result<()> {
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
                if let Some(file_name) = path.file_name() {
                    let file_name = file_name.to_string_lossy().to_string();
                    let destination = dest_panel_path.join(&file_name);
                    
                    // Always generate collision-free name
                    let final_destination = crate::fs::operations::generate_collision_free_name(&destination);
                    
                    operations.push((path.clone(), final_destination, file_name));
                }
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
            let total_bytes = entry.size;
            
            // Always generate collision-free name
            let final_destination = crate::fs::operations::generate_collision_free_name(&destination);
            
            let total_files = 1; // Single file or directory
            
            let operation = Operation::move_op(source, final_destination, total_bytes, total_files);
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

fn handle_rename_request(app: &mut AppState, include_extension: bool) -> Result<()> {
    // Get the current selected entry
    let panel = app.active_panel();
    if let Some(entry) = panel.selected_entry() {
        let old_path = entry.path.clone();
        let current_name = entry.name.clone();
        
        // For F2 (name only), extract just the name without extension
        let display_name = if !include_extension && old_path.is_file() {
            // Get stem (name without extension)
            old_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&current_name)
                .to_string()
        } else {
            // For directories or Shift+F2, use full name
            current_name.clone()
        };
        
        // Show rename dialog with appropriate name pre-loaded
        let prompt = if include_extension {
            format!("Rename '{}' to (with extension):", current_name)
        } else {
            format!("Rename '{}' to:", current_name)
        };
        
        app.dialog_state = Some(DialogState::Rename {
            prompt,
            value: display_name,
            old_path,
            include_extension,
        });
    }
    Ok(())
}

fn handle_input_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
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

fn handle_rename_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    use crossterm::event::{KeyCode, KeyEventKind};
    
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
                
                // Check if target already exists
                if new_path.exists() {
                    app.show_error(format!("Ya existe un archivo o directorio con el nombre '{}'", final_name));
                    return Ok(Action::None);
                }
                
                // Perform rename
                match std::fs::rename(old_path, &new_path) {
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

// T843: Handle password input dialog
fn handle_password_input_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    use crossterm::event::{KeyCode, KeyModifiers, KeyEventKind};
    
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

// T844: Handle collision dialog
fn handle_collision_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    use crossterm::event::{KeyCode, KeyModifiers, KeyEventKind};
    
    // BUG-002 FIX: Filter out key release events to prevent double processing
    if key.kind != KeyEventKind::Press {
        return Ok(Action::None);
    }
    
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
                        // Rename - BUG-003/BUG-005 FIX: Generate name with suffix
                        app.close_dialog();
                        match operation_type {
                            crate::app::CollisionOperation::Copy => start_copy_operation_with_rename(app)?,
                            crate::app::CollisionOperation::Move => start_move_operation_with_rename(app)?,
                            crate::app::CollisionOperation::Extract => {},
                        }
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
    
    // BUG-001 FIX: Check if directory already exists
    if new_path.exists() {
        if new_path.is_dir() {
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
    
    // Use blocking task for simplicity (create_dir is fast)
    match std::fs::create_dir(&new_path) {
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

fn start_delete_operation(app: &mut AppState) -> Result<()> {
    // T951: Validate source files exist before starting operation
    if app.has_selection() {
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        for path in &marked_paths {
            if !path.exists() {
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                app.show_error(format!("Archivo no encontrado: {}", file_name));
                return Ok(());
            }
        }
    } else {
        // Check single file exists
        if let Some(entry) = app.active_panel().selected_entry()
            && !entry.path.exists() {
                app.show_error(format!("Archivo no encontrado: {}", entry.name));
                return Ok(());
            }
    }
    
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
        let total_bytes = entry.size;
        
        let total_files = 1; // Single file or directory (estimate)
        
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
            // Keep the filtered results but exit search mode
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

/// Handle editor mode - TASK-030
fn handle_editor_mode(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    use crossterm::event::{KeyCode, KeyModifiers};
    use crate::ui::text_editor::EditorAction;
    
    if let Some(ref mut editor) = app.editor_state {
        // Check if this is a special action (Save, Close)
        let action = editor.handle_key(key.code, key.modifiers);
        
        match action {
            EditorAction::Save => {
                // Try to save
                match editor.save() {
                    Ok(_) => {
                        // Clear any error message
                        app.error_message = None;
                    }
                    Err(e) => {
                        app.error_message = Some(format!("Failed to save: {}", e));
                    }
                }
                Ok(Action::None)
            }
            EditorAction::Close => {
                // Close editor
                app.close_editor();
                Ok(Action::None)
            }
            EditorAction::ConfirmClose => {
                // Show unsaved changes dialog
                app.dialog_state = Some(DialogState::Confirm {
                    message: "File has unsaved changes. Close anyway?".to_string(),
                    confirm_action: ConfirmAction::CloseEditor,
                });
                Ok(Action::None)
            }
            EditorAction::Continue => {
                // Pass the key to textarea (only if not Ctrl+S or Esc)
                if !matches!((key.code, key.modifiers), 
                            (KeyCode::Char('s'), KeyModifiers::CONTROL) | 
                            (KeyCode::Char('S'), KeyModifiers::CONTROL) |
                            (KeyCode::Esc, _)) {
                    // Convert our KeyEvent to ratatui's crossterm KeyEvent
                    // We need to manually convert since they're different versions
                    use ratatui::crossterm::event::{
                        KeyCode as RKeyCode,
                        KeyModifiers as RKeyModifiers,
                        KeyEventKind as RKeyEventKind,
                        KeyEventState as RKeyEventState,
                    };
                    
                    // Convert KeyCode
                    let rcode = match key.code {
                        KeyCode::Backspace => RKeyCode::Backspace,
                        KeyCode::Enter => RKeyCode::Enter,
                        KeyCode::Left => RKeyCode::Left,
                        KeyCode::Right => RKeyCode::Right,
                        KeyCode::Up => RKeyCode::Up,
                        KeyCode::Down => RKeyCode::Down,
                        KeyCode::Home => RKeyCode::Home,
                        KeyCode::End => RKeyCode::End,
                        KeyCode::PageUp => RKeyCode::PageUp,
                        KeyCode::PageDown => RKeyCode::PageDown,
                        KeyCode::Tab => RKeyCode::Tab,
                        KeyCode::BackTab => RKeyCode::BackTab,
                        KeyCode::Delete => RKeyCode::Delete,
                        KeyCode::Insert => RKeyCode::Insert,
                        KeyCode::F(n) => RKeyCode::F(n),
                        KeyCode::Char(c) => RKeyCode::Char(c),
                        KeyCode::Null => RKeyCode::Null,
                        KeyCode::Esc => RKeyCode::Esc,
                        _ => RKeyCode::Null, // Default for unhandled keys
                    };
                    
                    // Convert KeyModifiers
                    let mut rmod = RKeyModifiers::empty();
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        rmod |= RKeyModifiers::SHIFT;
                    }
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        rmod |= RKeyModifiers::CONTROL;
                    }
                    if key.modifiers.contains(KeyModifiers::ALT) {
                        rmod |= RKeyModifiers::ALT;
                    }
                    
                    // Convert KeyEventKind
                    let rkind = match key.kind {
                        KeyEventKind::Press => RKeyEventKind::Press,
                        KeyEventKind::Repeat => RKeyEventKind::Repeat,
                        KeyEventKind::Release => RKeyEventKind::Release,
                    };
                    
                    // Convert KeyEventState
                    let mut rstate = RKeyEventState::empty();
                    if key.state.contains(KeyEventState::KEYPAD) {
                        rstate |= RKeyEventState::KEYPAD;
                    }
                    if key.state.contains(KeyEventState::CAPS_LOCK) {
                        rstate |= RKeyEventState::CAPS_LOCK;
                    }
                    if key.state.contains(KeyEventState::NUM_LOCK) {
                        rstate |= RKeyEventState::NUM_LOCK;
                    }
                    
                    let ratatui_key = ratatui::crossterm::event::KeyEvent {
                        code: rcode,
                        modifiers: rmod,
                        kind: rkind,
                        state: rstate,
                    };
                    editor.input_key(ratatui_key);
                }
                Ok(Action::None)
            }
        }
    } else {
        Ok(Action::None)
    }
}

/// Handle compress options dialog
fn handle_compress_options_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    use crossterm::event::{KeyCode, KeyModifiers, KeyEventKind};
    use crate::archive::formats::ArchiveFormat;
    use crate::archive::compressor::CompressionLevel;
    
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

// US4: Handle drive selector dialog key events
fn handle_drive_selector_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    use crossterm::event::{KeyCode, KeyModifiers};
    
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

// US5: Handle theme selector dialog key events
fn handle_theme_selector_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    use crossterm::event::{KeyCode, KeyModifiers};
    
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

// TASK-008: Handle bookmark manager dialog key events
fn handle_bookmark_manager_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    use crossterm::event::{KeyCode, KeyModifiers};
    
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

/// TASK-018: Handle navigation history viewer dialog input
fn handle_history_viewer_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    use crossterm::event::{KeyCode, KeyModifiers};
    
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

// TASK-021: Handle Go To Path dialog events
fn handle_goto_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    use crossterm::event::{KeyCode, KeyModifiers};
    
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

/// Expand environment variables and validate path
fn expand_and_validate_path(input: &str, current_dir: &std::path::Path) -> Result<std::path::PathBuf, String> {
    use std::path::PathBuf;
    
    let input = input.trim();
    
    // Expand ~ to home directory
    let expanded = if input.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            let rest = &input[1..];
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
    if path_str.starts_with(r"\\?\") {
        // Remove the \\?\ prefix
        PathBuf::from(&path_str[4..])
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
fn get_directory_children(path: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut children = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    children.push(entry.path());
                }
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
    let expanded = if input.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            let rest = &input[1..];
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

