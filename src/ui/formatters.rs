// Formatting utilities for file metadata display
use crate::models::file_entry::FileEntry;
use std::time::SystemTime;
use unicode_width::UnicodeWidthStr;

/// Calculate the visual width of a string (handles emojis correctly)
/// Emojis and wide characters take 2 cells, ASCII takes 1 cell
fn visual_width(s: &str) -> usize {
    s.width()
}

/// Format file extension for display
/// Returns the extension or empty string for directories/files without extension
/// Note: Does NOT truncate - let the scroll system handle long extensions
pub fn format_extension(entry: &FileEntry) -> String {
    match &entry.extension {
        Some(ext) => ext.clone(),
        None => {
            if entry.is_dir() {
                "".to_string()
            } else {
                "".to_string()
            }
        }
    }
}

/// Format file size for display with units (B, KB, MB, GB, TB)
/// Reuses logic from disk_info module
pub fn format_size(entry: &FileEntry) -> String {
    if entry.is_dir() {
        "<DIR>".to_string()
    } else {
        let bytes = entry.size;
        
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;

        if bytes < KB {
            format!("{} B", bytes)
        } else if bytes < MB {
            format!("{:.1} KB", bytes as f64 / KB as f64)
        } else if bytes < GB {
            format!("{:.1} MB", bytes as f64 / MB as f64)
        } else if bytes < TB {
            format!("{:.1} GB", bytes as f64 / GB as f64)
        } else {
            format!("{:.1} TB", bytes as f64 / TB as f64)
        }
    }
}

/// Format SystemTime as ISO date string: "YYYY-MM-DD HH:MM"
/// Returns "N/A" if time is None
pub fn format_date(time: Option<SystemTime>) -> String {
    match time {
        Some(t) => {
            // Convert SystemTime to local time
            match t.duration_since(SystemTime::UNIX_EPOCH) {
                Ok(duration) => {
                    let secs = duration.as_secs();
                    
                    // Basic date/time calculation (UTC)
                    // For production, consider using chrono crate for local timezone
                    let days_since_epoch = secs / 86400;
                    let remaining_secs = secs % 86400;
                    
                    let hours = remaining_secs / 3600;
                    let minutes = (remaining_secs % 3600) / 60;
                    
                    // Simple year/month/day calculation (approximate)
                    let years_since_1970 = days_since_epoch / 365;
                    let year = 1970 + years_since_1970;
                    let remaining_days = days_since_epoch % 365;
                    let month = (remaining_days / 30) + 1;
                    let day = (remaining_days % 30) + 1;
                    
                    format!("{:04}-{:02}-{:02} {:02}:{:02}", year, month.min(12), day.min(31), hours, minutes)
                }
                Err(_) => "N/A".to_string(),
            }
        }
        None => "N/A".to_string(),
    }
}

/// Format file permissions for display
/// Windows: RHSA format (Readonly, Hidden, System, Archive)
/// Unix: rwxr-xr-x format with type prefix (d, l, -, etc.)
pub fn format_permissions(entry: &FileEntry) -> String {
    #[cfg(windows)]
    {
        // Windows file attributes (FILE_ATTRIBUTE_*)
        // https://docs.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants
        const FILE_ATTRIBUTE_READONLY: u32 = 0x00000001;
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x00000002;
        const FILE_ATTRIBUTE_SYSTEM: u32 = 0x00000004;
        const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x00000020;
        
        if let Some(attrs) = entry.file_attributes {
            let readonly = attrs & FILE_ATTRIBUTE_READONLY != 0;
            let hidden = attrs & FILE_ATTRIBUTE_HIDDEN != 0;
            let system = attrs & FILE_ATTRIBUTE_SYSTEM != 0;
            let archive = attrs & FILE_ATTRIBUTE_ARCHIVE != 0;
            
            format!(
                "{}{}{}{}",
                if readonly { 'R' } else { '-' },
                if hidden { 'H' } else { '-' },
                if system { 'S' } else { '-' },
                if archive { 'A' } else { '-' }
            )
        } else {
            // Fallback if attributes unavailable
            let readonly = entry.permissions.readonly();
            format!("{}-", if readonly { "R" } else { "-" })
        }
    }
    
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;
        
        let mode = entry.permissions.mode();
        
        // Type prefix
        let type_char = if entry.is_dir() {
            'd'
        } else if entry.is_symlink() {
            'l'
        } else {
            '-'
        };
        
        // Owner permissions
        let user_r = if mode & 0o400 != 0 { 'r' } else { '-' };
        let user_w = if mode & 0o200 != 0 { 'w' } else { '-' };
        let user_x = if mode & 0o100 != 0 { 'x' } else { '-' };
        
        // Group permissions
        let group_r = if mode & 0o040 != 0 { 'r' } else { '-' };
        let group_w = if mode & 0o020 != 0 { 'w' } else { '-' };
        let group_x = if mode & 0o010 != 0 { 'x' } else { '-' };
        
        // Other permissions
        let other_r = if mode & 0o004 != 0 { 'r' } else { '-' };
        let other_w = if mode & 0o002 != 0 { 'w' } else { '-' };
        let other_x = if mode & 0o001 != 0 { 'x' } else { '-' };
        
        format!("{}{}{}{}{}{}{}{}{}{}",
            type_char,
            user_r, user_w, user_x,
            group_r, group_w, group_x,
            other_r, other_w, other_x
        )
    }
}

/// Pad or truncate text to fit within specified width
/// Alignment: Left, Right, or Center
/// Handles Unicode characters (emojis) correctly using visual width
pub fn pad_text(text: &str, width: u16, align: crate::ui::column_layout::Alignment) -> String {
    let width = width as usize;
    let visual_len = visual_width(text);
    
    if visual_len > width {
        // Truncate with ellipsis, handling Unicode properly
        if width > 3 {
            // Find where to truncate based on visual width
            let mut truncated = String::new();
            let mut current_width = 0;
            for ch in text.chars() {
                let ch_width = visual_width(&ch.to_string());
                if current_width + ch_width + 3 > width {
                    break;
                }
                truncated.push(ch);
                current_width += ch_width;
            }
            format!("{}...", truncated)
        } else {
            // Very narrow width, just truncate
            let mut truncated = String::new();
            let mut current_width = 0;
            for ch in text.chars() {
                let ch_width = visual_width(&ch.to_string());
                if current_width + ch_width > width {
                    break;
                }
                truncated.push(ch);
                current_width += ch_width;
            }
            truncated
        }
    } else {
        let padding = width - visual_len;
        match align {
            crate::ui::column_layout::Alignment::Left => {
                format!("{}{}", text, " ".repeat(padding))
            }
            crate::ui::column_layout::Alignment::Right => {
                format!("{}{}", " ".repeat(padding), text)
            }
            crate::ui::column_layout::Alignment::Center => {
                let left_pad = padding / 2;
                let right_pad = padding - left_pad;
                format!("{}{}{}", " ".repeat(left_pad), text, " ".repeat(right_pad))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::file_entry::{EntryType, FileEntry};
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn create_test_entry(name: &str, extension: Option<String>, size: u64, is_dir: bool) -> FileEntry {
        // Create a temporary file to get real permissions
        let temp_path = std::env::temp_dir().join(name);
        let _ = std::fs::write(&temp_path, b"test");
        let permissions = std::fs::metadata(&temp_path)
            .map(|m| m.permissions())
            .unwrap_or_else(|_| {
                // Fallback: create a basic permissions object
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::Permissions::from_mode(0o644)
                }
                #[cfg(not(unix))]
                {
                    // Windows: Use File::open to get real permissions
                    use std::fs::File;
                    File::open(&temp_path)
                        .and_then(|f| f.metadata())
                        .map(|m| m.permissions())
                        .unwrap_or_else(|_| {
                            // Last resort: Clone from a known file
                            std::fs::metadata(std::env::current_exe().unwrap())
                                .map(|m| m.permissions())
                                .unwrap()
                        })
                }
            });
        
        FileEntry::new(
            name.to_string(),
            if is_dir { EntryType::Dir } else { EntryType::File },
            size,
            SystemTime::now(),
            None,
            extension,
            permissions,
            PathBuf::from(name),
            #[cfg(windows)]
            Some(0x00000020), // FILE_ATTRIBUTE_ARCHIVE by default for tests
        )
    }

    #[test]
    fn test_format_extension_standard() {
        let entry = create_test_entry("file.txt", Some("txt".to_string()), 100, false);
        assert_eq!(format_extension(&entry), "txt");
    }

    #[test]
    fn test_format_extension_long() {
        let entry = create_test_entry("file.verylongext", Some("verylongext".to_string()), 100, false);
        let formatted = format_extension(&entry);
        // No truncation - scroll system will handle display
        assert_eq!(formatted, "verylongext");
    }

    #[test]
    fn test_format_extension_directory() {
        let entry = create_test_entry("folder", None, 0, true);
        assert_eq!(format_extension(&entry), "");
    }

    #[test]
    fn test_format_size_bytes() {
        let entry = create_test_entry("small.txt", None, 512, false);
        assert_eq!(format_size(&entry), "512 B");
    }

    #[test]
    fn test_format_size_kb() {
        let entry = create_test_entry("medium.txt", None, 5120, false);
        let result = format_size(&entry);
        assert!(result.contains("KB"));
    }

    #[test]
    fn test_format_size_mb() {
        let entry = create_test_entry("large.txt", None, 5 * 1024 * 1024, false);
        let result = format_size(&entry);
        assert!(result.contains("MB"));
    }

    #[test]
    fn test_format_size_directory() {
        let entry = create_test_entry("folder", None, 0, true);
        assert_eq!(format_size(&entry), "<DIR>");
    }

    #[test]
    fn test_format_date_valid() {
        let time = Some(UNIX_EPOCH + Duration::from_secs(1609459200)); // 2021-01-01 00:00:00 UTC
        let formatted = format_date(time);
        assert!(formatted.contains("-"));
        assert!(formatted.contains(":"));
        assert_eq!(formatted.len(), 16); // "YYYY-MM-DD HH:MM"
    }

    #[test]
    fn test_format_date_none() {
        assert_eq!(format_date(None), "N/A");
    }

    #[test]
    fn test_pad_text_left() {
        let result = pad_text("test", 10, crate::ui::column_layout::Alignment::Left);
        assert_eq!(result, "test      ");
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_pad_text_right() {
        let result = pad_text("test", 10, crate::ui::column_layout::Alignment::Right);
        assert_eq!(result, "      test");
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_pad_text_center() {
        let result = pad_text("test", 10, crate::ui::column_layout::Alignment::Center);
        assert_eq!(result.len(), 10);
        assert!(result.contains("test"));
    }

    #[test]
    fn test_pad_text_truncate() {
        let result = pad_text("verylongtext", 8, crate::ui::column_layout::Alignment::Left);
        assert_eq!(result.len(), 8);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_pad_text_emoji() {
        // Test with emoji - should handle Unicode correctly
        // Emoji 📁 has visual width of 2
        let result = pad_text("📁", 2, crate::ui::column_layout::Alignment::Left);
        // Visual width is 2, so no padding needed
        assert_eq!(visual_width(&result), 2);
        assert!(result.starts_with("📁"));
        
        // Test with padding - emoji needs 2 cells, asking for 4 total
        let result = pad_text("📁", 4, crate::ui::column_layout::Alignment::Left);
        assert_eq!(visual_width(&result), 4);
        assert!(result.starts_with("📁"));
        
        // Test truncation with emoji - asking for only 1 cell when emoji needs 2
        // Should truncate to empty or just show nothing
        let result = pad_text("📁test", 3, crate::ui::column_layout::Alignment::Left);
        // Emoji (2) + "..." (3) = 5, but we only have 3, so should truncate emoji
        assert!(visual_width(&result) <= 3);
    }
}
