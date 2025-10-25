// UI module
pub mod layout;
pub mod panel_widget;
pub mod dialog;
pub mod theme;
pub mod preview_modal;
pub mod file_icons;
pub mod welcome_screen;

use crate::app::{AppState, PanelSide, DialogState};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Parse a line containing ANSI escape codes into Ratatui Spans
/// Handles RGB color codes (38;2;R;G;B format)
/// 
/// Used by image preview and welcome screen to display colored ASCII art
pub fn parse_ansi_line(line: &str) -> Line<'static> {
    let mut spans = Vec::new();
    let mut current_text = String::new();
    let mut chars = line.chars().peekable();
    let mut current_color: Option<Color> = None;
    
    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            // Found ANSI escape sequence
            // Save current text if any
            if !current_text.is_empty() {
                let style = if let Some(color) = current_color {
                    Style::default().fg(color)
                } else {
                    Style::default()
                };
                spans.push(Span::styled(current_text.clone(), style));
                current_text.clear();
            }
            
            // Parse escape sequence
            chars.next(); // consume '['
            let mut code = String::new();
            while let Some(&ch) = chars.peek() {
                if ch == 'm' {
                    chars.next(); // consume 'm'
                    break;
                }
                if let Some(ch) = chars.next() {
                    code.push(ch);
                }
            }
            
            // Parse RGB color code (38;2;R;G;B)
            if code.starts_with("38;2;") {
                let parts: Vec<&str> = code.split(';').collect();
                if parts.len() >= 5
                    && let (Ok(r), Ok(g), Ok(b)) = (
                        parts[2].parse::<u8>(),
                        parts[3].parse::<u8>(),
                        parts[4].parse::<u8>(),
                    ) {
                        current_color = Some(Color::Rgb(r, g, b));
                    }
            } else if code == "0" {
                // Reset
                current_color = None;
            }
        } else {
            current_text.push(ch);
        }
    }
    
    // Add remaining text
    if !current_text.is_empty() {
        let style = if let Some(color) = current_color {
            Style::default().fg(color)
        } else {
            Style::default()
        };
        spans.push(Span::styled(current_text, style));
    }
    
    Line::from(spans)
}


/// Render both panels side by side
pub fn render_panels(frame: &mut Frame, app: &AppState, layout: &layout::AppLayout) {
    let is_left_active = app.active_panel == PanelSide::Left;
    
    panel_widget::render_panel(
        frame,
        &app.left_panel,
        layout.left_panel,
        is_left_active,
        app.search_mode && is_left_active,
        &app.search_pattern,
        &app.selection_state,
        PanelSide::Left,
    );
    
    panel_widget::render_panel(
        frame,
        &app.right_panel,
        layout.right_panel,
        !is_left_active,
        app.search_mode && !is_left_active,
        &app.search_pattern,
        &app.selection_state,
        PanelSide::Right,
    );
}

/// Render dialog if present
pub fn render_dialog_if_present(frame: &mut Frame, app: &AppState) {
    if let Some(dialog) = &app.dialog_state {
        match dialog {
            DialogState::Progress { message } => {
                if let Some(ref op) = app.current_operation {
                    dialog::render_progress_with_bar(
                        frame,
                        message,
                        &op.progress,
                        frame.area()
                    );
                } else {
                    dialog::render_dialog(frame, dialog, frame.area());
                }
            }
            _ => {
                dialog::render_dialog(frame, dialog, frame.area());
            }
        }
    }
}

pub fn render_header(frame: &mut Frame, app: &mut AppState, left_area: Rect, right_area: Rect) {
    // T070-T074: Show disk space information instead of redundant paths
    // T080: Use cached disk space to prevent UI lag
    
    // Clone paths to avoid borrow checker issues
    let left_path = app.left_panel.current_path.clone();
    let right_path = app.right_panel.current_path.clone();
    
    let left_space = if let Some(info) = app.get_cached_disk_space(&left_path) {
        crate::fs::disk_info::format_disk_space(&info)
    } else {
        "Space: N/A".to_string()
    };
    
    let right_space = if let Some(info) = app.get_cached_disk_space(&right_path) {
        crate::fs::disk_info::format_disk_space(&info)
    } else {
        "Space: N/A".to_string()
    };

    // Render left header
    let left_content = Line::from(vec![
        Span::styled(format!(" {} ", left_space), Style::default().fg(Color::Cyan)),
    ]);
    
    let left_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());
    
    let left_paragraph = Paragraph::new(left_content).block(left_block);
    frame.render_widget(left_paragraph, left_area);

    // Render right header
    let right_content = Line::from(vec![
        Span::styled(format!(" {} ", right_space), Style::default().fg(Color::Cyan)),
    ]);
    
    let right_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());
    
    let right_paragraph = Paragraph::new(right_content).block(right_block);
    frame.render_widget(right_paragraph, right_area);
}

pub fn render_footer(frame: &mut Frame, area: Rect) {
    // T561: Group keybindings in 2 lines for better space management
    let line1_bindings = vec![
        ("↑↓", "Nav", Color::Blue),
        ("PgUp/Dn", "5×", Color::Blue),      // T128j: Page navigation
        ("Home/End", "Start/End", Color::Blue), // T128j: Jump to edges
        ("Tab", "Switch", Color::Blue),
        ("Enter", "Open", Color::Green),
        ("Bksp", "Up", Color::Yellow),
        ("Space", "Select", Color::Magenta),
        ("a-z", "Jump", Color::Blue),        // T128e: Alphanumeric navigation
    ];
    
    let line2_bindings = vec![
        ("F2", "Rename", Color::Yellow),
        ("⇧F2", "Full", Color::Yellow),      // Shift+F2 for rename with extension
        ("F3", "Search", Color::Cyan),
        ("F4", "Preview", Color::Cyan),
        ("F5", "Copy", Color::Green),
        ("F6", "Move", Color::Green),
        ("F7", "NewDir", Color::Yellow),
        ("F8", "Delete", Color::Red),
        ("F9", "Extract", Color::Cyan),
        ("⇧F9", "Compress", Color::Cyan),    // Shift+F9 for compression
        ("Ctrl+Q", "Quit", Color::Gray),     // T128b: Changed from Q to Ctrl+Q
        ("Ctrl+A", "All", Color::Magenta),
    ];

    let create_line = |bindings: &[(&str, &str, Color)]| {
        Line::from(
            bindings
                .iter()
                .enumerate()
                .flat_map(|(i, &(key, action, color))| {
                    let bg = if i % 2 == 0 { Color::Black } else { Color::DarkGray };
                    vec![
                        Span::styled(
                            format!(" {} ", key),
                            Style::default()
                                .fg(color)
                                .bg(bg)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!(":{} ", action),
                            Style::default()
                                .fg(Color::White)
                                .bg(bg),
                        ),
                    ]
                })
                .collect::<Vec<_>>(),
        )
    };

    let content = vec![
        create_line(&line1_bindings),
        create_line(&line2_bindings),
    ];

    let paragraph = Paragraph::new(content)
        .style(Style::default().bg(Color::Black));

    frame.render_widget(paragraph, area);
}

/// Render welcome screen with logo and version
/// 
/// # Arguments
/// * `frame` - Ratatui frame for rendering
/// * `version` - Application version string
pub fn render_welcome(frame: &mut Frame, version: &str) {
    let area = frame.area();
    welcome_screen::render(frame, area, version);
}

