// Migration utility to clean up plain-text passwords from old connection files
use anyhow::Result;
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Migrate old connection files to remove plain-text passwords
pub fn migrate_connections_file(file_path: &Path) -> Result<bool> {
    if !file_path.exists() {
        return Ok(false);
    }
    
    let content = fs::read_to_string(file_path)?;
    let mut json: Value = serde_json::from_str(&content)?;
    
    let mut modified = false;
    
    // Check if there are connections with plain-text passwords
    if let Some(connections) = json.get_mut("connections").and_then(|c| c.as_array_mut()) {
        for conn in connections {
            if let Some(auth) = conn.get_mut("auth") {
                // Check for old Password format: "Password": "plaintext"
                if let Some(_password_str) = auth.as_str() {
                    log::warn!("Found plain-text password in connection, removing it");
                    // Replace with new format without password
                    *auth = serde_json::json!({
                        "Password": {
                            "password": null,
                            "stored": false
                        }
                    });
                    modified = true;
                }
                // Check for old Password object format
                else if let Some(obj) = auth.as_object_mut() {
                    if let Some(password_val) = obj.get("Password") {
                        // If it's a string (old format)
                        if password_val.is_string() {
                            log::warn!("Found plain-text password in connection object, removing it");
                            obj.insert("Password".to_string(), serde_json::json!({
                                "password": null,
                                "stored": false
                            }));
                            modified = true;
                        }
                        // If it's an object but has plain-text password field
                        else if let Some(pwd_obj) = password_val.as_object() {
                            if pwd_obj.contains_key("0") || pwd_obj.get("password").is_some() {
                                if let Some(pwd_str) = pwd_obj.get("password").and_then(|p| p.as_str()) {
                                    if !pwd_str.is_empty() {
                                        log::warn!("Found stored plain-text password, removing it");
                                        obj.insert("Password".to_string(), serde_json::json!({
                                            "password": null,
                                            "stored": false
                                        }));
                                        modified = true;
                                    }
                                }
                            }
                        }
                    }
                    
                    // Check PublicKey passphrase
                    if let Some(pk_val) = obj.get("PublicKey") {
                        if let Some(pk_obj) = pk_val.as_object() {
                            if let Some(pass_str) = pk_obj.get("passphrase").and_then(|p| p.as_str()) {
                                if !pass_str.is_empty() {
                                    log::warn!("Found stored plain-text passphrase, removing it");
                                    obj.insert("PublicKey".to_string(), serde_json::json!({
                                        "key_path": pk_obj.get("key_path"),
                                        "passphrase": null,
                                        "stored": false
                                    }));
                                    modified = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    if modified {
        // Create backup
        let backup_path = file_path.with_extension("json.backup");
        fs::copy(file_path, &backup_path)?;
        log::info!("Created backup at: {:?}", backup_path);
        
        // Write cleaned file
        let cleaned_content = serde_json::to_string_pretty(&json)?;
        fs::write(file_path, cleaned_content)?;
        log::info!("Cleaned connection file, removed plain-text passwords");
        
        Ok(true)
    } else {
        log::info!("No plain-text passwords found in connection file");
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_migrate_old_password_format() {
        let mut file = NamedTempFile::new().unwrap();
        let content = r#"{
            "connections": [
                {
                    "name": "Test Server",
                    "host": "example.com",
                    "auth": "my-secret-password"
                }
            ]
        }"#;
        file.write_all(content.as_bytes()).unwrap();
        
        let result = migrate_connections_file(file.path()).unwrap();
        assert!(result, "Should have modified the file");
        
        let new_content = fs::read_to_string(file.path()).unwrap();
        assert!(!new_content.contains("my-secret-password"));
    }
}
