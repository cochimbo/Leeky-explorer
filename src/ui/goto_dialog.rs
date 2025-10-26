// Go To Path dialog widget - TASK-022
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::ui::theme::Theme;

/// Render the Go To Path dialog
pub fn render(
    frame: &mut Frame,
    input: &str,
    error: &Option<String>,
    theme: &Theme,
) {
    let area = centered_rect(60, 30, frame.area());
    
    // Clear the background
    frame.render_widget(Clear, area);
    
    // Create main block
    let block = Block::default()
        .title(" Go To Path (Ctrl+G) ")
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
    
    // Split into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Header text
            Constraint::Length(1),  // Spacing
            Constraint::Length(3),  // Input box
            Constraint::Length(3),  // Error message area
            Constraint::Min(1),     // Spacer
            Constraint::Length(2),  // Footer instructions
        ])
        .split(inner);
    
    // Header text
    let header = Paragraph::new("Enter the directory path to navigate to:")
        .style(Style::default().fg(theme.info_color))
        .alignment(Alignment::Left);
    frame.render_widget(header, chunks[0]);
    
    // Input box with current input
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.highlight_bg))
        .style(Style::default().bg(Color::Black));
    
    let input_text = if input.is_empty() {
        // Show placeholder when empty
        Paragraph::new(Span::styled(
            "Type path here...",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        ))
    } else {
        // Show actual input with cursor
        let spans = vec![
            Span::styled(input, Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(theme.highlight_fg)), // Cursor
        ];
        Paragraph::new(Line::from(spans))
    };
    
    frame.render_widget(input_block, chunks[2]);
    
    // Render input text inside the box
    let input_inner = Rect {
        x: chunks[2].x + 1,
        y: chunks[2].y + 1,
        width: chunks[2].width.saturating_sub(2),
        height: 1,
    };
    frame.render_widget(input_text, input_inner);
    
    // Error message area (if any)
    if let Some(err_msg) = error {
        let error_text = Paragraph::new(err_msg.as_str())
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });
        frame.render_widget(error_text, chunks[3]);
    } else {
        // Show hints when no error
        let hints = vec![
            Line::from(vec![
                Span::styled("Examples: ", Style::default().fg(Color::DarkGray)),
                Span::styled("C:\\Users\\", Style::default().fg(theme.info_color)),
                Span::styled(" or ", Style::default().fg(Color::DarkGray)),
                Span::styled("../parent", Style::default().fg(theme.info_color)),
            ]),
            Line::from(vec![
                Span::styled("Variables: ", Style::default().fg(Color::DarkGray)),
                Span::styled("~", Style::default().fg(theme.info_color)),
                Span::styled(" or ", Style::default().fg(Color::DarkGray)),
                Span::styled("%USERPROFILE%", Style::default().fg(theme.info_color)),
            ]),
        ];
        let hints_text = Paragraph::new(hints)
            .alignment(Alignment::Left);
        frame.render_widget(hints_text, chunks[3]);
    }
    
    // Footer with instructions
    let footer = Paragraph::new("Enter: Navigate | Ctrl+V: Paste | Esc: Cancel")
        .style(Style::default().fg(theme.info_color))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[5]);
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

