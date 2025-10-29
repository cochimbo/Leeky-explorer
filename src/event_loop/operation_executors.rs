//! Operation executors for file operations
//!
//! Executes background file operations (copy, move, delete, extract) with
//! progress tracking and cancellation support.

use anyhow::Result;
use tokio::sync::mpsc;

use crate::models::operation::{Operation, OperationType, Progress};

/// Execute an operation (delegates to batch or single executor)
///
/// Main entry point for operation execution. Determines whether the operation
/// is a batch operation (multiple files) or single operation and delegates
/// to the appropriate executor.
///
/// # Arguments
///
/// * `operation` - Operation to execute
/// * `progress_tx` - Channel sender for progress updates
/// * `cancel_rx` - Watch channel receiver for cancellation signal
pub async fn execute_operation(
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

/// Execute a batch operation (multiple files)
///
/// Processes a batch of files one by one, sending progress updates after
/// each file completion. Supports cancellation between files.
///
/// # Arguments
///
/// * `operation` - Batch operation to execute
/// * `progress_tx` - Channel sender for progress updates
/// * `cancel_rx` - Watch channel receiver for cancellation signal
pub async fn execute_batch_operation(
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
    let source_vfs = operation.source_vfs.clone();
    let dest_vfs = operation.dest_vfs.clone();
    
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
            source_vfs.clone(),
            dest_vfs.clone(),
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
///
/// Performs the actual file operation (copy/move/delete) for one file in a batch.
/// Returns the file size for progress tracking.
///
/// # Arguments
///
/// * `operation_type` - Type of operation (Copy, Move, Delete)
/// * `source` - Source path
/// * `destination` - Destination path
/// * `progress_tx` - Channel sender for progress updates
/// * `cancel_rx` - Watch channel receiver for cancellation signal
/// * `source_vfs` - Optional VFS for source (if remote)
/// * `dest_vfs` - Optional VFS for destination (if remote)
///
/// # Returns
///
/// Size of the processed file in bytes
pub async fn execute_single_batch_item(
    operation_type: &OperationType,
    source: &std::path::Path,
    destination: &std::path::Path,
    progress_tx: mpsc::Sender<Progress>,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
    source_vfs: Option<std::sync::Arc<dyn crate::remote::VirtualFileSystem>>,
    dest_vfs: Option<std::sync::Arc<dyn crate::remote::VirtualFileSystem>>,
) -> Result<u64> {
    match operation_type {
        OperationType::Copy => {
            // Get metadata using VFS if available
            let (size, is_dir) = crate::fs::vfs_operations::get_metadata_vfs(source, source_vfs.clone()).await?;
            
            if is_dir {
                crate::fs::vfs_operations::copy_dir_recursive_vfs(
                    source,
                    destination,
                    source_vfs,
                    dest_vfs,
                    progress_tx,
                    size,
                    Some(cancel_rx),
                ).await?;
            } else {
                crate::fs::vfs_operations::copy_file_vfs(
                    source,
                    destination,
                    source_vfs,
                    dest_vfs,
                    progress_tx,
                    Some(cancel_rx),
                ).await?;
            }
            
            Ok(size)
        }
        OperationType::Move => {
            // Use VFS for metadata
            let (size, _) = crate::fs::vfs_operations::get_metadata_vfs(
                source,
                source_vfs.clone()
            ).await?;
            
            crate::fs::vfs_operations::move_item_vfs(
                source,
                destination,
                source_vfs,
                dest_vfs,
                progress_tx,
                Some(cancel_rx),
            ).await?;
            
            Ok(size)
        }
        OperationType::Delete => {
            // Use VFS for metadata check
            let (_, is_dir) = crate::fs::vfs_operations::get_metadata_vfs(
                source,
                source_vfs.clone()
            ).await?;
            
            let size = if let Some(vfs) = &source_vfs {
                // Remote file - get size from VFS
                let metadata = vfs.metadata(source)?;
                metadata.size
            } else {
                // Local file
                let metadata = tokio::fs::metadata(source).await?;
                metadata.len()
            };
            
            if is_dir {
                crate::fs::vfs_operations::delete_dir_recursive_vfs(
                    source,
                    source_vfs.clone(),
                    progress_tx,
                ).await?;
            } else {
                crate::fs::vfs_operations::delete_file_vfs(
                    source,
                    source_vfs.clone(),
                    progress_tx,
                ).await?;
            }
            
            Ok(size)
        }
        OperationType::Extract => {
            Err(anyhow::anyhow!("Batch extraction not supported"))
        }
    }
}

/// Execute a single operation (one file/directory)
///
/// Performs a single file operation (copy/move/delete/extract) with
/// progress tracking and cancellation support.
///
/// # Arguments
///
/// * `operation` - Operation to execute
/// * `progress_tx` - Channel sender for progress updates
/// * `cancel_rx` - Watch channel receiver for cancellation signal
pub async fn execute_single_operation(
    operation: Operation,
    progress_tx: mpsc::Sender<Progress>,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<()> {
    match operation.operation_type {
        OperationType::Copy => {
            // Get metadata using VFS if available
            let (_, is_dir) = crate::fs::vfs_operations::get_metadata_vfs(
                &operation.source,
                operation.source_vfs.clone()
            ).await?;
            
            if is_dir {
                let total_size = operation.progress.bytes_total;
                crate::fs::vfs_operations::copy_dir_recursive_vfs(
                    &operation.source,
                    &operation.destination,
                    operation.source_vfs,
                    operation.dest_vfs,
                    progress_tx,
                    total_size,
                    Some(cancel_rx),
                ).await?;
            } else {
                crate::fs::vfs_operations::copy_file_vfs(
                    &operation.source,
                    &operation.destination,
                    operation.source_vfs,
                    operation.dest_vfs,
                    progress_tx,
                    Some(cancel_rx),
                ).await?;
            }
        }
        OperationType::Move => {
            crate::fs::vfs_operations::move_item_vfs(
                &operation.source,
                &operation.destination,
                operation.source_vfs,
                operation.dest_vfs,
                progress_tx,
                Some(cancel_rx),
            ).await?;
        }
        OperationType::Delete => {
            // Use VFS for metadata check
            let (_, is_dir) = crate::fs::vfs_operations::get_metadata_vfs(
                &operation.source,
                operation.source_vfs.clone()
            ).await?;
            
            if is_dir {
                crate::fs::vfs_operations::delete_dir_recursive_vfs(
                    &operation.source,
                    operation.source_vfs,
                    progress_tx,
                ).await?;
            } else {
                crate::fs::vfs_operations::delete_file_vfs(
                    &operation.source,
                    operation.source_vfs,
                    progress_tx,
                ).await?;
            }
        }
        OperationType::Extract => {
            execute_extract_operation(operation, progress_tx, cancel_rx).await?;
        }
    }
    
    Ok(())
}

/// Execute extraction operation with progress and cancellation support
///
/// Extracts an archive to the destination directory. Supports password-protected
/// archives and handles cancellation with partial cleanup.
///
/// # Arguments
///
/// * `operation` - Extract operation to execute
/// * `progress_tx` - Channel sender for progress updates
/// * `cancel_rx` - Watch channel receiver for cancellation signal
pub async fn execute_extract_operation(
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
