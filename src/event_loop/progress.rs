//! Progress tracking and update handling for file operations
//!
//! Handles progress channel updates from background operations and manages
//! operation completion/failure states.

use anyhow::Result;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::app::{AppState, DialogState};
use crate::models::operation::Progress;

/// Process progress updates from the progress channel
///
/// Continuously tries to receive progress updates from the channel and updates
/// the current operation's progress. Stops when the channel is empty or closed.
///
/// # Arguments
///
/// * `app` - Application state to update with progress
/// * `progress_rx` - Channel receiver for progress updates
/// * `operation_task` - Operation task handle (used when channel closes)
pub async fn process_progress_updates(
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
pub async fn handle_channel_closed(
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
                super::refresh_panels(app);
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
pub async fn check_operation_completion(
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
                        super::refresh_panels(app);
                        
                        // Check if there are pending batch operations to continue
                        if let Some(pending) = app.pending_batch.take() {
                            log::info!("Found pending batch with {} remaining files, continuing...", pending.remaining_files.len());
                            match crate::events::handlers::collision::continue_batch_operation(
                                pending.remaining_files,
                                pending.dest_path,
                                pending.source_vfs,
                                pending.dest_vfs,
                                pending.operation,
                                app,
                            ) {
                                Ok(_) => log::info!("Successfully started continuation of batch operation"),
                                Err(e) => log::error!("Failed to continue batch operation: {}", e),
                            }
                        } else {
                            log::info!("No pending batch operations to continue");
                        }
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
