// Virtual File System abstraction for local and remote filesystems
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Entry type in the virtual filesystem
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VfsEntryType {
    File,
    Directory,
    Symlink,
}

/// Represents a file or directory entry in the virtual filesystem
#[derive(Debug, Clone)]
pub struct VfsEntry {
    pub name: String,
    pub path: PathBuf,
    pub entry_type: VfsEntryType,
    pub size: u64,
    pub modified: SystemTime,
    pub permissions: u32,  // Unix-style permissions
}

/// Virtual File System trait for abstracting local and remote filesystems
/// Note: Using sync methods because ssh2 is blocking anyway
pub trait VirtualFileSystem: Send + Sync {
    /// List entries in a directory
    fn list_dir(&self, path: &Path) -> Result<Vec<VfsEntry>>;
    
    /// Read file contents
    fn read_file(&self, path: &Path) -> Result<Vec<u8>>;
    
    /// Write file contents
    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()>;
    
    /// Create a directory
    fn create_dir(&self, path: &Path) -> Result<()>;
    
    /// Delete a file or directory
    fn delete(&self, path: &Path, recursive: bool) -> Result<()>;
    
    /// Rename/move a file or directory
    fn rename(&self, from: &Path, to: &Path) -> Result<()>;
    
    /// Get metadata for a path
    fn metadata(&self, path: &Path) -> Result<VfsEntry>;
    
    /// Check if a path exists
    fn exists(&self, path: &Path) -> Result<bool>;
    
    /// Get the filesystem type (for display)
    fn fs_type(&self) -> &str;
    
    /// Get connection info (for display in header)
    fn connection_info(&self) -> String;
}

/// Local filesystem implementation
pub struct LocalFileSystem;

impl VirtualFileSystem for LocalFileSystem {
    fn list_dir(&self, path: &Path) -> Result<Vec<VfsEntry>> {
        use std::fs;
        
        let mut entries = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let name = entry.file_name().to_string_lossy().to_string();
            
            let entry_type = if metadata.is_dir() {
                VfsEntryType::Directory
            } else if metadata.is_symlink() {
                VfsEntryType::Symlink
            } else {
                VfsEntryType::File
            };
            
            entries.push(VfsEntry {
                name,
                path: entry.path(),
                entry_type,
                size: metadata.len(),
                modified: metadata.modified()?,
                permissions: get_permissions(&metadata),
            });
        }
        
        Ok(entries)
    }
    
    fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        Ok(std::fs::read(path)?)
    }
    
    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()> {
        Ok(std::fs::write(path, contents)?)
    }
    
    fn create_dir(&self, path: &Path) -> Result<()> {
        Ok(std::fs::create_dir_all(path)?)
    }
    
    fn delete(&self, path: &Path, recursive: bool) -> Result<()> {
        if recursive {
            if path.is_dir() {
                std::fs::remove_dir_all(path)?;
            } else {
                std::fs::remove_file(path)?;
            }
        } else if path.is_dir() {
            std::fs::remove_dir(path)?;
        } else {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
    
    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        Ok(std::fs::rename(from, to)?)
    }
    
    fn metadata(&self, path: &Path) -> Result<VfsEntry> {
        let metadata = std::fs::metadata(path)?;
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        
        let entry_type = if metadata.is_dir() {
            VfsEntryType::Directory
        } else if metadata.is_symlink() {
            VfsEntryType::Symlink
        } else {
            VfsEntryType::File
        };
        
        Ok(VfsEntry {
            name,
            path: path.to_path_buf(),
            entry_type,
            size: metadata.len(),
            modified: metadata.modified()?,
            permissions: get_permissions(&metadata),
        })
    }
    
    fn exists(&self, path: &Path) -> Result<bool> {
        Ok(path.exists())
    }
    
    fn fs_type(&self) -> &str {
        "local"
    }
    
    fn connection_info(&self) -> String {
        "Local".to_string()
    }
}

#[cfg(unix)]
fn get_permissions(metadata: &std::fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode()
}

#[cfg(windows)]
fn get_permissions(metadata: &std::fs::Metadata) -> u32 {
    // Windows doesn't have Unix-style permissions
    // Return a default value indicating read/write
    if metadata.permissions().readonly() {
        0o444  // Read-only
    } else {
        0o666  // Read-write
    }
}
