// T823-T833: Archive extraction logic
use anyhow::{Result, Context, bail};
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::Read;
use tokio::sync::mpsc::Sender;
use crate::models::operation::Progress;

/// T824: Unbounded channel version for use in blocking context
pub fn extract_archive_unbounded(
    archive_path: &Path,
    dest_path: &Path,
    format: super::formats::ArchiveFormat,
    password: Option<String>,
    progress_tx: tokio::sync::mpsc::UnboundedSender<Progress>,
) -> Result<()> {
    // T851d: Log extraction start
    log::info!("Starting extraction: {:?} -> {:?} (format: {:?})", 
               archive_path, dest_path, format);
    
    // Create destination directory if it doesn't exist
    fs::create_dir_all(dest_path).context("Failed to create destination directory")?;
    
    let result = match format {
        super::formats::ArchiveFormat::ZIP => extract_zip_unbounded(archive_path, dest_path, password, progress_tx),
        super::formats::ArchiveFormat::TAR => extract_tar_unbounded(archive_path, dest_path, None, progress_tx),
        super::formats::ArchiveFormat::TarGz => extract_tar_unbounded(archive_path, dest_path, Some(CompressionType::Gzip), progress_tx),
        super::formats::ArchiveFormat::TarBz2 => extract_tar_unbounded(archive_path, dest_path, Some(CompressionType::Bzip2), progress_tx),
        super::formats::ArchiveFormat::TarXz => extract_tar_unbounded(archive_path, dest_path, Some(CompressionType::Xz), progress_tx),
        super::formats::ArchiveFormat::SEVENZ => extract_7z_unbounded(archive_path, dest_path, password, progress_tx),
        super::formats::ArchiveFormat::RAR => Err(anyhow::anyhow!("RAR extraction not yet implemented")),
        super::formats::ArchiveFormat::UNKNOWN => bail!("Unknown archive format"),
    };
    
    // T851d: Log extraction result
    match &result {
        Ok(_) => log::info!("Extraction completed successfully: {:?}", archive_path),
        Err(e) => log::error!("Extraction failed for {:?}: {}", archive_path, e),
    }
    
    result
}

/// T824: Sync version of extract for use in blocking context
pub fn extract_archive_sync(
    archive_path: &Path,
    dest_path: &Path,
    format: super::formats::ArchiveFormat,
    password: Option<String>,
    progress_tx: std::sync::mpsc::Sender<Progress>,
) -> Result<()> {
    // Create destination directory if it doesn't exist
    fs::create_dir_all(dest_path).context("Failed to create destination directory")?;
    
    match format {
        super::formats::ArchiveFormat::ZIP => extract_zip_sync(archive_path, dest_path, password, progress_tx),
        super::formats::ArchiveFormat::TAR => extract_tar_sync(archive_path, dest_path, None, progress_tx),
        super::formats::ArchiveFormat::TarGz => extract_tar_sync(archive_path, dest_path, Some(CompressionType::Gzip), progress_tx),
        super::formats::ArchiveFormat::TarBz2 => extract_tar_sync(archive_path, dest_path, Some(CompressionType::Bzip2), progress_tx),
        super::formats::ArchiveFormat::TarXz => extract_tar_sync(archive_path, dest_path, Some(CompressionType::Xz), progress_tx),
        super::formats::ArchiveFormat::SEVENZ => extract_7z_sync(archive_path, dest_path, password, progress_tx),
        super::formats::ArchiveFormat::RAR => Err(anyhow::anyhow!("RAR extraction not yet implemented")),
        super::formats::ArchiveFormat::UNKNOWN => bail!("Unknown archive format"),
    }
}

/// T825: Extract ZIP archive (unbounded version)
fn extract_zip_unbounded(
    archive_path: &Path,
    dest_path: &Path,
    _password: Option<String>,
    progress_tx: tokio::sync::mpsc::UnboundedSender<Progress>,
) -> Result<()> {
    let file = File::open(archive_path).context("Failed to open ZIP file")?;
    let mut archive = zip::ZipArchive::new(file).context("Failed to read ZIP archive")?;
    
    let total_files = archive.len();
    let mut bytes_extracted = 0u64;
    let total_bytes = total_files as u64 * 1024; // Approximation
    
    for i in 0..total_files {
        let mut file = archive.by_index(i)?;
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

/// T825: Extract ZIP archive (sync version)
fn extract_zip_sync(
    archive_path: &Path,
    dest_path: &Path,
    _password: Option<String>,
    progress_tx: std::sync::mpsc::Sender<Progress>,
) -> Result<()> {
    let file = File::open(archive_path).context("Failed to open ZIP file")?;
    let mut archive = zip::ZipArchive::new(file).context("Failed to read ZIP archive")?;
    
    let total_files = archive.len();
    let mut bytes_extracted = 0u64;
    let total_bytes = total_files as u64 * 1024; // Approximation
    
    for i in 0..total_files {
        let mut file = archive.by_index(i)?;
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

fn extract_tar_unbounded(
    archive_path: &Path,
    dest_path: &Path,
    compression: Option<CompressionType>,
    progress_tx: tokio::sync::mpsc::UnboundedSender<Progress>,
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

fn extract_tar_sync(
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

fn extract_7z_unbounded(
    archive_path: &Path,
    dest_path: &Path,
    password: Option<String>,
    progress_tx: tokio::sync::mpsc::UnboundedSender<Progress>,
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

fn extract_7z_sync(
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


/// T834: Extraction progress information
#[derive(Debug, Clone)]
pub struct ExtractionProgress {
    pub current_file: String,
    pub file_index: usize,
    pub total_files: usize,
    pub bytes_extracted: u64,
}

enum CompressionType {
    Gzip,
    Bzip2,
    Xz,
}

use super::formats::ArchiveFormat;

/// T845: Check if there's enough disk space for extraction
pub fn check_disk_space(dest_path: &Path, required_bytes: u64) -> Result<()> {
    use fs2::available_space;
    
    // T847: ZIP bomb protection - limit to 10GB
    const MAX_EXTRACTION_SIZE: u64 = 10 * 1024 * 1024 * 1024; // 10GB
    
    if required_bytes > MAX_EXTRACTION_SIZE {
        let size_gb = required_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        bail!(
            "Archive too large: {:.2} GB exceeds safety limit of 10 GB. This may be a ZIP bomb.",
            size_gb
        );
    }
    
    let available = available_space(dest_path)
        .context("Failed to query available disk space")?;
    
    // Add 10% safety margin
    let required_with_margin = required_bytes + (required_bytes / 10);
    
    if available < required_with_margin {
        let available_mb = available / (1024 * 1024);
        let required_mb = required_with_margin / (1024 * 1024);
        bail!(
            "Insufficient disk space: {} MB available, {} MB required (including 10% margin)",
            available_mb,
            required_mb
        );
    }
    
    Ok(())
}

/// T824: Main extraction function
pub async fn extract_archive(
    archive_path: &Path,
    dest_path: &Path,
    format: ArchiveFormat,
    password: Option<String>,
    progress_tx: Sender<Progress>,
    uncompressed_size: Option<u64>,  // T845: For disk space check
) -> Result<()> {
    // T845: Check disk space before extraction
    if let Some(size) = uncompressed_size {
        check_disk_space(dest_path, size)?;
    }
    
    // Create destination directory if it doesn't exist
    fs::create_dir_all(dest_path).context("Failed to create destination directory")?;
    
    match format {
        ArchiveFormat::ZIP => extract_zip(archive_path, dest_path, password, progress_tx).await,
        ArchiveFormat::TAR => extract_tar(archive_path, dest_path, None, progress_tx).await,
        ArchiveFormat::TarGz => extract_tar(archive_path, dest_path, Some(CompressionType::Gzip), progress_tx).await,
        ArchiveFormat::TarBz2 => extract_tar(archive_path, dest_path, Some(CompressionType::Bzip2), progress_tx).await,
        ArchiveFormat::TarXz => extract_tar(archive_path, dest_path, Some(CompressionType::Xz), progress_tx).await,
        ArchiveFormat::SEVENZ => extract_7z(archive_path, dest_path, password, progress_tx).await,
        ArchiveFormat::RAR => extract_rar(archive_path, dest_path, password, progress_tx).await,
        ArchiveFormat::UNKNOWN => bail!("Unknown archive format"),
    }
}

/// T825: Extract ZIP archive
async fn extract_zip(
    archive_path: &Path,
    dest_path: &Path,
    _password: Option<String>,
    progress_tx: Sender<Progress>,
) -> Result<()> {
    // T846: Better error handling for corrupt archives
    let file = File::open(archive_path)
        .context("Failed to open ZIP file - file may be locked or inaccessible")?;
    
    let mut archive = zip::ZipArchive::new(file)
        .context("Failed to read ZIP archive - file may be corrupt or not a valid ZIP")?;
    
    let total_files = archive.len();
    let mut bytes_extracted = 0u64;
    let total_bytes = total_files as u64 * 1024; // Approximation
    
    // T847: ZIP bomb protection - track actual extracted size
    const MAX_EXTRACTION_SIZE: u64 = 10 * 1024 * 1024 * 1024; // 10GB
    
    for i in 0..total_files {
        // T846: Handle individual file extraction errors
        let mut file = archive.by_index(i)
            .with_context(|| format!("Failed to access file #{} in archive - archive may be corrupt", i))?;
        let file_name = file.name().to_string();
        
        // T832: Sanitize path (convert absolute to relative)
        let sanitized_path = sanitize_path(&file_name);
        let out_path = dest_path.join(&sanitized_path);
        
        // T850: Handle duplicate filenames - overwrite with warning
        if !file.is_dir() && out_path.exists() {
            log::warn!("Overwriting existing file from archive: {}", out_path.display());
        }
        
        // T835: Send progress update
        let _ = progress_tx.blocking_send(Progress {
            bytes_done: bytes_extracted,
            bytes_total: total_bytes,
            files_done: i,
            files_total: total_files,
        });
        
        if file.is_dir() {
            // T829: Create directory
            // T848: Handle permission errors gracefully
            if let Err(e) = fs::create_dir_all(&out_path) {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    log::warn!("Permission denied creating directory: {} - skipping", out_path.display());
                    continue;
                } else {
                    return Err(e).context(format!("Failed to create directory: {}", out_path.display()));
                }
            }
        } else {
            // T829: Create parent directories
            // T848: Handle permission errors gracefully
            if let Some(parent) = out_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        log::warn!("Permission denied creating parent directory: {} - skipping file", parent.display());
                        continue;
                    } else {
                        return Err(e).context(format!("Failed to create parent directory: {}", parent.display()));
                    }
                }
            }
            
            // T846: Extract file with better error handling
            // T848: Handle permission errors during file creation
            let mut out_file = match File::create(&out_path) {
                Ok(f) => f,
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                    log::warn!("Permission denied creating file: {} - skipping", out_path.display());
                    continue;
                }
                Err(e) => return Err(e).with_context(|| format!("Failed to create output file: {}", out_path.display())),
            };
            
            // T848: Handle permission errors during write
            if let Err(e) = std::io::copy(&mut file, &mut out_file) {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    log::warn!("Permission denied writing file: {} - skipping", sanitized_path.display());
                    continue;
                } else {
                    return Err(e).with_context(|| format!("Failed to extract '{}' - archive may be corrupt", sanitized_path.display()));
                }
            }
            
            bytes_extracted += file.size();
            
            // T847: ZIP bomb protection - check if we've exceeded limit
            if bytes_extracted > MAX_EXTRACTION_SIZE {
                bail!(
                    "Extraction stopped: exceeded 10 GB safety limit ({:.2} GB extracted). This may be a ZIP bomb.",
                    bytes_extracted as f64 / (1024.0 * 1024.0 * 1024.0)
                );
            }
            
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
    let _ = progress_tx.blocking_send(Progress {
        bytes_done: bytes_extracted,
        bytes_total: bytes_extracted,
        files_done: total_files,
        files_total: total_files,
    });
    
    Ok(())
}

/// T826: Extract TAR archive (with optional compression)
async fn extract_tar(
    archive_path: &Path,
    dest_path: &Path,
    compression: Option<CompressionType>,
    progress_tx: Sender<Progress>,
) -> Result<()> {
    // T846: Better error handling for corrupt archives
    let file = File::open(archive_path)
        .context("Failed to open TAR file - file may be locked or inaccessible")?;
    
    let mut archive: tar::Archive<Box<dyn Read + Send>> = match compression {
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
    
    // Count total entries
    let entries_vec: Vec<_> = archive.entries()?.collect();
    let total_files = entries_vec.len();
    let mut bytes_extracted = 0u64;
    
    // Re-open archive for extraction
    let file = File::open(archive_path)?;
    let mut archive: tar::Archive<Box<dyn Read + Send>> = match compression {
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
    
    for (i, entry_result) in archive.entries()?.enumerate() {
        let mut entry = entry_result?;
        let path = entry.path()?.to_path_buf();
        let file_name = path.to_string_lossy().to_string();
        
        // T832: Sanitize path
        let sanitized_path = sanitize_path(&file_name);
        let out_path = dest_path.join(&sanitized_path);
        
        // T835: Send progress
        let _ = progress_tx.blocking_send(Progress {
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
    
    Ok(())
}

/// T827: Extract 7Z archive
async fn extract_7z(
    archive_path: &Path,
    dest_path: &Path,
    password: Option<String>,
    _progress_tx: Sender<Progress>,
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
    
    let _total_files = archive.archive().files.len();
    let mut _bytes_extracted = 0u64;
    
    // Extract all files
    archive.for_each_entries(|entry, reader| {
        let file_name = entry.name().to_string();
        
        // T832: Sanitize path
        let sanitized_path = sanitize_path(&file_name);
        let out_path = dest_path.join(&sanitized_path);
        
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
            _bytes_extracted += entry.size();
        }
        
        Ok(true) // Continue extraction
    })?;
    
    Ok(())
}

/// T828: Extract RAR archive (placeholder)
async fn extract_rar(
    _archive_path: &Path,
    _dest_path: &Path,
    _password: Option<String>,
    _progress_tx: Sender<Progress>,
) -> Result<()> {
    bail!("RAR extraction is not yet supported. Please install libunrar.")
}

/// T832: Sanitize path to prevent directory traversal attacks
fn sanitize_path(path_str: &str) -> PathBuf {
    let path = Path::new(path_str);
    
    // Remove any absolute path components and ".." references
    path.components()
        .filter(|c| matches!(c, std::path::Component::Normal(_)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sanitize_path() {
        assert_eq!(
            sanitize_path("/etc/passwd"),
            PathBuf::from("etc/passwd")
        );
        
        assert_eq!(
            sanitize_path("../../../etc/passwd"),
            PathBuf::from("etc/passwd")
        );
        
        assert_eq!(
            sanitize_path("normal/path/file.txt"),
            PathBuf::from("normal/path/file.txt")
        );
    }
}
