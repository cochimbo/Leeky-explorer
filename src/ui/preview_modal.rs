// Preview modal rendering
use crate::app::PreviewState;
use crate::ui::theme::Theme;
use humansize::{format_size, DECIMAL};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use super::parse_ansi_line;

/// Render the text preview modal
pub fn render_preview_modal(f: &mut Frame, preview_state: &PreviewState, theme: &Theme) {
    match preview_state {
        PreviewState::Text {
            content,
            scroll_offset,
            total_lines,
            file_path,
            file_size,
            warning,
        } => {
            // Calculate modal size: 80% of screen
            let area = f.area();
            let modal_width = (area.width as f32 * 0.8) as u16;
            let modal_height = (area.height as f32 * 0.8) as u16;

            // Center the modal
            let modal_area = Rect {
                x: (area.width.saturating_sub(modal_width)) / 2,
                y: (area.height.saturating_sub(modal_height)) / 2,
                width: modal_width,
                height: modal_height,
            };

            // Clear background behind modal
            f.render_widget(Clear, modal_area);

            // Create title with filename and size
            let filename = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown");
            let size_str = format_size(*file_size, DECIMAL);
            let title = if let Some(warn) = warning {
                format!("{} ({}) - {}", filename, size_str, warn)
            } else {
                format!("{} ({})", filename, size_str)
            };

            // Create block with border and title
            let block = Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(Style::default().bg(theme.dialog_bg).fg(theme.dialog_fg));

            // Calculate content area (inside border)
            let inner_area = block.inner(modal_area);

            // Render block first
            f.render_widget(block, modal_area);

            // Split inner area: content + footer
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),      // Content
                    Constraint::Length(1),   // Footer
                ])
                .split(inner_area);

            let content_area = chunks[0];
            let footer_area = chunks[1];

            // Calculate visible lines based on content area height
            let visible_lines = content_area.height as usize;
            let start_line = *scroll_offset;
            let _end_line = (start_line + visible_lines).min(*total_lines);

            // Prepare lines with line numbers
            let lines: Vec<Line> = content
                .lines()
                .enumerate()
                .skip(start_line)
                .take(visible_lines)
                .map(|(idx, line_content)| {
                    let line_num = idx + 1;
                    let num_width = 4; // Reserve 4 characters for line numbers

                    Line::from(vec![
                        Span::styled(
                            format!("{:>width$} ", line_num, width = num_width),
                            Style::default().fg(theme.info_color),
                        ),
                        Span::raw(line_content),
                    ])
                })
                .collect();

            // Render content
            let content_widget = Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .style(Style::default().bg(theme.dialog_bg).fg(theme.dialog_fg));

            f.render_widget(content_widget, content_area);

            // Calculate progress percentage
            let progress_percent = if *total_lines > 0 {
                (*scroll_offset * 100) / total_lines.saturating_sub(1).max(1)
            } else {
                0
            };

            // Render footer with hints and position
            let footer_text = vec![
                Span::raw("↑↓: Scroll | "),
                Span::raw("Home/End: Inicio/Fin | "),
                Span::raw("PgUp/PgDn: Página | "),
                Span::styled("Esc/Q: Cerrar", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!(
                    "  |  Línea {}/{} ({}%)",
                    start_line + 1,
                    total_lines,
                    progress_percent
                )),
            ];

            let footer = Paragraph::new(Line::from(footer_text))
                .style(Style::default().bg(theme.dialog_bg).fg(theme.info_color))
                .alignment(Alignment::Left);

            f.render_widget(footer, footer_area);
        }
        // T716-T721: Image preview rendering
        PreviewState::Image {
            ascii_art,
            metadata,
            file_path,
            file_size,
        } => {
            // T717: Calculate modal size: 95% of screen (maximize space for images)
            let area = f.area();
            let modal_width = (area.width as f32 * 0.95) as u16;
            let modal_height = (area.height as f32 * 0.95) as u16;

            // Center the modal
            let modal_area = Rect {
                x: (area.width.saturating_sub(modal_width)) / 2,
                y: (area.height.saturating_sub(modal_height)) / 2,
                width: modal_width,
                height: modal_height,
            };

            // Clear background behind modal
            f.render_widget(Clear, modal_area);

            // T718: Create title with filename, dimensions, size, and format
            let filename = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown");
            let size_str = format_size(*file_size, DECIMAL);
            
            // T721: Check if it's a GIF (might be animated)
            let gif_note = if metadata.format == "GIF" {
                " (GIF - frame 1)"
            } else {
                ""
            };
            
            let title = format!(
                "{} ({}x{}, {}, {}){}",
                filename,
                metadata.width,
                metadata.height,
                size_str,
                metadata.format,
                gif_note
            );

            // Create block with border and title
            let block = Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(Style::default().bg(theme.dialog_bg).fg(theme.dialog_fg));

            // Calculate content area (inside border)
            let inner_area = block.inner(modal_area);

            // Render block first
            f.render_widget(block, modal_area);

            // Split inner area: content + footer
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),      // Content
                    Constraint::Length(1),   // Footer
                ])
                .split(inner_area);

            let content_area = chunks[0];
            let footer_area = chunks[1];

            // T719: Parse ASCII art with ANSI color codes and center within modal area
            let art_lines: Vec<Line> = ascii_art
                .lines()
                .map(parse_ansi_line)
                .collect();

            let content_widget = Paragraph::new(art_lines)
                .style(Style::default().bg(theme.dialog_bg))
                .alignment(Alignment::Center);

            f.render_widget(content_widget, content_area);

            // T720: Render footer with hints
            let footer_text = vec![
                Span::styled("Esc/Q: Cerrar", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!(
                    "  |  Original: {}x{} pixels",
                    metadata.width, metadata.height
                )),
            ];

            let footer = Paragraph::new(Line::from(footer_text))
                .style(Style::default().bg(theme.dialog_bg).fg(theme.info_color))
                .alignment(Alignment::Left);

            f.render_widget(footer, footer_area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_preview_state_creation() {
        let content = "Line 1\nLine 2\nLine 3".to_string();
        let total_lines = content.lines().count();

        let preview = PreviewState::Text {
            content: content.clone(),
            scroll_offset: 0,
            total_lines,
            file_path: PathBuf::from("test.txt"),
            file_size: 100,
            warning: None,
        };

        assert_eq!(preview.scroll_offset(), 0);
        assert_eq!(preview.total_lines(), 3);
    }
}
