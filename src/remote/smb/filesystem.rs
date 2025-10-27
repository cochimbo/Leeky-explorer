// SMB Virtual Filesystem implementation
use crate::remote::{VfsEntry, VfsEntryType, VirtualFileSystem, ConnectionConfig};
use crate::remote::smb::SmbManager;
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

/// SMB filesystem implementation using VirtualFileSystem trait
pub struct SmbFileSystem {
    config: ConnectionConfig,
    manager: Arc<SmbManager>,
    unc_path: String,  // \\server\share
}

impl SmbFileSystem {
    /// Connect to an SMB share
    pub fn connect(config: ConnectionConfig) -> Result<Self> {
        log::info!("Connecting to SMB share: {}@{}", config.username, config.host);
        
        // Extract share name from initial_path (e.g., "/share_name" -> "share_name")
        let share = config.initial_path
            .as_ref()
            .and_then(|p| p.to_str())
            .and_then(|s| s.strip_prefix('/'))
            .ok_or_else(|| anyhow::anyhow!("Invalid share path"))?;
        
        let unc_path = format!("\\\\{}\\{}", config.host, share);
        
        // Extract password from config
        let password = match &config.auth {
            crate::remote::AuthMethod::Password { password, .. } => password.clone(),
            _ => None,
        };
        
        // Create connection params
        let params = crate::models::remote::smb::SmbConnectionParams {
            name: config.name.clone(),
            unc_path: unc_path.clone(),
            username: if config.username == "guest" { None } else { Some(config.username.clone()) },
            domain: None,  // TODO: Extract from config if needed
            save_password: false,  // Already handled by ConnectionConfig
            use_guest: config.username == "guest",
        };
        
        // Create SMB manager and connect
        let mut manager = SmbManager::new();
        let connection_id = manager.connect(params, password)
            .context("Failed to establish SMB connection")?;
        
        log::info!("Successfully connected to SMB share: {} (id: {})", unc_path, connection_id);
        
        Ok(Self {
            config,
            manager: Arc::new(manager),
            unc_path,
        })
    }
    
    /// Convert a relative VFS path to full UNC path
    /// E.g., "/" -> "\\server\share", "/folder" -> "\\server\share\folder"
    fn to_unc_path(&self, path: &Path) -> PathBuf {
        let path_str = path.to_string_lossy();
        let relative = path_str.trim_start_matches('/');
        
        if relative.is_empty() {
            PathBuf::from(&self.unc_path)
        } else {
            PathBuf::from(format!("{}\\{}", self.unc_path, relative.replace('/', "\\")))
        }
    }
    
    /// Convert UNC path back to VFS path
    /// E.g., "\\server\share\folder" -> "/folder"
    fn from_unc_path(&self, unc_path: &Path) -> PathBuf {
        let unc_str = unc_path.to_string_lossy();
        let relative = unc_str
            .strip_prefix(&self.unc_path)
            .unwrap_or(&unc_str)
            .trim_start_matches('\\');
        
        if relative.is_empty() {
            PathBuf::from("/")
        } else {
            PathBuf::from(format!("/{}", relative.replace('\\', "/")))
        }
    }
}

impl VirtualFileSystem for SmbFileSystem {
    fn list_dir(&self, path: &Path) -> Result<Vec<VfsEntry>> {
        let unc_path = self.to_unc_path(path);
        log::debug!("SMB list_dir: {} -> {}", path.display(), unc_path.display());
        
        // Use std::fs to list directory (Windows handles UNC paths natively)
        let mut entries = Vec::new();
        
        for entry in std::fs::read_dir(&unc_path)
            .context(format!("Failed to list directory: {}", unc_path.display()))? 
        {
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
            
            // Convert UNC path back to VFS path
            let vfs_path = self.from_unc_path(&entry.path());
            
            entries.push(VfsEntry {
                name,
                path: vfs_path,
                entry_type,
                size: metadata.len(),
                modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                permissions: 0o644,  // Default permissions for SMB
            });
        }
        
        Ok(entries)
    }
    
    fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        let unc_path = self.to_unc_path(path);
        log::debug!("SMB read_file: {} -> {}", path.display(), unc_path.display());
        
        std::fs::read(&unc_path)
            .context(format!("Failed to read file: {}", unc_path.display()))
    }
    
    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()> {
        let unc_path = self.to_unc_path(path);
        log::debug!("SMB write_file: {} -> {}", path.display(), unc_path.display());
        
        std::fs::write(&unc_path, contents)
            .context(format!("Failed to write file: {}", unc_path.display()))
    }
    
    fn create_dir(&self, path: &Path) -> Result<()> {
        let unc_path = self.to_unc_path(path);
        log::debug!("SMB create_dir: {} -> {}", path.display(), unc_path.display());
        
        std::fs::create_dir(&unc_path)
            .context(format!("Failed to create directory: {}", unc_path.display()))
    }
    
    fn delete(&self, path: &Path, recursive: bool) -> Result<()> {
        let unc_path = self.to_unc_path(path);
        log::debug!("SMB delete: {} -> {} (recursive: {})", path.display(), unc_path.display(), recursive);
        
        let metadata = std::fs::metadata(&unc_path)
            .context(format!("Failed to get metadata for deletion: {}", unc_path.display()))?;
        
        if metadata.is_dir() {
            if recursive {
                std::fs::remove_dir_all(&unc_path)
            } else {
                std::fs::remove_dir(&unc_path)
            }
        } else {
            std::fs::remove_file(&unc_path)
        }.context(format!("Failed to delete: {}", unc_path.display()))
    }
    
    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        let unc_from = self.to_unc_path(from);
        let unc_to = self.to_unc_path(to);
        log::debug!("SMB rename: {} -> {} (UNC: {} -> {})", 
                   from.display(), to.display(), unc_from.display(), unc_to.display());
        
        std::fs::rename(&unc_from, &unc_to)
            .context(format!("Failed to rename: {} -> {}", unc_from.display(), unc_to.display()))
    }
    
    fn metadata(&self, path: &Path) -> Result<VfsEntry> {
        let unc_path = self.to_unc_path(path);
        log::debug!("SMB metadata: {} -> {}", path.display(), unc_path.display());
        
        let metadata = std::fs::metadata(&unc_path)
            .context(format!("Failed to get metadata: {}", unc_path.display()))?;
        
        let entry_type = if metadata.is_dir() {
            VfsEntryType::Directory
        } else if metadata.is_symlink() {
            VfsEntryType::Symlink
        } else {
            VfsEntryType::File
        };
        
        let name = unc_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        
        Ok(VfsEntry {
            name,
            path: path.to_path_buf(),
            entry_type,
            size: metadata.len(),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            permissions: 0o644,
        })
    }
    
    fn exists(&self, path: &Path) -> Result<bool> {
        let unc_path = self.to_unc_path(path);
        Ok(unc_path.exists())
    }
    
    fn fs_type(&self) -> &str {
        "SMB"
    }
    
    fn connection_info(&self) -> String {
        self.unc_path.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_path_conversion() {
        let config = ConnectionConfig {
            name: "test".to_string(),
            connection_type: crate::remote::ConnectionType::Smb,
            host: "server".to_string(),
            port: 445,
            username: "user".to_string(),
            auth: crate::remote::AuthMethod::Password {
                password: Some("pass".to_string()),
                stored: false,
            },
            initial_path: Some(PathBuf::from("/share")),
        };
        
        // Can't actually connect without a real server, but we can test path conversion
        // by creating the struct directly (not using connect())
        let fs = SmbFileSystem {
            config,
            manager: Arc::new(SmbManager::new()),
            unc_path: "\\\\server\\share".to_string(),
        };
        
        // Test to_unc_path
        assert_eq!(fs.to_unc_path(Path::new("/")), PathBuf::from("\\\\server\\share"));
        assert_eq!(fs.to_unc_path(Path::new("/folder")), PathBuf::from("\\\\server\\share\\folder"));
        assert_eq!(fs.to_unc_path(Path::new("/folder/sub")), PathBuf::from("\\\\server\\share\\folder\\sub"));
        
        // Test from_unc_path
        assert_eq!(fs.from_unc_path(Path::new("\\\\server\\share")), PathBuf::from("/"));
        assert_eq!(fs.from_unc_path(Path::new("\\\\server\\share\\folder")), PathBuf::from("/folder"));
        assert_eq!(fs.from_unc_path(Path::new("\\\\server\\share\\folder\\sub")), PathBuf::from("/folder/sub"));
    }
}
