// SFTP filesystem implementation
use super::vfs::{VfsEntry, VfsEntryType, VirtualFileSystem};
use super::connection_manager::{AuthMethod, ConnectionConfig};
use anyhow::{Context, Result};
use ssh2::{FileStat, Session, Sftp, HashType};
use std::fs::{OpenOptions, create_dir_all};
use std::io::{Read, Write, BufRead, BufReader};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;
use base64::{Engine as _, engine::general_purpose};

/// Helper to normalize remote paths to always use forward slashes
/// This prevents Windows from mixing backslashes into Unix paths
fn normalize_remote_path(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    // Replace all backslashes with forward slashes
    let normalized = path_str.replace('\\', "/");
    PathBuf::from(normalized)
}

/// SFTP filesystem implementation
pub struct SftpFileSystem {
    #[allow(dead_code)]
    session: Arc<Mutex<Session>>,
    sftp: Arc<Mutex<Sftp>>,
    config: ConnectionConfig,
}

/// Get the path to the known_hosts file
fn get_known_hosts_path() -> Result<PathBuf> {
    let config_dir = crate::config::paths::get_config_dir()?;
    Ok(config_dir.join("known_hosts"))
}

/// Check if a host key is in known_hosts
fn check_known_host(host: &str, port: u16, key_hash: &[u8]) -> Result<bool> {
    let known_hosts_path = get_known_hosts_path()?;
    
    if !known_hosts_path.exists() {
        return Ok(false);
    }
    
    let file = std::fs::File::open(&known_hosts_path)
        .context("Failed to open known_hosts file")?;
    let reader = BufReader::new(file);
    
    let host_port = if port == 22 {
        host.to_string()
    } else {
        format!("[{}]:{}", host, port)
    };
    
    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[0] == host_port
            && let Ok(stored_key) = general_purpose::STANDARD.decode(parts[1])
            && stored_key == key_hash {
            return Ok(true);
        }
    }
    
    Ok(false)
}

/// Add a host key to known_hosts
fn add_known_host(host: &str, port: u16, key_hash: &[u8]) -> Result<()> {
    let known_hosts_path = get_known_hosts_path()?;
    
    // Create directory if it doesn't exist
    if let Some(parent) = known_hosts_path.parent() {
        create_dir_all(parent)?;
    }
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&known_hosts_path)
        .context("Failed to open known_hosts file for writing")?;
    
    let host_port = if port == 22 {
        host.to_string()
    } else {
        format!("[{}]:{}", host, port)
    };
    
    let key_b64 = general_purpose::STANDARD.encode(key_hash);
    writeln!(file, "{} {}", host_port, key_b64)
        .context("Failed to write to known_hosts file")?;
    
    Ok(())
}

impl SftpFileSystem {
    /// Create a new SFTP connection with host key verification
    pub fn connect(config: ConnectionConfig) -> Result<Self> {
        Self::connect_with_verification(config, true)
    }
    
    /// Create a new SFTP connection, optionally bypassing host key verification
    /// (trust_on_first_use=true will automatically accept new hosts)
    pub fn connect_with_verification(config: ConnectionConfig, trust_on_first_use: bool) -> Result<Self> {
        // Connect to the SSH server
        let tcp = TcpStream::connect(format!("{}:{}", config.host, config.port))
            .with_context(|| {
                format!(
                    "Failed to connect to SSH server at {}:{}. \
                    Please check the hostname, port, and your network connection.",
                    config.host, config.port
                )
            })?;
        
        let mut session = Session::new()
            .context("Failed to create SSH session")?;
        session.set_tcp_stream(tcp);
        session.handshake()
            .with_context(|| {
                format!(
                    "SSH handshake failed with {}:{}. \
                    Server may not be running SSH or connection was interrupted.",
                    config.host, config.port
                )
            })?;
        
        // Get and verify host key
        let _host_key = session.host_key()
            .context("Failed to get host key from server")?;
        let key_hash = session.host_key_hash(HashType::Sha256)
            .context("Failed to get host key hash")?;
        
        // Check known_hosts
        let is_known = check_known_host(&config.host, config.port, key_hash)?;
        
        if !is_known {
            if trust_on_first_use {
                // Automatically trust on first use
                add_known_host(&config.host, config.port, key_hash)?;
            } else {
                anyhow::bail!(
                    "Host key verification failed: {} is not in known_hosts. \
                    Fingerprint: {:?}\n\n\
                    This could indicate a man-in-the-middle attack or that the server's host key has changed.\n\
                    If you trust this server, connect again to accept the new key.", 
                    config.host, 
                    key_hash
                );
            }
        }
        
        // Authenticate
        match &config.auth {
            AuthMethod::Password { password: Some(password), .. } => {
                session.userauth_password(&config.username, password)
                    .with_context(|| {
                        format!(
                            "SSH password authentication failed for user '{}' on {}. \
                            Please check your username and password.",
                            config.username, config.host
                        )
                    })?;
            }
            AuthMethod::Password { password: None, .. } => {
                anyhow::bail!("Password required for SFTP authentication but none provided");
            }
            AuthMethod::PublicKey { key_path, passphrase, .. } => {
                session.userauth_pubkey_file(
                    &config.username,
                    None,  // No separate public key file
                    key_path,
                    passphrase.as_deref(),
                ).with_context(|| {
                    format!(
                        "SSH public key authentication failed for user '{}' on {}. \
                        Key file: {}. \
                        Please check that the key file exists and is in the correct format.",
                        config.username, config.host, key_path.display()
                    )
                })?;
            }
            AuthMethod::Anonymous => {
                anyhow::bail!("Anonymous authentication is not supported for SFTP");
            }
        }
        
        if !session.authenticated() {
            anyhow::bail!(
                "SSH authentication failed for user '{}' on {}. \
                All authentication methods exhausted.",
                config.username, config.host
            );
        }
        
        // Create SFTP channel
        let sftp = session.sftp()
            .with_context(|| {
                format!(
                    "Failed to create SFTP channel on {}. \
                    Server may not have SFTP enabled.",
                    config.host
                )
            })?;
        
        Ok(Self {
            session: Arc::new(Mutex::new(session)),
            sftp: Arc::new(Mutex::new(sftp)),
            config,
        })
    }
    
    /// Convert SFTP FileStat to VfsEntry
    fn filestat_to_entry(&self, path: &Path, stat: &FileStat) -> Result<VfsEntry> {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        
        let entry_type = if stat.is_dir() {
            VfsEntryType::Directory
        } else if stat.is_file() {
            VfsEntryType::File
        } else {
            VfsEntryType::Symlink
        };
        
        let size = stat.size.unwrap_or(0);
    let mtime = stat.mtime.unwrap_or(0);
        let modified = UNIX_EPOCH + std::time::Duration::from_secs(mtime);
        let permissions = stat.perm.unwrap_or(0o644);
        
        Ok(VfsEntry {
            name,
            path: normalize_remote_path(path),  // Normalize to use forward slashes
            entry_type,
            size,
            modified,
            permissions,
        })
    }
}

impl VirtualFileSystem for SftpFileSystem {
    fn list_dir(&self, path: &Path) -> Result<Vec<VfsEntry>> {
        let normalized_path = normalize_remote_path(path);
        log::debug!("SFTP list_dir called with path: {:?} (normalized: {:?})", path, normalized_path);
        
        let sftp = self.sftp.lock().expect("SFTP mutex should not be poisoned");
        let mut entries = Vec::new();
        
        let result = sftp.readdir(&normalized_path);
        if let Err(ref e) = result {
            log::error!("SFTP readdir failed for path {:?}: {}", normalized_path, e);
        }
        
        for (remote_path, stat) in result.context("Failed to read remote directory")? 
        {
            // Skip . and ..
            if let Some(name) = remote_path.file_name() {
                let name_str = name.to_string_lossy();
                if name_str == "." || name_str == ".." {
                    continue;
                }
            }
            
            entries.push(self.filestat_to_entry(&remote_path, &stat)?);
        }
        
        log::debug!("SFTP list_dir returned {} entries", entries.len());
        Ok(entries)
    }
    
    fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        let normalized_path = normalize_remote_path(path);
        let sftp = self.sftp.lock().expect("SFTP mutex should not be poisoned");
        
        let mut file = sftp.open(&normalized_path)
            .with_context(|| {
                format!(
                    "Failed to open remote file: {}. \
                    File may not exist or you may lack read permissions.",
                    normalized_path.display()
                )
            })?;
        
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)
            .with_context(|| {
                format!(
                    "Failed to read remote file: {}. \
                    Connection may have been interrupted.",
                    normalized_path.display()
                )
            })?;
        
        Ok(contents)
    }
    
    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()> {
        use std::io::Write;
        
        let normalized_path = normalize_remote_path(path);
        let sftp = self.sftp.lock().expect("SFTP mutex should not be poisoned");
        
        let mut file = sftp.create(&normalized_path)
            .with_context(|| format!("Failed to create remote file: {}", normalized_path.display()))?;
        
        file.write_all(contents)
            .with_context(|| {
                format!(
                    "Failed to write {} bytes to remote file: {}. \
                    This could be due to insufficient disk space, permissions, or network issues.",
                    contents.len(),
                    normalized_path.display()
                )
            })?;
        
        Ok(())
    }
    
    fn create_dir(&self, path: &Path) -> Result<()> {
        let normalized_path = normalize_remote_path(path);
        let sftp = self.sftp.lock().expect("SFTP mutex should not be poisoned");
        
        sftp.mkdir(&normalized_path, 0o755)
            .with_context(|| {
                format!(
                    "Failed to create remote directory: {}. \
                    Check if parent directory exists and you have write permissions.",
                    normalized_path.display()
                )
            })?;
        Ok(())
    }
    
    fn delete(&self, path: &Path, recursive: bool) -> Result<()> {
        let normalized_path = normalize_remote_path(path);
        let sftp = self.sftp.lock().expect("SFTP mutex should not be poisoned");
        
        let stat = sftp.stat(&normalized_path)
            .with_context(|| {
                format!(
                    "Failed to access remote path for deletion: {}. \
                    Path may not exist.",
                    normalized_path.display()
                )
            })?;
        
        if stat.is_dir() {
            if recursive {
                // Recursively delete directory contents
                let entries: Vec<_> = sftp.readdir(&normalized_path)?
                    .into_iter()
                    .filter(|(entry_path, _)| {
                        if let Some(name) = entry_path.file_name() {
                            let name_str = name.to_string_lossy();
                            name_str != "." && name_str != ".."
                        } else {
                            false
                        }
                    })
                    .collect();
                
                // Delete each entry
                for (entry_path, entry_stat) in entries {
                    let normalized_entry = normalize_remote_path(&entry_path);
                    if entry_stat.is_dir() {
                        // For directories, we need to delete recursively
                        // This is simplified - in production should handle this better
                        let _ = sftp.rmdir(&normalized_entry);
                    } else {
                        let _ = sftp.unlink(&normalized_entry);
                    }
                }
            }
            sftp.rmdir(&normalized_path)
                .with_context(|| {
                    format!(
                        "Failed to remove remote directory: {}. \
                        Directory may not be empty or you may lack permissions.",
                        normalized_path.display()
                    )
                })?;
        } else {
            sftp.unlink(&normalized_path)
                .with_context(|| {
                    format!(
                        "Failed to delete remote file: {}. \
                        File may be in use or you may lack permissions.",
                        normalized_path.display()
                    )
                })?;
        }
        
        Ok(())
    }
    
    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        let normalized_from = normalize_remote_path(from);
        let normalized_to = normalize_remote_path(to);
        let sftp = self.sftp.lock().expect("SFTP mutex should not be poisoned");
        
        sftp.rename(&normalized_from, &normalized_to, None)
            .with_context(|| {
                format!(
                    "Failed to rename remote file from '{}' to '{}'. \
                    Target may already exist or you may lack permissions.",
                    normalized_from.display(),
                    normalized_to.display()
                )
            })?;
        Ok(())
    }
    
    fn metadata(&self, path: &Path) -> Result<VfsEntry> {
        let normalized_path = normalize_remote_path(path);
        let sftp = self.sftp.lock().expect("SFTP mutex should not be poisoned");
        
        let stat = sftp.stat(&normalized_path)
            .with_context(|| {
                format!(
                    "Failed to get metadata for remote path: {}. \
                    Path may not exist or you may lack permissions.",
                    normalized_path.display()
                )
            })?;
        
        self.filestat_to_entry(path, &stat)
    }
    
    fn exists(&self, path: &Path) -> Result<bool> {
        let normalized_path = normalize_remote_path(path);
        let sftp = self.sftp.lock().expect("SFTP mutex should not be poisoned");
        Ok(sftp.stat(&normalized_path).is_ok())
    }
    
    fn fs_type(&self) -> &str {
        "sftp"
    }
    
    fn connection_info(&self) -> String {
        format!("SFTP: {}@{}", self.config.username, self.config.host)
    }
}
