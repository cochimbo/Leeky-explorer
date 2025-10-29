// T823-T833: Archive extraction logic - modular structure
use anyhow::{Result, Context, bail};
use std::path::Path;
use std::fs;
use crate::models::operation::Progress;

// Re-export utilities for external use
pub use utils::{sanitize_path, check_disk_space, CompressionType};

// Internal modules
mod utils;
mod extractors {
    pub mod zip;
    pub mod tar;
    pub mod sevenz;
}
mod async_extractors;

use extractors::zip::{extract_zip_unbounded, extract_zip_sync};
use extractors::tar::{extract_tar_unbounded, extract_tar_sync};
use extractors::sevenz::{extract_7z_unbounded, extract_7z_sync};

// Re-export async dispatcher
pub use async_extractors::extract_archive;

/// Unbounded channel version for use in blocking context.
/// T824: Main dispatcher that delegates to format-specific extractors.
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

/// Sync version for use in blocking context with std::sync::mpsc channels.
/// T824: Main dispatcher that delegates to format-specific sync extractors.
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
