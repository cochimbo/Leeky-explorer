// UI module
pub mod layout;
pub mod panel_widget;
pub mod dialog;
pub mod theme;
pub mod preview_modal;
pub mod file_icons;

use crate::app::AppState;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_header(frame: &mut Frame, app: &AppState, area: Rect) {
    let left_path = format!(" Left: {} ", app.left_panel.current_path.display());
    let right_path = format!(" Right: {} ", app.right_panel.current_path.display());

    let content = Line::from(vec![
        Span::styled(left_path, Style::default().fg(Color::Cyan)),
        Span::raw(" | "),
        Span::styled(right_path, Style::default().fg(Color::Cyan)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let paragraph = Paragraph::new(content).block(block);
    frame.render_widget(paragraph, area);
}

pub fn render_footer(frame: &mut Frame, area: Rect) {
    // T561: Group keybindings in 2 lines for better space management
    let line1_bindings = vec![
        ("↑↓", "Nav", Color::Blue),
        ("Tab", "Switch", Color::Blue),
        ("Enter", "Open", Color::Green),
        ("Bksp", "Up", Color::Yellow),
        ("Space", "Select", Color::Magenta),
        ("Ctrl+A", "All", Color::Magenta),
    ];
    
    let line2_bindings = vec![
        ("F3", "Search", Color::Cyan),
        ("F4", "Preview", Color::Cyan),
        ("F5", "Copy", Color::Green),
        ("F6", "Move", Color::Green),
        ("F7", "NewDir", Color::Yellow),
        ("F8", "Delete", Color::Red),
        ("F9", "Extract", Color::Cyan),
        ("Q", "Quit", Color::Gray),
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

