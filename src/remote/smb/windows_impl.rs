// Windows-specific SMB implementation using WinAPI
use crate::models::remote::smb::SmbCredentials;
use anyhow::{anyhow, Result};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::NetworkManagement::WNet::{
    WNetAddConnection2W, WNetCancelConnection2W, NETRESOURCEW, RESOURCETYPE_DISK,
    NET_CONNECT_FLAGS, RESOURCE_GLOBALNET,
};

/// Connect to an SMB share using WNetAddConnection2W
pub fn connect_share(unc_path: &str, credentials: &SmbCredentials) -> Result<()> {
    // Convert strings to wide strings (UTF-16)
    let unc_path_wide: Vec<u16> = OsStr::new(unc_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let username_wide: Option<Vec<u16>> = credentials.username.as_ref().map(|u| {
        let username_with_domain = if let Some(ref domain) = credentials.domain {
            format!("{}\\{}", domain, u)
        } else {
            u.clone()
        };
        OsStr::new(&username_with_domain)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    });

    let password_wide: Option<Vec<u16>> = credentials.password.as_ref().map(|p| {
        OsStr::new(p)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    });

    // Set up network resource structure
    let net_resource = NETRESOURCEW {
        dwType: RESOURCETYPE_DISK,
        lpLocalName: PWSTR::null(),
        lpRemoteName: PWSTR::from_raw(unc_path_wide.as_ptr() as *mut u16),
        lpProvider: PWSTR::null(),
        dwScope: RESOURCE_GLOBALNET,
        dwUsage: 0,
        dwDisplayType: 0,
        lpComment: PWSTR::null(),
    };

    let username_ptr = username_wide
        .as_ref()
        .map(|v| PWSTR::from_raw(v.as_ptr() as *mut u16))
        .unwrap_or(PWSTR::null());

    let password_ptr = password_wide
        .as_ref()
        .map(|v| PWSTR::from_raw(v.as_ptr() as *mut u16))
        .unwrap_or(PWSTR::null());

    // Call WNetAddConnection2W
    let result = unsafe {
        WNetAddConnection2W(
            &net_resource as *const _,
            password_ptr,
            username_ptr,
            NET_CONNECT_FLAGS(0x00000001), // CONNECT_TEMPORARY
        )
    };

    if result != ERROR_SUCCESS {
        return Err(anyhow!(
            "Failed to connect to SMB share '{}'. Windows error code: {}. {}",
            unc_path,
            result.0,
            get_error_message(result)
        ));
    }

    Ok(())
}

/// Disconnect from an SMB share using WNetCancelConnection2W
pub fn disconnect_share(unc_path: &str) -> Result<()> {
    let unc_path_wide: Vec<u16> = OsStr::new(unc_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let result = unsafe {
        WNetCancelConnection2W(
            PCWSTR::from_raw(unc_path_wide.as_ptr()),
            NET_CONNECT_FLAGS(0), // No flags
            true, // Force disconnect even if files are open
        )
    };

    if result != ERROR_SUCCESS {
        return Err(anyhow!(
            "Failed to disconnect from SMB share '{}'. Windows error code: {}. {}",
            unc_path,
            result.0,
            get_error_message(result)
        ));
    }

    Ok(())
}

/// Test if connection is still alive by attempting to access the share
pub fn test_connection(unc_path: &str) -> Result<bool> {
    use std::fs;
    
    // Try to read the root directory
    match fs::read_dir(unc_path) {
        Ok(_) => Ok(true),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(false)
            } else {
                Err(anyhow!("Failed to test connection: {}", e))
            }
        }
    }
}

/// Get a human-readable error message for common Windows network errors
fn get_error_message(error: WIN32_ERROR) -> String {
    match error.0 {
        53 => "Network path not found. Please check the server name and ensure the server is accessible.".to_string(),
        67 => "Network name not found. The share may not exist on the server.".to_string(),
        86 => "Invalid password or access denied. Please check your credentials.".to_string(),
        1219 => "Multiple connections to a server or shared resource by the same user are not allowed. Disconnect existing connections first.".to_string(),
        1326 => "Logon failure: unknown username or bad password.".to_string(),
        1331 => "Account currently disabled. Please contact your system administrator.".to_string(),
        1909 => "Account locked out. Please contact your system administrator.".to_string(),
        2202 => "Username not found in the domain.".to_string(),
        _ => format!("Unknown Windows error code: {}. Please check your network connection and credentials.", error.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        let msg = get_error_message(WIN32_ERROR(53));
        assert!(msg.contains("Network path not found"));

        let msg = get_error_message(WIN32_ERROR(67));
        assert!(msg.contains("Network name not found"));

        let msg = get_error_message(WIN32_ERROR(86));
        assert!(msg.contains("Invalid password"));
    }
}
