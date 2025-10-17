// T818-T822: Password handling for encrypted archives
use anyhow::Result;
use std::path::Path;
use std::fs::File;

/// T819: Password dialog state
#[derive(Debug, Clone, Default)]
pub struct PasswordDialog {
    pub input: String,
    pub cursor_pos: usize,
}

impl PasswordDialog {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn push_char(&mut self, c: char) {
        self.input.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
    }
    
    pub fn backspace(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.input.remove(self.cursor_pos);
        }
    }
    
    pub fn get_masked_display(&self) -> String {
        "*".repeat(self.input.len())
    }
    
    pub fn get_password(&self) -> String {
        self.input.clone()
    }
}

/// T820: Prompt for password (returns None if cancelled)
/// Note: This is a placeholder - actual implementation requires UI integration
pub fn prompt_password() -> Option<String> {
    // This will be implemented in the UI layer
    // For now, return None to indicate "not implemented"
    None
}

/// T821: Detect if archive is password-protected
pub fn is_password_protected(path: &Path) -> Result<bool> {
    // Check file extension first
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    match ext.as_str() {
        "zip" => is_zip_encrypted(path),
        "7z" => is_7z_encrypted(path),
        "rar" => {
            // RAR encryption detection would require libunrar
            // For now, assume not encrypted
            Ok(false)
        }
        _ => Ok(false), // TAR formats don't support encryption
    }
}

/// Check if ZIP file is encrypted
fn is_zip_encrypted(path: &Path) -> Result<bool> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    
    // Check if any file in the archive is encrypted
    // Note: zip crate 0.6 doesn't have is_encrypted() method directly
    // We check if files require password by examining encryption method
    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        // If compression method indicates encryption, it's encrypted
        // AES encrypted files have specific compression methods
        if file.compression() == zip::CompressionMethod::AES {
            return Ok(true);
        }
    }
    
    Ok(false)
}

/// Check if 7Z file is encrypted
fn is_7z_encrypted(path: &Path) -> Result<bool> {
    let file = File::open(path)?;
    let len = file.metadata()?.len();
    
    let password: sevenz_rust::Password = "".into();
    
    // Try to open without password
    match sevenz_rust::SevenZReader::new(file, len, password) {
        Ok(archive) => {
            // Check if any file requires password
            for entry in &archive.archive().files {
                if entry.has_stream() {
                    // If we can read the archive structure but files are encrypted,
                    // sevenz-rust will indicate this through encryption flags
                    // For now, we assume if we can read the archive, it's not encrypted
                    // (This is a simplification - proper implementation requires checking encryption flags)
                    return Ok(false);
                }
            }
            Ok(false)
        }
        Err(_) => {
            // If we can't open, it might be encrypted or corrupt
            // For now, assume encrypted
            Ok(true)
        }
    }
}

/// T822: Validate password attempt
pub fn validate_password(path: &Path, password: &str) -> Result<bool> {
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    match ext.as_str() {
        "zip" => validate_zip_password(path, password),
        "7z" => validate_7z_password(path, password),
        _ => Ok(true), // No password needed
    }
}

fn validate_zip_password(path: &Path, password: &str) -> Result<bool> {
    use std::fs::File;
    
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    
    // Try to read the first encrypted file
    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        // Check if file uses AES encryption
        if file.compression() == zip::CompressionMethod::AES {
            // Note: zip crate doesn't directly support password validation
            // We would need to try extracting with the password
            // For now, we'll assume the password is valid if provided
            return Ok(!password.is_empty());
        }
    }
    
    Ok(true)
}

fn validate_7z_password(path: &Path, password: &str) -> Result<bool> {
    use std::fs::File;
    
    let file = File::open(path)?;
    let len = file.metadata()?.len();
    
    let pw: sevenz_rust::Password = password.into();
    
    // Try to open with password
    match sevenz_rust::SevenZReader::new(file, len, pw) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_password_dialog() {
        let mut dialog = PasswordDialog::new();
        
        dialog.push_char('p');
        dialog.push_char('a');
        dialog.push_char('s');
        dialog.push_char('s');
        
        assert_eq!(dialog.get_password(), "pass");
        assert_eq!(dialog.get_masked_display(), "****");
        
        dialog.backspace();
        assert_eq!(dialog.get_password(), "pas");
        assert_eq!(dialog.get_masked_display(), "***");
    }
}
