// Navigation history dialog widget
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::models::panel::Panel;
use crate::ui::theme::Theme;
use crate::ui::utils::{centered_rect, create_bordered_block, SelectableState};

/// State for history dialog
#[derive(Debug, Clone)]
pub struct HistoryDialogState {
    pub selected: usize,
}

impl Default for HistoryDialogState {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryDialogState {
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    pub fn move_up(&mut self, count: usize) {
        if count > 0 && self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self, count: usize) {
        if count > 0 && self.selected + 1 < count {
            self.selected += 1;
        }
    }

    pub fn reset_selection(&mut self) {
        self.selected = 0;
    }
}

impl SelectableState for HistoryDialogState {
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
        if max_items > 0 && self.selected + 1 < max_items {
            self.selected += 1;
        }
    }

    fn reset_selection(&mut self) {
        self.selected = 0;
    }
}

/// Render the navigation history dialog
pub fn render(
    frame: &mut Frame,
    panel: &Panel,
    state: &HistoryDialogState,
    theme: &Theme,
) {
    let area = centered_rect(70, 60, frame.area());
    
    // Clear the background
    frame.render_widget(Clear, area);
    
    // Create main block
    let block = create_bordered_block(Some(" Navigation History (Ctrl+H) "), theme.active_border, Some(theme.dialog_bg))
        .title_alignment(Alignment::Center);
    
    frame.render_widget(block, area);
    
    // Inner area for content
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };
    
    // Split into header, list, and footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Header
            Constraint::Min(3),     // History list
            Constraint::Length(2),  // Footer instructions
        ])
        .split(inner);
    
    // Header
    let history_count = panel.history.count();
    let header_text = if history_count > 0 {
        format!("Last {} visited directories (most recent at top)", history_count)
    } else {
        "No history available".to_string()
    };
    
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(theme.info_color))
        .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);
    
    // History list (reversed so most recent is at top)
    let history_entries = panel.history.get_all();
    
    if history_entries.is_empty() {
        // Show empty message
        let empty_msg = Paragraph::new("Navigate through directories to build history...")
            .style(Style::default().fg(theme.info_color))
            .alignment(Alignment::Center);
        frame.render_widget(empty_msg, chunks[1]);
    } else {
        let items: Vec<ListItem> = history_entries
            .iter()
            .rev() // Reverse to show most recent first
            .enumerate()
            .map(|(i, path)| {
                let is_selected = i == state.selected;
                let style = if is_selected {
                    Style::default()
                        .fg(theme.highlight_fg)
                        .bg(theme.highlight_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.dialog_fg)
                };
                
                let prefix = if is_selected { "► " } else { "  " };
                
                // Check if path still exists
                let exists = path.exists();
                let path_display = path.display().to_string();
                
                let line = if exists {
                    Line::from(vec![
                        Span::styled(prefix, style),
                        Span::styled(path_display, style),
                    ])
                } else {
                    // Show invalid paths in dim style
                    Line::from(vec![
                        Span::styled(prefix, style),
                        Span::styled(path_display, style.add_modifier(Modifier::DIM)),
                        Span::styled(" [INVALID]", Style::default().fg(Color::Red)),
                    ])
                };
                
                ListItem::new(line).style(style)
            })
            .collect();
        
        let list = List::new(items)
            .style(Style::default().bg(theme.dialog_bg));
        
        frame.render_widget(list, chunks[1]);
    }
    
    // Footer with instructions
    let footer = Paragraph::new("↑↓: Navigate | Enter: Go | c: Clean invalid | Esc: Cancel")
        .style(Style::default().fg(theme.info_color))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_creation() {
        let state = HistoryDialogState::new();
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_move_up() {
        let mut state = HistoryDialogState::new();
        state.selected = 5;
        state.move_up(10);
        assert_eq!(state.selected, 4);
        
        state.move_up(10);
        assert_eq!(state.selected, 3);
        
        // Can't go below 0
        state.selected = 0;
        state.move_up(10);
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_move_down() {
        let mut state = HistoryDialogState::new();
        state.move_down(10);
        assert_eq!(state.selected, 1);
        
        state.move_down(10);
        assert_eq!(state.selected, 2);
        
        // Can't go above count - 1
        state.selected = 9;
        state.move_down(10);
        assert_eq!(state.selected, 9);
    }

    #[test]
    fn test_reset_selection() {
        let mut state = HistoryDialogState::new();
        state.selected = 5;
        state.reset_selection();
        assert_eq!(state.selected, 0);
    }
}
