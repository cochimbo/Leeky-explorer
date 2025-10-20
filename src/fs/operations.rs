// File operations
use anyhow::{Context, Result};
use async_recursion::async_recursion;
use std::path::Path;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;

use crate::models::operation::Progress;

/// Check if destination file exists (for collision detection)
pub async fn check_collision(dst: &Path) -> bool {
    fs::metadata(dst).await.is_ok()
}

/// Generate a new filename with suffix to avoid collision
pub fn generate_collision_free_name(dst: &Path) -> std::path::PathBuf {
    let parent = dst.parent().unwrap_or(Path::new("."));
    let stem = dst.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let extension = dst.extension().and_then(|s| s.to_str()).unwrap_or("");
    
    let mut counter = 1;
    loop {
        let new_name = if extension.is_empty() {
            format!("{}_{}", stem, counter)
        } else {
            format!("{}_{}.{}", stem, counter, extension)
        };
        
        let new_path = parent.join(new_name);
        if !std::path::Path::new(&new_path).exists() {
            return new_path;
        }
        counter += 1;
    }
}

pub async fn copy_file_with_progress(
    src: &Path,
    dst: &Path,
    tx: mpsc::Sender<Progress>,
    cancel_rx: Option<tokio::sync::watch::Receiver<bool>>,
) -> Result<()> {
    // T851d: Log file copy start
    log::info!("Copying file: {:?} -> {:?}", src, dst);
    
    let metadata = fs::metadata(src).await?;
    let total_size = metadata.len();
    let mut bytes_copied = 0u64;

    let mut reader = fs::File::open(src).await
        .with_context(|| format!("Failed to open source file: {}", src.display()))?;
    
    let mut writer = fs::File::create(dst).await
        .with_context(|| format!("Failed to create destination file: {}", dst.display()))?;

    let mut buffer = vec![0u8; 8192]; // 8KB buffer
    let dst_path = dst.to_path_buf(); // Clone for cleanup
    
    loop {
        // BUG-004 FIX: Check for cancellation
        if let Some(ref cancel_rx) = cancel_rx {
            if *cancel_rx.borrow() {
                log::info!("Copy operation cancelled by user");
                drop(writer);
                drop(reader);
                // Clean up partial file
                let _ = tokio::fs::remove_file(&dst_path).await;
                return Err(anyhow::anyhow!("Operation cancelled by user"));
            }
        }
        
        let n = reader.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        
        writer.write_all(&buffer[..n]).await?;
        bytes_copied += n as u64;
        
        let progress = Progress {
            bytes_done: bytes_copied,
            bytes_total: total_size,
            files_done: if bytes_copied >= total_size { 1 } else { 0 },
            files_total: 1,
        };
        
        // Send progress update
        if tx.send(progress).await.is_err() {
            // Receiver dropped, stop operation
            break;
        }
    }
    
    // Ensure file is flushed and closed
    writer.flush().await?;
    drop(writer);
    
    // T851d: Log successful copy
    log::info!("File copied successfully: {:?}", dst);
    
    // Send final progress update
    let final_progress = Progress {
        bytes_done: total_size,
        bytes_total: total_size,
        files_done: 1,
        files_total: 1,
    };
    let _ = tx.send(final_progress).await;
    
    Ok(())
}

pub async fn copy_dir_recursive(
    src: &Path,
    dst: &Path,
    tx: mpsc::Sender<Progress>,
    total_size: u64,
    cancel_rx: Option<tokio::sync::watch::Receiver<bool>>,
) -> Result<()> {
    // T851d: Log directory copy start
    log::info!("Copying directory recursively: {:?} -> {:?}", src, dst);
    
    fs::create_dir_all(dst).await?;
    
    let mut bytes_copied = 0u64;
    let mut files_copied = 0usize;
    let total_files = count_files(src).await?;
    
    copy_dir_recursive_impl(src, dst, &tx, &mut bytes_copied, &mut files_copied, total_size, total_files, cancel_rx).await?;
    
    // T851d: Log successful directory copy
    log::info!("Directory copied successfully: {:?}", dst);
    
    Ok(())
}

#[async_recursion::async_recursion]
async fn copy_dir_recursive_impl(
    src: &Path,
    dst: &Path,
    tx: &mpsc::Sender<Progress>,
    bytes_copied: &mut u64,
    files_copied: &mut usize,
    total_bytes: u64,
    total_files: usize,
    cancel_rx: Option<tokio::sync::watch::Receiver<bool>>,
) -> Result<()> {
    let mut entries = fs::read_dir(src).await?;
    
    while let Some(entry) = entries.next_entry().await? {
        // BUG-004 FIX: Check for cancellation
        if let Some(ref cancel_rx) = cancel_rx {
            if *cancel_rx.borrow() {
                log::info!("Copy directory operation cancelled by user");
                return Err(anyhow::anyhow!("Operation cancelled by user"));
            }
        }
        
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);
        
        let metadata = entry.metadata().await?;
        
        if metadata.is_dir() {
            fs::create_dir_all(&dst_path).await?;
            copy_dir_recursive_impl(&src_path, &dst_path, tx, bytes_copied, files_copied, total_bytes, total_files, cancel_rx.clone()).await?;
        } else {
            // Copy file
            let file_size = metadata.len();
            fs::copy(&src_path, &dst_path).await?;
            
            *bytes_copied += file_size;
            *files_copied += 1;
            
            let progress = Progress {
                bytes_done: *bytes_copied,
                bytes_total: total_bytes,
                files_done: *files_copied,
                files_total: total_files,
            };
            
            let _ = tx.send(progress).await;
        }
    }
    
    Ok(())
}

pub async fn move_item(
    src: &Path,
    dst: &Path,
    tx: mpsc::Sender<Progress>,
    cancel_rx: Option<tokio::sync::watch::Receiver<bool>>,
) -> Result<()> {
    // T851d: Log move start
    log::info!("Moving item: {:?} -> {:?}", src, dst);
    
    // Try simple rename first (fast if same filesystem)
    match fs::rename(src, dst).await {
        Ok(()) => {
            // T851d: Log successful move
            log::info!("Item moved successfully (rename): {:?}", dst);
            
            // Renamed successfully, report completion
            let size = get_total_size(dst).await.unwrap_or(0);
            let _ = tx.send(Progress {
                bytes_done: size,
                bytes_total: size,
                files_done: 1,
                files_total: 1,
            }).await;
            Ok(())
        }
        Err(_) => {
            // T851d: Log cross-device move
            log::info!("Cross-device move detected, using copy+delete: {:?}", src);
            
            // Cross-device move: copy then delete
            let total_size = get_total_size(src).await?;
            if src.is_dir() {
                copy_dir_recursive(src, dst, tx.clone(), total_size, cancel_rx.clone()).await?;
                fs::remove_dir_all(src).await?;
            } else {
                copy_file_with_progress(src, dst, tx.clone(), cancel_rx.clone()).await?;
                fs::remove_file(src).await?;
            }
            
            // T851d: Log successful cross-device move
            log::info!("Item moved successfully (copy+delete): {:?}", dst);
            
            Ok(())
        }
    }
}

#[async_recursion]
pub async fn get_total_size(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path).await?;
    
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    
    let mut total = 0u64;
    let mut entries = fs::read_dir(path).await?;
    
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let metadata = entry.metadata().await?;
        
        if metadata.is_dir() {
            total += get_total_size(&path).await?;
        } else {
            total += metadata.len();
        }
    }
    
    Ok(total)
}

#[async_recursion]
async fn count_files(path: &Path) -> Result<usize> {
    let metadata = fs::metadata(path).await?;
    
    if metadata.is_file() {
        return Ok(1);
    }
    
    let mut count = 0usize;
    let mut entries = fs::read_dir(path).await?;
    
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let metadata = entry.metadata().await?;
        
        if metadata.is_dir() {
            count += count_files(&path).await?;
        } else {
            count += 1;
        }
    }
    
    Ok(count)
}

// Delete operations
pub async fn delete_file(path: &Path) -> Result<()> {
    // T851d: Log file deletion
    log::info!("Deleting file: {:?}", path);
    
    fs::remove_file(path).await
        .with_context(|| format!("Failed to delete file: {}", path.display()))?;
    
    log::info!("File deleted successfully: {:?}", path);
    Ok(())
}

pub async fn delete_dir_recursive(
    path: &Path,
    tx: mpsc::Sender<Progress>,
) -> Result<()> {
    // T851d: Log directory deletion
    log::info!("Deleting directory recursively: {:?}", path);
    
    let total_files = count_files(path).await?;
    let mut files_deleted = 0usize;
    
    delete_dir_recursive_impl(path, &tx, &mut files_deleted, total_files).await?;
    
    // T851d: Log successful deletion
    log::info!("Directory deleted successfully: {:?}", path);
    
    // Send final progress
    let final_progress = Progress {
        bytes_done: files_deleted as u64,
        bytes_total: total_files as u64,
        files_done: files_deleted,
        files_total: total_files,
    };
    let _ = tx.send(final_progress).await;
    
    Ok(())
}

#[async_recursion]
async fn delete_dir_recursive_impl(
    path: &Path,
    tx: &mpsc::Sender<Progress>,
    files_deleted: &mut usize,
    total_files: usize,
) -> Result<()> {
    let mut entries = fs::read_dir(path).await?;
    
    while let Some(entry) = entries.next_entry().await? {
        let entry_path = entry.path();
        let metadata = entry.metadata().await?;
        
        if metadata.is_dir() {
            delete_dir_recursive_impl(&entry_path, tx, files_deleted, total_files).await?;
        } else {
            fs::remove_file(&entry_path).await?;
            *files_deleted += 1;
            
            let progress = Progress {
                bytes_done: *files_deleted as u64,
                bytes_total: total_files as u64,
                files_done: *files_deleted,
                files_total: total_files,
            };
            let _ = tx.send(progress.clone()).await;
        }
    }
    
    // Remove the now-empty directory
    fs::remove_dir(path).await?;
    
    Ok(())
}

pub async fn create_dir(path: &Path) -> Result<()> {
    fs::create_dir(path).await
        .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    Ok(())
}

pub async fn is_dir_empty(path: &Path) -> Result<bool> {
    let mut entries = fs::read_dir(path).await?;
    Ok(entries.next_entry().await?.is_none())
}
// TODO: Implement copy/move/delete for Phase 2
