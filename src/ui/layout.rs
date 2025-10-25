// Layout functions
use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub left_header: Rect,
    pub right_header: Rect,
    pub left_panel: Rect,
    pub right_panel: Rect,
    pub footer: Rect,
}

pub fn create_layout(area: Rect) -> AppLayout {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Headers
            Constraint::Min(10),    // Panels
            Constraint::Length(3),  // Footer (2 lines now)
        ])
        .split(area);

    // Split header area into two columns matching panels
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Left header
            Constraint::Percentage(50), // Right header
        ])
        .split(vertical_chunks[0]);

    let panel_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Left panel
            Constraint::Percentage(50), // Right panel
        ])
        .split(vertical_chunks[1]);

    AppLayout {
        left_header: header_chunks[0],
        right_header: header_chunks[1],
        left_panel: panel_chunks[0],
        right_panel: panel_chunks[1],
        footer: vertical_chunks[2],
    }
}
