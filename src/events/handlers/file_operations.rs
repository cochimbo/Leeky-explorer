//! File operation handlers
//! 
//! This module contains handlers for file operations (copy, move, delete).

use anyhow::Result;
use std::path::PathBuf;

use crate::app::{AppState, DialogState};
use crate::models::operation::Operation;

/// Start copy operation with collision detection
pub fn start_copy_operation(app: &mut AppState) -> Result<()> {
    let dest_panel_path = app.inactive_panel().current_path.clone();
    
    // Get VFS references from both panels
    let source_vfs = app.active_panel().vfs.clone();
    let dest_vfs = app.inactive_panel().vfs.clone();
    
    // T951: Validate source files exist before starting operation
    // Note: For remote files, we'll check during the actual operation
    if source_vfs.is_none() {
        // Only validate local files upfront
        if app.has_selection() {
            let marked_paths = app.selection_state.get_marked(app.active_panel);
            for path in &marked_paths {
                if !path.exists() {
                    let file_name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    app.show_error(format!("Archivo no encontrado: {}", file_name));
                    return Ok(());
                }
            }
        } else {
            // Check single file exists
            if let Some(entry) = app.active_panel().selected_entry() {
                let source_path = app.active_panel().current_path.join(&entry.name);
                if !source_path.exists() {
                    app.show_error(format!("Archivo no encontrado: {}", entry.name));
                    return Ok(());
                }
            }
        }
    }
    
    // BUG-003/BUG-005 FIX: Check if copying to same directory
    // If so, skip collision check and generate suffix automatically
    let copying_to_same_dir = if app.has_selection() {
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        marked_paths.iter().any(|path| {
            if let Some(src_parent) = path.parent() {
                src_parent == dest_panel_path
            } else {
                false
            }
        })
    } else {
        app.active_panel().selected_entry()
            .map(|entry| {
                entry.path.parent()
                    .map(|p| p == dest_panel_path)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    };
    
    // T844: Check for collisions first (but skip if copying to same directory)
    if !copying_to_same_dir {
        // Get VFS references
        let source_vfs = app.active_panel().vfs.clone();
        let dest_vfs = app.inactive_panel().vfs.clone();
        
        let collision_result: Option<(String, Vec<PathBuf>)> = if app.has_selection() {
            // Check marked items for collisions
            let marked_paths = app.selection_state.get_marked(app.active_panel);
            let mut collision_info = None;
            
            for (i, path) in marked_paths.iter().enumerate() {
                let file_name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let dest = dest_panel_path.join(&file_name);
                
                // Check if destination exists using VFS if available
                let exists = if let Some(vfs) = &dest_vfs {
                    vfs.exists(&dest).unwrap_or(false)
                } else {
                    dest.exists()
                };
                
                if exists {
                    // Found collision - collect remaining files
                    let remaining: Vec<PathBuf> = marked_paths.iter().skip(i + 1).cloned().collect();
                    collision_info = Some((dest.to_string_lossy().to_string(), remaining));
                    break;
                }
            }
            collision_info
        } else {
            // Check single item
            app.active_panel().selected_entry().and_then(|entry| {
                let dest = dest_panel_path.join(&entry.name);
                
                // Check if destination exists using VFS if available
                let exists = if let Some(vfs) = &dest_vfs {
                    vfs.exists(&dest).unwrap_or(false)
                } else {
                    dest.exists()
                };
                
                if exists {
                    Some((dest.to_string_lossy().to_string(), Vec::new()))
                } else {
                    None
                }
            })
        };
        
        // If collision detected, show collision dialog
        if let Some((collision_path, remaining_files)) = collision_result {
            app.dialog_state = Some(DialogState::CollisionPrompt {
                file_path: collision_path,
                selected: 0,
                operation: crate::app::CollisionOperation::Copy,
                remaining_files,
                dest_path: dest_panel_path.clone(),
                source_vfs: source_vfs.clone(),
                dest_vfs: dest_vfs.clone(),
            });
            return Ok(());
        }
    }
    
    // No collision, proceed with operation
    // T574: Check if batch operation or single
    if app.has_selection() {
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        let count = marked_paths.len();
        
        // Calculate total size and create batch operation
        let mut total_bytes = 0u64;
        let mut operations = Vec::new();
        
        for path in &marked_paths {
            if let Ok(metadata) = std::fs::metadata(path) {
                total_bytes += metadata.len();
                if let Some(file_name) = path.file_name() {
                    let file_name = file_name.to_string_lossy().to_string();
                    let destination = dest_panel_path.join(&file_name);
                    
                    // BUG-003/BUG-005 FIX: Generate new name if copying to same directory
                    let final_destination = if copying_to_same_dir {
                        crate::fs::operations::generate_collision_free_name(&destination)
                    } else {
                        destination
                    };
                    
                    operations.push((path.clone(), final_destination, file_name));
                }
            }
        }
        
        // T953: Check available disk space before copying (only for local destinations)
        if dest_vfs.is_none() {
            if let Ok(available_space) = fs2::available_space(&dest_panel_path)
                && available_space < total_bytes {
                    let size_mb = total_bytes / (1024 * 1024);
                    let avail_mb = available_space / (1024 * 1024);
                    app.show_error(format!(
                        "Espacio insuficiente. Se necesitan {} MB, disponibles {} MB",
                        size_mb, avail_mb
                    ));
                    return Ok(());
                }
        }
        
        // T956: Warn about large operations
        let size_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        if size_gb > 1.0 || count > 1000 {
            let warning_msg = if size_gb > 1.0 && count > 1000 {
                format!("Operación grande: {:.1} GB y {} archivos. ¿Continuar?", size_gb, count)
            } else if size_gb > 1.0 {
                format!("Operación grande: {:.1} GB. ¿Continuar?", size_gb)
            } else {
                format!("Operación grande: {} archivos. ¿Continuar?", count)
            };
            
            app.show_error(warning_msg);
            // TODO: En el futuro, mostrar diálogo de confirmación en lugar de error
            // Por ahora, mostramos advertencia pero continuamos
        }
        
        let operation = Operation::copy_batch_vfs(operations, total_bytes, count, source_vfs.clone(), dest_vfs.clone());
        app.current_operation = Some(operation);
        
        app.dialog_state = Some(DialogState::Progress {
            message: format!("Copying {} items...", count),
        });
    } else {
        // Single file operation
        let source_panel = app.active_panel();
        
        if let Some(entry) = source_panel.selected_entry() {
            let source = entry.path.clone();
            let mut destination = dest_panel_path.join(&entry.name);
            let entry_name = entry.name.clone();
            let total_bytes = entry.size;
            
            // BUG-003 FIX: Check if copying to same directory
            if let (Some(src_parent), Some(dst_parent)) = (source.parent(), destination.parent())
                && src_parent == dst_parent {
                    // Copying to same directory - generate new name with suffix
                    destination = crate::fs::operations::generate_collision_free_name(&destination);
                }
            
            // T953: Check available disk space before copying (only for local destinations)
            if dest_vfs.is_none() {
                if let Ok(available_space) = fs2::available_space(&dest_panel_path)
                    && available_space < total_bytes {
                        let size_mb = total_bytes / (1024 * 1024);
                        let avail_mb = available_space / (1024 * 1024);
                        app.show_error(format!(
                            "Espacio insuficiente. Se necesitan {} MB, disponibles {} MB",
                            size_mb, avail_mb
                        ));
                        return Ok(());
                    }
            }
            
            let total_files = 1; // Single file or directory
            
            let operation = Operation::copy_vfs(source, destination, total_bytes, total_files, source_vfs, dest_vfs);
            app.current_operation = Some(operation);
            
            app.dialog_state = Some(DialogState::Progress {
                message: format!("Copying '{}'...", entry_name),
            });
        }
    }
    
    Ok(())
}

/// Start copy operation skipping collision check (user confirmed overwrite)
pub fn start_copy_operation_skip_check(app: &mut AppState) -> Result<()> {
    let dest_panel_path = app.inactive_panel().current_path.clone();
    
    // Get VFS references from both panels
    let source_vfs = app.active_panel().vfs.clone();
    let dest_vfs = app.inactive_panel().vfs.clone();
    
    // T574: Check if batch operation or single
    if app.has_selection() {
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        let count = marked_paths.len();
        
        // Calculate total size and create batch operation
        let mut total_bytes = 0u64;
        let mut operations = Vec::new();
        
        for path in &marked_paths {
            // Get size using VFS if available
            let size = if let Some(vfs) = &source_vfs {
                vfs.metadata(path).map(|m| m.size).unwrap_or(0)
            } else {
                std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            };
            
            total_bytes += size;
            if let Some(file_name) = path.file_name() {
                let file_name = file_name.to_string_lossy().to_string();
                let mut destination = dest_panel_path.join(&file_name);
                
                // BUG-003 FIX: Check if copying to same directory
                if let (Some(src_parent), Some(dst_parent)) = (path.parent(), destination.parent())
                    && src_parent == dst_parent {
                        // Copying to same directory - generate new name with suffix
                        destination = crate::fs::operations::generate_collision_free_name(&destination);
                    }
                
                operations.push((path.clone(), destination, file_name));
            }
        }
        
        let operation = Operation::copy_batch_vfs(operations, total_bytes, count, source_vfs, dest_vfs);
        app.current_operation = Some(operation);
        
        app.dialog_state = Some(DialogState::Progress {
            message: format!("Copying {} items...", count),
        });
    } else {
        // Single file operation
        let source_panel = app.active_panel();
        
        if let Some(entry) = source_panel.selected_entry() {
            let source = entry.path.clone();
            let mut destination = dest_panel_path.join(&entry.name);
            let entry_name = entry.name.clone();
            let total_bytes = entry.size;
            
            // BUG-003 FIX: Check if copying to same directory
            if let (Some(src_parent), Some(dst_parent)) = (source.parent(), destination.parent())
                && src_parent == dst_parent {
                    // Copying to same directory - generate new name with suffix
                    destination = crate::fs::operations::generate_collision_free_name(&destination);
                }
            
            let total_files = 1; // Single file or directory
            
            let operation = Operation::copy_vfs(source, destination, total_bytes, total_files, source_vfs, dest_vfs);
            app.current_operation = Some(operation);
            
            app.dialog_state = Some(DialogState::Progress {
                message: format!("Copying '{}'...", entry_name),
            });
        }
    }
    
    Ok(())
}

/// Start copy operation with automatic rename (generate suffix to avoid collision)
pub fn start_copy_operation_with_rename(app: &mut AppState) -> Result<()> {
    let dest_panel_path = app.inactive_panel().current_path.clone();
    
    // Get VFS references from both panels
    let source_vfs = app.active_panel().vfs.clone();
    let dest_vfs = app.inactive_panel().vfs.clone();
    
    // T574: Check if batch operation or single
    if app.has_selection() {
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        let count = marked_paths.len();
        
        // Calculate total size and create batch operation
        let mut total_bytes = 0u64;
        let mut operations = Vec::new();
        
        for path in &marked_paths {
            // Get size using VFS if available
            let size = if let Some(vfs) = &source_vfs {
                vfs.metadata(path).map(|m| m.size).unwrap_or(0)
            } else {
                std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            };
            
            total_bytes += size;
            if let Some(file_name) = path.file_name() {
                let file_name = file_name.to_string_lossy().to_string();
                let destination = dest_panel_path.join(&file_name);
                
                // Always generate collision-free name
                let final_destination = crate::fs::operations::generate_collision_free_name(&destination);
                
                operations.push((path.clone(), final_destination, file_name));
            }
        }
        
        let operation = Operation::copy_batch_vfs(operations, total_bytes, count, source_vfs, dest_vfs);
        app.current_operation = Some(operation);
        
        app.dialog_state = Some(DialogState::Progress {
            message: format!("Copying {} items...", count),
        });
    } else {
        // Single file operation
        let source_panel = app.active_panel();
        
        if let Some(entry) = source_panel.selected_entry() {
            let source = entry.path.clone();
            let destination = dest_panel_path.join(&entry.name);
            let entry_name = entry.name.clone();
            let total_bytes = entry.size;
            
            // Always generate collision-free name
            let final_destination = crate::fs::operations::generate_collision_free_name(&destination);
            
            let total_files = 1; // Single file or directory
            
            let operation = Operation::copy_vfs(source, final_destination, total_bytes, total_files, source_vfs, dest_vfs);
            app.current_operation = Some(operation);
            
            app.dialog_state = Some(DialogState::Progress {
                message: format!("Copying '{}'...", entry_name),
            });
        }
    }
    
    Ok(())
}

/// Start move operation with collision detection
pub fn start_move_operation(app: &mut AppState) -> Result<()> {
    let dest_panel_path = app.inactive_panel().current_path.clone();
    
    // Get VFS references from both panels
    let source_vfs = app.active_panel().vfs.clone();
    let dest_vfs = app.inactive_panel().vfs.clone();
    
    // T951: Validate source files exist before starting operation
    // Only validate for local files (when source VFS is None)
    if source_vfs.is_none() {
        if app.has_selection() {
            let marked_paths = app.selection_state.get_marked(app.active_panel);
            for path in &marked_paths {
                if !path.exists() {
                    let file_name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    app.show_error(format!("Archivo no encontrado: {}", file_name));
                    return Ok(());
                }
            }
        } else {
            // Check single file exists
            if let Some(entry) = app.active_panel().selected_entry() {
                let source_path = app.active_panel().current_path.join(&entry.name);
                if !source_path.exists() {
                    app.show_error(format!("Archivo no encontrado: {}", entry.name));
                    return Ok(());
                }
            }
        }
    }
    
    // T844: Check for collisions first
    // For remote dest, use VFS exists check
    let collision_result: Option<(String, Vec<PathBuf>)> = if app.has_selection() {
        // Check marked items for collisions
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        let mut collision_info = None;
        
        for (i, path) in marked_paths.iter().enumerate() {
            let file_name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let dest = dest_panel_path.join(&file_name);
            
            let exists = if let Some(vfs) = &dest_vfs {
                vfs.exists(&dest).unwrap_or(false)
            } else {
                dest.exists()
            };
            
            if exists {
                // Found collision - collect remaining files
                let remaining: Vec<PathBuf> = marked_paths.iter().skip(i + 1).cloned().collect();
                collision_info = Some((dest.to_string_lossy().to_string(), remaining));
                break;
            }
        }
        collision_info
    } else {
        // Check single item
        app.active_panel().selected_entry().and_then(|entry| {
            let dest = dest_panel_path.join(&entry.name);
            
            let exists = if let Some(vfs) = &dest_vfs {
                vfs.exists(&dest).unwrap_or(false)
            } else {
                dest.exists()
            };
            
            if exists {
                Some((dest.to_string_lossy().to_string(), Vec::new()))
            } else {
                None
            }
        })
    };
    
    // If collision detected, show collision dialog
    if let Some((collision_path, remaining_files)) = collision_result {
        app.dialog_state = Some(DialogState::CollisionPrompt {
            file_path: collision_path,
            selected: 0,
            operation: crate::app::CollisionOperation::Move,
            remaining_files,
            dest_path: dest_panel_path.clone(),
            source_vfs: source_vfs.clone(),
            dest_vfs: dest_vfs.clone(),
        });
        return Ok(());
    }
    
    // No collision, proceed with operation
    // T574: Check if batch operation or single
    if app.has_selection() {
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        let count = marked_paths.len();
        
        let mut total_bytes = 0u64;
        let mut operations = Vec::new();
        
        for path in &marked_paths {
            // Get size using VFS if available
            let size = if let Some(vfs) = &source_vfs {
                vfs.metadata(path).map(|m| m.size).unwrap_or(0)
            } else {
                std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            };
            
            total_bytes += size;
            if let Some(file_name) = path.file_name() {
                let file_name = file_name.to_string_lossy().to_string();
                let destination = dest_panel_path.join(&file_name);
                operations.push((path.clone(), destination, file_name));
            }
        }
        
        let operation = Operation::move_batch_vfs(operations, total_bytes, count, source_vfs, dest_vfs);
        app.current_operation = Some(operation);
        
        app.dialog_state = Some(DialogState::Progress {
            message: format!("Moving {} items...", count),
        });
    } else {
        // Single file operation
        let source_panel = app.active_panel();
        
        if let Some(entry) = source_panel.selected_entry() {
            let source = entry.path.clone();
            let destination = dest_panel_path.join(&entry.name);
            let entry_name = entry.name.clone();
            let total_bytes = entry.size;
            
            let total_files = 1; // Single file or directory
            
            let operation = Operation::move_vfs(source, destination, total_bytes, total_files, source_vfs, dest_vfs);
            app.current_operation = Some(operation);
            
            app.dialog_state = Some(DialogState::Progress {
                message: format!("Moving '{}'...", entry_name),
            });
        }
    }
    
    Ok(())
}

/// Start move operation skipping collision check (user confirmed overwrite)
pub fn start_move_operation_skip_check(app: &mut AppState) -> Result<()> {
    let dest_panel_path = app.inactive_panel().current_path.clone();
    
    // Get VFS references from both panels
    let source_vfs = app.active_panel().vfs.clone();
    let dest_vfs = app.inactive_panel().vfs.clone();
    
    // T574: Check if batch operation or single
    if app.has_selection() {
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        let count = marked_paths.len();
        
        let mut total_bytes = 0u64;
        let mut operations = Vec::new();
        
        for path in &marked_paths {
            // Get size using VFS if available
            let size = if let Some(vfs) = &source_vfs {
                vfs.metadata(path).map(|m| m.size).unwrap_or(0)
            } else {
                std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            };
            
            total_bytes += size;
            if let Some(file_name) = path.file_name() {
                let file_name = file_name.to_string_lossy().to_string();
                let destination = dest_panel_path.join(&file_name);
                operations.push((path.clone(), destination, file_name));
            }
        }
        
        let operation = Operation::move_batch_vfs(operations, total_bytes, count, source_vfs, dest_vfs);
        app.current_operation = Some(operation);
        
        app.dialog_state = Some(DialogState::Progress {
            message: format!("Moving {} items...", count),
        });
    } else {
        // Single file operation
        let source_panel = app.active_panel();
        
        if let Some(entry) = source_panel.selected_entry() {
            let source = entry.path.clone();
            let destination = dest_panel_path.join(&entry.name);
            let entry_name = entry.name.clone();
            let total_bytes = entry.size;
            
            let total_files = 1; // Single file or directory
            
            let operation = Operation::move_vfs(source, destination, total_bytes, total_files, source_vfs, dest_vfs);
            app.current_operation = Some(operation);
            
            app.dialog_state = Some(DialogState::Progress {
                message: format!("Moving '{}'...", entry_name),
            });
        }
    }
    
    Ok(())
}

/// Start move operation with automatic rename (generate suffix to avoid collision)
pub fn start_move_operation_with_rename(app: &mut AppState) -> Result<()> {
    let dest_panel_path = app.inactive_panel().current_path.clone();
    
    // Get VFS references from both panels
    let source_vfs = app.active_panel().vfs.clone();
    let dest_vfs = app.inactive_panel().vfs.clone();
    
    // T574: Check if batch operation or single
    if app.has_selection() {
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        let count = marked_paths.len();
        
        let mut total_bytes = 0u64;
        let mut operations = Vec::new();
        
        for path in &marked_paths {
            // Get size using VFS if available
            let size = if let Some(vfs) = &source_vfs {
                vfs.metadata(path).map(|m| m.size).unwrap_or(0)
            } else {
                std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            };
            
            total_bytes += size;
            if let Some(file_name) = path.file_name() {
                let file_name = file_name.to_string_lossy().to_string();
                let destination = dest_panel_path.join(&file_name);
                
                // Always generate collision-free name
                let final_destination = crate::fs::operations::generate_collision_free_name(&destination);
                
                operations.push((path.clone(), final_destination, file_name));
            }
        }
        
        let operation = Operation::move_batch_vfs(operations, total_bytes, count, source_vfs, dest_vfs);
        app.current_operation = Some(operation);
        
        app.dialog_state = Some(DialogState::Progress {
            message: format!("Moving {} items...", count),
        });
    } else {
        // Single file operation
        let source_panel = app.active_panel();
        
        if let Some(entry) = source_panel.selected_entry() {
            let source = entry.path.clone();
            let destination = dest_panel_path.join(&entry.name);
            let entry_name = entry.name.clone();
            let total_bytes = entry.size;
            
            // Always generate collision-free name
            let final_destination = crate::fs::operations::generate_collision_free_name(&destination);
            
            let total_files = 1; // Single file or directory
            
            let operation = Operation::move_vfs(source, final_destination, total_bytes, total_files, source_vfs, dest_vfs);
            app.current_operation = Some(operation);
            
            app.dialog_state = Some(DialogState::Progress {
                message: format!("Moving '{}'...", entry_name),
            });
        }
    }
    
    Ok(())
}

/// Start delete operation (with VFS support for remote files)
pub fn start_delete_operation(app: &mut AppState) -> Result<()> {
    // Get VFS reference if we're on a remote filesystem
    let vfs = app.active_panel().vfs.clone();
    
    // T951: Validate source files exist before starting operation
    // Only validate for local files (when VFS is None)
    if vfs.is_none() {
        if app.has_selection() {
            let marked_paths = app.selection_state.get_marked(app.active_panel);
            for path in &marked_paths {
                if !path.exists() {
                    let file_name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    app.show_error(format!("Archivo no encontrado: {}", file_name));
                    return Ok(());
                }
            }
        } else {
            // Check single file exists
            if let Some(entry) = app.active_panel().selected_entry()
                && !entry.path.exists() {
                    app.show_error(format!("Archivo no encontrado: {}", entry.name));
                    return Ok(());
                }
        }
    }
    // For remote files (vfs.is_some()), we skip the exists check
    // The VFS delete operation will handle errors if files don't exist
    
    // T574: Check if we have marked items for batch delete
    if app.has_selection() {
        let panel = app.active_panel();
        let marked_paths = app.selection_state.get_marked(app.active_panel);
        
        if !marked_paths.is_empty() {
            // Calculate total size and prepare batch items
            let mut total_bytes = 0u64;
            let mut batch_items = Vec::new();
            
            for marked_path in marked_paths {
                if let Some(entry) = panel.entries.iter().find(|e| e.path == *marked_path) {
                    total_bytes += entry.size;
                    batch_items.push((
                        entry.path.clone(),
                        entry.path.clone(), // For delete, destination is same as source
                        entry.name.clone()
                    ));
                }
            }
            
            let total_files = batch_items.len();
            let operation = Operation::delete_batch_vfs(batch_items, total_bytes, total_files, vfs);
            app.current_operation = Some(operation);
            
            app.dialog_state = Some(DialogState::Progress {
                message: format!("Deleting {} items...", total_files),
            });
            
            return Ok(());
        }
    }
    
    // Single file delete
    let panel = app.active_panel();
    
    if let Some(entry) = panel.selected_entry() {
        let source = entry.path.clone();
        let entry_name = entry.name.clone();
        let total_bytes = entry.size;
        
        let total_files = 1; // Single file or directory (estimate)
        
        let operation = Operation::delete_vfs(source, total_bytes, total_files, vfs);
        app.current_operation = Some(operation);
        
        // Show progress dialog
        app.dialog_state = Some(DialogState::Progress {
            message: format!("Deleting '{}'...", entry_name),
        });
    }
    
    Ok(())
}
