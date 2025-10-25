// Theme system for customizable color schemes
use crate::models::file_entry::EntryType;
use ratatui::style::{Color, Style};

/// Complete theme definition with all color properties
/// Note: Theme is not serialized directly - only theme name is saved to config
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    
    // Panel colors
    pub panel_bg: Color,
    pub panel_fg: Color,
    
    // Border colors
    pub active_border: Color,
    pub inactive_border: Color,
    
    // Highlight colors
    pub highlight_bg: Color,
    pub highlight_fg: Color,
    
    // Entry type colors
    pub dir_color: Color,
    pub file_color: Color,
    pub symlink_color: Color,
    pub executable_color: Color,
    
    // Selection colors
    pub marked_bg: Color,
    
    // Footer colors
    pub footer_bg: Color,
    pub footer_fg: Color,
    
    // Dialog colors
    pub dialog_bg: Color,
    pub dialog_fg: Color,
    
    // Status colors
    pub error_color: Color,
    pub warning_color: Color,
    pub info_color: Color,
}

impl Theme {
    /// Get style for entry based on type
    pub fn get_entry_style(&self, entry_type: &EntryType) -> Style {
        match entry_type {
            EntryType::Dir => Style::default().fg(self.dir_color),
            EntryType::File => Style::default().fg(self.file_color),
            EntryType::Symlink => Style::default().fg(self.symlink_color),
        }
    }
    
    /// Classic theme (current default color scheme)
    pub fn classic() -> Self {
        Self {
            name: "Classic".to_string(),
            panel_bg: Color::Black,
            panel_fg: Color::White,
            active_border: Color::Cyan,
            inactive_border: Color::Gray,
            highlight_bg: Color::Blue,
            highlight_fg: Color::White,
            dir_color: Color::Blue,
            file_color: Color::White,
            symlink_color: Color::Cyan,
            executable_color: Color::Green,
            marked_bg: Color::DarkGray,
            footer_bg: Color::DarkGray,
            footer_fg: Color::White,
            dialog_bg: Color::Black,
            dialog_fg: Color::White,
            error_color: Color::Red,
            warning_color: Color::Yellow,
            info_color: Color::Cyan,
        }
    }
    
    /// Light theme for light terminal backgrounds
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            panel_bg: Color::Rgb(240, 240, 240), // Very light gray
            panel_fg: Color::Black,
            active_border: Color::Rgb(0, 100, 150), // Dark cyan
            inactive_border: Color::Rgb(150, 150, 150), // Medium gray
            highlight_bg: Color::Rgb(200, 220, 255), // Light blue
            highlight_fg: Color::Black,
            dir_color: Color::Rgb(0, 0, 200), // Dark blue
            file_color: Color::Black,
            symlink_color: Color::Rgb(0, 150, 150), // Teal
            executable_color: Color::Rgb(0, 150, 0), // Dark green
            marked_bg: Color::Rgb(220, 220, 220), // Light gray
            footer_bg: Color::Rgb(200, 200, 200),
            footer_fg: Color::Black,
            dialog_bg: Color::Rgb(250, 250, 250),
            dialog_fg: Color::Black,
            error_color: Color::Rgb(200, 0, 0), // Dark red
            warning_color: Color::Rgb(200, 150, 0), // Dark yellow
            info_color: Color::Rgb(0, 100, 150), // Dark cyan
        }
    }
    
    /// Enhanced dark theme
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            panel_bg: Color::Rgb(30, 30, 30), // Dark gray
            panel_fg: Color::Rgb(220, 220, 220), // Light gray
            active_border: Color::Rgb(100, 200, 255), // Bright cyan
            inactive_border: Color::Rgb(80, 80, 80), // Dark gray
            highlight_bg: Color::Rgb(60, 100, 180), // Medium blue
            highlight_fg: Color::White,
            dir_color: Color::Rgb(100, 150, 255), // Light blue
            file_color: Color::Rgb(220, 220, 220), // Light gray
            symlink_color: Color::Rgb(100, 200, 200), // Cyan
            executable_color: Color::Rgb(100, 200, 100), // Light green
            marked_bg: Color::Rgb(60, 60, 60), // Medium dark gray
            footer_bg: Color::Rgb(50, 50, 50),
            footer_fg: Color::Rgb(200, 200, 200),
            dialog_bg: Color::Rgb(40, 40, 40),
            dialog_fg: Color::Rgb(220, 220, 220),
            error_color: Color::Rgb(255, 100, 100), // Light red
            warning_color: Color::Rgb(255, 200, 100), // Light yellow
            info_color: Color::Rgb(100, 200, 255), // Light cyan
        }
    }
    
    /// High contrast theme (black and white only)
    pub fn high_contrast() -> Self {
        Self {
            name: "High Contrast".to_string(),
            panel_bg: Color::Black,
            panel_fg: Color::White,
            active_border: Color::White,
            inactive_border: Color::Rgb(128, 128, 128), // Medium gray
            highlight_bg: Color::White,
            highlight_fg: Color::Black,
            dir_color: Color::White,
            file_color: Color::White,
            symlink_color: Color::White,
            executable_color: Color::White,
            marked_bg: Color::Rgb(80, 80, 80), // Dark gray
            footer_bg: Color::White,
            footer_fg: Color::Black,
            dialog_bg: Color::Black,
            dialog_fg: Color::White,
            error_color: Color::White,
            warning_color: Color::White,
            info_color: Color::White,
        }
    }
    
    /// Nord theme (blue-gray aesthetic)
    pub fn nord() -> Self {
        Self {
            name: "Nord".to_string(),
            panel_bg: Color::Rgb(46, 52, 64), // Nord0
            panel_fg: Color::Rgb(216, 222, 233), // Nord4
            active_border: Color::Rgb(136, 192, 208), // Nord8
            inactive_border: Color::Rgb(76, 86, 106), // Nord2
            highlight_bg: Color::Rgb(94, 129, 172), // Nord10
            highlight_fg: Color::Rgb(236, 239, 244), // Nord6
            dir_color: Color::Rgb(136, 192, 208), // Nord8
            file_color: Color::Rgb(216, 222, 233), // Nord4
            symlink_color: Color::Rgb(143, 188, 187), // Nord7
            executable_color: Color::Rgb(163, 190, 140), // Nord14
            marked_bg: Color::Rgb(59, 66, 82), // Nord1
            footer_bg: Color::Rgb(67, 76, 94), // Nord2
            footer_fg: Color::Rgb(216, 222, 233), // Nord4
            dialog_bg: Color::Rgb(46, 52, 64), // Nord0
            dialog_fg: Color::Rgb(216, 222, 233), // Nord4
            error_color: Color::Rgb(191, 97, 106), // Nord11
            warning_color: Color::Rgb(235, 203, 139), // Nord13
            info_color: Color::Rgb(136, 192, 208), // Nord8
        }
    }
    
    /// Dracula theme (purple and pink accents)
    pub fn dracula() -> Self {
        Self {
            name: "Dracula".to_string(),
            panel_bg: Color::Rgb(40, 42, 54), // Background
            panel_fg: Color::Rgb(248, 248, 242), // Foreground
            active_border: Color::Rgb(139, 233, 253), // Cyan
            inactive_border: Color::Rgb(68, 71, 90), // Current Line
            highlight_bg: Color::Rgb(98, 114, 164), // Comment
            highlight_fg: Color::Rgb(248, 248, 242), // Foreground
            dir_color: Color::Rgb(189, 147, 249), // Purple
            file_color: Color::Rgb(248, 248, 242), // Foreground
            symlink_color: Color::Rgb(139, 233, 253), // Cyan
            executable_color: Color::Rgb(80, 250, 123), // Green
            marked_bg: Color::Rgb(68, 71, 90), // Current Line
            footer_bg: Color::Rgb(68, 71, 90),
            footer_fg: Color::Rgb(248, 248, 242),
            dialog_bg: Color::Rgb(40, 42, 54),
            dialog_fg: Color::Rgb(248, 248, 242),
            error_color: Color::Rgb(255, 85, 85), // Red
            warning_color: Color::Rgb(241, 250, 140), // Yellow
            info_color: Color::Rgb(139, 233, 253), // Cyan
        }
    }
    
    /// Solarized Dark theme
    pub fn solarized_dark() -> Self {
        Self {
            name: "Solarized Dark".to_string(),
            panel_bg: Color::Rgb(0, 43, 54), // Base03
            panel_fg: Color::Rgb(131, 148, 150), // Base0
            active_border: Color::Rgb(42, 161, 152), // Cyan
            inactive_border: Color::Rgb(7, 54, 66), // Base02
            highlight_bg: Color::Rgb(38, 139, 210), // Blue
            highlight_fg: Color::Rgb(253, 246, 227), // Base3
            dir_color: Color::Rgb(38, 139, 210), // Blue
            file_color: Color::Rgb(131, 148, 150), // Base0
            symlink_color: Color::Rgb(42, 161, 152), // Cyan
            executable_color: Color::Rgb(133, 153, 0), // Green
            marked_bg: Color::Rgb(7, 54, 66), // Base02
            footer_bg: Color::Rgb(7, 54, 66),
            footer_fg: Color::Rgb(131, 148, 150),
            dialog_bg: Color::Rgb(0, 43, 54),
            dialog_fg: Color::Rgb(131, 148, 150),
            error_color: Color::Rgb(220, 50, 47), // Red
            warning_color: Color::Rgb(181, 137, 0), // Yellow
            info_color: Color::Rgb(42, 161, 152), // Cyan
        }
    }
    
    /// Solarized Light theme
    pub fn solarized_light() -> Self {
        Self {
            name: "Solarized Light".to_string(),
            panel_bg: Color::Rgb(253, 246, 227), // Base3
            panel_fg: Color::Rgb(101, 123, 131), // Base00
            active_border: Color::Rgb(42, 161, 152), // Cyan
            inactive_border: Color::Rgb(238, 232, 213), // Base2
            highlight_bg: Color::Rgb(38, 139, 210), // Blue
            highlight_fg: Color::Rgb(253, 246, 227), // Base3
            dir_color: Color::Rgb(38, 139, 210), // Blue
            file_color: Color::Rgb(101, 123, 131), // Base00
            symlink_color: Color::Rgb(42, 161, 152), // Cyan
            executable_color: Color::Rgb(133, 153, 0), // Green
            marked_bg: Color::Rgb(238, 232, 213), // Base2
            footer_bg: Color::Rgb(238, 232, 213),
            footer_fg: Color::Rgb(101, 123, 131),
            dialog_bg: Color::Rgb(253, 246, 227),
            dialog_fg: Color::Rgb(101, 123, 131),
            error_color: Color::Rgb(220, 50, 47), // Red
            warning_color: Color::Rgb(181, 137, 0), // Yellow
            info_color: Color::Rgb(42, 161, 152), // Cyan
        }
    }
    
    /// Get all built-in themes
    pub fn all_themes() -> Vec<Theme> {
        vec![
            Self::classic(),
            Self::light(),
            Self::dark(),
            Self::high_contrast(),
            Self::nord(),
            Self::dracula(),
            Self::solarized_dark(),
            Self::solarized_light(),
        ]
    }
    
    /// Get theme by name (case-insensitive)
    pub fn by_name(name: &str) -> Option<Theme> {
        Self::all_themes()
            .into_iter()
            .find(|t| t.name.eq_ignore_ascii_case(name))
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::classic()
    }
}

// Backward compatibility: Export constants for current classic theme
// These will be deprecated once all code uses the Theme struct
pub const ACTIVE_BORDER: Color = Color::Cyan;
pub const INACTIVE_BORDER: Color = Color::Gray;
pub const HIGHLIGHT_BG: Color = Color::Blue;
pub const HIGHLIGHT_FG: Color = Color::White;
pub const DIR_COLOR: Color = Color::Blue;
pub const FILE_COLOR: Color = Color::White;
pub const SYMLINK_COLOR: Color = Color::Cyan;
pub const EXECUTABLE_COLOR: Color = Color::Green;
pub const FOOTER_BG: Color = Color::DarkGray;
pub const FOOTER_FG: Color = Color::White;
pub const ERROR: Color = Color::Red;
pub const WARNING: Color = Color::Yellow;
pub const MARKED_BG: Color = Color::DarkGray;

/// Legacy function for backward compatibility
pub fn get_entry_style(entry_type: &EntryType) -> Style {
    Theme::default().get_entry_style(entry_type)
}
