// Theme and colors
use crate::models::file_entry::EntryType;
use ratatui::style::{Color, Style};

// Border colors
pub const ACTIVE_BORDER: Color = Color::Cyan;
pub const INACTIVE_BORDER: Color = Color::Gray;

// Highlight colors
pub const HIGHLIGHT_BG: Color = Color::Blue;
pub const HIGHLIGHT_FG: Color = Color::White;

// Entry type colors
pub const DIR_COLOR: Color = Color::Blue;
pub const FILE_COLOR: Color = Color::White;
pub const SYMLINK_COLOR: Color = Color::Cyan;
pub const EXECUTABLE_COLOR: Color = Color::Green;

// Footer colors
pub const FOOTER_BG: Color = Color::DarkGray;
pub const FOOTER_FG: Color = Color::White;

// Error/warning colors
pub const ERROR: Color = Color::Red;
pub const WARNING: Color = Color::Yellow;

// Selection colors (T559)
pub const MARKED_BG: Color = Color::DarkGray;

pub fn get_entry_style(entry_type: &EntryType) -> Style {
    match entry_type {
        EntryType::Dir => Style::default().fg(DIR_COLOR),
        EntryType::File => Style::default().fg(FILE_COLOR),
        EntryType::Symlink => Style::default().fg(SYMLINK_COLOR),
    }
}
