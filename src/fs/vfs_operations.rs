// VFS-aware file operations (for mixed local/remote operations)
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::models::operation::Progress;
use crate::remote::{VirtualFileSystem, VfsEntry};

/// Copy file with VFS support (can handle local, remote, or mixed)
pub async fn copy_file_vfs(
    src: &Path,
    dst: &Path,
    src_vfs: Option<Arc<dyn VirtualFileSystem>>,
    dst_vfs: Option<Arc<dyn VirtualFileSystem>>,
    tx: mpsc::Sender<Progress>,
    cancel_rx: Option<tokio::sync::watch::Receiver<bool>>,
) -> Result<()> {
    log::info!("VFS copy: {:?} -> {:?} (src_remote={}, dst_remote={})", 
               src, dst, src_vfs.is_some(), dst_vfs.is_some());
    
    // If both are local (no VFS), use the optimized local copy with incremental progress
    if src_vfs.is_none() && dst_vfs.is_none() {
        log::info!("Both local: using copy_file_with_progress for incremental updates");
        return crate::fs::operations::copy_file_with_progress(src, dst, tx, cancel_rx).await;
    }
    
    // Get file size for progress tracking
    let total_size = if let Some(vfs) = &src_vfs {
        // Remote source
        let metadata = vfs.metadata(src)?;
        metadata.size
    } else {
        // Local source
        tokio::fs::metadata(src).await?.len()
    };
    
    // Read file content
    let content = if let Some(vfs) = &src_vfs {
        // Read from remote
        log::info!("Reading from remote VFS: {:?}", src);
        tokio::task::spawn_blocking({
            let vfs = vfs.clone();
            let src = src.to_path_buf();
            move || vfs.read_file(&src)
        }).await??
    } else {
        // Read from local
        log::info!("Reading from local filesystem: {:?}", src);
        tokio::fs::read(src).await?
    };
    
    // Check for cancellation
    if let Some(ref cancel_rx) = cancel_rx && *cancel_rx.borrow() {
        log::info!("Copy operation cancelled by user");
        return Err(anyhow::anyhow!("Operation cancelled by user"));
    }
    
    // Write file content
    if let Some(vfs) = &dst_vfs {
        // Write to remote
        log::info!("Writing to remote VFS: {:?}", dst);
        tokio::task::spawn_blocking({
            let vfs = vfs.clone();
            let dst = dst.to_path_buf();
            let content = content.clone();
            move || vfs.write_file(&dst, &content)
        }).await??;
    } else {
        // Write to local
        log::info!("Writing to local filesystem: {:?}", dst);
        // Create parent directory if needed
        if let Some(parent) = dst.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(dst, &content).await?;
    }
    
    // Send final progress
    let final_progress = Progress {
        bytes_done: total_size,
        bytes_total: total_size,
        files_done: 1,
        files_total: 1,
    };
    let _ = tx.send(final_progress).await;
    
    log::info!("VFS copy completed: {:?}", dst);
    Ok(())
}

/// Get metadata using VFS if available, otherwise local fs
pub async fn get_metadata_vfs(
    path: &Path,
    vfs: Option<Arc<dyn VirtualFileSystem>>,
) -> Result<(u64, bool)> {
    log::info!("[GET_METADATA_VFS] Getting metadata for: {}", path.display());
    log::info!("[GET_METADATA_VFS] VFS present: {}", vfs.is_some());
    
    if let Some(vfs) = vfs {
        // Remote filesystem
        log::info!("[GET_METADATA_VFS] Using VFS for remote metadata");
        let result = tokio::task::spawn_blocking({
            let vfs = vfs.clone();
            let path = path.to_path_buf();
            move || {
                log::info!("[GET_METADATA_VFS] Inside spawn_blocking, calling vfs.metadata");
                let result = vfs.metadata(&path);
                log::info!("[GET_METADATA_VFS] vfs.metadata result: {:?}", result);
                result
            }
        }).await?;
        
        match result {
            Ok(entry) => {
                log::info!("[GET_METADATA_VFS] Got metadata: size={}, is_dir={}", entry.size, entry.entry_type == crate::remote::VfsEntryType::Directory);
                Ok((entry.size, entry.entry_type == crate::remote::VfsEntryType::Directory))
            }
            Err(e) => {
                log::error!("[GET_METADATA_VFS] ERROR getting metadata: {}", e);
                Err(e)
            }
        }
    } else {
        // Local filesystem
        log::info!("[GET_METADATA_VFS] Using tokio::fs for local metadata");
        let result = tokio::fs::metadata(path).await;
        match result {
            Ok(metadata) => {
                log::info!("[GET_METADATA_VFS] Got local metadata: size={}, is_dir={}", metadata.len(), metadata.is_dir());
                Ok((metadata.len(), metadata.is_dir()))
            }
            Err(e) => {
                log::error!("[GET_METADATA_VFS] ERROR getting local metadata: {}", e);
                Err(e.into())
            }
        }
    }
}

/// Check if path exists using VFS if available
pub async fn exists_vfs(
    path: &Path,
    vfs: Option<Arc<dyn VirtualFileSystem>>,
) -> Result<bool> {
    if let Some(vfs) = vfs {
        // Remote filesystem
        tokio::task::spawn_blocking({
            let vfs = vfs.clone();
            let path = path.to_path_buf();
            move || vfs.exists(&path)
        }).await?
    } else {
        // Local filesystem
        Ok(path.exists())
    }
}

/// Create directory using VFS if available
pub async fn create_dir_vfs(
    path: &Path,
    vfs: Option<Arc<dyn VirtualFileSystem>>,
) -> Result<()> {
    if let Some(vfs) = vfs {
        // Remote filesystem
        tokio::task::spawn_blocking({
            let vfs = vfs.clone();
            let path = path.to_path_buf();
            move || vfs.create_dir(&path)
        }).await?
    } else {
        // Local filesystem
        tokio::fs::create_dir_all(path).await?;
        Ok(())
    }
}

/// Copy directory recursively with VFS support
#[async_recursion::async_recursion]
pub async fn copy_dir_recursive_vfs(
    src: &Path,
    dst: &Path,
    src_vfs: Option<Arc<dyn VirtualFileSystem>>,
    dst_vfs: Option<Arc<dyn VirtualFileSystem>>,
    tx: mpsc::Sender<Progress>,
    total_size: u64,
    cancel_rx: Option<tokio::sync::watch::Receiver<bool>>,
) -> Result<()> {
    log::info!("VFS copy directory: {:?} -> {:?}", src, dst);
    
    // If both are local (no VFS), use the optimized local copy with better progress
    if src_vfs.is_none() && dst_vfs.is_none() {
        log::info!("Both local: using copy_dir_recursive for better progress");
        return crate::fs::operations::copy_dir_recursive(src, dst, tx, total_size, cancel_rx).await;
    }
    
    // Create destination directory
    create_dir_vfs(dst, dst_vfs.clone()).await?;
    
    let mut bytes_copied = 0u64;
    let mut files_copied = 0usize;
    
    // Get entries from source
    let entries = if let Some(vfs) = &src_vfs {
        // Remote source
        tokio::task::spawn_blocking({
            let vfs = vfs.clone();
            let src = src.to_path_buf();
            move || vfs.list_dir(&src)
        }).await??
    } else {
        // Local source
        let mut entries = Vec::new();
        let mut dir_reader = tokio::fs::read_dir(src).await?;
        while let Some(entry) = dir_reader.next_entry().await? {
            let metadata = entry.metadata().await?;
            let name = entry.file_name().to_string_lossy().to_string();
            
            let entry_type = if metadata.is_dir() {
                crate::remote::VfsEntryType::Directory
            } else {
                crate::remote::VfsEntryType::File
            };
            
            entries.push(VfsEntry {
                name,
                path: entry.path(),
                entry_type,
                size: metadata.len(),
                modified: metadata.modified()?,
                permissions: 0,
            });
        }
        entries
    };
    
    // Process each entry
    for entry in entries {
        // Check for cancellation
        if let Some(ref cancel_rx) = cancel_rx && *cancel_rx.borrow() {
            log::info!("Copy directory operation cancelled by user");
            return Err(anyhow::anyhow!("Operation cancelled by user"));
        }
        
        let src_path = &entry.path;
        
        // Build destination path - normalize if dest is remote
        let dst_path = if dst_vfs.is_some() {
            // Remote destination - ensure Unix-style path
            let dst_str = dst.to_string_lossy();
            let normalized = if dst_str.ends_with('/') {
                format!("{}{}", dst_str, entry.name)
            } else {
                format!("{}/{}", dst_str, entry.name)
            };
            PathBuf::from(normalized)
        } else {
            // Local destination - use normal join
            dst.join(&entry.name)
        };
        
        if entry.entry_type == crate::remote::VfsEntryType::Directory {
            // Recursive copy
            copy_dir_recursive_vfs(
                src_path,
                &dst_path,
                src_vfs.clone(),
                dst_vfs.clone(),
                tx.clone(),
                total_size,
                cancel_rx.clone(),
            ).await?;
        } else {
            // Copy file
            let file_tx = tx.clone();
            copy_file_vfs(
                src_path,
                &dst_path,
                src_vfs.clone(),
                dst_vfs.clone(),
                file_tx,
                cancel_rx.clone(),
            ).await?;
            
            bytes_copied += entry.size;
            files_copied += 1;
            
            let progress = Progress {
                bytes_done: bytes_copied,
                bytes_total: total_size,
                files_done: files_copied,
                files_total: 0, // Unknown total
            };
            
            let _ = tx.send(progress).await;
        }
    }
    
    log::info!("VFS copy directory completed: {:?}", dst);
    Ok(())
}

/// Calculate directory size recursively with VFS support
#[async_recursion::async_recursion]
pub async fn calculate_size_vfs(
    path: &Path,
    vfs: Option<Arc<dyn VirtualFileSystem>>,
) -> Result<(u64, usize)> {
    let (size, is_dir) = get_metadata_vfs(path, vfs.clone()).await?;
    
    if !is_dir {
        return Ok((size, 1));
    }
    
    // Get directory entries
    let entries = if let Some(vfs) = &vfs {
        tokio::task::spawn_blocking({
            let vfs = vfs.clone();
            let path = path.to_path_buf();
            move || vfs.list_dir(&path)
        }).await??
    } else {
        let mut entries = Vec::new();
        let mut dir_reader = tokio::fs::read_dir(path).await?;
        while let Some(entry) = dir_reader.next_entry().await? {
            let metadata = entry.metadata().await?;
            let name = entry.file_name().to_string_lossy().to_string();
            
            let entry_type = if metadata.is_dir() {
                crate::remote::VfsEntryType::Directory
            } else {
                crate::remote::VfsEntryType::File
            };
            
            entries.push(VfsEntry {
                name,
                path: entry.path(),
                entry_type,
                size: metadata.len(),
                modified: metadata.modified()?,
                permissions: 0,
            });
        }
        entries
    };
    
    let mut total_size = 0u64;
    let mut total_files = 0usize;
    
    for entry in entries {
        if entry.entry_type == crate::remote::VfsEntryType::Directory {
            let (dir_size, dir_files) = calculate_size_vfs(&entry.path, vfs.clone()).await?;
            total_size += dir_size;
            total_files += dir_files;
        } else {
            total_size += entry.size;
            total_files += 1;
        }
    }
    
    Ok((total_size, total_files))
}

/// Delete file with VFS support
pub async fn delete_file_vfs(
    path: &Path,
    vfs: Option<Arc<dyn VirtualFileSystem>>,
    tx: mpsc::Sender<Progress>,
) -> Result<()> {
    if let Some(vfs) = vfs {
        // Remote delete
        log::info!("Deleting remote file: {:?}", path);
        tokio::task::spawn_blocking({
            let vfs = vfs.clone();
            let path = path.to_path_buf();
            move || vfs.delete(&path, false)
        }).await??;
    } else {
        // Local delete
        log::info!("Deleting local file: {:?}", path);
        tokio::fs::remove_file(path).await?;
    }
    
    // Send completion progress
    let _ = tx.send(Progress {
        bytes_done: 1,
        bytes_total: 1,
        files_done: 1,
        files_total: 1,
    }).await;
    
    Ok(())
}

/// Delete directory recursively with VFS support
pub async fn delete_dir_recursive_vfs(
    path: &Path,
    vfs: Option<Arc<dyn VirtualFileSystem>>,
    tx: mpsc::Sender<Progress>,
) -> Result<()> {
    if let Some(vfs) = vfs {
        // Remote delete (recursive)
        log::info!("Deleting remote directory recursively: {:?}", path);
        tokio::task::spawn_blocking({
            let vfs = vfs.clone();
            let path = path.to_path_buf();
            move || vfs.delete(&path, true)  // recursive = true
        }).await??;
        
        // Send completion progress
        let _ = tx.send(Progress {
            bytes_done: 1,
            bytes_total: 1,
            files_done: 1,
            files_total: 1,
        }).await;
    } else {
        // Local delete (use existing implementation)
        log::info!("Deleting local directory recursively: {:?}", path);
        crate::fs::operations::delete_dir_recursive(path, tx).await?;
    }
    
    Ok(())
}

/// Move/rename item with VFS support
/// For same-filesystem moves, this is just a rename
/// For cross-filesystem moves (local<->remote), this is copy+delete
pub async fn move_item_vfs(
    src: &Path,
    dst: &Path,
    src_vfs: Option<Arc<dyn VirtualFileSystem>>,
    dst_vfs: Option<Arc<dyn VirtualFileSystem>>,
    tx: mpsc::Sender<Progress>,
    cancel_rx: Option<tokio::sync::watch::Receiver<bool>>,
) -> Result<()> {
    log::info!("[VFS_MOVE] Starting move: {} -> {}", src.display(), dst.display());
    log::info!("[VFS_MOVE] src_vfs: {}, dst_vfs: {}", src_vfs.is_some(), dst_vfs.is_some());
    
    // If both are local (no VFS), use the optimized local move
    if src_vfs.is_none() && dst_vfs.is_none() {
        log::info!("Both local: using move_item for better progress");
        return crate::fs::operations::move_item(src, dst, tx, cancel_rx).await;
    }
    
    // Check if both source and dest are on same filesystem
    let same_fs = match (&src_vfs, &dst_vfs) {
        (None, None) => {
            log::info!("[VFS_MOVE] Both local filesystem");
            true  // Both local
        }
        (Some(s), Some(d)) => {
            // Both remote - check if same VFS instance
            let is_same = Arc::ptr_eq(s, d);
            log::info!("[VFS_MOVE] Both remote, same instance: {}", is_same);
            is_same
        }
        _ => {
            log::info!("[VFS_MOVE] Different filesystems (local<->remote)");
            false  // One local, one remote - different filesystems
        }
    };
    
    if same_fs {
        // Same filesystem - use rename (fast)
        log::info!("[VFS_MOVE] Using rename (same filesystem)");
        if let Some(vfs) = src_vfs {
            // Remote rename
            log::info!("[VFS_MOVE] Calling remote rename");
            log::info!("Renaming remote file: {:?} -> {:?}", src, dst);
            tokio::task::spawn_blocking({
                let vfs = vfs.clone();
                let src = src.to_path_buf();
                let dst = dst.to_path_buf();
                move || {
                    log::info!("[VFS_MOVE] Inside spawn_blocking, calling vfs.rename");
                    let result = vfs.rename(&src, &dst);
                    log::info!("[VFS_MOVE] vfs.rename result: {:?}", result);
                    result
                }
            }).await??;
            log::info!("[VFS_MOVE] Remote rename completed");
        } else {
            // Local rename
            log::info!("[VFS_MOVE] Calling tokio::fs::rename");
            log::info!("Renaming local file: {:?} -> {:?}", src, dst);
            let result = tokio::fs::rename(src, dst).await;
            log::info!("[VFS_MOVE] tokio::fs::rename result: {:?}", result);
            result?;
        }
        
        // Send completion progress
        let _ = tx.send(Progress {
            bytes_done: 1,
            bytes_total: 1,
            files_done: 1,
            files_total: 1,
        }).await;
        log::info!("[VFS_MOVE] Progress sent");
    } else {
        // Different filesystems - copy then delete
        log::info!("[VFS_MOVE] Using copy+delete (different filesystems)");
        log::info!("Moving across filesystems (copy+delete): {:?} -> {:?}", src, dst);
        
        // Get metadata to check if it's a directory
        let (size, is_dir) = get_metadata_vfs(src, src_vfs.clone()).await?;
        
        // First copy
        log::info!("[VFS_MOVE] Starting copy phase (is_dir: {})", is_dir);
        if is_dir {
            copy_dir_recursive_vfs(src, dst, src_vfs.clone(), dst_vfs, tx.clone(), size, cancel_rx).await?;
        } else {
            copy_file_vfs(src, dst, src_vfs.clone(), dst_vfs, tx.clone(), cancel_rx).await?;
        }
        log::info!("[VFS_MOVE] Copy completed, starting delete phase");
        
        // Then delete source
        if is_dir {
            delete_dir_recursive_vfs(src, src_vfs, tx).await?;
        } else {
            delete_file_vfs(src, src_vfs, tx).await?;
        }
        log::info!("[VFS_MOVE] Delete completed");
    }
    
    log::info!("[VFS_MOVE] Move operation completed successfully");
    Ok(())
}
