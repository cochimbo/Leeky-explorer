//! Collision handling for file operations
//! 
//! This module handles file collisions when copying or moving files,
//! managing the dialog state and processing files based on user decisions.

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

use crate::app::{AppState, DialogState};
use crate::models::operation::Operation;

/// Process a single file with collision handling
pub fn process_single_file_operation(
    source: &PathBuf,
    dest_dir: &PathBuf,
    source_vfs: &Option<Arc<dyn crate::remote::vfs::VirtualFileSystem>>,
    dest_vfs: &Option<Arc<dyn crate::remote::vfs::VirtualFileSystem>>,
    operation: crate::app::CollisionOperation,
    allow_overwrite: bool,
    app: &mut AppState,
) -> Result<()> {
    let file_name = source.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    
    let mut destination = dest_dir.join(&file_name);
    
    // If not allowing overwrite, generate a new name
    if !allow_overwrite {
        destination = crate::fs::operations::generate_collision_free_name(&destination);
    }
    
    // Get file size for progress
    let size = if let Some(vfs) = source_vfs {
        vfs.metadata(source).map(|m| m.size).unwrap_or(0)
    } else {
        std::fs::metadata(source).map(|m| m.len()).unwrap_or(0)
    };
    
    // Create the operation
    let op = match operation {
        crate::app::CollisionOperation::Copy => {
            Operation::copy_vfs(
                source.clone(),
                destination,
                size,
                1, // total_files
                (*source_vfs).clone(),
                (*dest_vfs).clone(),
            )
        }
        crate::app::CollisionOperation::Move => {
            Operation::move_vfs(
                source.clone(),
                destination,
                size,
                1, // total_files
                (*source_vfs).clone(),
                (*dest_vfs).clone(),
            )
        }
        crate::app::CollisionOperation::Extract => return Ok(()), // Not implemented
    };
    
    app.current_operation = Some(op);
    app.dialog_state = Some(DialogState::Progress {
        message: format!("Processing '{}'...", file_name),
    });
    
    Ok(())
}

/// Continue processing remaining files with collision checks
pub fn continue_batch_operation(
    mut remaining_files: Vec<PathBuf>,
    dest_dir: PathBuf,
    source_vfs: Option<Arc<dyn crate::remote::vfs::VirtualFileSystem>>,
    dest_vfs: Option<Arc<dyn crate::remote::vfs::VirtualFileSystem>>,
    operation: crate::app::CollisionOperation,
    app: &mut AppState,
) -> Result<()> {
    // Check if next file has a collision
    while let Some(source) = remaining_files.first() {
        let file_name = source.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let dest = dest_dir.join(&file_name);
        
        // Check if destination exists using VFS if available
        let exists = if let Some(vfs) = &dest_vfs {
            vfs.exists(&dest).unwrap_or(false)
        } else {
            dest.exists()
        };
        
        if exists {
            // Found another collision - show dialog again
            let next_remaining: Vec<PathBuf> = remaining_files.iter().skip(1).cloned().collect();
            app.dialog_state = Some(DialogState::CollisionPrompt {
                file_path: dest.to_string_lossy().to_string(),
                selected: 0,
                operation: operation.clone(),
                remaining_files: next_remaining,
                dest_path: dest_dir,
                source_vfs,
                dest_vfs,
            });
            return Ok(());
        }
        
        // No collision - process this file and continue
        process_single_file_operation(source, &dest_dir, &source_vfs, &dest_vfs, operation.clone(), false, app)?;
        remaining_files.remove(0);
        
        // If there are more files, they will be processed after this operation completes
        // For now, just process one at a time
        return Ok(());
    }
    
    Ok(())
}

/// Process all files without collision checks (used for "Overwrite All")
pub fn process_batch_without_collision_check(
    files: Vec<PathBuf>,
    dest_dir: PathBuf,
    source_vfs: Option<Arc<dyn crate::remote::vfs::VirtualFileSystem>>,
    dest_vfs: Option<Arc<dyn crate::remote::vfs::VirtualFileSystem>>,
    operation: crate::app::CollisionOperation,
    app: &mut AppState,
) -> Result<()> {
    let mut total_bytes = 0u64;
    let mut operations = Vec::new();
    
    for path in &files {
        // Get size using VFS if available
        let size = if let Some(vfs) = &source_vfs {
            vfs.metadata(path).map(|m| m.size).unwrap_or(0)
        } else {
            std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
        };
        
        total_bytes += size;
        if let Some(file_name) = path.file_name() {
            let file_name = file_name.to_string_lossy().to_string();
            let destination = dest_dir.join(&file_name);
            
            operations.push((path.clone(), destination, file_name));
        }
    }
    
    let count = operations.len();
    
    // Create batch operation
    let op = match operation {
        crate::app::CollisionOperation::Copy => {
            Operation::copy_batch_vfs(operations, total_bytes, count, source_vfs, dest_vfs)
        }
        crate::app::CollisionOperation::Move => {
            Operation::move_batch_vfs(operations, total_bytes, count, source_vfs, dest_vfs)
        }
        crate::app::CollisionOperation::Extract => return Ok(()), // Not implemented
    };
    
    app.current_operation = Some(op);
    app.dialog_state = Some(DialogState::Progress {
        message: format!("Processing {} files...", count),
    });
    
    Ok(())
}
