// US5: Theme selector dialog widget
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::ui::theme::Theme;

/// Render the theme selector dialog
pub fn render(
    frame: &mut Frame,
    themes: &[Theme],
    selected: usize,
    active_theme: &Theme,
) {
    let area = centered_rect(70, 80, frame.area());
    
    // Clear the background
    frame.render_widget(Clear, area);
    
    // Create main block - use active theme colors
    let block = Block::default()
        .title(" Select Theme ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(active_theme.active_border))
        .style(Style::default().bg(active_theme.dialog_bg));
    
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
            Constraint::Min(5),     // Theme list with previews
            Constraint::Length(2),  // Footer instructions
        ])
        .split(inner);
    
    // Header
    let header = Paragraph::new("Use ↑↓ to navigate, Enter to apply theme, Esc to cancel")
        .style(Style::default().fg(active_theme.info_color))
        .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);
    
    // Theme list with color previews
    let items: Vec<ListItem> = themes
        .iter()
        .enumerate()
        .map(|(i, theme)| {
            let is_selected = i == selected;
            
            // Build preview line with color squares
            let mut spans = vec![];
            
            // Selection indicator
            if is_selected {
                spans.push(Span::styled("►", Style::default().fg(active_theme.highlight_fg).add_modifier(Modifier::BOLD)));
            } else {
                spans.push(Span::raw(" "));
            }
            
            spans.push(Span::raw(" "));
            
            // Theme name
            let name_style = if is_selected {
                Style::default().fg(active_theme.highlight_fg).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(active_theme.dialog_fg)
            };
            spans.push(Span::styled(format!("{:<18}", theme.name), name_style));
            
            // Color preview squares
            spans.push(Span::raw(" ["));
            spans.push(Span::styled("██", Style::default().fg(theme.active_border)));
            spans.push(Span::raw("|"));
            spans.push(Span::styled("██", Style::default().fg(theme.dir_color)));
            spans.push(Span::raw("|"));
            spans.push(Span::styled("██", Style::default().fg(theme.file_color)));
            spans.push(Span::raw("|"));
            spans.push(Span::styled("██", Style::default().fg(theme.highlight_bg)));
            spans.push(Span::raw("]"));
            
            let line = Line::from(spans);
            
            let item_style = if is_selected {
                Style::default().bg(active_theme.marked_bg)
            } else {
                Style::default()
            };
            
            ListItem::new(line).style(item_style)
        })
        .collect();
    
    let list = List::new(items)
        .style(Style::default().bg(active_theme.dialog_bg));
    
    frame.render_widget(list, chunks[1]);
    
    // Footer with legend
    let footer_text = "Preview: Border | Dir | File | Highlight";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(active_theme.info_color))
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
