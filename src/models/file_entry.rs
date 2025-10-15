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
    pub permissions: Permissions,
    pub path: PathBuf,
}

impl FileEntry {
    pub fn new(
        name: String,
        entry_type: EntryType,
        size: u64,
        modified: SystemTime,
        permissions: Permissions,
        path: PathBuf,
    ) -> Self {
        Self {
            name,
            entry_type,
            size,
            modified,
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
