// Event handler
use anyhow::Result;
use crossterm::event::{KeyEvent, KeyCode, KeyEventKind, KeyEventState};
use std::path::PathBuf;
use std::sync::Arc;

use crate::app::{AppState, ConfirmAction, DialogState};
use crate::events::keybindings::{map_key_to_action, map_key_to_input_action, Action};
use crate::models::operation::Operation;

// Import modular handlers
use crate::events::handlers;

pub fn handle_key(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    // Filter out non-Press events early (ignore Release and Repeat for most actions)
    if key.kind != KeyEventKind::Press {
        return Ok(Action::None);
    }

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
        return handlers::dialogs::handle_input_dialog(app, key);
    }

    if let Some(DialogState::PasswordInput { .. }) = &app.dialog_state {
        return handlers::dialogs::handle_password_input_dialog(app, key);
    }

    if let Some(DialogState::DriveSelector { .. }) = &app.dialog_state {
        return handlers::dialogs::handle_drive_selector_dialog(app, key);
    }

    if let Some(DialogState::ThemeSelector { .. }) = &app.dialog_state {
        return handlers::dialogs::handle_theme_selector_dialog(app, key);
    }

    if let Some(DialogState::BookmarkManager { .. }) = &app.dialog_state {
        return handlers::dialogs::handle_bookmark_manager_dialog(app, key);
    }

    if let Some(DialogState::HistoryViewer { .. }) = &app.dialog_state {
        return handlers::dialogs::handle_history_viewer_dialog(app, key);
    }

    if let Some(DialogState::GoToPath { .. }) = &app.dialog_state {
        return handlers::dialogs::handle_goto_dialog(app, key);
    }

    if let Some(DialogState::CompressOptions { .. }) = &app.dialog_state {
        return handlers::dialogs::handle_compress_options_dialog(app, key);
    }
    
    // Handle remote connection dialog
    if let Some(DialogState::RemoteConnection { .. }) = &app.dialog_state {
        return handlers::dialogs::handle_connection_dialog(app, key);
    }
    
    // Handle Help dialog (F1) - close with ESC or any key
    if let Some(DialogState::Help) = &app.dialog_state {
        // Close help dialog with ESC or F1
        let action = map_key_to_action(key);
        if matches!(action, Action::Cancel | Action::ShowHelp) {
            app.close_dialog();
        }
        return Ok(Action::None);
    }
    
    // Handle Confirm, Progress, Error dialogs before checking editor/preview
    if let Some(DialogState::Confirm { .. }) = &app.dialog_state {
        let action = map_key_to_action(key);
        return handle_dialog_action(app, action);
    }
    
    if let Some(DialogState::Progress { .. }) = &app.dialog_state {
        let action = map_key_to_action(key);
        return handle_dialog_action(app, action);
    }
    
    if let Some(DialogState::Error { .. }) = &app.dialog_state {
        let action = map_key_to_action(key);
        return handle_dialog_action(app, action);
    }
    
    // TASK-040: Handle recursive search dialog (before preview/editor)
    if app.has_search_dialog() {
        return handlers::dialogs::handle_search_dialog(app, key);
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
        return handlers::dialogs::handle_rename_dialog(app, key);
    }
    
    // T844: Special handling for collision prompts
    if let Some(DialogState::CollisionPrompt { .. }) = &app.dialog_state {
        return handlers::dialogs::handle_collision(app, key);
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
            // Smart Enter: directory = enter, file = preview
            if let Some(entry) = app.active_panel().selected_entry() {
                use crate::models::file_entry::EntryType;
                
                if entry.entry_type == EntryType::Dir {
                    // Directory: enter it (original behavior)
                    app.active_panel_mut().enter_dir()?;
                    refresh_and_store(app)?;
                    // T578: Clear marks when navigating to different directory
                    app.selection_state.clear(app.active_panel);
                    // Exit search mode when entering a directory
                    if app.search_mode {
                        app.search_mode = false;
                        app.search_pattern.clear();
                    }
                } else {
                    // File: show preview (return action for async handling in main.rs)
                    return Ok(Action::OpenPreview);
                }
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
            handle_rename_request(app, true)?; // true = allow extension change with Ctrl+R
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
            
            if let Some(selected_entry) = panel.selected_entry()
                && selected_entry.is_file() {
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
        Action::AddBookmark => {
            // Quick add current directory to bookmarks (Ctrl+Shift+D)
            let current_path = app.active_panel().current_path.clone();
            let default_name = current_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Bookmark")
                .to_string();
            
            // Use error_message to pass the path as context (like existing bookmark code does)
            app.error_message = Some(current_path.to_string_lossy().to_string());
            app.dialog_state = Some(DialogState::Input {
                prompt: "Bookmark name:".to_string(),
                value: default_name,
            });
        }
        Action::ShowHelp => {
            // F1: Show help dialog with all keybindings
            app.dialog_state = Some(DialogState::Help);
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
            let initial_suggestions = handlers::dialogs::get_directory_children(&current_path);
            
            // Initialize input with current path
            let initial_input = current_path.to_string_lossy().to_string();
            
            app.dialog_state = Some(DialogState::GoToPath {
                input: initial_input,
                error_message: None,
                suggestions: initial_suggestions,
                selected_suggestion: 0,
            });
        }
        Action::OpenRecursiveSearch => {
            // TASK-040: Open recursive search dialog
            app.open_search_dialog();
        }
        Action::OpenRemoteConnection => {
            // Open remote connection dialog
            let state = crate::ui::connection_dialog::ConnectionDialogState::new();
            app.dialog_state = Some(DialogState::RemoteConnection { state });
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
                        return Ok(Action::None);
                    }
                    ConfirmAction::ExtractArchive { .. } => {
                        // Return action to main.rs for async extraction
                        return Ok(Action::ConfirmYes);
                    }
                    ConfirmAction::AddBookmark(_path) => {
                        // This shouldn't happen as AddBookmark uses Input dialog, not Confirm
                        // But we need to handle it for exhaustive match
                        app.close_dialog();
                        return Ok(Action::None);
                    }
                }
            }
        }
        Action::ConfirmNo | Action::Cancel => {
            app.close_dialog();
            return Ok(Action::None);
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
    
    // Get VFS references from both panels
    let source_vfs = app.active_panel().vfs.clone();
    let dest_vfs = app.inactive_panel().vfs.clone();
    
    // T951: Validate source files exist before starting operation
    // Note: For remote files, we'll check during the actual operation
    if source_vfs.is_none() {
        // Only validate local files upfront
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
        // Get VFS references
        let source_vfs = app.active_panel().vfs.clone();
        let dest_vfs = app.inactive_panel().vfs.clone();
        
        let collision_result: Option<(String, Vec<PathBuf>)> = if app.has_selection() {
            // Check marked items for collisions
            let marked_paths = app.selection_state.get_marked(app.active_panel);
            let mut collision_info = None;
            
            for (i, path) in marked_paths.iter().enumerate() {
                let file_name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let dest = dest_panel_path.join(&file_name);
                
                // Check if destination exists using VFS if available
                let exists = if let Some(vfs) = &dest_vfs {
                    vfs.exists(&dest).unwrap_or(false)
                } else {
                    dest.exists()
                };
                
                if exists {
                    // Found collision - collect remaining files
                    let remaining: Vec<PathBuf> = marked_paths.iter().skip(i + 1).cloned().collect();
                    collision_info = Some((dest.to_string_lossy().to_string(), remaining));
                    break;
                }
            }
            collision_info
        } else {
            // Check single item
            app.active_panel().selected_entry().and_then(|entry| {
                let dest = dest_panel_path.join(&entry.name);
                
                // Check if destination exists using VFS if available
                let exists = if let Some(vfs) = &dest_vfs {
                    vfs.exists(&dest).unwrap_or(false)
                } else {
                    dest.exists()
                };
                
                if exists {
                    Some((dest.to_string_lossy().to_string(), Vec::new()))
                } else {
                    None
                }
            })
        };
        
        // If collision detected, show collision dialog
        if let Some((collision_path, remaining_files)) = collision_result {
            app.dialog_state = Some(DialogState::CollisionPrompt {
                file_path: collision_path,
                selected: 0,
                operation: crate::app::CollisionOperation::Copy,
                remaining_files,
                dest_path: dest_panel_path.clone(),
                source_vfs: source_vfs.clone(),
                dest_vfs: dest_vfs.clone(),
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
        
        // T953: Check available disk space before copying (only for local destinations)
        if dest_vfs.is_none() {
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
        
        let operation = Operation::copy_batch_vfs(operations, total_bytes, count, source_vfs.clone(), dest_vfs.clone());
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
            
            // T953: Check available disk space before copying (only for local destinations)
            if dest_vfs.is_none() {
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
            }
            
            let total_files = 1; // Single file or directory
            
            let operation = Operation::copy_vfs(source, destination, total_bytes, total_files, source_vfs, dest_vfs);
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
    
    // Get VFS references from both panels
    let source_vfs = app.active_panel().vfs.clone();
    let dest_vfs = app.inactive_panel().vfs.clone();
    
    // T574: Check if batch operation or single
    if app.has_selection() {
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        let count = marked_paths.len();
        
        // Calculate total size and create batch operation
        let mut total_bytes = 0u64;
        let mut operations = Vec::new();
        
        for path in &marked_paths {
            // Get size using VFS if available
            let size = if let Some(vfs) = &source_vfs {
                vfs.metadata(path).map(|m| m.size).unwrap_or(0)
            } else {
                std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            };
            
            total_bytes += size;
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
        
        let operation = Operation::copy_batch_vfs(operations, total_bytes, count, source_vfs, dest_vfs);
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
            
            let operation = Operation::copy_vfs(source, destination, total_bytes, total_files, source_vfs, dest_vfs);
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
    
    // Get VFS references from both panels
    let source_vfs = app.active_panel().vfs.clone();
    let dest_vfs = app.inactive_panel().vfs.clone();
    
    // T574: Check if batch operation or single
    if app.has_selection() {
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        let count = marked_paths.len();
        
        // Calculate total size and create batch operation
        let mut total_bytes = 0u64;
        let mut operations = Vec::new();
        
        for path in &marked_paths {
            // Get size using VFS if available
            let size = if let Some(vfs) = &source_vfs {
                vfs.metadata(path).map(|m| m.size).unwrap_or(0)
            } else {
                std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            };
            
            total_bytes += size;
            if let Some(file_name) = path.file_name() {
                let file_name = file_name.to_string_lossy().to_string();
                let destination = dest_panel_path.join(&file_name);
                
                // Always generate collision-free name
                let final_destination = crate::fs::operations::generate_collision_free_name(&destination);
                
                operations.push((path.clone(), final_destination, file_name));
            }
        }
        
        let operation = Operation::copy_batch_vfs(operations, total_bytes, count, source_vfs, dest_vfs);
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
            
            let operation = Operation::copy_vfs(source, final_destination, total_bytes, total_files, source_vfs, dest_vfs);
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
    
    // Get VFS references from both panels
    let source_vfs = app.active_panel().vfs.clone();
    let dest_vfs = app.inactive_panel().vfs.clone();
    
    // T951: Validate source files exist before starting operation
    // Only validate for local files (when source VFS is None)
    if source_vfs.is_none() {
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
    }
    
    // T844: Check for collisions first
    // For remote dest, use VFS exists check
    let collision_result: Option<(String, Vec<PathBuf>)> = if app.has_selection() {
        // Check marked items for collisions
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        let mut collision_info = None;
        
        for (i, path) in marked_paths.iter().enumerate() {
            let file_name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let dest = dest_panel_path.join(&file_name);
            
            let exists = if let Some(vfs) = &dest_vfs {
                vfs.exists(&dest).unwrap_or(false)
            } else {
                dest.exists()
            };
            
            if exists {
                // Found collision - collect remaining files
                let remaining: Vec<PathBuf> = marked_paths.iter().skip(i + 1).cloned().collect();
                collision_info = Some((dest.to_string_lossy().to_string(), remaining));
                break;
            }
        }
        collision_info
    } else {
        // Check single item
        app.active_panel().selected_entry().and_then(|entry| {
            let dest = dest_panel_path.join(&entry.name);
            
            let exists = if let Some(vfs) = &dest_vfs {
                vfs.exists(&dest).unwrap_or(false)
            } else {
                dest.exists()
            };
            
            if exists {
                Some((dest.to_string_lossy().to_string(), Vec::new()))
            } else {
                None
            }
        })
    };
    
    // If collision detected, show collision dialog
    if let Some((collision_path, remaining_files)) = collision_result {
        app.dialog_state = Some(DialogState::CollisionPrompt {
            file_path: collision_path,
            selected: 0,
            operation: crate::app::CollisionOperation::Move,
            remaining_files,
            dest_path: dest_panel_path.clone(),
            source_vfs: source_vfs.clone(),
            dest_vfs: dest_vfs.clone(),
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
            // Get size using VFS if available
            let size = if let Some(vfs) = &source_vfs {
                vfs.metadata(path).map(|m| m.size).unwrap_or(0)
            } else {
                std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            };
            
            total_bytes += size;
            if let Some(file_name) = path.file_name() {
                let file_name = file_name.to_string_lossy().to_string();
                let destination = dest_panel_path.join(&file_name);
                operations.push((path.clone(), destination, file_name));
            }
        }
        
        let operation = Operation::move_batch_vfs(operations, total_bytes, count, source_vfs, dest_vfs);
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
            
            let operation = Operation::move_vfs(source, destination, total_bytes, total_files, source_vfs, dest_vfs);
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
    
    // Get VFS references from both panels
    let source_vfs = app.active_panel().vfs.clone();
    let dest_vfs = app.inactive_panel().vfs.clone();
    
    // T574: Check if batch operation or single
    if app.has_selection() {
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        let count = marked_paths.len();
        
        let mut total_bytes = 0u64;
        let mut operations = Vec::new();
        
        for path in &marked_paths {
            // Get size using VFS if available
            let size = if let Some(vfs) = &source_vfs {
                vfs.metadata(path).map(|m| m.size).unwrap_or(0)
            } else {
                std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            };
            
            total_bytes += size;
            if let Some(file_name) = path.file_name() {
                let file_name = file_name.to_string_lossy().to_string();
                let destination = dest_panel_path.join(&file_name);
                operations.push((path.clone(), destination, file_name));
            }
        }
        
        let operation = Operation::move_batch_vfs(operations, total_bytes, count, source_vfs, dest_vfs);
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
            
            let operation = Operation::move_vfs(source, destination, total_bytes, total_files, source_vfs, dest_vfs);
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
    
    // Get VFS references from both panels
    let source_vfs = app.active_panel().vfs.clone();
    let dest_vfs = app.inactive_panel().vfs.clone();
    
    // T574: Check if batch operation or single
    if app.has_selection() {
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        let count = marked_paths.len();
        
        let mut total_bytes = 0u64;
        let mut operations = Vec::new();
        
        for path in &marked_paths {
            // Get size using VFS if available
            let size = if let Some(vfs) = &source_vfs {
                vfs.metadata(path).map(|m| m.size).unwrap_or(0)
            } else {
                std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            };
            
            total_bytes += size;
            if let Some(file_name) = path.file_name() {
                let file_name = file_name.to_string_lossy().to_string();
                let destination = dest_panel_path.join(&file_name);
                
                // Always generate collision-free name
                let final_destination = crate::fs::operations::generate_collision_free_name(&destination);
                
                operations.push((path.clone(), final_destination, file_name));
            }
        }
        
        let operation = Operation::move_batch_vfs(operations, total_bytes, count, source_vfs, dest_vfs);
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
            
            let operation = Operation::move_vfs(source, final_destination, total_bytes, total_files, source_vfs, dest_vfs);
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

fn start_delete_operation(app: &mut AppState) -> Result<()> {
    // Get VFS reference if we're on a remote filesystem
    let vfs = app.active_panel().vfs.clone();
    
    // T951: Validate source files exist before starting operation
    // Only validate for local files (when VFS is None)
    if vfs.is_none() {
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
    }
    // For remote files (vfs.is_some()), we skip the exists check
    // The VFS delete operation will handle errors if files don't exist
    
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
            let operation = Operation::delete_batch_vfs(batch_items, total_bytes, total_files, vfs);
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
        
        let operation = Operation::delete_vfs(source, total_bytes, total_files, vfs);
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
