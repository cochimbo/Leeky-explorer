// T805: Archive extraction module
pub mod formats;
pub mod extractor;
pub mod password;

pub use formats::{ArchiveFormat, ArchiveEntry, detect_format, list_archive_contents};
pub use extractor::extract_archive;
pub use password::{prompt_password, PasswordDialog};

use std::path::Path;

/// T845: Get uncompressed size of an archive
pub fn get_uncompressed_size(path: &Path, format: ArchiveFormat) -> Option<u64> {
    // Try to get the uncompressed size from the archive
    // This is a best-effort - some formats don't reliably report this
    match list_archive_contents(path, format) {
        Ok(entries) => {
            let total: u64 = entries.iter().map(|e| e.size_uncompressed).sum();
            if total > 0 {
                Some(total)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

