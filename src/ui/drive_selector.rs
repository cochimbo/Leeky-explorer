// US4: Drive selector dialog widget
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::ui::theme::Theme;
use crate::fs::disk_info::{format_size, get_disk_space};

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
    
    // Calculate maximum label width for alignment
    let max_label_width = drives.iter()
        .map(|(_, label)| label.len())
        .max()
        .unwrap_or(10)
        .max(10); // At least 10 characters
    
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
                let bar_width = 25; // Wider bar for better visibility
                
                // Create gradient bar as multiple colored spans
                let bar_spans = create_gradient_bar_spans(usage_pct, bar_width);
                
                // Build complete line with all components
                let mut spans = vec![
                    Span::styled(prefix, base_style),
                    Span::styled(format!("{:<width$}", label, width = max_label_width), base_style),
                    Span::raw(" "), // Spacing before bar
                ];
                
                // Add the gradient bar spans
                spans.extend(bar_spans);
                
                // Add percentage and free space with consistent spacing
                spans.push(Span::styled(format!(" {:>3.0}%", usage_pct), base_style));
                spans.push(Span::styled(format!("  ({:>8})", format_size(info.free_bytes)), base_style));
                
                Line::from(spans)
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

/// Create gradient usage bar as multiple colored spans
/// Each character gets its own color in the gradient
/// 
/// # Arguments
/// * `percentage` - Usage percentage (0.0 - 100.0)
/// * `width` - Width of the bar in characters
/// 
/// # Returns
/// Vector of Spans with gradient colors from green to yellow to red
fn create_gradient_bar_spans(percentage: f64, width: usize) -> Vec<Span<'static>> {
    let filled_chars = ((percentage / 100.0) * width as f64).round() as usize;
    let filled_chars = filled_chars.min(width);
    
    let mut spans = Vec::new();
    
    // Opening bracket
    spans.push(Span::styled("[", Style::default().fg(Color::White)));
    
    // Filled portion with gradient
    for i in 0..filled_chars {
        // Calculate percentage for this specific character
        let char_pct = ((i + 1) as f64 / width as f64) * 100.0;
        let color = calculate_gradient_color(char_pct);
        spans.push(Span::styled("█", Style::default().fg(color)));
    }
    
    // Empty portion in dark gray
    if filled_chars < width {
        let empty_str = "░".repeat(width - filled_chars);
        spans.push(Span::styled(empty_str, Style::default().fg(Color::DarkGray)));
    }
    
    // Closing bracket
    spans.push(Span::styled("]", Style::default().fg(Color::White)));
    
    spans
}

/// Calculate gradient color from green (0%) to red (100%)
/// 
/// # Arguments
/// * `percentage` - Usage percentage (0.0 - 100.0)
/// 
/// # Returns
/// Color with smooth gradient from green through yellow to red
fn calculate_gradient_color(percentage: f64) -> Color {
    let pct = percentage.clamp(0.0, 100.0);
    
    if pct < 50.0 {
        // Green to Yellow (0% - 50%)
        let ratio = pct / 50.0;
        let red = (255.0 * ratio) as u8;
        let green = 255;
        Color::Rgb(red, green, 0)
    } else {
        // Yellow to Red (50% - 100%)
        let ratio = (pct - 50.0) / 50.0;
        let red = 255;
        let green = (255.0 * (1.0 - ratio)) as u8;
        Color::Rgb(red, green, 0)
    }
}
