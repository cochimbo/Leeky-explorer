use std::fs::{self, File};
use std::path::Path;
use anyhow::{Result, Context};
use crate::models::operation::Progress;
use crate::archive::progress_reader::ProgressReader;
use super::super::utils::sanitize_path;

/// Extract 7Z archive with unbounded tokio channel.
/// Supports password-protected archives.
pub fn extract_7z_unbounded(
    archive_path: &Path,
    dest_path: &Path,
    password: Option<String>,
    tokio_tx: tokio::sync::mpsc::UnboundedSender<Progress>,
) -> Result<()> {
    log::info!("extract_7z_unbounded: Starting extraction from {:?} to {:?}", archive_path, dest_path);
    
    // Create a std::sync::mpsc channel for thread-safe progress updates
    let (std_tx, std_rx) = std::sync::mpsc::channel::<Progress>();
    
    // Spawn a bridge thread to forward from std::sync::mpsc to tokio::sync::mpsc
    std::thread::spawn(move || {
        while let Ok(progress) = std_rx.recv() {
            if tokio_tx.send(progress).is_err() {
                log::warn!("7Z progress bridge: tokio channel closed");
                break;
            }
        }
        log::debug!("7Z progress bridge thread finished");
    });
    
    let file = File::open(archive_path).context("Failed to open 7Z file")?;
    let len = file.metadata()?.len();
    
    // Convert password to the required format for sevenz-rust
    let password_bytes: sevenz_rust::Password = password
        .as_deref()
        .unwrap_or("")
        .into();
    
    let mut archive = sevenz_rust::SevenZReader::new(file, len, password_bytes)
        .context("Failed to read 7Z archive (wrong password?)")?;
    
    let total_files = archive.archive().files.len();
    
    // Calculate real total bytes
    let total_bytes: u64 = archive.archive().files.iter()
        .map(|f| f.size())
        .sum();
    
    let mut bytes_extracted = 0u64;
    let mut file_index = 0;
    
    log::info!("7Z archive has {} files, {} total bytes", total_files, total_bytes);
    
    // Extract all files
    archive.for_each_entries(|entry, reader| {
        let file_name = entry.name().to_string();
        
        // T832: Sanitize path
        let sanitized_path = sanitize_path(&file_name);
        let out_path = dest_path.join(&sanitized_path);
        
        // Send progress update before processing file
        log::debug!("7Z: Processing file {}/{}: {}", file_index, total_files, file_name);
        let _ = std_tx.send(Progress {
            bytes_done: bytes_extracted,
            bytes_total: total_bytes,
            files_done: file_index,
            files_total: total_files,
        });
        
        if entry.is_directory() {
            fs::create_dir_all(&out_path)?;
        } else {
            // Create parent directories
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Extract file with progress tracking
            let mut out_file = File::create(&out_path)?;
            
            // Wrap reader with progress tracking
            let mut progress_reader = ProgressReader::new(
                reader,
                std_tx.clone(),
                file_index,
                total_files,
                bytes_extracted,
                total_bytes,
            );
            
            let bytes_copied = std::io::copy(&mut progress_reader, &mut out_file)?;
            log::debug!("7Z: File {} extracted ({} bytes)", file_name, bytes_copied);
            
            bytes_extracted += entry.size();
        }
        
        file_index += 1;
        Ok(true) // Continue extraction
    })?;
    
    // Send final progress
    let _ = std_tx.send(Progress {
        bytes_done: bytes_extracted,
        bytes_total: total_bytes,
        files_done: total_files,
        files_total: total_files,
    });
    
    log::info!("7Z extraction completed: {} bytes extracted", bytes_extracted);
    
    Ok(())
}

/// Extract 7Z archive with sync channel (simplified progress tracking).
pub fn extract_7z_sync(
    archive_path: &Path,
    dest_path: &Path,
    password: Option<String>,
    progress_tx: std::sync::mpsc::Sender<Progress>,
) -> Result<()> {
    let file = File::open(archive_path).context("Failed to open 7Z file")?;
    let len = file.metadata()?.len();
    
    // Convert password to the required format for sevenz-rust
    let password_bytes: sevenz_rust::Password = password
        .as_deref()
        .unwrap_or("")
        .into();
    
    let mut archive = sevenz_rust::SevenZReader::new(file, len, password_bytes)
        .context("Failed to read 7Z archive (wrong password?)")?;
    
    let total_files = archive.archive().files.len();
    let mut bytes_extracted = 0u64;
    let mut file_index = 0;
    
    // Extract all files
    archive.for_each_entries(|entry, reader| {
        let file_name = entry.name().to_string();
        
        // T832: Sanitize path
        let sanitized_path = sanitize_path(&file_name);
        let out_path = dest_path.join(&sanitized_path);
        
        // T835: Send progress update
        let _ = progress_tx.send(Progress {
            bytes_done: bytes_extracted,
            bytes_total: total_files as u64 * 1024,
            files_done: file_index,
            files_total: total_files,
        });
        
        if entry.is_directory() {
            fs::create_dir_all(&out_path)?;
        } else {
            // Create parent directories
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Extract file
            let mut out_file = File::create(&out_path)?;
            std::io::copy(reader, &mut out_file)?;
            bytes_extracted += entry.size();
        }
        
        file_index += 1;
        Ok(true) // Continue extraction
    })?;
    
    // Send final progress
    let _ = progress_tx.send(Progress {
        bytes_done: bytes_extracted,
        bytes_total: bytes_extracted,
        files_done: total_files,
        files_total: total_files,
    });
    
    Ok(())
}
