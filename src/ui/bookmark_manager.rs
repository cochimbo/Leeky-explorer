// TASK-006: Bookmark manager dialog widget
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::config::bookmarks::BookmarkManager;
use crate::models::bookmark::Bookmark;
use crate::ui::theme::Theme;

/// State for the bookmark manager dialog
#[derive(Debug, Clone)]
pub struct BookmarkManagerState {
    pub selected: usize,
    pub show_input: bool,
    pub input_mode: InputMode,
    pub input_value: String,
    pub selected_bookmark_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    AddBookmark,
    RenameBookmark,
}

impl BookmarkManagerState {
    pub fn new() -> Self {
        Self {
            selected: 0,
            show_input: false,
            input_mode: InputMode::AddBookmark,
            input_value: String::new(),
            selected_bookmark_name: None,
        }
    }

    /// Start adding a new bookmark
    pub fn start_add(&mut self, default_name: String) {
        self.show_input = true;
        self.input_mode = InputMode::AddBookmark;
        self.input_value = default_name;
    }

    /// Start renaming a bookmark
    pub fn start_rename(&mut self, bookmark_name: String) {
        self.show_input = true;
        self.input_mode = InputMode::RenameBookmark;
        self.input_value = bookmark_name.clone();
        self.selected_bookmark_name = Some(bookmark_name);
    }

    /// Cancel input mode
    pub fn cancel_input(&mut self) {
        self.show_input = false;
        self.input_value.clear();
        self.selected_bookmark_name = None;
    }

    /// Move selection up
    pub fn move_up(&mut self, max: usize) {
        if max > 0 && self.selected > 0 {
            self.selected = self.selected.saturating_sub(1);
        }
    }

    /// Move selection down
    pub fn move_down(&mut self, max: usize) {
        if max > 0 && self.selected < max - 1 {
            self.selected = self.selected.saturating_add(1);
        }
    }

    /// Reset selection to 0
    pub fn reset_selection(&mut self) {
        self.selected = 0;
    }
}

impl Default for BookmarkManagerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the bookmark manager dialog
pub fn render(
    frame: &mut Frame,
    bookmarks: &BookmarkManager,
    state: &BookmarkManagerState,
    theme: &Theme,
) {
    let area = centered_rect(75, 85, frame.area());
    
    // Clear the background
    frame.render_widget(Clear, area);
    
    // Create main block
    let title = if state.show_input {
        match state.input_mode {
            InputMode::AddBookmark => " Add Bookmark ",
            InputMode::RenameBookmark => " Rename Bookmark ",
        }
    } else {
        " Bookmark Manager (Ctrl+B) "
    };
    
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.active_border))
        .style(Style::default().bg(theme.dialog_bg));
    
    frame.render_widget(block, area);
    
    // Inner area for content
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };
    
    if state.show_input {
        render_input_mode(frame, inner, state, theme);
    } else {
        render_list_mode(frame, inner, bookmarks, state, theme);
    }
}

/// Render input mode for adding/renaming bookmarks
fn render_input_mode(
    frame: &mut Frame,
    area: Rect,
    state: &BookmarkManagerState,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Prompt
            Constraint::Length(3),  // Input box
            Constraint::Min(1),     // Spacer
            Constraint::Length(2),  // Footer
        ])
        .split(area);
    
    // Prompt
    let prompt_text = match state.input_mode {
        InputMode::AddBookmark => "Enter a name for this bookmark:",
        InputMode::RenameBookmark => "Enter new name:",
    };
    
    let prompt = Paragraph::new(prompt_text)
        .style(Style::default().fg(theme.dialog_fg))
        .alignment(Alignment::Left);
    frame.render_widget(prompt, chunks[0]);
    
    // Input box
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.highlight_fg));
    
    let input_text = format!("{}_", state.input_value);
    let input_paragraph = Paragraph::new(input_text)
        .style(Style::default().fg(theme.dialog_fg))
        .block(input_block);
    
    frame.render_widget(input_paragraph, chunks[1]);
    
    // Footer instructions
    let footer = Paragraph::new("Enter: Confirm | Esc: Cancel")
        .style(Style::default().fg(theme.info_color))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[3]);
}

/// Render list mode showing all bookmarks
fn render_list_mode(
    frame: &mut Frame,
    area: Rect,
    bookmarks: &BookmarkManager,
    state: &BookmarkManagerState,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Header info
            Constraint::Min(5),     // Bookmark list
            Constraint::Length(4),  // Footer instructions
        ])
        .split(area);
    
    // Header with count
    let all_bookmarks = bookmarks.get_all();
    let count_text = if all_bookmarks.is_empty() {
        "No bookmarks yet. Press 'a' to add current directory.".to_string()
    } else {
        format!("{} bookmark(s) | Use ↑↓ to navigate", all_bookmarks.len())
    };
    
    let header = Paragraph::new(count_text)
        .style(Style::default().fg(theme.info_color))
        .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);
    
    // Bookmark list
    if all_bookmarks.is_empty() {
        let empty_msg = Paragraph::new("Press 'a' to bookmark the current directory")
            .style(Style::default().fg(theme.dialog_fg))
            .alignment(Alignment::Center);
        frame.render_widget(empty_msg, chunks[1]);
    } else {
        let items: Vec<ListItem> = all_bookmarks
            .iter()
            .enumerate()
            .map(|(i, bookmark)| {
                create_bookmark_item(i, bookmark, state.selected, theme)
            })
            .collect();
        
        let list = List::new(items)
            .style(Style::default().bg(theme.dialog_bg));
        
        frame.render_widget(list, chunks[1]);
    }
    
    // Footer instructions
    let footer_text = if all_bookmarks.is_empty() {
        vec![
            Line::from(vec![
                Span::styled("a", Style::default().fg(theme.highlight_fg).add_modifier(Modifier::BOLD)),
                Span::raw(": Add | "),
                Span::styled("Esc", Style::default().fg(theme.highlight_fg).add_modifier(Modifier::BOLD)),
                Span::raw(": Close"),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(theme.highlight_fg).add_modifier(Modifier::BOLD)),
                Span::raw(": Go | "),
                Span::styled("a", Style::default().fg(theme.highlight_fg).add_modifier(Modifier::BOLD)),
                Span::raw(": Add | "),
                Span::styled("r", Style::default().fg(theme.highlight_fg).add_modifier(Modifier::BOLD)),
                Span::raw(": Rename | "),
                Span::styled("d", Style::default().fg(theme.highlight_fg).add_modifier(Modifier::BOLD)),
                Span::raw(": Delete"),
            ]),
            Line::from(vec![
                Span::styled("c", Style::default().fg(theme.highlight_fg).add_modifier(Modifier::BOLD)),
                Span::raw(": Clean invalid | "),
                Span::styled("Esc", Style::default().fg(theme.highlight_fg).add_modifier(Modifier::BOLD)),
                Span::raw(": Close"),
            ]),
        ]
    };
    
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(theme.dialog_fg))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

/// Create a list item for a bookmark
fn create_bookmark_item(
    index: usize,
    bookmark: &Bookmark,
    selected: usize,
    theme: &Theme,
) -> ListItem<'static> {
    let is_selected = index == selected;
    let exists = bookmark.path_exists();
    
    let mut spans = vec![];
    
    // Selection indicator
    if is_selected {
        spans.push(Span::styled(
            "► ",
            Style::default()
                .fg(theme.highlight_fg)
                .add_modifier(Modifier::BOLD),
        ));
    } else {
        spans.push(Span::raw("  "));
    }
    
    // Bookmark name (bold if selected)
    let name_style = if is_selected {
        Style::default()
            .fg(theme.highlight_fg)
            .add_modifier(Modifier::BOLD)
    } else if !exists {
        Style::default().fg(theme.error_color)
    } else {
        Style::default().fg(theme.dialog_fg)
    };
    
    spans.push(Span::styled(
        format!("{:<25}", bookmark.name),
        name_style,
    ));
    
    // Path with existence indicator
    let path_display = bookmark.path.display().to_string();
    let path_text = if !exists {
        format!("{} [PATH NOT FOUND]", path_display)
    } else {
        path_display
    };
    
    let path_style = if is_selected {
        Style::default().fg(theme.info_color)
    } else if !exists {
        Style::default().fg(theme.error_color)
    } else {
        Style::default().fg(theme.info_color)
    };
    
    spans.push(Span::styled(path_text, path_style));
    
    let line = Line::from(spans);
    
    let item_style = if is_selected {
        Style::default().bg(theme.marked_bg)
    } else {
        Style::default()
    };
    
    ListItem::new(line).style(item_style)
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_creation() {
        let state = BookmarkManagerState::new();
        assert_eq!(state.selected, 0);
        assert!(!state.show_input);
        assert!(state.input_value.is_empty());
    }

    #[test]
    fn test_start_add() {
        let mut state = BookmarkManagerState::new();
        state.start_add("TestName".to_string());
        
        assert!(state.show_input);
        assert_eq!(state.input_mode, InputMode::AddBookmark);
        assert_eq!(state.input_value, "TestName");
    }

    #[test]
    fn test_start_rename() {
        let mut state = BookmarkManagerState::new();
        state.start_rename("OldName".to_string());
        
        assert!(state.show_input);
        assert_eq!(state.input_mode, InputMode::RenameBookmark);
        assert_eq!(state.input_value, "OldName");
        assert_eq!(state.selected_bookmark_name, Some("OldName".to_string()));
    }

    #[test]
    fn test_cancel_input() {
        let mut state = BookmarkManagerState::new();
        state.start_add("Test".to_string());
        state.cancel_input();
        
        assert!(!state.show_input);
        assert!(state.input_value.is_empty());
        assert!(state.selected_bookmark_name.is_none());
    }

    #[test]
    fn test_move_up() {
        let mut state = BookmarkManagerState::new();
        state.selected = 5;
        state.move_up(10);
        assert_eq!(state.selected, 4);
        
        state.selected = 0;
        state.move_up(10);
        assert_eq!(state.selected, 0); // Shouldn't go below 0
    }

    #[test]
    fn test_move_down() {
        let mut state = BookmarkManagerState::new();
        state.move_down(10);
        assert_eq!(state.selected, 1);
        
        state.selected = 9;
        state.move_down(10);
        assert_eq!(state.selected, 9); // Shouldn't exceed max-1
    }

    #[test]
    fn test_reset_selection() {
        let mut state = BookmarkManagerState::new();
        state.selected = 5;
        state.reset_selection();
        assert_eq!(state.selected, 0);
    }
}
