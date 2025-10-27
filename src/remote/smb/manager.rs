use crate::models::remote::smb::{SmbConnection, SmbConnectionParams, SmbCredentials};
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(windows)]
use super::windows_impl;

#[cfg(unix)]
use super::unix_impl;

/// Manager for SMB/CIFS connections
pub struct SmbManager {
    connections: HashMap<String, SmbConnection>,
}

impl SmbManager {
    /// Create a new SMB manager
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    /// Connect to an SMB share
    pub fn connect(&mut self, params: SmbConnectionParams, password: Option<String>) -> Result<String> {
        log::info!("SmbManager::connect called with params: {:?}", params);
        
        // Validate UNC path
        params.validate_unc_path()
            .map_err(|e| anyhow!("Invalid UNC path: {}", e))?;

        // Parse server and share
        let (server, share) = params.parse_unc_path()
            .ok_or_else(|| anyhow!("Failed to parse UNC path"))?;
        
        log::debug!("Parsed server: {}, share: {}", server, share);

        // Create credentials
        let credentials = if params.use_guest {
            log::info!("Using guest credentials");
            SmbCredentials::guest()
        } else {
            log::info!("Using password credentials for user: {:?}", params.username);
            SmbCredentials::with_password(
                params.username.clone().unwrap_or_default(),
                password.unwrap_or_default(),
                params.domain.clone(),
            )
        };

        // Platform-specific connection
        let connection_id = Uuid::new_v4().to_string();
        
        log::info!("Calling platform-specific connect_share for: {}", params.unc_path);
        
        #[cfg(windows)]
        windows_impl::connect_share(&params.unc_path, &credentials)
            .with_context(|| format!("Failed to connect to SMB share: {}", params.unc_path))?;
        
        #[cfg(unix)]
        unix_impl::connect_share(&params.unc_path, &credentials)
            .with_context(|| format!("Failed to connect to SMB share: {}", params.unc_path))?;
        
        log::info!("Platform-specific connection successful");

        // Create connection object
        let connection = SmbConnection::new(
            connection_id.clone(),
            params.unc_path.clone(),
            server,
            share,
            credentials,
        );

        self.connections.insert(connection_id.clone(), connection);

        Ok(connection_id)
    }

    /// Disconnect from an SMB share
    pub fn disconnect(&mut self, connection_id: &str) -> Result<()> {
        let connection = self.connections.get(connection_id)
            .ok_or_else(|| anyhow!("Connection not found: {}", connection_id))?;

        #[cfg(windows)]
        windows_impl::disconnect_share(&connection.unc_path)?;
        
        #[cfg(unix)]
        unix_impl::disconnect_share(&connection.unc_path)?;

        self.connections.remove(connection_id);

        Ok(())
    }

    /// Get a connection by ID
    pub fn get_connection(&self, connection_id: &str) -> Option<&SmbConnection> {
        self.connections.get(connection_id)
    }

    /// Get all active connections
    pub fn list_connections(&self) -> Vec<&SmbConnection> {
        self.connections.values().collect()
    }

    /// Test if a connection is still alive
    pub fn test_connection(&self, connection_id: &str) -> Result<bool> {
        let connection = self.connections.get(connection_id)
            .ok_or_else(|| anyhow!("Connection not found: {}", connection_id))?;

        #[cfg(windows)]
        return windows_impl::test_connection(&connection.unc_path);
        
        #[cfg(unix)]
        return unix_impl::test_connection(&connection.unc_path);
    }
}

impl Default for SmbManager {
    fn default() -> Self {
        Self::new()
    }
}
