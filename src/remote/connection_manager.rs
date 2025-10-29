// Connection Manager for remote filesystems
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;
use keyring::Entry;

/// Type of remote connection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    Sftp,
    Ftp,
    Ftps,
    Smb,
}

impl ConnectionType {
    pub fn as_str(&self) -> &str {
        match self {
            ConnectionType::Sftp => "SFTP",
            ConnectionType::Ftp => "FTP",
            ConnectionType::Ftps => "FTPS",
            ConnectionType::Smb => "SMB",
        }
    }
}

/// Authentication method (passwords stored in OS keychain, not serialized)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    Password {
        #[serde(skip)]  // Don't serialize password
        password: Option<String>,  // In-memory only
        stored: bool,  // Indicates if password is in keychain
    },
    PublicKey { 
        key_path: PathBuf, 
        #[serde(skip)]  // Don't serialize passphrase
        passphrase: Option<String>,  // In-memory only
        stored: bool,  // Indicates if passphrase is in keychain
    },
    Anonymous,
}

/// Configuration for a remote connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub name: String,  // User-friendly name
    pub connection_type: ConnectionType,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: AuthMethod,
    pub initial_path: Option<PathBuf>,  // Default directory to open
}

impl ConnectionConfig {
    pub fn new_sftp(name: String, host: String, username: String, auth: AuthMethod) -> Self {
        Self {
            name,
            connection_type: ConnectionType::Sftp,
            host,
            port: 22,  // Default SFTP port
            username,
            auth,
            initial_path: Some(PathBuf::from("/")),
        }
    }
    
    pub fn new_ftp(name: String, host: String, username: String, password: Option<String>, use_tls: bool, store_password: bool) -> Self {
        Self {
            name,
            connection_type: if use_tls { ConnectionType::Ftps } else { ConnectionType::Ftp },
            host,
            port: 21,  // Default FTP port
            username,
            auth: AuthMethod::Password { password, stored: store_password },
            initial_path: Some(PathBuf::from("/")),
        }
    }
    
    pub fn new_smb(name: String, host: String, username: String, password: Option<String>, share: String, store_password: bool) -> Self {
        Self {
            name,
            connection_type: ConnectionType::Smb,
            host,
            port: 445,  // Default SMB port
            username,
            auth: AuthMethod::Password { password, stored: store_password },
            initial_path: Some(PathBuf::from(format!("/{}", share))),
        }
    }
    
    /// Get keyring service name for this connection
    fn keyring_service(&self) -> String {
        format!("leeky-explorer-{}", self.connection_type.as_str().to_lowercase())
    }
    
    /// Get keyring account name (unique identifier)
    fn keyring_account(&self) -> String {
        format!("{}@{}:{}", self.username, self.host, self.port)
    }
    
    /// Store password in OS keychain
    pub fn store_password(&self, password: &str) -> Result<()> {
        let service = self.keyring_service();
        let account = self.keyring_account();
        log::debug!("Attempting to store password in keychain - service: {}, account: {}", service, account);
        
        let entry = Entry::new(&service, &account)?;
        entry.set_password(password)?;
        
        log::info!("✓ Successfully stored password in OS keychain for: {}", account);
        Ok(())
    }
    
    /// Retrieve password from OS keychain
    pub fn get_password(&self) -> Result<String> {
        let service = self.keyring_service();
        let account = self.keyring_account();
        log::debug!("Attempting to retrieve password from keychain - service: {}, account: {}", service, account);
        
        let entry = Entry::new(&service, &account)?;
        let password = entry.get_password()?;
        
        log::info!("✓ Successfully retrieved password from OS keychain for: {}", account);
        Ok(password)
    }
    
    /// Delete password from OS keychain
    pub fn delete_password(&self) -> Result<()> {
        let entry = Entry::new(&self.keyring_service(), &self.keyring_account())?;
        entry.delete_credential()?;
        log::info!("Deleted password from keychain for: {}", self.keyring_account());
        Ok(())
    }
    
    /// Get the actual password (from memory or keychain)
    pub fn resolve_password(&mut self) -> Result<Option<String>> {
        match &self.auth {
            AuthMethod::Password { password: Some(pwd), .. } => {
                Ok(Some(pwd.clone()))
            }
            AuthMethod::Password { password: None, stored: true } => {
                // Try to load from keychain
                match self.get_password() {
                    Ok(pwd) => {
                        // Update the auth with the retrieved password
                        if let AuthMethod::Password { password, .. } = &mut self.auth {
                            *password = Some(pwd.clone());
                        }
                        Ok(Some(pwd))
                    }
                    Err(e) => {
                        log::warn!("Failed to retrieve password from keychain: {}", e);
                        // Mark as not stored
                        if let AuthMethod::Password { stored, .. } = &mut self.auth {
                            *stored = false;
                        }
                        Ok(None)
                    }
                }
            }
            AuthMethod::Password { .. } => {
                Ok(None)
            }
            AuthMethod::PublicKey { passphrase: Some(pp), .. } => {
                Ok(Some(pp.clone()))
            }
            AuthMethod::PublicKey { passphrase: None, stored: true, .. } => {
                match self.get_password() {
                    Ok(pp) => {
                        // Update the auth with the retrieved passphrase
                        if let AuthMethod::PublicKey { passphrase, .. } = &mut self.auth {
                            *passphrase = Some(pp.clone());
                        }
                        Ok(Some(pp))
                    }
                    Err(e) => {
                        log::warn!("Failed to retrieve passphrase from keychain: {}", e);
                        if let AuthMethod::PublicKey { stored, .. } = &mut self.auth {
                            *stored = false;
                        }
                        Ok(None)
                    }
                }
            }
            AuthMethod::PublicKey { .. } => {
                Ok(None)
            }
            AuthMethod::Anonymous => {
                Ok(None)
            }
        }
    }
}

/// Manager for saved connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionManager {
    connections: Vec<ConnectionConfig>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
        }
    }
    
    pub fn load() -> Result<Self> {
        let path = crate::config::paths::get_connections_file_path()?;
        
        // Run migration to clean up any plain-text passwords
        if path.exists() && let Err(e) = crate::remote::migration::migrate_connections_file(&path) {
            log::warn!("Failed to migrate connection file: {}", e);
        }
        
        if path.exists() {
            let contents = std::fs::read_to_string(&path)?;
            let manager: ConnectionManager = serde_json::from_str(&contents)?;
            Ok(manager)
        } else {
            Ok(Self::new())
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let path = crate::config::paths::get_connections_file_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, contents)?;
        Ok(())
    }
    
    pub fn add(&mut self, config: ConnectionConfig) -> Result<()> {
        // Check for duplicate names
        if self.connections.iter().any(|c| c.name == config.name) {
            anyhow::bail!("Connection with name '{}' already exists", config.name);
        }
        self.connections.push(config);
        self.save()?;
        Ok(())
    }
    
    pub fn remove(&mut self, index: usize) -> Result<()> {
        if index >= self.connections.len() {
            anyhow::bail!("Invalid connection index");
        }
        self.connections.remove(index);
        self.save()?;
        Ok(())
    }
    
    pub fn get(&self, index: usize) -> Option<&ConnectionConfig> {
        self.connections.get(index)
    }
    
    pub fn list(&self) -> &[ConnectionConfig] {
        &self.connections
    }
    
    pub fn is_empty(&self) -> bool {
        self.connections.is_empty()
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
