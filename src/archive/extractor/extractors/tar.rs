use std::fs::File;
use std::io::Read;
use std::path::Path;
use anyhow::{Result, Context};
use crate::models::operation::Progress;
use super::super::utils::{sanitize_path, CompressionType};

/// Extract TAR archive with unbounded tokio channel.
/// Supports gzip, bzip2, and xz compression.
pub fn extract_tar_unbounded(
    archive_path: &Path,
    dest_path: &Path,
    compression: Option<CompressionType>,
    tokio_tx: tokio::sync::mpsc::UnboundedSender<Progress>,
) -> Result<()> {
    log::info!("extract_tar_unbounded: Starting extraction from {:?} to {:?}", archive_path, dest_path);
    
    // Create a std::sync::mpsc channel for thread-safe progress updates
    let (std_tx, std_rx) = std::sync::mpsc::channel::<Progress>();
    
    // Spawn a bridge thread to forward from std::sync::mpsc to tokio::sync::mpsc
    std::thread::spawn(move || {
        while let Ok(progress) = std_rx.recv() {
            if tokio_tx.send(progress).is_err() {
                log::warn!("TAR progress bridge: tokio channel closed");
                break;
            }
        }
        log::debug!("TAR progress bridge thread finished");
    });
    
    let file = File::open(archive_path).context("Failed to open TAR file")?;
    
    // Create archive with appropriate decompressor
    let mut archive: tar::Archive<Box<dyn Read>> = match compression {
        None => {
            tar::Archive::new(Box::new(file))
        }
        Some(CompressionType::Gzip) => {
            let decoder = flate2::read::GzDecoder::new(file);
            tar::Archive::new(Box::new(decoder))
        }
        Some(CompressionType::Bzip2) => {
            let decoder = bzip2::read::BzDecoder::new(file);
            tar::Archive::new(Box::new(decoder))
        }
        Some(CompressionType::Xz) => {
            let decoder = xz2::read::XzDecoder::new(file);
            tar::Archive::new(Box::new(decoder))
        }
    };
    
    // Count total entries and calculate total bytes
    let entries_vec: Vec<_> = archive.entries()?.collect();
    let total_files = entries_vec.len();
    let total_bytes: u64 = entries_vec.iter()
        .filter_map(|e| e.as_ref().ok())
        .map(|e| e.header().size().unwrap_or(0))
        .sum();
    
    log::info!("TAR archive has {} files, {} total bytes", total_files, total_bytes);
    
    let mut bytes_extracted = 0u64;
    
    // Re-open archive for extraction
    let file = File::open(archive_path)?;
    let mut archive: tar::Archive<Box<dyn Read>> = match compression {
        None => tar::Archive::new(Box::new(file)),
        Some(CompressionType::Gzip) => {
            let decoder = flate2::read::GzDecoder::new(file);
            tar::Archive::new(Box::new(decoder))
        }
        Some(CompressionType::Bzip2) => {
            let decoder = bzip2::read::BzDecoder::new(file);
            tar::Archive::new(Box::new(decoder))
        }
        Some(CompressionType::Xz) => {
            let decoder = xz2::read::XzDecoder::new(file);
            tar::Archive::new(Box::new(decoder))
        }
    };
    
    // Extract files
    for (i, entry_result) in archive.entries()?.enumerate() {
        let mut entry = entry_result?;
        let path = entry.path()?.to_path_buf();
        let file_name = path.to_string_lossy().to_string();
        
        // T832: Sanitize path
        let sanitized_path = sanitize_path(&file_name);
        let out_path = dest_path.join(&sanitized_path);
        
        // Send progress update before processing file
        log::debug!("TAR: Processing file {}/{}: {}", i, total_files, file_name);
        let _ = std_tx.send(Progress {
            bytes_done: bytes_extracted,
            bytes_total: total_bytes,
            files_done: i,
            files_total: total_files,
        });
        
        // Get entry size before extraction
        let entry_size = entry.size();
        
        // T831: Handle symlinks (Unix only)
        #[cfg(unix)]
        {
            if entry.header().entry_type().is_symlink() {
                // Extract symlink
                entry.unpack(&out_path)?;
                bytes_extracted += entry_size;
                continue;
            }
        }
        
        // T829: Extract with directory creation
        entry.unpack(&out_path)?;
        bytes_extracted += entry_size;
    }
    
    // Send final progress
    let _ = std_tx.send(Progress {
        bytes_done: bytes_extracted,
        bytes_total: bytes_extracted,
        files_done: total_files,
        files_total: total_files,
    });
    
    Ok(())
}

/// Extract TAR archive with sync channel (simplified progress tracking).
pub fn extract_tar_sync(
    archive_path: &Path,
    dest_path: &Path,
    compression: Option<CompressionType>,
    progress_tx: std::sync::mpsc::Sender<Progress>,
) -> Result<()> {
    let file = File::open(archive_path).context("Failed to open TAR file")?;
    
    // Create archive with appropriate decompressor
    let mut archive: tar::Archive<Box<dyn Read>> = match compression {
        None => {
            tar::Archive::new(Box::new(file))
        }
        Some(CompressionType::Gzip) => {
            let decoder = flate2::read::GzDecoder::new(file);
            tar::Archive::new(Box::new(decoder))
        }
        Some(CompressionType::Bzip2) => {
            let decoder = bzip2::read::BzDecoder::new(file);
            tar::Archive::new(Box::new(decoder))
        }
        Some(CompressionType::Xz) => {
            let decoder = xz2::read::XzDecoder::new(file);
            tar::Archive::new(Box::new(decoder))
        }
    };
    
    // Count total entries first
    let entries_vec: Vec<_> = archive.entries()?.collect();
    let total_files = entries_vec.len();
    let mut bytes_extracted = 0u64;
    
    // Re-open archive for extraction
    let file = File::open(archive_path)?;
    let mut archive: tar::Archive<Box<dyn Read>> = match compression {
        None => tar::Archive::new(Box::new(file)),
        Some(CompressionType::Gzip) => {
            let decoder = flate2::read::GzDecoder::new(file);
            tar::Archive::new(Box::new(decoder))
        }
        Some(CompressionType::Bzip2) => {
            let decoder = bzip2::read::BzDecoder::new(file);
            tar::Archive::new(Box::new(decoder))
        }
        Some(CompressionType::Xz) => {
            let decoder = xz2::read::XzDecoder::new(file);
            tar::Archive::new(Box::new(decoder))
        }
    };
    
    // Extract files
    for (i, entry_result) in archive.entries()?.enumerate() {
        let mut entry = entry_result?;
        let path = entry.path()?.to_path_buf();
        let file_name = path.to_string_lossy().to_string();
        
        // T832: Sanitize path
        let sanitized_path = sanitize_path(&file_name);
        let out_path = dest_path.join(&sanitized_path);
        
        // T835: Send progress update
        let _ = progress_tx.send(Progress {
            bytes_done: bytes_extracted,
            bytes_total: total_files as u64 * 1024,
            files_done: i,
            files_total: total_files,
        });
        
        // T831: Handle symlinks (Unix only)
        #[cfg(unix)]
        {
            if entry.header().entry_type().is_symlink() {
                // Extract symlink
                entry.unpack(&out_path)?;
                continue;
            }
        }
        
        // T829: Extract with directory creation
        entry.unpack(&out_path)?;
        bytes_extracted += entry.size();
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
