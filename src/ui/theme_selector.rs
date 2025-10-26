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
    
    // Split into header, list, preview and footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Header
            Constraint::Min(5),     // Theme list
            Constraint::Length(7),  // Preview box
            Constraint::Length(2),  // Footer instructions
        ])
        .split(inner);
    
    // Header
    let header = Paragraph::new("Use ↑↓ to navigate, Enter to apply theme, Esc to cancel")
        .style(Style::default().fg(active_theme.info_color))
        .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);
    
    // Theme list
    let items: Vec<ListItem> = themes
        .iter()
        .enumerate()
        .map(|(i, theme)| {
            let is_selected = i == selected;
            let is_active = theme.name == active_theme.name;
            
            // Build item line
            let mut spans = vec![];
            
            // Selection indicator
            if is_selected {
                spans.push(Span::styled("►", Style::default().fg(active_theme.highlight_fg).add_modifier(Modifier::BOLD)));
            } else {
                spans.push(Span::raw(" "));
            }
            
            spans.push(Span::raw(" "));
            
            // Theme name with active indicator
            let name_text = if is_active {
                format!("{:<18} ✓", theme.name)
            } else {
                format!("{:<20}", theme.name)
            };
            
            let name_style = if is_selected {
                Style::default().fg(active_theme.highlight_fg).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(active_theme.dialog_fg)
            };
            spans.push(Span::styled(name_text, name_style));
            
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
    
    // Preview of selected theme
    if let Some(preview_theme) = themes.get(selected) {
        let preview_area = chunks[2];
        
        // Create a mini preview box showing the theme colors
        let preview_block = Block::default()
            .title(" Preview ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(preview_theme.active_border))
            .style(Style::default().bg(preview_theme.panel_bg));
        
        let preview_inner = preview_block.inner(preview_area);
        frame.render_widget(preview_block, preview_area);
        
        // Build preview content showing different elements
        let preview_lines = vec![
            Line::from(vec![
                Span::styled("📁 ", Style::default().fg(preview_theme.dir_color)),
                Span::styled("Directory", Style::default().fg(preview_theme.dir_color)),
                Span::raw("    "),
                Span::styled("📄 ", Style::default().fg(preview_theme.file_color)),
                Span::styled("File.txt", Style::default().fg(preview_theme.file_color)),
            ]),
            Line::from(vec![
                Span::styled("🔗 ", Style::default().fg(preview_theme.symlink_color)),
                Span::styled("Symlink", Style::default().fg(preview_theme.symlink_color)),
                Span::raw("    "),
                Span::styled("⚡ ", Style::default().fg(preview_theme.executable_color)),
                Span::styled("program.exe", Style::default().fg(preview_theme.executable_color)),
            ]),
            Line::from(vec![
                Span::styled("  Selected item", Style::default().bg(preview_theme.highlight_bg).fg(preview_theme.highlight_fg)),
            ]),
            Line::from(vec![
                Span::styled("* ", Style::default().fg(preview_theme.marked_bg)),
                Span::styled("Marked item", Style::default().bg(preview_theme.marked_bg).fg(preview_theme.panel_fg)),
            ]),
        ];
        
        let preview_content = Paragraph::new(preview_lines)
            .style(Style::default().fg(preview_theme.panel_fg))
            .alignment(Alignment::Left);
        
        frame.render_widget(preview_content, preview_inner);
    }
    
    // Footer
    let footer = Paragraph::new("Live preview of selected theme above")
        .style(Style::default().fg(active_theme.info_color))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[3]);
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
