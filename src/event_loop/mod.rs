//! Event loop and asynchronous operation execution
//!
//! This module implements the main event loop architecture for the file manager,
//! handling user input, UI rendering, and background file operations.
//!
//! # Architecture
//!
//! The event loop follows a concurrent model with three main components:
//!
//! 1. **Input Handling**: Processes keyboard events via crossterm with 50ms timeout
//! 2. **Operation Execution**: Runs file operations (copy/move/delete/compress) in background tasks
//! 3. **Progress Updates**: Receives progress updates via mpsc channel and updates UI
//!
//! # Module Organization
//!
//! - `progress` - Progress tracking and operation completion handling
//! - `archive_handlers` - Archive extraction and compression dialog handlers
//! - `operation_executors` - Background task execution for file operations
//!
//! # Operation Lifecycle
//!
//! ```text
//! User Input → Action → Operation Created → Background Task Spawned
//!     ↓            ↓           ↓                      ↓
//! handle_key → Confirm → app.current_operation → tokio::spawn
//!                                                      ↓
//!                                              Progress Updates
//!                                                      ↓
//!                                              UI Refresh (50ms)
//!                                                      ↓
//!                                              Completion/Error
//! ```
//!
//! # Cancellation
//!
//! Operations support graceful cancellation via `tokio::sync::watch` channel.
//! User presses Esc during operation → cancel signal sent → task cleanup → UI update.

use std::time::Duration;
use anyhow::Result;
use crossterm::event::{self, Event};
use ratatui::Terminal;
use tokio::sync::mpsc;

use crate::app::{AppState, DialogState, ConfirmAction};
use crate::events::handler::handle_key;
use crate::events::keybindings::Action;
use crate::models::operation::Progress;
use crate::ui;

// Submodules
mod progress;
mod archive_handlers;
mod operation_executors;

// Re-export commonly used functions
pub use operation_executors::execute_operation;

// Internal module functions
use progress::{process_progress_updates, check_operation_completion};
use archive_handlers::{
    handle_extract_options,
    handle_password_input,
    handle_compress_options,
    handle_extract_confirm,
};
use operation_executors::execute_operation as _execute_operation;

/// Main event loop for the application
///
/// Runs the application's main loop, handling user input, rendering the UI,
/// and managing background file operations.
///
/// # Architecture
///
/// The loop operates in cycles:
/// 1. Process progress updates from background operations
/// 2. Check if operations completed (success or error)
/// 3. Start any queued operations
/// 4. Update text scroll animations
/// 5. Render the current UI state
/// 6. Auto-refresh panels (every 5 seconds)
/// 7. Poll for user input (50ms timeout)
/// 8. Handle cancellation (Esc during Progress dialog)
///
/// # Arguments
///
/// * `terminal` - Ratatui terminal instance for rendering
/// * `app` - Application state containing panels, dialogs, and operations
///
/// # Cancellation
///
/// Press Esc during a Progress dialog to cancel the current operation.
/// The operation will cleanup partial files and return to normal state.
///
/// # Returns
///
/// `Ok(())` when user quits normally, or an error if rendering/IO fails.
pub async fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut AppState,
) -> Result<()> {
    let (progress_tx, mut progress_rx) = mpsc::channel::<Progress>(1000);
    let mut operation_task: Option<tokio::task::JoinHandle<Result<()>>> = None;
    
    // T955: Cancellation channel - watch channel to signal cancellation
    let (_cancel_tx, _cancel_rx) = tokio::sync::watch::channel(false);
    let mut current_cancel_tx: Option<tokio::sync::watch::Sender<bool>> = None;
    
    loop {
        // Process progress updates
        process_progress_updates(app, &mut progress_rx, &mut operation_task).await;
        
        // Check if current operation completed
        check_operation_completion(app, &mut operation_task).await;
        
        // Start new operation if one is queued
        start_queued_operation(app, &mut operation_task, &progress_tx, &mut current_cancel_tx);
        
        // US3: Update text scroll animation for active panel
        let active_panel = app.active_panel_mut();
        let term_size = terminal.size()?;
        let panel_width = (term_size.width / 2).saturating_sub(4); // Account for borders, half screen
        let column_layout = ui::column_layout::ColumnLayout::calculate(panel_width, &active_panel.entries);
        let _needs_refresh = active_panel.update_text_scroll(
            column_layout.name_width as usize,
            column_layout.ext_width as usize,
            column_layout.size_width as usize,
            column_layout.modified_width as usize,
            column_layout.created_width as usize,
            column_layout.perms_width as usize,
        );
        
        // Draw UI
        render_ui(terminal, app)?;
        
        // Auto-refresh: Check if directories have changed externally (every 5 seconds)
        if let Err(e) = app.check_and_refresh_panels() {
            log::warn!("Auto-refresh error: {}", e);
        }
        
        // Handle input
        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()? {
                // Only process key press events, ignore release and repeat
                use crossterm::event::KeyEventKind;
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                
                // T955: Check if user pressed Esc during progress dialog to cancel
                if let Some(DialogState::Progress { .. }) = &app.dialog_state
                    && matches!(key.code, crossterm::event::KeyCode::Esc) {
                        log::info!("User requested operation cancellation");
                        if let Some(ref cancel_sender) = current_cancel_tx {
                            let _ = cancel_sender.send(true);
                            log::info!("Cancellation signal sent");
                        }
                        continue; // Don't process other actions while canceling
                    }
                
                let action = handle_key(app, key)?;
                
                if action == Action::Quit {
                    cancel_operation(&mut operation_task);
                    break;
                }
                
                handle_action(app, action, current_cancel_tx.as_ref()).await?;
            }
    }
    
    Ok(())
}

/// Start a queued operation if one is waiting and no task is running
///
/// Checks if `app.current_operation` exists and no task is currently executing.
/// If so, spawns a new tokio task to execute the operation in the background.
/// Creates a new cancellation channel for this operation.
///
/// # Arguments
///
/// * `app` - Application state containing the queued operation
/// * `operation_task` - Mutable reference to store the spawned task handle
/// * `progress_tx` - Channel sender for progress updates
/// * `cancel_tx_holder` - Mutable reference to store the cancellation sender
fn start_queued_operation(
    app: &AppState,
    operation_task: &mut Option<tokio::task::JoinHandle<Result<()>>>,
    progress_tx: &mpsc::Sender<Progress>,
    cancel_tx_holder: &mut Option<tokio::sync::watch::Sender<bool>>,
) {
    if let Some(op) = &app.current_operation
        && operation_task.is_none() {
            let op = op.clone();
            let tx = progress_tx.clone();
            
            // T955: Create new cancellation channel for this operation
            let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
            *cancel_tx_holder = Some(cancel_tx);
            
            *operation_task = Some(tokio::spawn(async move {
                _execute_operation(op, tx, cancel_rx).await
            }));
        }
}

/// Refresh both panels after operation completion
///
/// Refreshes directory entries for both left and right panels and updates
/// the stored "all entries" state for each panel.
///
/// # Arguments
///
/// * `app` - Application state with panels to refresh
pub(crate) fn refresh_panels(app: &mut AppState) {
    let _ = app.left_panel.refresh_entries();
    app.left_all_entries = app.left_panel.entries.clone();
    let _ = app.right_panel.refresh_entries();
    app.right_all_entries = app.right_panel.entries.clone();
}

/// Cancel running operation by aborting its task
///
/// Aborts the tokio task running the operation. Task cleanup and
/// partial file removal is handled by the individual executors.
///
/// # Arguments
///
/// * `operation_task` - Optional task handle to abort
fn cancel_operation(operation_task: &mut Option<tokio::task::JoinHandle<Result<()>>>) {
    if let Some(task) = operation_task.take() {
        task.abort();
    }
}

/// Render the UI to the terminal
///
/// Orchestrates all UI rendering including headers, panels, footer,
/// dialogs, preview modal, text editor, and search dialog.
///
/// # Arguments
///
/// * `terminal` - Ratatui terminal backend
/// * `app` - Application state to render
fn render_ui<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut AppState,
) -> Result<()> {
    terminal.draw(|f| {
        // Show welcome screen if flag is set
        if app.show_welcome {
            ui::render_welcome(f, env!("CARGO_PKG_VERSION"), &app.theme);
            return;
        }

        let layout = ui::layout::create_layout(f.area());
        
        // Render headers (T075-T076: two separate blocks aligned with panels)
        ui::render_header(f, app, layout.left_header, layout.right_header);
        
        // Render panels
        ui::render_panels(f, app, &layout);
        
        // Render footer
        ui::render_footer(f, layout.footer, &app.theme);
        
        // Render preview modal if present
        if let Some(preview) = &app.preview_state {
            ui::preview_modal::render_preview_modal(f, preview, &app.theme);
        }
        
        // Render text editor if present (TASK-030)
        if let Some(ref mut editor) = app.editor_state {
            editor.render(f, f.area(), &app.theme);
        }
        
        // Render search dialog if present (TASK-040)
        // TASK-041: Update dialog state (debouncing + results polling)
        if let Some(ref mut dialog) = app.search_dialog {
            dialog.update();
            dialog.render(f, f.area(), &app.theme);
        }
        
        // Render dialog LAST so it appears on top of everything
        ui::render_dialog_if_present(f, app);
    })?;
    
    Ok(())
}

/// Handle different action types from key input
///
/// Processes high-level actions like opening previews, starting archive
/// operations, and handling confirmations.
///
/// # Arguments
///
/// * `app` - Application state
/// * `action` - Action to handle
/// * `cancel_tx` - Optional cancellation sender for current operation
async fn handle_action(
    app: &mut AppState,
    action: Action,
    cancel_tx: Option<&tokio::sync::watch::Sender<bool>>,
) -> Result<()> {
    match action {
        Action::OpenPreview => {
            app.open_text_preview().await?;
        }
        Action::ExtractArchive => {
            app.start_extract_archive()?;
        }
        Action::CompressArchive => {
            app.start_compress_archive()?;
        }
        Action::ConfirmYes => {
            handle_confirm_action(app, cancel_tx).await?;
        }
        _ => {}
    }
    
    Ok(())
}

/// Handle confirmation actions for dialogs
///
/// Processes confirmations for various dialog types including extraction options,
/// password input, and compression settings. Delegates to specialized handlers
/// in the archive_handlers module.
///
/// # Arguments
///
/// * `app` - Application state
/// * `cancel_tx` - Optional cancellation sender for current operation
async fn handle_confirm_action(
    app: &mut AppState,
    cancel_tx: Option<&tokio::sync::watch::Sender<bool>>,
) -> Result<()> {
    log::info!("handle_confirm_action called");
    // Clone the dialog state to avoid borrow checker issues
    let dialog_state = app.dialog_state.clone();
    
    if let Some(dialog) = dialog_state {
        log::info!("Dialog state: {:?}", std::mem::discriminant(&dialog));
        match dialog {
            DialogState::ExtractOptions { source, dest, format, archive_name, selected } => {
                log::info!("Handling ExtractOptions dialog");
                handle_extract_options(app, &source, &dest, &format, &archive_name, selected)?;
                log::info!("After handle_extract_options, dialog state is: {:?}", 
                    app.dialog_state.as_ref().map(std::mem::discriminant));
            }
            DialogState::PasswordInput { archive_path, dest_path, format, value, .. } => {
                log::info!("Handling PasswordInput dialog");
                handle_password_input(app, &archive_path, &dest_path, &format, &value)?;
            }
            DialogState::CompressOptions { sources, output_name, format, level, use_password, password, .. } => {
                // Create cancel receiver from cancel_tx
                let (_cancel_tx_local, cancel_rx) = tokio::sync::watch::channel(false);
                let cancel_rx_to_use = if let Some(tx) = cancel_tx {
                    tx.subscribe()
                } else {
                    cancel_rx
                };
                handle_compress_options(app, sources, &output_name, format, level, use_password, &password, cancel_rx_to_use).await?;
            }
            DialogState::Confirm { confirm_action: ConfirmAction::ExtractArchive { source, dest, format }, .. } => {
                handle_extract_confirm(app, &source, &dest, format).await?;
            }
            _ => {}
        }
    }
    
    Ok(())
}
