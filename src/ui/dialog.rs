// Dialog rendering
use crate::app::DialogState;
use crate::models::operation::Progress;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Wrap},
    Frame,
};

pub fn render_dialog(frame: &mut Frame, dialog: &DialogState, area: Rect) {
    match dialog {
        DialogState::Confirm { message, .. } => {
            render_confirm_dialog(frame, message, area);
        }
        DialogState::Input { prompt, value } => {
            render_input_dialog(frame, prompt, value, area);
        }
        DialogState::Progress { message } => {
            render_progress_dialog(frame, message, area);
        }
        DialogState::Error { message } => {
            render_error_dialog(frame, message, area);
        }
        DialogState::ExtractOptions { archive_name, dest, selected, .. } => {
            render_extract_options_dialog(frame, archive_name, dest, *selected, area);
        }
        DialogState::PasswordInput { prompt, value, show_password, .. } => {
            render_password_input_dialog(frame, prompt, value, *show_password, area);
        }
        DialogState::CollisionPrompt { file_path, selected, operation: _ } => {
            render_collision_dialog(frame, file_path, *selected, area);
        }
    }
}

pub fn render_extract_options_dialog(frame: &mut Frame, archive_name: &str, dest: &std::path::Path, selected: usize, area: Rect) {
    let dialog_area = centered_rect(70, 35, area);
    
    frame.render_widget(Clear, dialog_area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Extract Archive ")
        .style(Style::default().bg(Color::Black).fg(Color::Cyan));
    
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Title
            Constraint::Length(3),  // Option 1
            Constraint::Length(3),  // Option 2
            Constraint::Length(2),  // Help
        ])
        .split(inner);
    
    // Title
    let title = Paragraph::new(format!("Extract '{}' to:", archive_name))
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::White));
    frame.render_widget(title, chunks[0]);
    
    // Option 1: Extract here
    let option1_style = if selected == 0 {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let option1 = Paragraph::new(vec![
        Line::from(vec![
            Span::raw(if selected == 0 { "▶ " } else { "  " }),
            Span::styled("Extract files here", option1_style),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("→ {}", dest.display()), Style::default().fg(Color::DarkGray)),
        ]),
    ]);
    frame.render_widget(option1, chunks[1]);
    
    // Option 2: Create folder
    let option2_style = if selected == 1 {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let dest_with_folder = dest.join(archive_name);
    let option2 = Paragraph::new(vec![
        Line::from(vec![
            Span::raw(if selected == 1 { "▶ " } else { "  " }),
            Span::styled("Create folder and extract", option2_style),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("→ {}", dest_with_folder.display()), Style::default().fg(Color::DarkGray)),
        ]),
    ]);
    frame.render_widget(option2, chunks[2]);
    
    // Help
    let help_text = Paragraph::new(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(": Select | "),
        Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(": Confirm | "),
        Span::styled("Esc", Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        Span::raw(": Cancel"),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(help_text, chunks[3]);
}

pub fn render_confirm_dialog(frame: &mut Frame, message: &str, area: Rect) {
    let dialog_area = centered_rect(60, 30, area);
    
    frame.render_widget(Clear, dialog_area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Confirm ")
        .style(Style::default().bg(Color::Black).fg(Color::Yellow));
    
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(inner);
    
    let text = Paragraph::new(message)
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Center);
    frame.render_widget(text, chunks[0]);
    
    let confirm_text = Paragraph::new(Line::from(vec![
        Span::styled("Y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw("es / "),
        Span::styled("N", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw("o / "),
        Span::styled("Esc", Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(confirm_text, chunks[1]);
}

pub fn render_input_dialog(frame: &mut Frame, prompt: &str, value: &str, area: Rect) {
    let dialog_area = centered_rect(60, 25, area);
    
    frame.render_widget(Clear, dialog_area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Input ")
        .style(Style::default().bg(Color::Black).fg(Color::Cyan));
    
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(inner);
    
    let prompt_text = Paragraph::new(prompt)
        .alignment(Alignment::Left);
    frame.render_widget(prompt_text, chunks[0]);
    
    let input_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));
    let input_text = Paragraph::new(value)
        .block(input_block);
    frame.render_widget(input_text, chunks[1]);
    
    let help_text = Paragraph::new("Enter: Confirm | Esc: Cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(help_text, chunks[2]);
}

pub fn render_progress_dialog(frame: &mut Frame, message: &str, area: Rect) {
    let dialog_area = centered_rect(70, 30, area);
    
    frame.render_widget(Clear, dialog_area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Operation in Progress ")
        .style(Style::default().bg(Color::Black).fg(Color::Green));
    
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);
    
    let text = Paragraph::new(message)
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Center);
    frame.render_widget(text, inner);
}

pub fn render_progress_with_bar(
    frame: &mut Frame,
    message: &str,
    progress: &Progress,
    area: Rect,
) {
    let dialog_area = centered_rect(70, 35, area);
    
    frame.render_widget(Clear, dialog_area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Operation in Progress ")
        .style(Style::default().bg(Color::Black).fg(Color::Green));
    
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(inner);
    
    let text = Paragraph::new(message)
        .alignment(Alignment::Center);
    frame.render_widget(text, chunks[0]);
    
    let percentage = progress.percentage();
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
        .percent(percentage as u16)
        .label(format!("{:.1}%", percentage));
    frame.render_widget(gauge, chunks[1]);
    
    let stats = format!(
        "{} / {} files | {} / {}",
        progress.files_done,
        progress.files_total,
        humansize::format_size(progress.bytes_done, humansize::DECIMAL),
        humansize::format_size(progress.bytes_total, humansize::DECIMAL)
    );
    let stats_text = Paragraph::new(stats)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(stats_text, chunks[2]);
}

pub fn render_error_dialog(frame: &mut Frame, message: &str, area: Rect) {
    let dialog_area = centered_rect(60, 30, area);
    
    frame.render_widget(Clear, dialog_area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Error ")
        .style(Style::default().bg(Color::Black).fg(Color::Red));
    
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(2),
        ])
        .split(inner);
    
    let text = Paragraph::new(message)
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Red));
    frame.render_widget(text, chunks[0]);
    
    let help_text = Paragraph::new("Press any key to close")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(help_text, chunks[1]);
}

pub fn render_double_confirm_dialog(frame: &mut Frame, message: &str, area: Rect) {
    let dialog_area = centered_rect(70, 35, area);
    
    frame.render_widget(Clear, dialog_area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" ⚠ WARNING ⚠ ")
        .style(Style::default().bg(Color::Black).fg(Color::Red));
    
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(inner);
    
    let text = Paragraph::new(message)
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(text, chunks[0]);
    
    let confirm_text = Paragraph::new(Line::from(vec![
        Span::styled("This action cannot be undone! ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw("\n"),
        Span::styled("Y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw("es to delete / "),
        Span::styled("N", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw("o / "),
        Span::styled("Esc", Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(confirm_text, chunks[1]);
}

// T843: Password input dialog for encrypted archives
pub fn render_password_input_dialog(frame: &mut Frame, prompt: &str, value: &str, show_password: bool, area: Rect) {
    let dialog_area = centered_rect(60, 30, area);
    
    frame.render_widget(Clear, dialog_area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 🔒 Password Required ")
        .style(Style::default().bg(Color::Black).fg(Color::Yellow));
    
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Prompt
            Constraint::Length(3),  // Input field
            Constraint::Length(2),  // Show password toggle
            Constraint::Length(1),  // Spacer
            Constraint::Length(2),  // Instructions
        ])
        .split(inner);
    
    // Prompt
    let prompt_text = Paragraph::new(prompt)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White));
    frame.render_widget(prompt_text, chunks[0]);
    
    // Input field (show asterisks if password hidden)
    let display_value = if show_password {
        value.to_string()
    } else {
        "*".repeat(value.len())
    };
    
    let input_text = Paragraph::new(format!(" {} ", display_value))
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(input_text, chunks[1]);
    
    // Show password toggle
    let toggle_text = if show_password {
        "[ ] Show password (press Tab to toggle)"
    } else {
        "[✓] Hide password (press Tab to toggle)"
    };
    let toggle_para = Paragraph::new(toggle_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(toggle_para, chunks[2]);
    
    // Instructions
    let instructions = Paragraph::new(Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(": Confirm | "),
        Span::styled("Tab", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(": Toggle visibility | "),
        Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw(": Cancel"),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(instructions, chunks[4]);
}

// T844: Collision handling dialog
pub fn render_collision_dialog(frame: &mut Frame, file_path: &str, selected: usize, area: Rect) {
    let dialog_area = centered_rect(70, 40, area);
    
    frame.render_widget(Clear, dialog_area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" ⚠️  File Already Exists ")
        .style(Style::default().bg(Color::Black).fg(Color::Yellow));
    
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Message
            Constraint::Length(1),  // Spacer
            Constraint::Length(2),  // Option 0: Overwrite
            Constraint::Length(2),  // Option 1: Overwrite All
            Constraint::Length(2),  // Option 2: Rename
            Constraint::Length(2),  // Option 3: Skip
            Constraint::Length(2),  // Option 4: Cancel
            Constraint::Length(1),  // Spacer
            Constraint::Length(2),  // Instructions
        ])
        .split(inner);
    
    // Message
    let message = format!("File already exists:\n{}", file_path);
    let message_text = Paragraph::new(message)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White));
    frame.render_widget(message_text, chunks[0]);
    
    // Options
    let options = [
        ("S", "Sobreescribir este archivo"),
        ("T", "Sobreescribir Todos"),
        ("R", "Renombrar (agregar sufijo)"),
        ("O", "Omitir este archivo"),
        ("C", "Cancelar extracción"),
    ];
    
    for (i, (key, text)) in options.iter().enumerate() {
        let style = if i == selected {
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        
        let option_text = Paragraph::new(Line::from(vec![
            Span::styled(format!(" [{}] ", key), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(*text, style),
        ]));
        frame.render_widget(option_text, chunks[2 + i]);
    }
    
    // Instructions
    let instructions = Paragraph::new(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(": Navegar | "),
        Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(" o "),
        Span::styled("Letra", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(": Seleccionar | "),
        Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw(": Cancelar"),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(instructions, chunks[8]);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
