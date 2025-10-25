// Disk space information utilities
use std::path::Path;
use anyhow::{Result, Context};

/// Disk space information for a filesystem
#[derive(Debug, Clone)]
pub struct DiskSpaceInfo {
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub drive_label: String,
}

/// Get disk space information for the filesystem containing the given path
pub fn get_disk_space(path: &Path) -> Result<DiskSpaceInfo> {
    // Canonicalize path to resolve symlinks and relative paths
    let canonical_path = path.canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {}", path.display()))?;
    
    // Get filesystem statistics
    let available = fs2::available_space(&canonical_path)
        .with_context(|| format!("Failed to get available space for: {}", canonical_path.display()))?;
    
    let total = fs2::total_space(&canonical_path)
        .with_context(|| format!("Failed to get total space for: {}", canonical_path.display()))?;
    
    let used = total.saturating_sub(available);
    let free = available;
    
    // Get drive label (platform-specific)
    let drive_label = get_drive_label(&canonical_path);
    
    Ok(DiskSpaceInfo {
        used_bytes: used,
        total_bytes: total,
        free_bytes: free,
        drive_label,
    })
}

/// Extract drive label from path (platform-specific)
fn get_drive_label(path: &Path) -> String {
    #[cfg(target_os = "windows")]
    {
        // Extract drive letter (e.g., "C:", "D:")
        if let Some(prefix) = path.components().next() {
            if let std::path::Component::Prefix(prefix_component) = prefix {
                // Format the disk letter properly
                if let std::path::Prefix::Disk(letter) | std::path::Prefix::VerbatimDisk(letter) = prefix_component.kind() {
                    return format!("{}:", letter as char);
                }
            }
        }
        "Unknown".to_string()
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // For Unix-like systems, try to determine mount point
        // This is a simplified version - in production, you'd parse /proc/mounts or use statvfs
        let path_str = path.to_string_lossy();
        
        // Common mount points
        if path_str.starts_with("/Volumes/") {
            // macOS external volumes
            if let Some(volume_name) = path_str.strip_prefix("/Volumes/") {
                if let Some(first_part) = volume_name.split('/').next() {
                    return format!("/Volumes/{}", first_part);
                }
            }
        }
        
        if path_str.starts_with("/mnt/") {
            // Linux mount points
            if let Some(mount_name) = path_str.strip_prefix("/mnt/") {
                if let Some(first_part) = mount_name.split('/').next() {
                    return format!("/mnt/{}", first_part);
                }
            }
        }
        
        if path_str.starts_with("/home") {
            return "/home".to_string();
        }
        
        // Default to root
        "/".to_string()
    }
}

/// Format bytes to human-readable size (KB, MB, GB, TB)
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    
    if bytes >= TB {
        format!("{:.1}TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

/// Format disk space information as a compact string
/// Format: "Drive: UsedGB / TotalGB (XX% free)"
pub fn format_disk_space(info: &DiskSpaceInfo) -> String {
    let used_str = format_size(info.used_bytes);
    let total_str = format_size(info.total_bytes);
    
    // Calculate percentage free
    let percent_free = if info.total_bytes > 0 {
        (info.free_bytes as f64 / info.total_bytes as f64 * 100.0) as u8
    } else {
        0
    };
    
    format!(
        "{}: {} / {} ({}% free)",
        info.drive_label,
        used_str,
        total_str,
        percent_free
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512B");
        assert_eq!(format_size(1024), "1.0KB");
        assert_eq!(format_size(1536), "1.5KB");
        assert_eq!(format_size(1024 * 1024), "1.0MB");
        assert_eq!(format_size(1536 * 1024), "1.5MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0GB");
        assert_eq!(format_size(1536 * 1024 * 1024), "1.5GB");
        assert_eq!(format_size(1024 * 1024 * 1024 * 1024), "1.0TB");
        assert_eq!(format_size(1536 * 1024 * 1024 * 1024), "1.5TB");
    }
    
    #[test]
    #[cfg(target_os = "windows")]
    fn test_get_drive_label_windows() {
        // Test with canonicalized current directory
        let current_dir = std::env::current_dir().expect("Failed to get current dir");
        let label = get_drive_label(&current_dir);
        println!("Current dir: {:?}, Label: {}", current_dir, label);
        // On Windows, should have a drive letter
        assert!(label.len() > 0 && label != "Unknown", "Expected drive label, got: {}", label);
    }
    
    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_get_drive_label_unix() {
        use std::path::PathBuf;
        
        let path = PathBuf::from("/");
        let label = get_drive_label(&path);
        assert_eq!(label, "/");
        
        let path = PathBuf::from("/home/user/documents");
        let label = get_drive_label(&path);
        assert_eq!(label, "/home");
        
        let path = PathBuf::from("/mnt/data/files");
        let label = get_drive_label(&path);
        assert_eq!(label, "/mnt/data");
        
        let path = PathBuf::from("/Volumes/External/files");
        let label = get_drive_label(&path);
        assert_eq!(label, "/Volumes/External");
    }
    
    #[test]
    fn test_format_disk_space() {
        let info = DiskSpaceInfo {
            used_bytes: 45 * 1024 * 1024 * 1024, // 45GB
            total_bytes: 120 * 1024 * 1024 * 1024, // 120GB
            free_bytes: 75 * 1024 * 1024 * 1024, // 75GB
            drive_label: "C:".to_string(),
        };
        
        let formatted = format_disk_space(&info);
        assert!(formatted.contains("C:"));
        assert!(formatted.contains("45"));
        assert!(formatted.contains("120"));
        assert!(formatted.contains("62% free")); // 75/120 = 62.5%
    }
}
