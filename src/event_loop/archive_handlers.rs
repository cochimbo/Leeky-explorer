//! Archive operation handlers (extraction and compression)
//!
//! Handles dialogs and operations for archive extraction and compression,
//! including password protection, disk space checks, and progress tracking.

use std::path::Path;
use anyhow::Result;

use crate::app::{AppState, DialogState};
use crate::models::operation::Operation;

/// Handle extract options dialog selection
///
/// Processes the user's choice from the extraction options dialog:
/// - Option 0: Extract to current directory
/// - Option 1: Extract to new folder with archive name
///
/// Performs pre-flight checks including disk space and password protection.
///
/// # Arguments
///
/// * `app` - Application state
/// * `source` - Path to archive file
/// * `dest` - Base destination path
/// * `format` - Archive format
/// * `archive_name` - Name of archive (without extension)
/// * `selected` - Selected option index (0 or 1)
pub fn handle_extract_options(
    app: &mut AppState,
    source: &Path,
    dest: &Path,
    format: &crate::archive::formats::ArchiveFormat,
    archive_name: &str,
    selected: usize,
) -> Result<()> {
    let source = source.to_path_buf();
    let mut dest = dest.to_path_buf();
    let format = *format;
    let create_folder = selected == 1;
    
    // If option 1 selected, create a folder with archive name
    if create_folder {
        dest = dest.join(archive_name);
    }
    
    // T954: Check if destination already exists
    if dest.exists() {
        app.show_error(format!(
            "El directorio de destino ya existe: {}",
            dest.display()
        ));
        return Ok(());
    }
    
    // T953: Check available disk space before extraction
    let archive_size = std::fs::metadata(&source)
        .map(|m| m.len())
        .unwrap_or(0);
    
    // Estimate extracted size (typically 2-3x compressed size, use 3x to be safe)
    let estimated_extracted_size = archive_size * 3;
    
    // Get available space on destination
    if let Ok(available_space) = fs2::available_space(&dest)
        && available_space < estimated_extracted_size {
            let size_mb = estimated_extracted_size / (1024 * 1024);
            let avail_mb = available_space / (1024 * 1024);
            app.show_error(format!(
                "Espacio insuficiente. Se necesitan ~{} MB, disponibles {} MB",
                size_mb, avail_mb
            ));
            return Ok(());
        }
    
    // Check if archive is password-protected
    let is_encrypted = crate::archive::password::is_password_protected(&source)
        .unwrap_or(false);
    
    log::info!("Archive encryption check: {} - encrypted: {}", archive_name, is_encrypted);
    
    if is_encrypted {
        log::info!("Showing password input dialog for {}", archive_name);
        // Show password input dialog
        app.dialog_state = Some(DialogState::PasswordInput {
            prompt: format!("Enter password for {}:", archive_name),
            value: String::new(),
            show_password: false,
            archive_path: source,
            dest_path: dest,
            format,
        });
        log::info!("Password input dialog set successfully");
    } else {
        log::info!("No encryption detected, starting extraction directly");
        // No password needed, start extraction immediately
        start_extraction(app, source, dest, format, "Extrayendo archivo...");
    }
    
    Ok(())
}

/// Handle password input dialog submission
///
/// Processes password entered by user and starts extraction with that password.
///
/// # Arguments
///
/// * `app` - Application state
/// * `archive_path` - Path to password-protected archive
/// * `dest_path` - Extraction destination path
/// * `format` - Archive format
/// * `value` - Password entered by user
pub fn handle_password_input(
    app: &mut AppState,
    archive_path: &Path,
    dest_path: &Path,
    format: &crate::archive::formats::ArchiveFormat,
    value: &str,
) -> Result<()> {
    let source = archive_path.to_path_buf();
    let dest = dest_path.to_path_buf();
    let format = *format;
    let password = value.to_string();
    
    start_extraction_with_password(app, source, dest, format, password, "Extracting archive with password...");
    
    Ok(())
}

/// Handle compress options dialog submission
///
/// Processes compression settings and starts the compression operation.
/// Performs pre-flight checks including disk space and file existence.
///
/// # Arguments
///
/// * `app` - Application state
/// * `sources` - List of files/directories to compress
/// * `output_name` - Base name for output archive (without extension)
/// * `format` - Archive format (ZIP, TAR.GZ, 7Z, etc.)
/// * `level` - Compression level (Store, Fast, Normal, Best, Ultra)
/// * `use_password` - Whether to use password protection
/// * `password` - Password string (if use_password is true)
/// * `cancel_rx` - Cancellation watch channel receiver
#[allow(clippy::too_many_arguments)]
pub async fn handle_compress_options(
    app: &mut AppState,
    sources: Vec<std::path::PathBuf>,
    output_name: &str,
    format: crate::archive::formats::ArchiveFormat,
    level: crate::archive::compressor::CompressionLevel,
    use_password: bool,
    password: &str,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<()> {
    use crate::archive::formats::ArchiveFormat;
    
    // Add appropriate extension based on format
    let extension = match format {
        ArchiveFormat::ZIP => ".zip",
        ArchiveFormat::TarGz => ".tar.gz",
        ArchiveFormat::TarBz2 => ".tar.bz2",
        ArchiveFormat::TarXz => ".tar.xz",
        ArchiveFormat::TAR => ".tar",
        ArchiveFormat::SEVENZ => ".7z",
        _ => ".zip", // fallback
    };
    
    let full_output_name = format!("{}{}", output_name, extension);
    
    // Get active panel path (where source files are)
    let dest_dir = if app.active_panel == crate::app::PanelSide::Left {
        app.left_panel.current_path.clone()
    } else {
        app.right_panel.current_path.clone()
    };
    let dest_path = dest_dir.join(&full_output_name);
    
    // Check if output file already exists
    if dest_path.exists() {
        app.show_error(format!("El archivo {} ya existe", full_output_name));
        return Ok(());
    }
    
    // Estimate total size
    let total_size = crate::archive::estimate_compressed_size(&sources).unwrap_or(0);
    
    // Check disk space
    let available_space = match fs2::available_space(&dest_dir) {
        Ok(space) => space,
        Err(_) => {
            // If we can't get space, just proceed anyway
            u64::MAX
        }
    };
    
    if available_space < total_size {
        let size_mb = total_size / (1024 * 1024);
        let avail_mb = available_space / (1024 * 1024);
        app.show_error(format!(
            "Espacio insuficiente. Necesitas {} MB, tienes {} MB",
            size_mb, avail_mb
        ));
        return Ok(());
    }
    
    // Show progress dialog
    app.dialog_state = Some(DialogState::Progress {
        message: format!("Comprimiendo {}...", full_output_name),
    });
    
    // Create progress channel
    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel();
    
    // Prepare compression options
    let opts = crate::archive::compressor::CompressionOptions {
        output_path: dest_path.clone(),
        format,
        level,
        password: if use_password && !password.is_empty() {
            Some(password.to_string())
        } else {
            None
        },
    };
    
    // Spawn compression task
    let sources_clone = sources.clone();
    let dest_path_clone = dest_path.clone();
    let task = tokio::task::spawn_blocking(move || {
        crate::archive::compress_archive(&sources_clone, opts, progress_tx)
    });
    
    // Use tokio::select! to handle both progress updates and cancellation
    let mut cancel_rx_clone = cancel_rx.clone();
    loop {
        tokio::select! {
            // Process progress updates
            progress_result = progress_rx.recv() => {
                match progress_result {
                    Some(progress) => {
                        if let Some(op) = &mut app.current_operation {
                            op.progress = progress;
                        }
                    }
                    None => {
                        // Progress channel closed, task finished
                        break;
                    }
                }
            }
            
            // Check for cancellation
            _ = cancel_rx_clone.changed() => {
                if *cancel_rx_clone.borrow() {
                    log::info!("Compression cancellation requested");
                    
                    // Abort the compression task
                    task.abort();
                    
                    // Small delay to let the task abort
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    
                    // Clean up partial archive if it exists
                    if dest_path_clone.exists() {
                        log::info!("Removing partial archive: {:?}", dest_path_clone);
                        let _ = tokio::fs::remove_file(&dest_path_clone).await;
                    }
                    
                    app.close_dialog();
                    app.show_error("Compression cancelled by user".to_string());
                    
                    return Err(anyhow::anyhow!("Compression cancelled by user"));
                }
            }
        }
    }
    
    // Wait for task to complete
    match task.await {
        Ok(Ok(())) => {
            log::info!("Compression completed successfully");
            app.close_dialog();
            
            // Refresh active panel (where archive was created)
            let active_panel = if app.active_panel == crate::app::PanelSide::Left {
                &mut app.left_panel
            } else {
                &mut app.right_panel
            };
            
            active_panel.refresh_entries()?;
            
            // Try to select the newly created archive
            if let Some(file_name) = dest_path.file_name() {
                let file_name_str = file_name.to_string_lossy().to_string();
                if let Some(idx) = active_panel.entries.iter().position(|e| e.name == file_name_str) {
                    active_panel.cursor = idx;
                }
            }
        }
        Ok(Err(e)) => {
            log::error!("Compression failed: {}", e);
            app.show_error(format!("Error al comprimir: {}", e));
        }
        Err(e) => {
            log::error!("Compression task failed: {}", e);
            app.show_error(format!("Compression task error: {}", e));
        }
    }
    
    Ok(())
}

/// Handle extract confirm dialog (legacy path for simple extraction)
///
/// Legacy handler for direct extraction without options dialog.
/// Creates operation and starts extraction immediately.
///
/// # Arguments
///
/// * `app` - Application state
/// * `source` - Archive path
/// * `dest` - Destination path
/// * `format` - Archive format
pub async fn handle_extract_confirm(
    app: &mut AppState,
    source: &Path,
    dest: &Path,
    format: crate::archive::formats::ArchiveFormat,
) -> Result<()> {
    let source = source.to_path_buf();
    let dest = dest.to_path_buf();
    
    app.close_dialog();
    
    // Create a dummy progress channel
    let (progress_tx, _progress_rx) = tokio::sync::mpsc::channel(100);
    
    // Get uncompressed size for disk space check
    let uncompressed_size = crate::archive::get_uncompressed_size(&source, format);
    
    // Extract archive
    let _ = crate::archive::extractor::extract_archive(
        &source,
        &dest,
        format,
        None,
        progress_tx,
        uncompressed_size,
    ).await;
    
    // Refresh both panels
    app.left_panel.refresh_entries()?;
    app.right_panel.refresh_entries()?;
    
    Ok(())
}

/// Start extraction operation without password
///
/// Creates an extraction operation and queues it for execution.
/// Shows progress dialog and updates operation state.
///
/// # Arguments
///
/// * `app` - Application state
/// * `source` - Archive path
/// * `dest` - Extraction destination
/// * `format` - Archive format
/// * `message` - Message to show in progress dialog
pub fn start_extraction(
    app: &mut AppState,
    source: std::path::PathBuf,
    dest: std::path::PathBuf,
    format: crate::archive::formats::ArchiveFormat,
    message: &str,
) {
    // Get archive size for progress
    let archive_size = std::fs::metadata(&source)
        .map(|m| m.len())
        .unwrap_or(0);
    
    // Create extract operation with progress
    app.current_operation = Some(Operation::extract(
        source,
        dest,
        archive_size,
        1,
        format,
    ));
    
    // Show progress dialog
    app.dialog_state = Some(DialogState::Progress {
        message: message.to_string(),
    });
}

/// Start extraction operation with password
///
/// Creates a password-protected extraction operation and queues it for execution.
/// Shows progress dialog and updates operation state.
///
/// # Arguments
///
/// * `app` - Application state
/// * `source` - Archive path
/// * `dest` - Extraction destination
/// * `format` - Archive format
/// * `password` - Password for encrypted archive
/// * `message` - Message to show in progress dialog
pub fn start_extraction_with_password(
    app: &mut AppState,
    source: std::path::PathBuf,
    dest: std::path::PathBuf,
    format: crate::archive::formats::ArchiveFormat,
    password: String,
    message: &str,
) {
    // Get archive size for progress
    let archive_size = std::fs::metadata(&source)
        .map(|m| m.len())
        .unwrap_or(0);
    
    // Create extract operation with password
    app.current_operation = Some(Operation::extract_with_password(
        source,
        dest,
        archive_size,
        1,
        format,
        password,
    ));
    
    // Show progress dialog
    app.dialog_state = Some(DialogState::Progress {
        message: message.to_string(),
    });
}
