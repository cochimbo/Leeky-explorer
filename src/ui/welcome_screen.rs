//! Welcome screen module
//!
//! Displays ASCII art logo and version information on application startup.
//! User presses Enter to proceed to the main file manager interface.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::preview::image_viewer::image_to_ascii;
use crate::ui::theme::Theme;
use super::parse_ansi_line;



/// Load and convert the logo PNG to ASCII art
/// 
/// # Arguments
/// * `terminal_area` - Full terminal area to calculate appropriate dimensions
/// 
/// # Returns
/// ASCII art string with ANSI color codes, or fallback text on error
fn load_logo(terminal_area: Rect) -> String {
    // Calculate max dimensions similar to image preview
    // Use most of the terminal space, leaving room for version and instruction
    let max_width = terminal_area.width.saturating_sub(4); // Leave margin
    let max_height = terminal_area.height.saturating_sub(8); // Leave space for version + instruction
    
    // Try to load the PNG logo using tokio blocking (since we're in sync context)
    let logo_path = std::path::Path::new("assets/images/leekpc.png");
    
    // Use image crate directly for synchronous loading
    match image::open(logo_path) {
        Ok(img) => {
            // Convert to ASCII art with calculated dimensions
            match image_to_ascii(&img, max_width as u32, max_height as u32) {
                Ok(ascii) => ascii,
                Err(_) => fallback_logo(),
            }
        }
        Err(_) => fallback_logo(),
    }
}

/// Fallback logo text when PNG conversion fails
fn fallback_logo() -> String {
    r#"
    ╔═══════════════════════════════╗
    ║                               ║
    ║    LEEKY FILE MANAGER         ║
    ║                               ║
    ╔═══════════════════════════════╝
    "#.to_string()
}

/// Render the welcome screen
/// 
/// # Arguments
/// * `frame` - Ratatui frame for rendering
/// * `area` - Full terminal area
/// * `version` - Application version string (e.g., "0.3.0")
/// * `theme` - Theme to use for colors
pub fn render(frame: &mut Frame, area: Rect, version: &str, theme: &Theme) {
    // Handle very small terminals
    if area.width < 40 || area.height < 10 {
        render_minimal(frame, area, version, theme);
        return;
    }

    // Load and convert logo with full area for optimal sizing
    let logo_text = load_logo(area);

    // Parse ASCII art with ANSI color codes
    let logo_lines: Vec<Line> = logo_text
        .lines()
        .map(parse_ansi_line)
        .collect();

    // Create vertical layout: logo, version, instruction
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),           // Logo area (takes most space)
            Constraint::Length(3),         // Version display
            Constraint::Length(2),         // Instruction
        ])
        .split(area);

    // Render logo centered with parsed ANSI codes
    let logo_paragraph = Paragraph::new(logo_lines)
        .alignment(Alignment::Center)
        .style(Style::default().bg(theme.panel_bg));
    frame.render_widget(logo_paragraph, chunks[0]);

    // Render version
    let version_line = Line::from(vec![
        Span::styled(
            format!("Version {}", version),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let version_paragraph = Paragraph::new(version_line)
        .alignment(Alignment::Center);
    frame.render_widget(version_paragraph, chunks[1]);

    // Render instruction
    let instruction = Line::from(vec![
        Span::styled(
            "Press ",
            Style::default().fg(theme.info_color),
        ),
        Span::styled(
            "Enter",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " to continue...",
            Style::default().fg(theme.info_color),
        ),
    ]);
    let instruction_paragraph = Paragraph::new(instruction)
        .alignment(Alignment::Center);
    frame.render_widget(instruction_paragraph, chunks[2]);
}

/// Render minimal welcome screen for very small terminals
fn render_minimal(frame: &mut Frame, area: Rect, version: &str, theme: &Theme) {
    let lines = vec![
        Line::from(Span::styled(
            "Leeky File Manager",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("v{}", version),
            Style::default().fg(theme.info_color),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(theme.info_color)),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    // Center the minimal block
    let centered = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(6),
            Constraint::Percentage(30),
        ])
        .split(area);

    frame.render_widget(paragraph, centered[1]);
}
