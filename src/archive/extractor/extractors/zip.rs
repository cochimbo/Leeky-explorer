use std::fs::{self, File};
use std::path::Path;
use anyhow::{Result, Context};
use crate::models::operation::Progress;
use crate::archive::progress_reader::ProgressReader;
use super::super::utils::sanitize_path;

/// Extract ZIP archive with unbounded tokio channel.
/// Supports password-protected archives and preserves Unix permissions.
pub fn extract_zip_unbounded(
    archive_path: &Path,
    dest_path: &Path,
    password: Option<String>,
    tokio_tx: tokio::sync::mpsc::UnboundedSender<Progress>,
) -> Result<()> {
    log::info!("extract_zip_unbounded: Starting extraction from {:?} to {:?}", archive_path, dest_path);
    
    // Create a std::sync::mpsc channel for thread-safe progress updates
    let (std_tx, std_rx) = std::sync::mpsc::channel::<Progress>();
    
    // Spawn a bridge thread to forward from std::sync::mpsc to tokio::sync::mpsc
    std::thread::spawn(move || {
        while let Ok(progress) = std_rx.recv() {
            if tokio_tx.send(progress).is_err() {
                log::warn!("Progress bridge: tokio channel closed");
                break;
            }
        }
        log::debug!("Progress bridge thread finished");
    });
    
    let file = File::open(archive_path).context("Failed to open ZIP file")?;
    let mut archive = zip::ZipArchive::new(file).context("Failed to read ZIP archive")?;
    
    let total_files = archive.len();
    let mut bytes_extracted = 0u64;
    
    // Calculate real total bytes by iterating files
    let mut total_bytes = 0u64;
    for i in 0..total_files {
        // Use by_index_decrypt if password provided, otherwise by_index
        let file_result = if let Some(ref pwd) = password {
            archive.by_index_decrypt(i, pwd.as_bytes())
        } else {
            archive.by_index(i)
        };
        
        if let Ok(file) = file_result {
            total_bytes += file.size();
        }
    }
    
    log::info!("ZIP archive has {} files, {} total bytes", total_files, total_bytes);
    
    for i in 0..total_files {
        // Use by_index_decrypt if password provided, otherwise by_index
        let file = if let Some(ref pwd) = password {
            archive.by_index_decrypt(i, pwd.as_bytes())
                .context("Failed to decrypt file - wrong password?")?
        } else {
            archive.by_index(i)?
        };
        let file_name = file.name().to_string();
        
        // T832: Sanitize path (convert absolute to relative)
        let sanitized_path = sanitize_path(&file_name);
        let out_path = dest_path.join(&sanitized_path);
        
        // Send progress update before processing file
        log::debug!("Processing file {}/{}: {}", i, total_files, file_name);
        let _ = std_tx.send(Progress {
            bytes_done: bytes_extracted,
            bytes_total: total_bytes,
            files_done: i,
            files_total: total_files,
        });
        
        if file.is_dir() {
            // T829: Create directory
            fs::create_dir_all(&out_path)?;
        } else {
            // T829: Create parent directories
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Get file size and unix mode before moving file
            let file_size = file.size();
            #[cfg(unix)]
            let unix_mode = file.unix_mode();
            
            // Extract file with real-time progress updates
            let mut out_file = File::create(&out_path)?;
            
            // Wrap reader with progress tracking
            let mut progress_reader = ProgressReader::new(
                file,
                std_tx.clone(),
                i,
                total_files,
                bytes_extracted,
                total_bytes,
            );
            
            let bytes_copied = std::io::copy(&mut progress_reader, &mut out_file)?;
            log::debug!("ZIP: File {} extracted ({} bytes)", file_name, bytes_copied);
            
            bytes_extracted += file_size;
            
            // T830: Preserve file permissions (Unix only)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = unix_mode {
                    let permissions = std::fs::Permissions::from_mode(mode);
                    fs::set_permissions(&out_path, permissions)?;
                }
            }
        }
    }
    
    // Send final progress
    let _ = std_tx.send(Progress {
        bytes_done: bytes_extracted,
        bytes_total: total_bytes,
        files_done: total_files,
        files_total: total_files,
    });
    
    log::info!("ZIP extraction completed: {} bytes extracted", bytes_extracted);
    
    Ok(())
}

/// Extract ZIP archive with sync channel (simplified progress tracking).
pub fn extract_zip_sync(
    archive_path: &Path,
    dest_path: &Path,
    password: Option<String>,
    progress_tx: std::sync::mpsc::Sender<Progress>,
) -> Result<()> {
    let file = File::open(archive_path).context("Failed to open ZIP file")?;
    let mut archive = zip::ZipArchive::new(file).context("Failed to read ZIP archive")?;
    
    let total_files = archive.len();
    let mut bytes_extracted = 0u64;
    let total_bytes = total_files as u64 * 1024; // Approximation
    
    for i in 0..total_files {
        // Use by_index_decrypt if password provided, otherwise by_index
        let mut file = if let Some(ref pwd) = password {
            archive.by_index_decrypt(i, pwd.as_bytes())
                .context("Failed to decrypt file - wrong password?")?
        } else {
            archive.by_index(i)?
        };
        let file_name = file.name().to_string();
        
        // T832: Sanitize path (convert absolute to relative)
        let sanitized_path = sanitize_path(&file_name);
        let out_path = dest_path.join(&sanitized_path);
        
        // T835: Send progress update
        let _ = progress_tx.send(Progress {
            bytes_done: bytes_extracted,
            bytes_total: total_bytes,
            files_done: i,
            files_total: total_files,
        });
        
        if file.is_dir() {
            // T829: Create directory
            fs::create_dir_all(&out_path)?;
        } else {
            // T829: Create parent directories
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Extract file
            let mut out_file = File::create(&out_path)?;
            std::io::copy(&mut file, &mut out_file)?;
            
            bytes_extracted += file.size();
            
            // T830: Preserve file permissions (Unix only)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    let permissions = std::fs::Permissions::from_mode(mode);
                    fs::set_permissions(&out_path, permissions)?;
                }
            }
        }
    }
    
    // Send final progress
    let _ = progress_tx.send(Progress {
        bytes_done: bytes_extracted,
        bytes_total: bytes_extracted,
        files_done: total_files,
        files_total: total_files,
    });
    
    Ok(())
}
