// Event handler
use anyhow::Result;
use crossterm::event::{KeyEvent, KeyCode, KeyEventKind};

use crate::app::{AppState, ConfirmAction, DialogState};
use crate::events::keybindings::{map_key_to_action, Action};

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
        return handlers::modes::handle_preview_mode(app, key);
    }
    
    // TASK-030: Special handling for editor mode (after dialogs and preview)
    if app.has_editor() {
        return handlers::modes::handle_editor_mode(app, key);
    }
    
    // Special handling for search mode
    if app.search_mode {
        return handlers::modes::handle_search_mode(app, key);
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
            handlers::file_operations::handle_copy_request(app)?;
        }
        Action::Move => {
            handlers::file_operations::handle_move_request(app)?;
        }
        Action::Delete => {
            handlers::file_operations::handle_delete_request(app)?;
        }
        Action::CreateFolder => {
            handlers::file_operations::handle_create_folder_request(app)?;
        }
        Action::Rename => {
            handlers::file_operations::handle_rename_request(app, true)?; // true = allow extension change with Ctrl+R
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
            // Check if we're in remote mode - bookmarks don't work with remote connections
            if app.active_panel().is_remote() {
                app.dialog_state = Some(DialogState::Error {
                    message: "Bookmarks are not available for remote connections.\n\nUse saved connections instead (Ctrl+M to manage).".to_string()
                });
            } else {
                let current_path = app.active_panel().current_path.clone();
                let default_name = current_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Bookmark")
                    .to_string();
                
                // Use error_message to pass the path as context with BOOKMARK: prefix
                app.error_message = Some(format!("BOOKMARK:{}", current_path.to_string_lossy()));
                app.dialog_state = Some(DialogState::Input {
                    prompt: "Bookmark name:".to_string(),
                    value: default_name,
                });
            }
        }
        Action::ShowHelp => {
            // F1: Show help dialog with all keybindings
            app.dialog_state = Some(DialogState::Help);
        }
        Action::ToggleBookmarkManager => {
            // TASK-008: Open bookmark manager dialog
            // Check if we're in remote mode - bookmarks don't work with remote connections
            if app.active_panel().is_remote() {
                app.dialog_state = Some(DialogState::Error {
                    message: "Bookmarks are not available for remote connections.\n\nUse saved connections instead (Ctrl+M to manage).".to_string()
                });
            } else {
                let state = crate::ui::bookmark_manager::BookmarkManagerState::new();
                app.dialog_state = Some(DialogState::BookmarkManager { state });
            }
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
        Action::DisconnectRemote => {
            // Disconnect from remote filesystem
            let panel = app.active_panel_mut();
            if panel.vfs.is_some() {
                // Get home directory as fallback
                let fallback = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"));
                panel.disconnect_remote(fallback.clone());
                
                // Try to refresh panel
                if let Err(e) = panel.refresh_entries() {
                    log::error!("Failed to refresh panel after disconnect: {}", e);
                    app.error_message = Some(format!("Disconnected but failed to load directory: {}", e));
                } else {
                    app.error_message = Some("Disconnected from remote filesystem".to_string());
                }
            } else {
                app.error_message = Some("Not connected to any remote filesystem".to_string());
            }
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
                        handlers::file_operations::start_copy_operation(app)?;
                        // Don't close dialog - start_copy_operation sets progress dialog
                    }
                    ConfirmAction::Move => {
                        handlers::file_operations::start_move_operation(app)?;
                        // Don't close dialog - start_move_operation sets progress dialog
                    }
                    ConfirmAction::Delete => {
                        handlers::file_operations::start_delete_operation(app)?;
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

// Helper to refresh panel entries and store unfiltered list
fn refresh_and_store(app: &mut AppState) -> Result<()> {
    app.active_panel_mut().refresh_entries()?;
    let entries = app.active_panel().entries.clone();
    app.store_all_entries(entries);
    Ok(())
}
