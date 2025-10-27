use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents an active SMB/CIFS connection
#[derive(Debug, Clone)]
pub struct SmbConnection {
    /// Unique identifier for this connection
    pub id: String,
    /// UNC path (Windows: \\server\share, Unix: smb://server/share)
    pub unc_path: String,
    /// Server hostname or IP
    pub server: String,
    /// Share name
    pub share: String,
    /// Current working directory within the share
    pub current_path: PathBuf,
    /// Connection credentials
    pub credentials: SmbCredentials,
    /// Timestamp when connection was established
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

/// Authentication credentials for SMB connection
#[derive(Debug, Clone)]
pub struct SmbCredentials {
    /// Username (None for guest access)
    pub username: Option<String>,
    /// Domain or workgroup (None for default)
    pub domain: Option<String>,
    /// Password (None for guest access)
    pub password: Option<String>,
    /// Whether this is a guest connection
    pub is_guest: bool,
}

/// Parameters for establishing a new SMB connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmbConnectionParams {
    /// Display name for this connection
    pub name: String,
    /// UNC path to connect to
    pub unc_path: String,
    /// Username for authentication
    pub username: Option<String>,
    /// Domain or workgroup
    pub domain: Option<String>,
    /// Whether to save password in keyring
    pub save_password: bool,
    /// Whether to use guest access
    pub use_guest: bool,
}

impl SmbConnectionParams {
    /// Create new connection parameters with default values
    pub fn new(name: String, unc_path: String) -> Self {
        Self {
            name,
            unc_path,
            username: None,
            domain: None,
            save_password: false,
            use_guest: false,
        }
    }

    /// Parse server and share from UNC path
    /// Windows: \\server\share
    /// Unix: smb://server/share
    pub fn parse_unc_path(&self) -> Option<(String, String)> {
        if self.unc_path.starts_with("\\\\") {
            // Windows UNC path: \\server\share
            let parts: Vec<&str> = self.unc_path.trim_start_matches("\\\\").split('\\').collect();
            if parts.len() >= 2 {
                return Some((parts[0].to_string(), parts[1].to_string()));
            }
        } else if self.unc_path.starts_with("smb://") {
            // Unix SMB URL: smb://server/share
            let path = self.unc_path.trim_start_matches("smb://");
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 2 {
                return Some((parts[0].to_string(), parts[1].to_string()));
            }
        }
        None
    }

    /// Validate UNC path format
    pub fn validate_unc_path(&self) -> Result<(), String> {
        if self.unc_path.is_empty() {
            return Err("UNC path cannot be empty".to_string());
        }

        if !self.unc_path.starts_with("\\\\") && !self.unc_path.starts_with("smb://") {
            return Err(
                "Invalid UNC path format. Use \\\\server\\share (Windows) or smb://server/share (Unix)"
                    .to_string(),
            );
        }

        if self.parse_unc_path().is_none() {
            return Err("Could not parse server and share from UNC path".to_string());
        }

        Ok(())
    }
}

impl SmbConnection {
    /// Create a new SMB connection
    pub fn new(
        id: String,
        unc_path: String,
        server: String,
        share: String,
        credentials: SmbCredentials,
    ) -> Self {
        Self {
            id,
            unc_path,
            server,
            share,
            current_path: PathBuf::from("/"),
            credentials,
            connected_at: chrono::Utc::now(),
        }
    }

    /// Get the full UNC path for a relative path within the share
    pub fn full_path(&self, relative_path: &PathBuf) -> PathBuf {
        if relative_path.is_absolute() {
            relative_path.clone()
        } else {
            self.current_path.join(relative_path)
        }
    }

    /// Get display string for this connection
    pub fn display_name(&self) -> String {
        format!("{}\\{}", self.server, self.share)
    }
}

impl SmbCredentials {
    /// Create credentials for guest access
    pub fn guest() -> Self {
        Self {
            username: None,
            domain: None,
            password: None,
            is_guest: true,
        }
    }

    /// Create credentials with username and password
    pub fn with_password(username: String, password: String, domain: Option<String>) -> Self {
        Self {
            username: Some(username),
            domain,
            password: Some(password),
            is_guest: false,
        }
    }

    /// Check if credentials are for guest access
    pub fn is_guest(&self) -> bool {
        self.is_guest || (self.username.is_none() && self.password.is_none())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_windows_unc_path() {
        let params = SmbConnectionParams::new(
            "test".to_string(),
            "\\\\server\\share".to_string(),
        );
        let (server, share) = params.parse_unc_path().unwrap();
        assert_eq!(server, "server");
        assert_eq!(share, "share");
    }

    #[test]
    fn test_parse_unix_smb_url() {
        let params = SmbConnectionParams::new(
            "test".to_string(),
            "smb://server/share".to_string(),
        );
        let (server, share) = params.parse_unc_path().unwrap();
        assert_eq!(server, "server");
        assert_eq!(share, "share");
    }

    #[test]
    fn test_validate_unc_path_empty() {
        let params = SmbConnectionParams::new("test".to_string(), "".to_string());
        assert!(params.validate_unc_path().is_err());
    }

    #[test]
    fn test_validate_unc_path_invalid_format() {
        let params = SmbConnectionParams::new("test".to_string(), "/invalid/path".to_string());
        assert!(params.validate_unc_path().is_err());
    }

    #[test]
    fn test_validate_unc_path_valid() {
        let params = SmbConnectionParams::new(
            "test".to_string(),
            "\\\\server\\share".to_string(),
        );
        assert!(params.validate_unc_path().is_ok());
    }

    #[test]
    fn test_guest_credentials() {
        let creds = SmbCredentials::guest();
        assert!(creds.is_guest());
        assert!(creds.username.is_none());
        assert!(creds.password.is_none());
    }

    #[test]
    fn test_password_credentials() {
        let creds = SmbCredentials::with_password(
            "user".to_string(),
            "pass".to_string(),
            Some("DOMAIN".to_string()),
        );
        assert!(!creds.is_guest());
        assert_eq!(creds.username.as_ref().unwrap(), "user");
        assert_eq!(creds.password.as_ref().unwrap(), "pass");
        assert_eq!(creds.domain.as_ref().unwrap(), "DOMAIN");
    }
}
