// US4: Drive selector dialog widget
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::ui::theme::Theme;

/// Render the drive selector dialog
pub fn render(
    frame: &mut Frame,
    drives: &[(String, String)],
    selected: usize,
    theme: &Theme,
) {
    let area = centered_rect(60, 70, frame.area());
    
    // Clear the background
    frame.render_widget(Clear, area);
    
    // Create main block
    let block = Block::default()
        .title(" Select Drive ")
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
    
    // Split into header and list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Header
            Constraint::Min(3),     // Drive list
            Constraint::Length(2),  // Footer instructions
        ])
        .split(inner);
    
    // Header
    let header = Paragraph::new("Use ↑↓ to navigate, Enter to select, Esc to cancel")
        .style(Style::default().fg(theme.info_color))
        .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);
    
    // Drive list
    let items: Vec<ListItem> = drives
        .iter()
        .enumerate()
        .map(|(i, (_, label))| {
            let style = if i == selected {
                Style::default()
                    .fg(theme.highlight_fg)
                    .bg(theme.highlight_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.dialog_fg)
            };
            
            let content = if i == selected {
                format!("► {}", label)
            } else {
                format!("  {}", label)
            };
            
            ListItem::new(content).style(style)
        })
        .collect();
    
    let list = List::new(items)
        .style(Style::default().bg(theme.dialog_bg));
    
    frame.render_widget(list, chunks[1]);
    
    // Footer with count
    let footer = Paragraph::new(format!("{} drive(s) available", drives.len()))
        .style(Style::default().fg(theme.info_color))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
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
