// Leeky Explorer - Dual-pane TUI file explorer
pub mod models;
pub mod ui;
pub mod fs;
pub mod events;
pub mod config;
pub mod app;

use anyhow::Result;
use app::{AppState, DialogState, PanelSide};
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use events::handler::handle_key;
use events::keybindings::Action;
use models::operation::{Operation, OperationType, Progress};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::time::Duration;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize app state
    let mut app = AppState::new()?;
    
    // Load initial directory contents and store for filtering
    app.left_panel.refresh_entries()?;
    app.left_all_entries = app.left_panel.entries.clone();
    app.right_panel.refresh_entries()?;
    app.right_all_entries = app.right_panel.entries.clone();

    // Run the application
    let result = run_app(&mut terminal, &mut app).await;

    // T508: Save state on exit
    if result.is_ok() {
        let _ = app.save_state(); // Ignore errors during cleanup
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut AppState,
) -> Result<()> {
    let (progress_tx, mut progress_rx) = mpsc::channel::<Progress>(1000);
    let mut operation_task: Option<tokio::task::JoinHandle<Result<()>>> = None;
    
    loop {
        // Process ALL available progress updates (not just one)
        loop {
            match progress_rx.try_recv() {
                Ok(progress) => {
                    if let Some(ref mut op) = app.current_operation {
                        op.progress = progress;
                    }
                }
                Err(mpsc::error::TryRecvError::Empty) => {
                    break; // No more updates available right now
                }
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    // Channel closed, check task status
                    if let Some(task) = operation_task.take() {
                        match task.await {
                            Ok(Ok(())) => {
                                // T575: Clear marks after successful batch operation
                                app.selection_state.clear(app.active_panel);
                                app.close_dialog();
                                app.current_operation = None;
                                // Refresh and store entries for both panels
                                let _ = app.left_panel.refresh_entries();
                                app.left_all_entries = app.left_panel.entries.clone();
                                let _ = app.right_panel.refresh_entries();
                                app.right_all_entries = app.right_panel.entries.clone();
                            }
                            Ok(Err(e)) => {
                                app.show_error(format!("Operation failed: {}", e));
                                app.current_operation = None;
                            }
                            Err(e) => {
                                app.show_error(format!("Task error: {}", e));
                                app.current_operation = None;
                            }
                        }
                    }
                    break;
                }
            }
        }
        
        // Check if current operation completed
        if let Some(ref op) = app.current_operation {
            if op.progress.is_complete() && operation_task.is_some() {
                if let Some(task) = operation_task.take() {
                    match task.await {
                        Ok(Ok(())) => {
                            // T575: Clear marks after successful batch operation
                            app.selection_state.clear(app.active_panel);
                            app.close_dialog();
                            app.current_operation = None;
                            let _ = app.left_panel.refresh_entries();
                            let _ = app.right_panel.refresh_entries();
                        }
                        Ok(Err(e)) => {
                            app.show_error(format!("Operation failed: {}", e));
                            app.current_operation = None;
                        }
                        Err(e) => {
                            app.show_error(format!("Task error: {}", e));
                            app.current_operation = None;
                        }
                    }
                }
            }
        }
        
        // Start new operation if one is queued
        if app.current_operation.is_some() && operation_task.is_none() {
            let op = app.current_operation.clone().unwrap();
            let tx = progress_tx.clone();
            
            operation_task = Some(tokio::spawn(async move {
                execute_operation(op, tx).await
            }));
        }
        
        // Draw UI
        terminal.draw(|f| {
            let layout = ui::layout::create_layout(f.size());

            // Render header
            ui::render_header(f, app, layout.header);

            // Render panels
            let is_left_active = app.active_panel == PanelSide::Left;
            ui::panel_widget::render_panel(
                f,
                &app.left_panel,
                layout.left_panel,
                is_left_active,
                app.search_mode && is_left_active,
                &app.search_pattern,
                &app.selection_state,
                PanelSide::Left,
            );
            ui::panel_widget::render_panel(
                f,
                &app.right_panel,
                layout.right_panel,
                !is_left_active,
                app.search_mode && !is_left_active,
                &app.search_pattern,
                &app.selection_state,
                PanelSide::Right,
            );

            // Render footer
            ui::render_footer(f, layout.footer);
            
            // Render dialog if present with progress
            if let Some(dialog) = &app.dialog_state {
                match dialog {
                    DialogState::Progress { message } => {
                        if let Some(ref op) = app.current_operation {
                            ui::dialog::render_progress_with_bar(
                                f,
                                message,
                                &op.progress,
                                f.size()
                            );
                        } else {
                            ui::dialog::render_dialog(f, dialog, f.size());
                        }
                    }
                    _ => {
                        ui::dialog::render_dialog(f, dialog, f.size());
                    }
                }
            }
        })?;

        // Handle input with shorter timeout for better progress updates
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    let action = handle_key(app, key)?;
                    if action == Action::Quit {
                        // Cancel any running operation
                        if let Some(task) = operation_task.take() {
                            task.abort();
                        }
                        break;
                    }
                }
                Event::Resize(_, _) => {
                    // Terminal resized, will redraw on next iteration
                }
                _ => {}
            }
        }
    }

    Ok(())
}

async fn execute_operation(
    operation: Operation,
    progress_tx: mpsc::Sender<Progress>,
) -> Result<()> {
    // T574: Handle batch operations
    if operation.is_batch() {
        if let Some(batch_items) = &operation.batch_items {
            let total_files = batch_items.len();
            let mut files_done = 0;
            let mut bytes_done = 0;
            let bytes_total = operation.progress.bytes_total;
            
            // Create a dummy channel for individual operations to prevent progress conflicts
            let (dummy_tx, mut dummy_rx) = mpsc::channel::<Progress>(100);
            
            // Spawn a task to drain the dummy channel
            let drain_task = tokio::spawn(async move {
                while dummy_rx.recv().await.is_some() {
                    // Discard individual progress updates
                }
            });
            
            for (source, destination, _name) in batch_items {
                // Execute individual operation
                match operation.operation_type {
                    OperationType::Copy => {
                        let metadata = tokio::fs::metadata(source).await?;
                        if metadata.is_dir() {
                            fs::operations::copy_dir_recursive(
                                source,
                                destination,
                                dummy_tx.clone(),
                                metadata.len(),
                            ).await?;
                        } else {
                            fs::operations::copy_file_with_progress(
                                source,
                                destination,
                                dummy_tx.clone(),
                            ).await?;
                        }
                        bytes_done += metadata.len();
                    }
                    OperationType::Move => {
                        fs::operations::move_item(
                            source,
                            destination,
                            dummy_tx.clone(),
                        ).await?;
                        let metadata = tokio::fs::metadata(destination).await?;
                        bytes_done += metadata.len();
                    }
                    OperationType::Delete => {
                        let metadata = tokio::fs::metadata(source).await?;
                        let file_size = metadata.len();
                        
                        if metadata.is_dir() {
                            fs::operations::delete_dir_recursive(
                                source,
                                dummy_tx.clone(),
                            ).await?;
                        } else {
                            fs::operations::delete_file(source).await?;
                        }
                        bytes_done += file_size;
                    }
                }
                
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
        }
    } else {
        // Single file operation (original code)
        match operation.operation_type {
            OperationType::Copy => {
                let metadata = tokio::fs::metadata(&operation.source).await?;
                if metadata.is_dir() {
                    let total_size = operation.progress.bytes_total;
                    fs::operations::copy_dir_recursive(
                        &operation.source,
                        &operation.destination,
                        progress_tx,
                        total_size,
                    ).await?;
                } else {
                    fs::operations::copy_file_with_progress(
                        &operation.source,
                        &operation.destination,
                        progress_tx,
                    ).await?;
                }
            }
            OperationType::Move => {
                fs::operations::move_item(
                    &operation.source,
                    &operation.destination,
                    progress_tx,
                ).await?;
            }
            OperationType::Delete => {
                let metadata = tokio::fs::metadata(&operation.source).await?;
                if metadata.is_dir() {
                    fs::operations::delete_dir_recursive(
                        &operation.source,
                        progress_tx,
                    ).await?;
                } else {
                    fs::operations::delete_file(&operation.source).await?;
                    // Send completion progress
                    let _ = progress_tx.send(Progress {
                        bytes_done: 1,
                        bytes_total: 1,
                        files_done: 1,
                        files_total: 1,
                    }).await;
                }
            }
        }
    }
    
    Ok(())
}
