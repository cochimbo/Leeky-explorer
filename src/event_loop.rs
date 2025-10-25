//! Event loop and asynchronous operation execution
//!
//! This module implements the main event loop architecture for the file manager,
//! handling user input, UI rendering, and background file operations.
//!
//! # Architecture
//!
//! The event loop follows a concurrent model with three main components:
//!
//! 1. **Input Handling**: Processes keyboard events via crossterm with 100ms timeout
//! 2. **Operation Execution**: Runs file operations (copy/move/delete/compress) in background tasks
//! 3. **Progress Updates**: Receives progress updates via mpsc channel and updates UI
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
//!                                              UI Refresh (100ms)
//!                                                      ↓
//!                                              Completion/Error
//! ```
//!
//! # Cancellation
//!
//! Operations support graceful cancellation via `tokio::sync::watch` channel.
//! User presses Esc during operation → cancel signal sent → task cleanup → UI update.
//!
//! # Error Handling
//!
//! - File not found: Validated before operation starts (T951)
//! - Permission denied: Detected in completion handler (T952)
//! - Disk space: Checked before extraction/copy (T953)
//! - Collisions: User prompted before overwrite (T954)

use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event};
use ratatui::Terminal;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::app::{AppState, DialogState, ConfirmAction};
use crate::events::handler::handle_key;
use crate::events::keybindings::Action;
use crate::models::operation::{Operation, OperationType, Progress};
use crate::ui;

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
/// 4. Render the current UI state
/// 5. Poll for user input (50ms timeout)
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
        let _needs_refresh = active_panel.update_text_scroll(column_layout.name_width as usize);
        
        // Draw UI
        render_ui(terminal, app)?;
        
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

/// Process all available progress updates from background operations
///
/// Drains the progress channel and updates the application state with
/// the latest progress information. Handles both incremental progress
/// and operation completion messages.
///
/// # Arguments
///
/// * `app` - Application state to update with progress
/// * `progress_rx` - Channel receiving progress updates from background tasks
/// * `operation_task` - Current operation task handle (for logging)
async fn process_progress_updates(
    app: &mut AppState,
    progress_rx: &mut mpsc::Receiver<Progress>,
    operation_task: &mut Option<JoinHandle<Result<()>>>,
) {
    let mut count = 0;
    loop {
        match progress_rx.try_recv() {
            Ok(progress) => {
                count += 1;
                log::debug!("Event loop: Received progress update #{}: files {}/{}, bytes {}/{}",
                    count, progress.files_done, progress.files_total,
                    progress.bytes_done, progress.bytes_total);
                    
                if let Some(op) = &mut app.current_operation {
                    op.progress = progress;
                }
            }
            Err(mpsc::error::TryRecvError::Empty) => {
                if count > 0 {
                    log::debug!("Event loop: Processed {} progress updates this cycle", count);
                }
                break;
            }
            Err(mpsc::error::TryRecvError::Disconnected) => {
                handle_channel_closed(app, operation_task).await;
                break;
            }
        }
    }
}

/// Handle progress channel closure (operation completed or failed)
///
/// Called when the progress channel is disconnected, indicating the background
/// operation has finished. Awaits the task to get the result and updates UI.
///
/// # Arguments
///
/// * `app` - Application state to update with completion result
/// * `operation_task` - Operation task handle to await and check result
async fn handle_channel_closed(
    app: &mut AppState,
    operation_task: &mut Option<tokio::task::JoinHandle<Result<()>>>,
) {
    if let Some(task) = operation_task.take() {
        match task.await {
            Ok(Ok(())) => {
                log::info!("Operation completed successfully");
                app.selection_state.clear(app.active_panel);
                app.close_dialog();
                app.current_operation = None;
                refresh_panels(app);
            }
            Ok(Err(e)) => {
                let error_msg = format!("{}", e);
                log::error!("Operation failed: {}", error_msg);
                
                // Check if it's a password error during extraction
                if error_msg.contains("wrong password") || error_msg.contains("Password required") {
                    // Close progress dialog and reopen password input dialog
                    app.close_dialog();
                    
                    // Get operation details to recreate password dialog
                    if let Some(ref op) = app.current_operation
                        && let Some(format) = op.archive_format {
                            let source = op.source.clone();
                            let dest = op.destination.clone();
                            let archive_name = source.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("archive");
                            
                            // Show password dialog again with error message
                            app.dialog_state = Some(DialogState::PasswordInput {
                                prompt: format!("❌ Incorrect password for {}. Try again:", archive_name),
                                value: String::new(),
                                show_password: false,
                                archive_path: source.clone(),
                                dest_path: dest.clone(),
                                format,
                            });
                            
                            log::info!("Reopening password dialog after wrong password (from channel_closed)");
                        }
                    // CRITICAL: Clear current_operation to prevent auto-restart by start_queued_operation
                    app.current_operation = None;
                    log::info!("current_operation cleared to prevent auto-retry loop (from channel_closed)");
                } else {
                    // Other errors: show error and clear operation
                    app.show_error(format!("Operation failed: {}", error_msg));
                    app.current_operation = None;
                }
            }
            Err(e) => {
                log::error!("Task join error: {}", e);
                app.show_error(format!("Task error: {}", e));
                app.current_operation = None;
            }
        }
    }
}

/// Check if the current operation has completed
///
/// Polls the operation task handle to see if it has finished without blocking.
/// If completed, processes the result and updates the UI accordingly.
///
/// # Arguments
///
/// * `app` - Application state to update on completion
/// * `operation_task` - Operation task handle to poll
async fn check_operation_completion(
    app: &mut AppState,
    operation_task: &mut Option<tokio::task::JoinHandle<Result<()>>>,
) {
    // Check if we have an operation and a task
    if app.current_operation.is_some() && operation_task.is_some() {
        // Check if task is finished (either success or error)
        let task_finished = if let Some(task) = operation_task.as_ref() {
            task.is_finished()
        } else {
            false
        };
        
        if task_finished {
            log::info!("Operation task has finished, checking result");
            if let Some(task) = operation_task.take() {
                log::info!("Processing finished task result");
                match task.await {
                    Ok(Ok(())) => {
                        log::info!("Operation completion check: success");
                        app.selection_state.clear(app.active_panel);
                        app.close_dialog();
                        app.current_operation = None;
                        refresh_panels(app);
                    }
                    Ok(Err(e)) => {
                        let error_msg = format!("{}", e);
                        log::error!("Operation completed with error: {}", error_msg);
                        
                        // T952: Check for permission errors
                        if error_msg.contains("Permission denied") || error_msg.contains("Access denied") 
                            || error_msg.contains("permission") || error_msg.contains("access") {
                            log::info!("Detected permission error");
                            app.close_dialog();
                            app.show_error(format!("Permiso denegado: {}", error_msg));
                            app.current_operation = None;
                        }
                        // Check if it's a password error during extraction
                        else if error_msg.contains("wrong password") || error_msg.contains("Password required") {
                            log::info!("Detected password error, reopening dialog");
                            // Close progress dialog and reopen password input dialog
                            app.close_dialog();
                            
                            // Get operation details to recreate password dialog
                            if let Some(ref op) = app.current_operation
                                && let Some(format) = op.archive_format {
                                    let source = op.source.clone();
                                    let dest = op.destination.clone();
                                    let archive_name = source.file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("archive");
                                    
                                    // Show password dialog again with error message
                                    app.dialog_state = Some(DialogState::PasswordInput {
                                        prompt: format!("❌ Incorrect password for {}. Try again:", archive_name),
                                        value: String::new(),
                                        show_password: false,
                                        archive_path: source.clone(),
                                        dest_path: dest.clone(),
                                        format,
                                    });
                                    
                                    log::info!("Password dialog reopened successfully, task cleared");
                                }
                            // CRITICAL: Clear current_operation to prevent auto-restart by start_queued_operation
                            app.current_operation = None;
                            log::info!("current_operation cleared to prevent auto-retry loop");
                        } else {
                            // Other errors: show error and clear operation
                            log::info!("Non-password error, showing error dialog");
                            app.show_error(format!("Operation failed: {}", error_msg));
                            app.current_operation = None;
                        }
                    }
                    Err(e) => {
                        log::error!("Task join error during completion check: {}", e);
                        app.show_error(format!("Task error: {}", e));
                        app.current_operation = None;
                    }
                }
                log::info!("Task processing complete, operation_task is now None");
            }
        }
    }
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
/// * `cancel_tx_holder` - Mutable reference to store cancellation sender
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
                execute_operation(op, tx, cancel_rx).await
            }));
        }
}

/// Refresh both panels
fn refresh_panels(app: &mut AppState) {
    let _ = app.left_panel.refresh_entries();
    app.left_all_entries = app.left_panel.entries.clone();
    let _ = app.right_panel.refresh_entries();
    app.right_all_entries = app.right_panel.entries.clone();
}

/// Cancel running operation
fn cancel_operation(operation_task: &mut Option<tokio::task::JoinHandle<Result<()>>>) {
    if let Some(task) = operation_task.take() {
        task.abort();
    }
}

/// Render the UI
fn render_ui<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut AppState,
) -> Result<()> {
    terminal.draw(|f| {
        // Show welcome screen if flag is set
        if app.show_welcome {
            ui::render_welcome(f, env!("CARGO_PKG_VERSION"));
            return;
        }

        let layout = ui::layout::create_layout(f.area());
        
        // Render headers (T075-T076: two separate blocks aligned with panels)
        ui::render_header(f, app, layout.left_header, layout.right_header);
        
        // Render panels
        ui::render_panels(f, app, &layout);
        
        // Render footer
        ui::render_footer(f, layout.footer);
        
        // Render dialog if present
        ui::render_dialog_if_present(f, app);
        
        // Render preview modal if present
        if let Some(preview) = &app.preview_state {
            ui::preview_modal::render_preview_modal(f, preview);
        }
    })?;
    
    Ok(())
}

/// Handle different actions
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

/// Handle confirmation actions (extraction dialogs)
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

/// Handle extract options dialog
fn handle_extract_options(
    app: &mut AppState,
    source: &Path,
    dest: &Path,
    format: &crate::archive::formats::ArchiveFormat,
    archive_name: &str,
    selected: usize,
) -> Result<()> {
    let source = source.to_path_buf();
    let mut dest = dest.to_path_buf();
    let format = *format;
    let create_folder = selected == 1;
    
    // If option 1 selected, create a folder with archive name
    if create_folder {
        dest = dest.join(archive_name);
    }
    
    // T954: Check if destination already exists
    if dest.exists() {
        app.show_error(format!(
            "El directorio de destino ya existe: {}",
            dest.display()
        ));
        return Ok(());
    }
    
    // T953: Check available disk space before extraction
    let archive_size = std::fs::metadata(&source)
        .map(|m| m.len())
        .unwrap_or(0);
    
    // Estimate extracted size (typically 2-3x compressed size, use 3x to be safe)
    let estimated_extracted_size = archive_size * 3;
    
    // Get available space on destination
    if let Ok(available_space) = fs2::available_space(&dest)
        && available_space < estimated_extracted_size {
            let size_mb = estimated_extracted_size / (1024 * 1024);
            let avail_mb = available_space / (1024 * 1024);
            app.show_error(format!(
                "Espacio insuficiente. Se necesitan ~{} MB, disponibles {} MB",
                size_mb, avail_mb
            ));
            return Ok(());
        }
    
    // Check if archive is password-protected
    let is_encrypted = crate::archive::password::is_password_protected(&source)
        .unwrap_or(false);
    
    log::info!("Archive encryption check: {} - encrypted: {}", archive_name, is_encrypted);
    
    if is_encrypted {
        log::info!("Showing password input dialog for {}", archive_name);
        // Show password input dialog
        app.dialog_state = Some(DialogState::PasswordInput {
            prompt: format!("Enter password for {}:", archive_name),
            value: String::new(),
            show_password: false,
            archive_path: source,
            dest_path: dest,
            format,
        });
        log::info!("Password input dialog set successfully");
    } else {
        log::info!("No encryption detected, starting extraction directly");
        // No password needed, start extraction immediately
        start_extraction(app, source, dest, format, "Extrayendo archivo...");
    }
    
    Ok(())
}

/// Handle password input dialog
fn handle_password_input(
    app: &mut AppState,
    archive_path: &Path,
    dest_path: &Path,
    format: &crate::archive::formats::ArchiveFormat,
    value: &str,
) -> Result<()> {
    let source = archive_path.to_path_buf();
    let dest = dest_path.to_path_buf();
    let format = *format;
    let password = value.to_string();
    
    start_extraction_with_password(app, source, dest, format, password, "Extrayendo archivo con contraseña...");
    
    Ok(())
}

/// Handle compress options dialog
#[allow(clippy::too_many_arguments)]
async fn handle_compress_options(
    app: &mut AppState,
    sources: Vec<std::path::PathBuf>,
    output_name: &str,
    format: crate::archive::formats::ArchiveFormat,
    level: crate::archive::compressor::CompressionLevel,
    use_password: bool,
    password: &str,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<()> {
    use crate::archive::formats::ArchiveFormat;
    
    // Add appropriate extension based on format
    let extension = match format {
        ArchiveFormat::ZIP => ".zip",
        ArchiveFormat::TarGz => ".tar.gz",
        ArchiveFormat::TarBz2 => ".tar.bz2",
        ArchiveFormat::TarXz => ".tar.xz",
        ArchiveFormat::TAR => ".tar",
        ArchiveFormat::SEVENZ => ".7z",
        _ => ".zip", // fallback
    };
    
    let full_output_name = format!("{}{}", output_name, extension);
    
    // Get active panel path (where source files are)
    let dest_dir = if app.active_panel == crate::app::PanelSide::Left {
        app.left_panel.current_path.clone()
    } else {
        app.right_panel.current_path.clone()
    };
    let dest_path = dest_dir.join(&full_output_name);
    
    // Check if output file already exists
    if dest_path.exists() {
        app.show_error(format!("El archivo {} ya existe", full_output_name));
        return Ok(());
    }
    
    // Estimate total size
    let total_size = crate::archive::estimate_compressed_size(&sources).unwrap_or(0);
    
    // Check disk space
    let available_space = match fs2::available_space(&dest_dir) {
        Ok(space) => space,
        Err(_) => {
            // If we can't get space, just proceed anyway
            u64::MAX
        }
    };
    
    if available_space < total_size {
        let size_mb = total_size / (1024 * 1024);
        let avail_mb = available_space / (1024 * 1024);
        app.show_error(format!(
            "Espacio insuficiente. Necesitas {} MB, tienes {} MB",
            size_mb, avail_mb
        ));
        return Ok(());
    }
    
    // Show progress dialog
    app.dialog_state = Some(DialogState::Progress {
        message: format!("Comprimiendo {}...", full_output_name),
    });
    
    // Create progress channel
    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel();
    
    // Prepare compression options
    let opts = crate::archive::compressor::CompressionOptions {
        output_path: dest_path.clone(),
        format,
        level,
        password: if use_password && !password.is_empty() {
            Some(password.to_string())
        } else {
            None
        },
    };
    
    // Spawn compression task
    let sources_clone = sources.clone();
    let dest_path_clone = dest_path.clone();
    let task = tokio::task::spawn_blocking(move || {
        crate::archive::compress_archive(&sources_clone, opts, progress_tx)
    });
    
    // Use tokio::select! to handle both progress updates and cancellation
    let mut cancel_rx_clone = cancel_rx.clone();
    loop {
        tokio::select! {
            // Process progress updates
            progress_result = progress_rx.recv() => {
                match progress_result {
                    Some(progress) => {
                        if let Some(op) = &mut app.current_operation {
                            op.progress = progress;
                        }
                    }
                    None => {
                        // Progress channel closed, task finished
                        break;
                    }
                }
            }
            
            // Check for cancellation
            _ = cancel_rx_clone.changed() => {
                if *cancel_rx_clone.borrow() {
                    log::info!("Compression cancellation requested");
                    
                    // Abort the compression task
                    task.abort();
                    
                    // Small delay to let the task abort
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    
                    // Clean up partial archive if it exists
                    if dest_path_clone.exists() {
                        log::info!("Removing partial archive: {:?}", dest_path_clone);
                        let _ = tokio::fs::remove_file(&dest_path_clone).await;
                    }
                    
                    app.close_dialog();
                    app.show_error("Compresión cancelada por el usuario".to_string());
                    
                    return Err(anyhow::anyhow!("Compression cancelled by user"));
                }
            }
        }
    }
    
    // Wait for task to complete
    match task.await {
        Ok(Ok(())) => {
            log::info!("Compression completed successfully");
            app.close_dialog();
            
            // Refresh active panel (where archive was created)
            let active_panel = if app.active_panel == crate::app::PanelSide::Left {
                &mut app.left_panel
            } else {
                &mut app.right_panel
            };
            
            active_panel.refresh_entries()?;
            
            // Try to select the newly created archive
            if let Some(file_name) = dest_path.file_name() {
                let file_name_str = file_name.to_string_lossy().to_string();
                if let Some(idx) = active_panel.entries.iter().position(|e| e.name == file_name_str) {
                    active_panel.cursor = idx;
                }
            }
        }
        Ok(Err(e)) => {
            log::error!("Compression failed: {}", e);
            app.show_error(format!("Error al comprimir: {}", e));
        }
        Err(e) => {
            log::error!("Compression task failed: {}", e);
            app.show_error(format!("Error en tarea de compresión: {}", e));
        }
    }
    
    Ok(())
}

/// Handle extract confirm dialog (legacy path)
async fn handle_extract_confirm(
    app: &mut AppState,
    source: &Path,
    dest: &Path,
    format: crate::archive::formats::ArchiveFormat,
) -> Result<()> {
    let source = source.to_path_buf();
    let dest = dest.to_path_buf();
    
    app.close_dialog();
    
    // Create a dummy progress channel
    let (progress_tx, _progress_rx) = tokio::sync::mpsc::channel(100);
    
    // Get uncompressed size for disk space check
    let uncompressed_size = crate::archive::get_uncompressed_size(&source, format);
    
    // Extract archive
    let _ = crate::archive::extractor::extract_archive(
        &source,
        &dest,
        format,
        None,
        progress_tx,
        uncompressed_size,
    ).await;
    
    // Refresh both panels
    app.left_panel.refresh_entries()?;
    app.right_panel.refresh_entries()?;
    
    Ok(())
}

/// Start extraction operation
fn start_extraction(
    app: &mut AppState,
    source: std::path::PathBuf,
    dest: std::path::PathBuf,
    format: crate::archive::formats::ArchiveFormat,
    message: &str,
) {
    // Get archive size for progress
    let archive_size = std::fs::metadata(&source)
        .map(|m| m.len())
        .unwrap_or(0);
    
    // Create extract operation with progress
    app.current_operation = Some(Operation::extract(
        source,
        dest,
        archive_size,
        1,
        format,
    ));
    
    // Show progress dialog
    app.dialog_state = Some(DialogState::Progress {
        message: message.to_string(),
    });
}

fn start_extraction_with_password(
    app: &mut AppState,
    source: std::path::PathBuf,
    dest: std::path::PathBuf,
    format: crate::archive::formats::ArchiveFormat,
    password: String,
    message: &str,
) {
    // Get archive size for progress
    let archive_size = std::fs::metadata(&source)
        .map(|m| m.len())
        .unwrap_or(0);
    
    // Create extract operation with password
    app.current_operation = Some(Operation::extract_with_password(
        source,
        dest,
        archive_size,
        1,
        format,
        password,
    ));
    
    // Show progress dialog
    app.dialog_state = Some(DialogState::Progress {
        message: message.to_string(),
    });
}

/// Execute an operation (copy, move, delete, extract)
async fn execute_operation(
    operation: Operation,
    progress_tx: mpsc::Sender<Progress>,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<()> {
    log::info!("Executing operation: {:?}", operation.operation_type);
    
    if operation.is_batch() {
        execute_batch_operation(operation, progress_tx, cancel_rx).await
    } else {
        execute_single_operation(operation, progress_tx, cancel_rx).await
    }
}

/// Execute a batch operation
async fn execute_batch_operation(
    operation: Operation,
    progress_tx: mpsc::Sender<Progress>,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<()> {
    let Some(batch_items) = &operation.batch_items else {
        return Err(anyhow::anyhow!("Batch operation without batch items"));
    };
    
    let total_files = batch_items.len();
    let mut files_done = 0;
    let mut bytes_done = 0;
    let bytes_total = operation.progress.bytes_total;
    
    // Create a dummy channel for individual operations
    let (dummy_tx, mut dummy_rx) = mpsc::channel::<Progress>(100);
    
    // Spawn a task to drain the dummy channel
    let drain_task = tokio::spawn(async move {
        while dummy_rx.recv().await.is_some() {}
    });
    
    for (source, destination, _name) in batch_items {
        // T955: Check if operation was cancelled
        if *cancel_rx.borrow() {
            log::info!("Batch operation cancelled by user");
            return Err(anyhow::anyhow!("Operation cancelled by user"));
        }
        
        let file_size = execute_single_batch_item(
            &operation.operation_type,
            source,
            destination,
            dummy_tx.clone(),
            cancel_rx.clone(),
        ).await?;
        
        bytes_done += file_size;
        files_done += 1;
        
        // Send progress update after each file
        let _ = progress_tx.send(Progress {
            bytes_done,
            bytes_total,
            files_done,
            files_total: total_files,
        }).await;
    }
    
    // Close dummy channel and wait for drain task
    drop(dummy_tx);
    let _ = drain_task.await;
    
    Ok(())
}

/// Execute a single item in a batch operation
async fn execute_single_batch_item(
    operation_type: &OperationType,
    source: &std::path::Path,
    destination: &std::path::Path,
    progress_tx: mpsc::Sender<Progress>,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<u64> {
    match operation_type {
        OperationType::Copy => {
            let metadata = tokio::fs::metadata(source).await?;
            let size = metadata.len();
            
            if metadata.is_dir() {
                crate::fs::operations::copy_dir_recursive(
                    source,
                    destination,
                    progress_tx,
                    size,
                    Some(cancel_rx),
                ).await?;
            } else {
                crate::fs::operations::copy_file_with_progress(
                    source,
                    destination,
                    progress_tx,
                    Some(cancel_rx),
                ).await?;
            }
            
            Ok(size)
        }
        OperationType::Move => {
            let metadata = tokio::fs::metadata(source).await?;
            let size = metadata.len();
            
            crate::fs::operations::move_item(
                source,
                destination,
                progress_tx,
                Some(cancel_rx),
            ).await?;
            
            Ok(size)
        }
        OperationType::Delete => {
            let metadata = tokio::fs::metadata(source).await?;
            let size = metadata.len();
            
            if metadata.is_dir() {
                crate::fs::operations::delete_dir_recursive(
                    source,
                    progress_tx,
                ).await?;
            } else {
                crate::fs::operations::delete_file(source).await?;
            }
            
            Ok(size)
        }
        OperationType::Extract => {
            Err(anyhow::anyhow!("Batch extraction not supported"))
        }
    }
}

/// Execute a single operation
async fn execute_single_operation(
    operation: Operation,
    progress_tx: mpsc::Sender<Progress>,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<()> {
    match operation.operation_type {
        OperationType::Copy => {
            let metadata = tokio::fs::metadata(&operation.source).await?;
            if metadata.is_dir() {
                let total_size = operation.progress.bytes_total;
                crate::fs::operations::copy_dir_recursive(
                    &operation.source,
                    &operation.destination,
                    progress_tx,
                    total_size,
                    Some(cancel_rx),
                ).await?;
            } else {
                crate::fs::operations::copy_file_with_progress(
                    &operation.source,
                    &operation.destination,
                    progress_tx,
                    Some(cancel_rx),
                ).await?;
            }
        }
        OperationType::Move => {
            crate::fs::operations::move_item(
                &operation.source,
                &operation.destination,
                progress_tx,
                Some(cancel_rx),
            ).await?;
        }
        OperationType::Delete => {
            let metadata = tokio::fs::metadata(&operation.source).await?;
            if metadata.is_dir() {
                crate::fs::operations::delete_dir_recursive(
                    &operation.source,
                    progress_tx,
                ).await?;
            } else {
                crate::fs::operations::delete_file(&operation.source).await?;
                // Send completion progress
                let _ = progress_tx.send(Progress {
                    bytes_done: 1,
                    bytes_total: 1,
                    files_done: 1,
                    files_total: 1,
                }).await;
            }
        }
        OperationType::Extract => {
            execute_extract_operation(operation, progress_tx, cancel_rx).await?;
        }
    }
    
    Ok(())
}

/// Execute extraction operation
async fn execute_extract_operation(
    operation: Operation,
    progress_tx: mpsc::Sender<Progress>,
    mut cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<()> {
    let Some(format) = operation.archive_format else {
        return Err(anyhow::anyhow!("Extract operation missing archive format"));
    };
    
    let source = operation.source.clone();
    let destination = operation.destination.clone();
    let destination_for_cleanup = destination.clone(); // For cleanup on cancel
    let password = operation.password.clone();
    let tx = progress_tx.clone();
    
    // Create an unbounded channel for progress from blocking context
    let (progress_sender, mut progress_receiver) = tokio::sync::mpsc::unbounded_channel::<Progress>();
    
    log::info!("Created unbounded channel for extraction progress forwarding");
    
    // Spawn forwarding task
    let forward_handle = tokio::spawn(async move {
        log::info!("Forwarding task started");
        let mut count = 0;
        while let Some(progress) = progress_receiver.recv().await {
            count += 1;
            log::debug!("Forwarding progress update #{}: files {}/{}", 
                count, progress.files_done, progress.files_total);
            
            match tx.send(progress).await {
                Ok(_) => log::debug!("Progress #{} forwarded successfully", count),
                Err(e) => log::error!("Failed to forward progress #{}: {}", count, e),
            }
        }
        log::info!("Forwarding task completed after {} messages", count);
    });
    
    // Extract in blocking task
    log::info!("Spawning blocking extraction task");
    let mut extract_handle = tokio::task::spawn_blocking(move || {
        crate::archive::extractor::extract_archive_unbounded(&source, &destination, format, password, progress_sender)
    });
    
    // T955: Wait for extraction to complete OR cancellation
    loop {
        tokio::select! {
            result = &mut extract_handle => {
                // Extraction completed (or failed)
                let extract_result = result?;
                
                // Wait for all progress messages to be forwarded
                let _ = forward_handle.await;
                
                extract_result?;
                
                return Ok(());
            }
            _ = cancel_rx.changed() => {
                // User requested cancellation
                if *cancel_rx.borrow() {
                    log::info!("Extraction cancelled by user, aborting task");
                    extract_handle.abort();
                    
                    // Wait a bit for cleanup
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    
                    // Try to remove partial extraction
                    if destination_for_cleanup.exists() {
                        log::info!("Removing partial extraction: {:?}", destination_for_cleanup);
                        let _ = tokio::fs::remove_dir_all(&destination_for_cleanup).await;
                    }
                    
                    return Err(anyhow::anyhow!("Extraction cancelled by user"));
                }
                // If false alarm, continue loop
            }
        }
    }
}
