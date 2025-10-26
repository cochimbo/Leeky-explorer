// US4: Drive selector dialog widget
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::ui::theme::Theme;
use crate::fs::disk_info::{UsageLevel, format_size, get_disk_space};

/// Render the drive selector dialog with usage bars
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
    
    // Drive list with usage bars
    let items: Vec<ListItem> = drives
        .iter()
        .enumerate()
        .map(|(i, (path, label))| {
            // Try to get disk space info for this drive
            let disk_info = get_disk_space(std::path::Path::new(path)).ok();
            
            let is_selected = i == selected;
            let base_style = if is_selected {
                Style::default()
                    .fg(theme.highlight_fg)
                    .bg(theme.highlight_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.dialog_fg)
            };
            
            let prefix = if is_selected { "► " } else { "  " };
            
            // Build the line with usage bar if disk info available
            let line = if let Some(info) = disk_info {
                let usage_pct = info.usage_percentage();
                let level = info.warning_level();
                let bar = create_usage_bar(usage_pct, 20); // 20 char wide bar
                
                // Color based on usage level
                let bar_color = match level {
                    UsageLevel::Normal => Color::Green,
                    UsageLevel::Warning => Color::Yellow,
                    UsageLevel::Critical => Color::Red,
                };
                
                Line::from(vec![
                    Span::styled(prefix, base_style),
                    Span::styled(format!("{:<10}", label), base_style),
                    Span::styled(bar, Style::default().fg(bar_color)),
                    Span::styled(format!(" {:>3.0}% ", usage_pct), base_style),
                    Span::styled(format!("({} free)", format_size(info.free_bytes)), base_style),
                ])
            } else {
                // Fallback if disk info unavailable
                Line::from(vec![
                    Span::styled(format!("{}{}", prefix, label), base_style),
                ])
            };
            
            ListItem::new(line).style(base_style)
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

/// Create a visual usage bar
/// 
/// # Arguments
/// * `percentage` - Usage percentage (0.0 - 100.0)
/// * `width` - Width of the bar in characters
/// 
/// # Returns
/// String representation of the usage bar (e.g., "[████████░░░░]")
fn create_usage_bar(percentage: f64, width: usize) -> String {
    let filled_chars = ((percentage / 100.0) * width as f64).round() as usize;
    let filled_chars = filled_chars.min(width); // Ensure we don't overflow
    
    let filled = "█".repeat(filled_chars);
    let empty = "░".repeat(width.saturating_sub(filled_chars));
    
    format!("[{}{}]", filled, empty)
}
