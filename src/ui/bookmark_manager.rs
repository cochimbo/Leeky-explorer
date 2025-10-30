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
use crate::ui::utils::SelectableState;

/// State for the bookmark manager dialog
#[derive(Debug, Clone)]
pub struct BookmarkManagerState {
    pub selected: usize,
}

impl BookmarkManagerState {
    pub fn new() -> Self {
        Self {
            selected: 0,
        }
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

impl SelectableState for BookmarkManagerState {
    fn selected(&self) -> usize {
        self.selected
    }

    fn set_selected(&mut self, index: usize) {
        self.selected = index;
    }

    fn move_up(&mut self, max_items: usize) {
        if max_items > 0 && self.selected > 0 {
            self.selected = self.selected.saturating_sub(1);
        }
    }

    fn move_down(&mut self, max_items: usize) {
        if max_items > 0 && self.selected < max_items - 1 {
            self.selected = self.selected.saturating_add(1);
        }
    }

    fn reset_selection(&mut self) {
        self.selected = 0;
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
    let block = Block::default()
        .title(" Bookmark Manager (Ctrl+B) ")
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
    
    render_list_mode(frame, inner, bookmarks, state, theme);
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
    
    // Bookmark name
    let name_style = if is_selected {
        Style::default()
            .fg(theme.highlight_fg)
            .add_modifier(Modifier::BOLD)
    } else if !exists {
        Style::default()
            .fg(theme.error_color)
            .add_modifier(Modifier::DIM)
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
        format!("{} [INVALID]", path_display)
    } else {
        path_display
    };
    
    let path_style = if is_selected {
        Style::default().fg(theme.info_color)
    } else if !exists {
        Style::default()
            .fg(theme.error_color)
            .add_modifier(Modifier::DIM)
    } else {
        Style::default()
            .fg(theme.info_color)
            .add_modifier(Modifier::DIM)
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
