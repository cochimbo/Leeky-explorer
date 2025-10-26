// Filesystem navigation
use crate::models::file_entry::{EntryType, FileEntry};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn read_dir(path: &Path) -> Result<Vec<FileEntry>> {
    let mut entries = Vec::new();

    let dir_entries = fs::read_dir(path)
        .with_context(|| format!("Failed to read directory: {}", path.display()))?;

    for entry_result in dir_entries {
        // Skip entries that cannot be read (permission denied, etc.)
        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue, // Skip entries we can't read
        };

        // Skip entries whose metadata cannot be read
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue, // Skip entries with inaccessible metadata (permission denied, etc.)
        };

        let file_name = entry
            .file_name()
            .to_string_lossy()
            .to_string();

        let entry_type = if metadata.is_symlink() {
            EntryType::Symlink
        } else if metadata.is_dir() {
            EntryType::Dir
        } else {
            EntryType::File
        };

        // Query creation time (available on Windows, may not be on all Unix systems)
        let created = metadata.created().ok();

        // Extract file extension using FileEntry helper
        let extension = FileEntry::extract_extension(&file_name, entry_type == EntryType::Dir);

        // Get Windows file attributes if on Windows
        #[cfg(windows)]
        let file_attributes = {
            use std::os::windows::fs::MetadataExt;
            Some(metadata.file_attributes())
        };

        let file_entry = FileEntry::new(
            file_name,
            entry_type,
            metadata.len(),
            metadata.modified().unwrap_or(std::time::UNIX_EPOCH),
            created,
            extension,
            metadata.permissions(),
            entry.path(),
            #[cfg(windows)]
            file_attributes,
        );

        entries.push(file_entry);
    }

    // Sort: directories first, then files, alphabetically
    entries.sort_by(|a, b| {
        use std::cmp::Ordering;
        match (&a.entry_type, &b.entry_type) {
            (EntryType::Dir, EntryType::Dir) => {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
            (EntryType::Dir, _) => Ordering::Less,
            (_, EntryType::Dir) => Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(entries)
}
