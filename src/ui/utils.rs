// UI utility functions shared across modules
use ratatui::layout::Rect;
use ratatui::widgets::Block;
use ratatui::style::{Style, Color};
use crate::ui::theme::Theme;

/// Create a centered rectangle with given percentage dimensions
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
            ratatui::layout::Constraint::Percentage(percent_y),
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
            ratatui::layout::Constraint::Percentage(percent_x),
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Trait for dialog states that manage selection
pub trait SelectableState {
    fn selected(&self) -> usize;
    fn set_selected(&mut self, index: usize);
    fn move_up(&mut self, max_items: usize);
    fn move_down(&mut self, max_items: usize);
    fn reset_selection(&mut self);
}

/// Generic implementation for structs with a `selected: usize` field
pub trait SelectableStateImpl: SelectableState {
    fn selected_mut(&mut self) -> &mut usize;
}

impl<T: SelectableStateImpl> SelectableState for T {
    fn selected(&self) -> usize {
        // This would need to be implemented by each struct
        // For now, we'll provide concrete implementations
        0
    }

    fn set_selected(&mut self, index: usize) {
        *self.selected_mut() = index;
    }

    fn move_up(&mut self, max_items: usize) {
        if max_items > 0 && *self.selected_mut() > 0 {
            *self.selected_mut() -= 1;
        }
    }

    fn move_down(&mut self, max_items: usize) {
        if max_items > 0 && *self.selected_mut() < max_items - 1 {
            *self.selected_mut() += 1;
        }
    }

    fn reset_selection(&mut self) {
        *self.selected_mut() = 0;
    }
}

/// Create a standard dialog block with borders and title
pub fn create_dialog_block<'a>(title: &'a str, theme: &Theme) -> Block<'a> {
    Block::default()
        .borders(ratatui::widgets::Borders::ALL)
        .title(title)
        .style(Style::default().bg(theme.dialog_bg).fg(theme.dialog_fg))
}

/// Create a bordered block with custom style
pub fn create_bordered_block<'a>(title: Option<&'a str>, border_color: Color, background_color: Option<Color>) -> Block<'a> {
    let mut block = Block::default()
        .borders(ratatui::widgets::Borders::ALL)
        .border_style(Style::default().fg(border_color));

    if let Some(title) = title {
        block = block.title(title);
    }

    if let Some(bg_color) = background_color {
        block = block.style(Style::default().bg(bg_color));
    }

    block
}

/// Create a simple block with just borders
pub fn create_simple_block() -> Block<'static> {
    Block::default().borders(ratatui::widgets::Borders::ALL)
}