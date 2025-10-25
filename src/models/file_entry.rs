// File entry data structure
use std::fs::Permissions;
use std::path::PathBuf;
use std::time::SystemTime;
use std::fmt;
use humansize::{format_size, DECIMAL};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryType {
    File,
    Dir,
    Symlink,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub entry_type: EntryType,
    pub size: u64,
    pub modified: SystemTime,
    pub created: Option<SystemTime>,
    pub extension: Option<String>,
    pub permissions: Permissions,
    pub path: PathBuf,
    
    // Windows file attributes (FILE_ATTRIBUTE_*)
    #[cfg(windows)]
    pub file_attributes: Option<u32>,
}

impl FileEntry {
    #[cfg(windows)]
    pub fn new(
        name: String,
        entry_type: EntryType,
        size: u64,
        modified: SystemTime,
        created: Option<SystemTime>,
        extension: Option<String>,
        permissions: Permissions,
        path: PathBuf,
        file_attributes: Option<u32>,
    ) -> Self {
        Self {
            name,
            entry_type,
            size,
            modified,
            created,
            extension,
            permissions,
            path,
            file_attributes,
        }
    }
    
    #[cfg(not(windows))]
    pub fn new(
        name: String,
        entry_type: EntryType,
        size: u64,
        modified: SystemTime,
        created: Option<SystemTime>,
        extension: Option<String>,
        permissions: Permissions,
        path: PathBuf,
    ) -> Self {
        Self {
            name,
            entry_type,
            size,
            modified,
            created,
            extension,
            permissions,
            path,
        }
    }

    pub fn is_dir(&self) -> bool {
        matches!(self.entry_type, EntryType::Dir)
    }

    pub fn is_file(&self) -> bool {
        matches!(self.entry_type, EntryType::File)
    }

    pub fn is_symlink(&self) -> bool {
        matches!(self.entry_type, EntryType::Symlink)
    }

    /// Extract file extension from filename
    /// Returns None for directories, dotfiles without extension, or files without extension
    /// Handles multi-part extensions like .tar.gz
    pub fn extract_extension(name: &str, is_dir: bool) -> Option<String> {
        if is_dir {
            return None;
        }

        // Handle dotfiles (e.g., .gitignore, .bashrc)
        if name.starts_with('.') && !name[1..].contains('.') {
            return None;
        }

        // Find the last dot
        if let Some(dot_pos) = name.rfind('.') {
            let ext = &name[dot_pos + 1..];
            
            // Check for multi-part extensions like .tar.gz
            if let Some(prev_dot_pos) = name[..dot_pos].rfind('.') {
                let prev_ext = &name[prev_dot_pos + 1..dot_pos];
                // Known multi-part extensions
                if matches!(prev_ext, "tar" | "backup" | "test") {
                    return Some(format!("{}.{}", prev_ext, ext));
                }
            }
            
            Some(ext.to_string())
        } else {
            None
        }
    }
}

impl fmt::Display for FileEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size_str = if self.is_dir() {
            "<DIR>".to_string()
        } else {
            format_size(self.size, DECIMAL)
        };

        let type_indicator = match self.entry_type {
            EntryType::Dir => "/",
            EntryType::Symlink => "@",
            EntryType::File => "",
        };

        write!(f, "{}{} ({})", self.name, type_indicator, size_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_extension_standard() {
        assert_eq!(
            FileEntry::extract_extension("file.txt", false),
            Some("txt".to_string())
        );
        assert_eq!(
            FileEntry::extract_extension("document.rs", false),
            Some("rs".to_string())
        );
        assert_eq!(
            FileEntry::extract_extension("data.json", false),
            Some("json".to_string())
        );
    }

    #[test]
    fn test_extract_extension_multipart() {
        assert_eq!(
            FileEntry::extract_extension("archive.tar.gz", false),
            Some("tar.gz".to_string())
        );
        assert_eq!(
            FileEntry::extract_extension("config.backup.json", false),
            Some("backup.json".to_string())
        );
        assert_eq!(
            FileEntry::extract_extension("main.test.rs", false),
            Some("test.rs".to_string())
        );
    }

    #[test]
    fn test_extract_extension_dotfiles() {
        // Dotfiles without extension
        assert_eq!(FileEntry::extract_extension(".gitignore", false), None);
        assert_eq!(FileEntry::extract_extension(".bashrc", false), None);
        
        // Dotfiles with extension
        assert_eq!(
            FileEntry::extract_extension(".vscode.json", false),
            Some("json".to_string())
        );
    }

    #[test]
    fn test_extract_extension_no_extension() {
        assert_eq!(FileEntry::extract_extension("README", false), None);
        assert_eq!(FileEntry::extract_extension("Makefile", false), None);
        assert_eq!(FileEntry::extract_extension("LICENSE", false), None);
    }

    #[test]
    fn test_extract_extension_directories() {
        // Directories should always return None
        assert_eq!(FileEntry::extract_extension("folder", true), None);
        assert_eq!(FileEntry::extract_extension("my.folder", true), None);
    }
}
