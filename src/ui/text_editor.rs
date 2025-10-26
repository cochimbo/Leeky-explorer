// Text editor widget - TASK-027/028
use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use tui_textarea::TextArea;

use crate::ui::theme::Theme;

/// Actions that the editor can perform
#[derive(Debug, Clone, PartialEq)]
pub enum EditorAction {
    Continue,
    Save,
    Close,
    ConfirmClose,
}

/// Text editor state with tui-textarea integration
pub struct TextEditor<'a> {
    textarea: TextArea<'a>,
    file_path: PathBuf,
    original_content: String,  // Original file content for change detection
    file_mtime: Option<SystemTime>,  // File modification time when loaded
    modified: bool,
    read_only: bool,
    last_error: Option<String>,
}

impl<'a> TextEditor<'a> {
    /// Create a new text editor from a file path
    pub fn from_file(path: PathBuf, theme: &Theme) -> Result<Self> {
        // Check if file is read-only and get modification time
        let metadata = fs::metadata(&path)?;
        let read_only = metadata.permissions().readonly();
        let file_mtime = metadata.modified().ok();
        
        // Read file content
        let content = fs::read_to_string(&path)?;
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        
        // Create textarea
        let mut textarea = TextArea::new(lines);
        
        // Apply theme styling
        Self::apply_theme(&mut textarea, theme);
        
        Ok(Self {
            textarea,
            file_path: path,
            original_content: content,  // Store original for comparison
            file_mtime,  // Store modification time
            modified: false,
            read_only,
            last_error: None,
        })
    }
    
    /// Apply theme colors to textarea
    fn apply_theme(textarea: &mut TextArea, theme: &Theme) {
        // Line numbers
        textarea.set_line_number_style(Style::default().fg(theme.info_color));
        
        // No cursor line highlight - just normal background
        textarea.set_cursor_line_style(Style::default());
        
        // Cursor itself - simple block cursor
        textarea.set_cursor_style(
            Style::default()
                .fg(theme.dialog_bg)
                .bg(theme.highlight_fg)
        );
        
        // Block border
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.active_border))
                .style(Style::default().bg(theme.panel_bg).fg(theme.panel_fg))
        );
    }
    
    /// Render the text editor
    pub fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Calculate modal size: 80% of screen (like preview)
        let modal_width = (area.width as f32 * 0.8) as u16;
        let modal_height = (area.height as f32 * 0.8) as u16;
        
        // Center the modal
        let editor_area = Rect {
            x: (area.width.saturating_sub(modal_width)) / 2,
            y: (area.height.saturating_sub(modal_height)) / 2,
            width: modal_width,
            height: modal_height,
        };
        
        // Clear background behind editor (this creates the solid background effect)
        frame.render_widget(Clear, editor_area);
        
        // Split editor area into main editor and status bar
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),      // Editor
                Constraint::Length(1),   // Status bar
            ])
            .split(editor_area);
        
        // Update title with file name and status
        let file_name = self.file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");
        
        let modified_indicator = if self.modified { " [Modified]" } else { "" };
        let readonly_indicator = if self.read_only { " [Read-Only]" } else { "" };
        
        let title = format!(" Text Editor: {} {}{}", file_name, modified_indicator, readonly_indicator);
        
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.active_border))
            .style(Style::default().bg(theme.dialog_bg).fg(theme.dialog_fg));
        
        self.textarea.set_block(block);
        
        // Render textarea (tui-textarea 0.7+ can be passed directly)
        frame.render_widget(&self.textarea, chunks[0]);
        
        // Render status bar
        self.render_status_bar(frame, chunks[1], theme);
    }
    
    /// Render status bar with cursor position and keybindings
    fn render_status_bar(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let (row, col) = self.textarea.cursor();
        let line_count = self.textarea.lines().len();
        
        // Left side: cursor position
        let left_text = format!(" Line {}/{}, Col {} ", row + 1, line_count, col + 1);
        
        // Right side: keybindings
        let right_text = if self.read_only {
            " [Read-Only] Esc: Close "
        } else if self.modified {
            " Ctrl+S: Save | Esc: Close (unsaved changes) "
        } else {
            " Ctrl+S: Save | Esc: Close "
        };
        
        // Show error if any
        let status_text = if let Some(ref error) = self.last_error {
            format!(" ERROR: {} ", error)
        } else {
            format!("{}{}", left_text, right_text)
        };
        
        let style = if self.last_error.is_some() {
            Style::default()
                .bg(theme.dialog_bg)
                .fg(ratatui::style::Color::Red)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .bg(theme.dialog_bg)
                .fg(theme.info_color)
        };
        
        let status = Paragraph::new(status_text)
            .style(style)
            .alignment(Alignment::Left);
        
        frame.render_widget(status, area);
    }
    
    /// Handle keyboard input - returns EditorAction
    /// The actual key will be passed to textarea separately
    pub fn handle_key(&mut self, key_code: KeyCode, key_modifiers: KeyModifiers) -> EditorAction {
        // Clear previous errors
        self.last_error = None;
        
        match (key_code, key_modifiers) {
            // Save file
            (KeyCode::Char('s'), KeyModifiers::CONTROL) | 
            (KeyCode::Char('S'), KeyModifiers::CONTROL) => {
                if self.read_only {
                    self.last_error = Some("File is read-only".to_string());
                    EditorAction::Continue
                } else {
                    EditorAction::Save
                }
            }
            
            // Close editor
            (KeyCode::Esc, _) => {
                if self.modified && !self.read_only {
                    EditorAction::ConfirmClose
                } else {
                    EditorAction::Close
                }
            }
            
            // All other keys should be handled by caller and passed to input_key
            _ => EditorAction::Continue
        }
    }
    
    /// Pass the actual input to textarea
    /// This should be called AFTER handle_key returns Continue
    pub fn input_key(&mut self, key: ratatui::crossterm::event::KeyEvent) {
        if !self.read_only {
            self.textarea.input(key);
            // Check if content has actually changed
            self.update_modified_status();
        }
    }
    
    /// Check if current content differs from original
    fn update_modified_status(&mut self) {
        let current_content = self.textarea.lines().join("\n");
        // Normalize both sides: remove trailing newline for comparison
        let original_normalized = self.original_content.trim_end_matches('\n');
        let current_normalized = current_content.trim_end_matches('\n');
        self.modified = current_normalized != original_normalized;
    }
    
    /// Save the file
    pub fn save(&mut self) -> Result<()> {
        if self.read_only {
            anyhow::bail!("File is read-only");
        }
        
        // Check if file still exists
        if !self.file_path.exists() {
            anyhow::bail!("File has been deleted externally");
        }
        
        // Check if file was modified externally
        if self.check_external_modifications() {
            anyhow::bail!("File has been modified externally. Please reload the file.");
        }
        
        let content = self.textarea.lines().join("\n");
        fs::write(&self.file_path, &content)?;
        
        // Update mtime after successful save
        if let Ok(metadata) = fs::metadata(&self.file_path) {
            self.file_mtime = metadata.modified().ok();
        }
        
        self.original_content = content;  // Update original after save
        self.modified = false;
        Ok(())
    }
    
    /// Check if file has been modified externally since it was loaded
    pub fn check_external_modifications(&self) -> bool {
        if let Some(original_mtime) = self.file_mtime {
            if let Ok(metadata) = fs::metadata(&self.file_path) {
                if let Ok(current_mtime) = metadata.modified() {
                    return current_mtime > original_mtime;
                }
            }
        }
        false
    }
    
    /// Check if file has been modified
    pub fn is_modified(&self) -> bool {
        self.modified
    }
    
    /// Check if file is read-only
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }
    
    /// Get file path
    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }
    
    /// Get the number of lines in the editor (for testing)
    pub fn line_count(&self) -> usize {
        self.textarea.lines().len()
    }
    
    /// Get a specific line content (for testing)
    pub fn get_line(&self, index: usize) -> Option<&str> {
        self.textarea.lines().get(index).map(|s| s.as_str())
    }
    
    /// Set modified flag (for testing)
    pub fn set_modified(&mut self, modified: bool) {
        self.modified = modified;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::Theme;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_editor_creation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Line 1\nLine 2\nLine 3").unwrap();
        
        let theme = Theme::default();
        let editor = TextEditor::from_file(temp_file.path().to_path_buf(), &theme);
        
        assert!(editor.is_ok());
        let editor = editor.unwrap();
        assert_eq!(editor.textarea.lines().len(), 3);
        assert!(!editor.is_modified());
    }
    
    #[test]
    fn test_editor_save() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Original content").unwrap();
        
        let theme = Theme::default();
        let mut editor = TextEditor::from_file(temp_file.path().to_path_buf(), &theme).unwrap();
        
        // Simulate modification
        editor.modified = true;
        
        assert!(editor.is_modified());
        
        // Save
        let result = editor.save();
        assert!(result.is_ok());
        assert!(!editor.is_modified());
    }
    
    #[test]
    fn test_readonly_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        fs::write(&path, "Read-only content").unwrap();
        
        // Set read-only
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&path, perms).unwrap();
        
        let theme = Theme::default();
        let editor = TextEditor::from_file(path.clone(), &theme).unwrap();
        assert!(editor.is_read_only());
        
        // Clean up: remove read-only flag
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_readonly(false);
        fs::set_permissions(&path, perms).unwrap();
    }
}
