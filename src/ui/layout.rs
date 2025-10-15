// Layout functions
use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub header: Rect,
    pub left_panel: Rect,
    pub right_panel: Rect,
    pub footer: Rect,
}

pub fn create_layout(area: Rect) -> AppLayout {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Panels
            Constraint::Length(3),  // Footer (2 lines now)
        ])
        .split(area);

    let panel_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Left panel
            Constraint::Percentage(50), // Right panel
        ])
        .split(vertical_chunks[1]);

    AppLayout {
        header: vertical_chunks[0],
        left_panel: panel_chunks[0],
        right_panel: panel_chunks[1],
        footer: vertical_chunks[2],
    }
}
