//! Navigation dialog handlers
//!
//! Handles dialogs for navigation:
//! - Drive selector (change drives)
//! - Bookmark manager (add/remove/navigate bookmarks)
//! - Navigation history viewer
//! - Go To Path dialog (with autocompletion)

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{AppState, DialogState};
use crate::events::keybindings::Action;

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
                
                // Disconnect from remote if currently connected
                if panel.is_remote() {
                    panel.vfs = None;
                    panel.connection_info = None;
                }
                
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
                    
                    // Disconnect from remote if currently connected
                    if panel.is_remote() {
                        panel.vfs = None;
                        panel.connection_info = None;
                    }
                    
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
            app.error_message = Some(format!("BOOKMARK:{}", current_path.display()));
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
                        
                        // Disconnect from remote if currently connected
                        if panel.is_remote() {
                            panel.vfs = None;
                            panel.connection_info = None;
                        }
                        
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
                        
                        // Disconnect from remote if currently connected and navigating to local path
                        if panel.is_remote() {
                            panel.vfs = None;
                            panel.connection_info = None;
                        }
                        
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
