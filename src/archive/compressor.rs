// T901-T904: Archive compression module
use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc::UnboundedSender;
use walkdir::WalkDir;

use crate::models::operation::Progress;
use super::formats::ArchiveFormat;
use super::progress_reader::ProgressReader;

/// T903: Compression level for archives
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    Fast,    // 1
    Normal,  // 6
    Maximum, // 9
}

impl CompressionLevel {
    pub fn to_value(&self) -> u32 {
        match self {
            CompressionLevel::Fast => 1,
            CompressionLevel::Normal => 6,
            CompressionLevel::Maximum => 9,
        }
    }
}

/// T904: Options for compression
#[derive(Debug, Clone)]
pub struct CompressionOptions {
    pub format: ArchiveFormat,
    pub level: CompressionLevel,
    pub password: Option<String>,
    pub output_path: PathBuf,
}

/// T921-T922: Estimate compressed size based on heuristics
pub fn estimate_compressed_size(sources: &[PathBuf]) -> Result<u64> {
    let mut total_size = 0u64;
    
    for source in sources {
        if source.is_file() {
            if let Ok(metadata) = std::fs::metadata(source) {
                let size = metadata.len();
                // Heuristic: estimate compression based on file extension
                let ratio = if let Some(ext) = source.extension() {
                    let ext = ext.to_string_lossy().to_lowercase();
                    match ext.as_str() {
                        // Already compressed
                        "zip" | "7z" | "rar" | "gz" | "bz2" | "xz" | "jpg" | "jpeg" | "png" | "mp3" | "mp4" | "avi" | "mkv" => 1.0,
                        // Text files (good compression)
                        "txt" | "md" | "log" | "json" | "xml" | "html" | "css" | "js" | "py" | "rs" | "c" | "cpp" | "h" => 0.4,
                        // Default
                        _ => 0.7,
                    }
                } else {
                    0.7 // Default ratio
                };
                total_size += (size as f64 * ratio) as u64;
            }
        } else if source.is_dir() {
            for entry in WalkDir::new(source).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    if let Ok(metadata) = entry.metadata() {
                        // Use 0.6 as average ratio for directories
                        total_size += (metadata.len() as f64 * 0.6) as u64;
                    }
                }
            }
        }
    }
    
    Ok(total_size)
}

/// Get total size of all sources (for progress tracking)
fn get_total_size(sources: &[PathBuf]) -> Result<u64> {
    let mut total = 0u64;
    
    for source in sources {
        if source.is_file() {
            if let Ok(metadata) = std::fs::metadata(source) {
                total += metadata.len();
            }
        } else if source.is_dir() {
            for entry in WalkDir::new(source).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    if let Ok(metadata) = entry.metadata() {
                        total += metadata.len();
                    }
                }
            }
        }
    }
    
    Ok(total)
}

/// Helper: Count total files in sources
fn count_files(sources: &[PathBuf]) -> Result<usize> {
    let mut count = 0;
    
    for source in sources {
        if source.is_file() {
            count += 1;
        } else if source.is_dir() {
            for entry in WalkDir::new(source).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    count += 1;
                }
            }
        }
    }
    
    Ok(count)
}

/// T905-T910: Compress files to ZIP format with progress tracking
pub fn compress_zip(
    sources: &[PathBuf],
    dest: &Path,
    opts: CompressionOptions,
    progress_tx: UnboundedSender<Progress>,
) -> Result<()> {
    use zip::write::{FileOptions, ExtendedFileOptions};
    use zip::CompressionMethod;

    log::info!("Starting ZIP compression to {:?}", dest);
    
    // Create bridge channel for progress updates
    let (std_tx, std_rx) = std::sync::mpsc::channel::<Progress>();
    let progress_bridge = progress_tx.clone();
    
    std::thread::spawn(move || {
        while let Ok(progress) = std_rx.recv() {
            if progress_bridge.send(progress).is_err() {
                break;
            }
        }
    });

    // Calculate total size for progress
    let total_bytes = get_total_size(sources).context("Failed to calculate total size")?;
    let total_files = count_files(sources)?;
    
    let output_file = File::create(dest).context("Failed to create output file")?;
    let mut zip = zip::ZipWriter::new(output_file);
    
    // T906: Set compression method and level
    let compression_level = opts.level.to_value();
    let password_clone = opts.password.clone();
    
    // Helper to create file options (needed for each file)
    let create_file_options = || -> FileOptions<ExtendedFileOptions> {
        if let Some(ref pwd) = password_clone {
            FileOptions::<ExtendedFileOptions>::default()
                .compression_method(CompressionMethod::Deflated)
                .compression_level(Some(compression_level as i64))
                .with_aes_encryption(zip::AesMode::Aes256, pwd)
        } else {
            FileOptions::<ExtendedFileOptions>::default()
                .compression_method(CompressionMethod::Deflated)
                .compression_level(Some(compression_level as i64))
        }
    };
    
    let mut bytes_processed = 0u64;
    let mut file_index = 0usize;
    
    // Process each source
    for source in sources {
        if source.is_file() {
            let file_name = source.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file");
            
            log::debug!("Compressing file: {}", file_name);
            
            // Send progress update
            std_tx.send(Progress {
                bytes_done: bytes_processed,
                bytes_total: total_bytes,
                files_done: file_index,
                files_total: total_files,
            }).ok();
            
            // T909: Start file with metadata
            zip.start_file(file_name, create_file_options())
                .context("Failed to start file in ZIP")?;
            
            // Read and write with progress
            let mut file = File::open(source).context("Failed to open source file")?;
            let mut progress_reader = ProgressReader::new(
                &mut file,
                std_tx.clone(),
                file_index,
                total_files,
                bytes_processed,
                total_bytes,
            );
            
            std::io::copy(&mut progress_reader, &mut zip)
                .context("Failed to write file to ZIP")?;
            
            bytes_processed += std::fs::metadata(source)?.len();
            file_index += 1;
            
        } else if source.is_dir() {
            // T910: Add directory recursively
            let base_path = source.parent().unwrap_or(source);
            
            for entry in WalkDir::new(source).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                
                if path.is_file() {
                    let relative_path = path.strip_prefix(base_path)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();
                    
                    log::debug!("Compressing: {}", relative_path);
                    
                    // Send progress
                    std_tx.send(Progress {
                        bytes_done: bytes_processed,
                        bytes_total: total_bytes,
                        files_done: file_index,
                        files_total: total_files,
                    }).ok();
                    
                    zip.start_file(&relative_path, create_file_options())
                        .context("Failed to start file in ZIP")?;
                    
                    let mut file = File::open(path).context("Failed to open file")?;
                    let mut progress_reader = ProgressReader::new(
                        &mut file,
                        std_tx.clone(),
                        file_index,
                        total_files,
                        bytes_processed,
                        total_bytes,
                    );
                    
                    std::io::copy(&mut progress_reader, &mut zip)
                        .context("Failed to write file")?;
                    
                    bytes_processed += std::fs::metadata(path)?.len();
                    file_index += 1;
                }
            }
        }
    }
    
    zip.finish().context("Failed to finalize ZIP")?;
    
    // Send final progress
    std_tx.send(Progress {
        bytes_done: total_bytes,
        bytes_total: total_bytes,
        files_done: total_files,
        files_total: total_files,
    }).ok();
    
    log::info!("ZIP compression completed: {:?}", dest);
    Ok(())
}

/// T911-T916: Compress files to TAR format with optional compression
pub fn compress_tar(
    sources: &[PathBuf],
    dest: &Path,
    compression: ArchiveFormat,
    opts: CompressionOptions,
    progress_tx: UnboundedSender<Progress>,
) -> Result<()> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    
    log::info!("Starting TAR compression to {:?}", dest);
    
    // Create bridge channel
    let (std_tx, std_rx) = std::sync::mpsc::channel::<Progress>();
    let progress_bridge = progress_tx.clone();
    
    std::thread::spawn(move || {
        while let Ok(progress) = std_rx.recv() {
            if progress_bridge.send(progress).is_err() {
                break;
            }
        }
    });
    
    let total_bytes = get_total_size(sources)?;
    let total_files = count_files(sources)?;
    
    let output_file = File::create(dest).context("Failed to create output file")?;
    
    // T913: Wrap with compression encoder
    let compression_level = Compression::new(opts.level.to_value());
    let writer: Box<dyn Write> = match compression {
        ArchiveFormat::TarGz => {
            Box::new(GzEncoder::new(output_file, compression_level))
        }
        ArchiveFormat::TarBz2 => {
            use bzip2::write::BzEncoder;
            Box::new(BzEncoder::new(output_file, bzip2::Compression::new(opts.level.to_value())))
        }
        ArchiveFormat::TarXz => {
            use xz2::write::XzEncoder;
            Box::new(XzEncoder::new(output_file, opts.level.to_value()))
        }
        _ => Box::new(output_file), // Plain TAR
    };
    
    // T912: Create TAR builder
    let mut tar = tar::Builder::new(writer);
    
    let mut bytes_processed = 0u64;
    let mut file_index = 0usize;
    
    for source in sources {
        if source.is_file() {
            let file_name = source.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file");
            
            std_tx.send(Progress {
                bytes_done: bytes_processed,
                bytes_total: total_bytes,
                files_done: file_index,
                files_total: total_files,
            }).ok();
            
            // T914: Append file (preserves permissions automatically)
            tar.append_path_with_name(source, file_name)
                .context("Failed to append file to TAR")?;
            
            bytes_processed += std::fs::metadata(source)?.len();
            file_index += 1;
            
        } else if source.is_dir() {
            let base_path = source.parent().unwrap_or(source);
            
            for entry in WalkDir::new(source).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                let relative_path = path.strip_prefix(base_path).unwrap_or(path);
                
                std_tx.send(Progress {
                    bytes_done: bytes_processed,
                    bytes_total: total_bytes,
                    files_done: file_index,
                    files_total: total_files,
                }).ok();
                
                if path.is_file() {
                    // T914: Preserves Unix permissions
                    tar.append_path_with_name(path, relative_path)
                        .context("Failed to append file")?;
                    
                    bytes_processed += std::fs::metadata(path)?.len();
                    file_index += 1;
                    
                } else if path.is_dir() && path != source {
                    // Append directory entry
                    tar.append_dir(relative_path, path)
                        .context("Failed to append directory")?;
                }
                // T915: Symlinks are handled automatically by append_path_with_name
            }
        }
    }
    
    tar.finish().context("Failed to finalize TAR")?;
    
    std_tx.send(Progress {
        bytes_done: total_bytes,
        bytes_total: total_bytes,
        files_done: total_files,
        files_total: total_files,
    }).ok();
    
    log::info!("TAR compression completed");
    Ok(())
}

/// T917-T920: Compress files to 7Z format
pub fn compress_7z(
    _sources: &[PathBuf],
    _dest: &Path,
    _opts: CompressionOptions,
    _progress_tx: UnboundedSender<Progress>,
) -> Result<()> {
    log::warn!("7Z compression not yet fully implemented");
    
    // Note: sevenz-rust doesn't have a stable compression API yet
    // This is a placeholder for future implementation
    
    anyhow::bail!("7Z compression is not yet supported. Use ZIP or TAR.GZ instead.")
}

/// Main compression function that dispatches to format-specific functions
pub fn compress_archive(
    sources: &[PathBuf],
    opts: CompressionOptions,
    progress_tx: UnboundedSender<Progress>,
) -> Result<()> {
    log::info!("Starting compression with format: {:?}", opts.format);
    
    let dest = opts.output_path.clone();
    let format = opts.format;
    
    match format {
        ArchiveFormat::ZIP => {
            compress_zip(sources, &dest, opts, progress_tx)
        }
        ArchiveFormat::TAR => {
            compress_tar(sources, &dest, ArchiveFormat::TAR, opts, progress_tx)
        }
        ArchiveFormat::TarGz => {
            compress_tar(sources, &dest, ArchiveFormat::TarGz, opts, progress_tx)
        }
        ArchiveFormat::TarBz2 => {
            compress_tar(sources, &dest, ArchiveFormat::TarBz2, opts, progress_tx)
        }
        ArchiveFormat::TarXz => {
            compress_tar(sources, &dest, ArchiveFormat::TarXz, opts, progress_tx)
        }
        ArchiveFormat::SEVENZ => {
            compress_7z(sources, &dest, opts, progress_tx)
        }
        _ => {
            anyhow::bail!("Unsupported compression format: {:?}", format)
        }
    }
}
