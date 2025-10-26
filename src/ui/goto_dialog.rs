// Go To Path dialog widget - TASK-022
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::path::PathBuf;

use crate::ui::theme::Theme;

/// Render the Go To Path dialog with autocomplete suggestions
pub fn render(
    frame: &mut Frame,
    input: &str,
    error: &Option<String>,
    suggestions: &[PathBuf],
    selected_suggestion: usize,
    theme: &Theme,
) {
    let area = centered_rect(70, 60, frame.area());
    
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
            Constraint::Length(2),  // Error message or hint
            Constraint::Min(5),     // Suggestions list
            Constraint::Length(2),  // Footer instructions
        ])
        .split(inner);
    
    // Header text
    let header = Paragraph::new("Enter directory path (Tab to autocomplete):")
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
    
    // Error message or suggestion count
    if let Some(err_msg) = error {
        let error_text = Paragraph::new(err_msg.as_str())
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });
        frame.render_widget(error_text, chunks[3]);
    } else if !suggestions.is_empty() {
        let count_text = format!("{} suggestion(s) - Press Enter to select highlighted, Tab for prefix", suggestions.len());
        let hint = Paragraph::new(count_text)
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Left);
        frame.render_widget(hint, chunks[3]);
    } else {
        // Show examples when no suggestions
        let hints = Line::from(vec![
            Span::styled("Examples: ", Style::default().fg(Color::DarkGray)),
            Span::styled("C:\\Users\\", Style::default().fg(theme.info_color)),
            Span::styled(" or ", Style::default().fg(Color::DarkGray)),
            Span::styled("../parent", Style::default().fg(theme.info_color)),
        ]);
        let hints_text = Paragraph::new(hints).alignment(Alignment::Left);
        frame.render_widget(hints_text, chunks[3]);
    }
    
    // Suggestions list
    if !suggestions.is_empty() {
        let suggestions_block = Block::default()
            .title(" Subdirectories ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .style(Style::default().bg(theme.dialog_bg));
        
        let items: Vec<ListItem> = suggestions
            .iter()
            .enumerate()
            .map(|(i, path)| {
                let is_selected = i == selected_suggestion;
                let style = if is_selected {
                    Style::default()
                        .fg(theme.highlight_fg)
                        .bg(theme.highlight_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.dialog_fg)
                };
                
                let prefix = if is_selected { "► " } else { "  " };
                let display_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("<invalid>");
                
                let line = Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(display_name, style),
                ]);
                
                ListItem::new(line).style(style)
            })
            .collect();
        
        let list = List::new(items).block(suggestions_block);
        frame.render_widget(list, chunks[4]);
    } else {
        // Show "no subdirectories" message
        let empty_block = Block::default()
            .title(" Subdirectories ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .style(Style::default().bg(theme.dialog_bg));
        
        frame.render_widget(empty_block, chunks[4]);
        
        let empty_msg = Paragraph::new("No subdirectories available")
            .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))
            .alignment(Alignment::Center);
        
        let empty_inner = Rect {
            x: chunks[4].x + 1,
            y: chunks[4].y + (chunks[4].height / 2),
            width: chunks[4].width.saturating_sub(2),
            height: 1,
        };
        frame.render_widget(empty_msg, empty_inner);
    }
    
    // Footer with instructions - dynamic based on suggestions
    let footer_text = if !suggestions.is_empty() {
        "Enter: Select | Tab: Complete prefix | ↑↓: Navigate | Esc: Cancel"
    } else {
        "Enter: Navigate to path | Tab: Autocomplete | Esc: Cancel"
    };
    
    let footer = Paragraph::new(footer_text)
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


