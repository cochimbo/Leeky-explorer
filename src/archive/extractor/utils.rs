use std::path::{Path, PathBuf};
use anyhow::{Result, bail, Context};

/// Compression types supported for TAR archives
#[derive(Debug, Clone, Copy)]
pub enum CompressionType {
    Gzip,
    Bzip2,
    Xz,
}

/// Sanitize a path to prevent directory traversal attacks.
/// Removes absolute path components and ".." references.
pub fn sanitize_path(path_str: &str) -> PathBuf {
    let path = Path::new(path_str);
    
    // Remove any absolute path components and ".." references
    path.components()
        .filter(|c| matches!(c, std::path::Component::Normal(_)))
        .collect()
}

/// Check if there's enough disk space for extraction.
/// T845: Includes ZIP bomb protection with 10GB limit.
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
