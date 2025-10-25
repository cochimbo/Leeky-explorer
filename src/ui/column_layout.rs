// Column layout calculator for detailed file view
use crate::models::file_entry::FileEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone)]
pub struct ColumnLayout {
    pub icon_width: u16,
    pub mark_width: u16,
    pub name_width: u16,
    pub ext_width: u16,
    pub size_width: u16,
    pub modified_width: u16,
    pub created_width: u16,
    pub perms_width: u16,
    pub show_extension: bool,
    pub show_created: bool,
    pub show_permissions: bool,
}

impl ColumnLayout {
    /// Calculate column widths based on available terminal width
    /// Minimum width: 80 columns (icon, mark, name, size, modified)
    /// Comfortable width: 120 columns (all columns visible)
    pub fn calculate(available_width: u16, _entries: &[FileEntry]) -> Self {
        // Fixed column widths
        let icon_width = 2;  // Emoji takes exactly 2 visual cells
        let mark_width = 1;
        let ext_width = 5;   // Reduced from 8 - most extensions are 3-4 chars (.rs, .txt, .json)
        let size_width = 10;
        let modified_width = 16; // "YYYY-MM-DD HH:MM"
        let created_width = 16;
        
        // Platform-specific permissions width
        #[cfg(windows)]
        let perms_width = 6; // "RHSA  " with padding
        
        #[cfg(not(windows))]
        let perms_width = 10; // "drwxr-xr-x"

        // Column separators (spaces between columns)
        let separator_count = 7; // Between 8 columns
        let separator_width = separator_count * 2; // 2 spaces between each column

        // Determine which columns to show based on available width
        let show_extension = available_width >= 100;
        let show_created = available_width >= 120;
        let show_permissions = available_width >= 120;

        // Calculate fixed columns width
        let mut fixed_width = icon_width + mark_width + size_width + modified_width + separator_width;
        
        if show_extension {
            fixed_width += ext_width + 2;
        }
        if show_created {
            fixed_width += created_width + 2;
        }
        if show_permissions {
            fixed_width += perms_width + 2;
        }

        // Name column gets remaining space
        let name_width = if available_width > fixed_width {
            available_width - fixed_width
        } else {
            // Fallback: minimum name width
            20
        };

        Self {
            icon_width,
            mark_width,
            name_width,
            ext_width,
            size_width,
            modified_width,
            created_width,
            perms_width,
            show_extension,
            show_created,
            show_permissions,
        }
    }

    /// Get alignment for a specific column
    pub fn get_alignment(&self, column: &str) -> Alignment {
        match column {
            "icon" | "mark" | "name" | "ext" => Alignment::Left,
            "size" => Alignment::Right,
            "modified" | "created" | "perms" => Alignment::Center,
            _ => Alignment::Left,
        }
    }

    /// Total width used by all visible columns (including separators)
    pub fn total_width(&self) -> u16 {
        let mut width = self.icon_width + self.mark_width + self.name_width + self.size_width + self.modified_width;
        
        // Count separators (2 spaces between columns)
        let mut separator_count = 4; // icon|mark, mark|name, name|size, size|modified
        
        if self.show_extension {
            width += self.ext_width;
            separator_count += 1;
        }
        if self.show_created {
            width += self.created_width;
            separator_count += 1;
        }
        if self.show_permissions {
            width += self.perms_width;
            separator_count += 1;
        }
        
        width + (separator_count * 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_minimum_width() {
        // 80 columns - minimum viable
        let layout = ColumnLayout::calculate(80, &[]);
        assert_eq!(layout.icon_width, 2);
        assert_eq!(layout.mark_width, 1);
        assert!(layout.name_width >= 20);
        assert!(!layout.show_extension);
        assert!(!layout.show_created);
        assert!(!layout.show_permissions);
    }

    #[test]
    fn test_calculate_comfortable_width() {
        // 120 columns - all columns visible
        let layout = ColumnLayout::calculate(120, &[]);
        assert!(layout.show_extension);
        assert!(layout.show_created);
        assert!(layout.show_permissions);
        assert!(layout.name_width >= 20);
    }

    #[test]
    fn test_calculate_wide_width() {
        // 200 columns - lots of space for name
        let layout = ColumnLayout::calculate(200, &[]);
        assert!(layout.show_extension);
        assert!(layout.show_created);
        assert!(layout.show_permissions);
        assert!(layout.name_width > 50);
    }

    #[test]
    fn test_get_alignment() {
        let layout = ColumnLayout::calculate(120, &[]);
        assert_eq!(layout.get_alignment("icon"), Alignment::Left);
        assert_eq!(layout.get_alignment("name"), Alignment::Left);
        assert_eq!(layout.get_alignment("size"), Alignment::Right);
        assert_eq!(layout.get_alignment("modified"), Alignment::Center);
    }

    #[test]
    fn test_total_width_calculation() {
        let layout = ColumnLayout::calculate(120, &[]);
        let total = layout.total_width();
        // Total should be reasonable and not exceed available width
        assert!(total > 0);
        assert!(total <= 120);
    }
}
