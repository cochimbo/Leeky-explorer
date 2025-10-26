// Dialog rendering
use crate::app::{AppState, DialogState};
use crate::models::operation::Progress;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Wrap},
    Frame,
};

pub fn render_dialog(frame: &mut Frame, dialog: &DialogState, area: Rect, theme: &Theme, app: &AppState) {
    match dialog {
        DialogState::Confirm { message, .. } => {
            render_confirm_dialog(frame, message, area, theme);
        }
        DialogState::Input { prompt, value } => {
            render_input_dialog(frame, prompt, value, area, theme);
        }
        DialogState::Rename { prompt, value, .. } => {
            render_input_dialog(frame, prompt, value, area, theme);
        }
        DialogState::Progress { message } => {
            render_progress_dialog(frame, message, area, theme);
        }
        DialogState::Error { message } => {
            render_error_dialog(frame, message, area, theme);
        }
        DialogState::ExtractOptions { archive_name, dest, selected, .. } => {
            render_extract_options_dialog(frame, archive_name, dest, *selected, area, theme);
        }
        DialogState::PasswordInput { prompt, value, show_password, .. } => {
            render_password_input_dialog(frame, prompt, value, *show_password, area, theme);
        }
        DialogState::CollisionPrompt { file_path, selected, operation: _ } => {
            render_collision_dialog(frame, file_path, *selected, area, theme);
        }
        // T930: Render compression options dialog
        DialogState::CompressOptions { sources, output_name, format, level, use_password, password, confirm_password, selected_field } => {
            render_compress_options_dialog(frame, sources, output_name, format, level, *use_password, password, confirm_password, *selected_field, area, theme);
        }
        // US4: Render drive selector dialog
        DialogState::DriveSelector { drives, selected } => {
            crate::ui::drive_selector::render(frame, drives, *selected, theme);
        }
        // US5: Render theme selector dialog
        DialogState::ThemeSelector { themes, selected } => {
            crate::ui::theme_selector::render(frame, themes, *selected, theme);
        }
        // TASK-008: Render bookmark manager dialog
        DialogState::BookmarkManager { state } => {
            crate::ui::bookmark_manager::render(frame, &app.bookmarks, state, theme);
        }
        // TASK-018: Render navigation history dialog
        DialogState::HistoryViewer { state } => {
            let panel = app.active_panel();
            crate::ui::history_dialog::render(frame, panel, state, theme);
        }
        // TASK-021: Render Go To Path dialog (will be implemented in TASK-022)
        DialogState::GoToPath { input, error_message, suggestions, selected_suggestion } => {
            crate::ui::goto_dialog::render(frame, input, error_message, suggestions, *selected_suggestion, theme);
        }
    }
}

pub fn render_extract_options_dialog(frame: &mut Frame, archive_name: &str, dest: &std::path::Path, selected: usize, area: Rect, theme: &Theme) {
    let dialog_area = centered_rect(70, 35, area);
    
    frame.render_widget(Clear, dialog_area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Extract Archive ")
        .style(Style::default().bg(theme.dialog_bg).fg(theme.dialog_fg));
    
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
        .style(Style::default().fg(theme.dialog_fg));
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
            Span::styled(format!("→ {}", dest.display()), Style::default().fg(theme.info_color)),
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
            Span::styled(format!("→ {}", dest_with_folder.display()), Style::default().fg(theme.info_color)),
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

pub fn render_confirm_dialog(frame: &mut Frame, message: &str, area: Rect, theme: &Theme) {
    let dialog_area = centered_rect(60, 30, area);
    
    frame.render_widget(Clear, dialog_area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Confirm ")
        .style(Style::default().bg(theme.dialog_bg).fg(theme.warning_color));
    
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
        .style(Style::default().fg(theme.dialog_fg));
    frame.render_widget(text, chunks[0]);
    
    let confirm_text = Paragraph::new(Line::from(vec![
        Span::styled("y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw("es / "),
        Span::styled("n", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw("o / "),
        Span::styled("Esc", Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(confirm_text, chunks[1]);
}

pub fn render_input_dialog(frame: &mut Frame, prompt: &str, value: &str, area: Rect, theme: &Theme) {
    let dialog_area = centered_rect(60, 25, area);
    
    frame.render_widget(Clear, dialog_area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Input ")
        .style(Style::default().bg(theme.dialog_bg).fg(theme.dialog_fg));
    
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
        .alignment(Alignment::Left)
        .style(Style::default().fg(theme.dialog_fg));
    frame.render_widget(prompt_text, chunks[0]);
    
    let input_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(theme.dialog_fg));
    let input_text = Paragraph::new(value)
        .block(input_block);
    frame.render_widget(input_text, chunks[1]);
    
    let help_text = Paragraph::new("Enter: Confirm | Esc: Cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.info_color));
    frame.render_widget(help_text, chunks[2]);
}

pub fn render_progress_dialog(frame: &mut Frame, message: &str, area: Rect, theme: &Theme) {
    let dialog_area = centered_rect(70, 30, area);
    
    frame.render_widget(Clear, dialog_area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Operation in Progress ")
        .style(Style::default().bg(theme.dialog_bg).fg(Color::Green));
    
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);
    
    let text = Paragraph::new(message)
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.dialog_fg));
    frame.render_widget(text, inner);
}

pub fn render_progress_with_bar(
    frame: &mut Frame,
    message: &str,
    progress: &Progress,
    area: Rect,
    theme: &Theme,
) {
    let dialog_area = centered_rect(70, 35, area);
    
    frame.render_widget(Clear, dialog_area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Operation in Progress ")
        .style(Style::default().bg(theme.dialog_bg).fg(Color::Green));
    
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
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.dialog_fg));
    frame.render_widget(text, chunks[0]);
    
    let percentage = progress.percentage();
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green).bg(theme.dialog_bg))
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
        .style(Style::default().fg(theme.info_color));
    frame.render_widget(stats_text, chunks[2]);
}

pub fn render_error_dialog(frame: &mut Frame, message: &str, area: Rect, theme: &Theme) {
    let dialog_area = centered_rect(60, 30, area);
    
    frame.render_widget(Clear, dialog_area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Error ")
        .style(Style::default().bg(theme.dialog_bg).fg(theme.error_color));
    
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
        .style(Style::default().fg(theme.error_color));
    frame.render_widget(text, chunks[0]);
    
    // BUG-001 FIX: Make it clearer that ESC closes the error dialog
    let help_text = Paragraph::new("Presiona ESC para cerrar")
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.info_color));
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
pub fn render_password_input_dialog(frame: &mut Frame, prompt: &str, value: &str, show_password: bool, area: Rect, theme: &Theme) {
    let dialog_area = centered_rect(60, 30, area);
    
    frame.render_widget(Clear, dialog_area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 🔒 Password Required ")
        .style(Style::default().bg(theme.dialog_bg).fg(theme.warning_color));
    
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
        .style(Style::default().fg(theme.dialog_fg));
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
        .style(Style::default().fg(theme.info_color));
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
pub fn render_collision_dialog(frame: &mut Frame, file_path: &str, selected: usize, area: Rect, theme: &Theme) {
    let dialog_area = centered_rect(70, 40, area);
    
    frame.render_widget(Clear, dialog_area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" ⚠️  File Already Exists ")
        .style(Style::default().bg(theme.dialog_bg).fg(theme.warning_color));
    
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

// T930-T936: Render compression options dialog
#[allow(clippy::too_many_arguments)]
fn render_compress_options_dialog(
    frame: &mut Frame,
    sources: &[std::path::PathBuf],
    output_name: &str,
    format: &crate::archive::formats::ArchiveFormat,
    level: &crate::archive::compressor::CompressionLevel,
    use_password: bool,
    password: &str,
    confirm_password: &str,
    selected_field: usize,
    area: Rect,
    theme: &Theme,
) {
    let dialog_area = centered_rect(80, 70, area);
    
    frame.render_widget(Clear, dialog_area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Comprimir Archivos ")
        .style(Style::default().bg(theme.dialog_bg).fg(theme.dialog_fg));
    
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);
    
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2), // Source count
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Output name
            Constraint::Length(3), // Format selector
            Constraint::Length(3), // Compression level
            Constraint::Length(3), // Password checkbox
            Constraint::Length(3), // Password field (if enabled)
            Constraint::Length(3), // Confirm password (if enabled)
            Constraint::Length(2), // Estimated size
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Instructions
        ])
        .split(inner);
    
    // Source count
    let count_text = if sources.len() == 1 {
        "📦 Comprimir 1 elemento".to_string()
    } else {
        format!("📦 Comprimir {} elementos", sources.len())
    };
    let source_info = Paragraph::new(count_text)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(source_info, chunks[0]);
    
    // Output name field
    let name_style = if selected_field == 0 {
        Style::default().fg(Color::Black).bg(Color::Cyan)
    } else {
        Style::default().fg(Color::White)
    };
    
    // Get extension for current format
    let extension = match format {
        crate::archive::formats::ArchiveFormat::ZIP => ".zip",
        crate::archive::formats::ArchiveFormat::TarGz => ".tar.gz",
        crate::archive::formats::ArchiveFormat::TarBz2 => ".tar.bz2",
        crate::archive::formats::ArchiveFormat::TarXz => ".tar.xz",
        crate::archive::formats::ArchiveFormat::TAR => ".tar",
        crate::archive::formats::ArchiveFormat::SEVENZ => ".7z",
        _ => ".zip",
    };
    
    let name_field = Paragraph::new(Line::from(vec![
        Span::styled(" Nombre: ", Style::default().fg(Color::Green)),
        Span::styled(output_name, name_style),
        Span::styled("_", name_style),
        Span::styled(extension, Style::default().fg(Color::DarkGray)), // Show extension as hint
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(
        if selected_field == 0 {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    ));
    frame.render_widget(name_field, chunks[2]);
    
    // T931: Format selector
    let format_text = match format {
        crate::archive::formats::ArchiveFormat::ZIP => "ZIP (rápido, compatible)",
        crate::archive::formats::ArchiveFormat::TarGz => "TAR.GZ (Linux, buena compresión)",
        crate::archive::formats::ArchiveFormat::TarBz2 => "TAR.BZ2 (mejor compresión)",
        crate::archive::formats::ArchiveFormat::TarXz => "TAR.XZ (máxima compresión)",
        _ => "ZIP (rápido, compatible)",
    };
    let format_style = if selected_field == 1 {
        Style::default().fg(Color::Black).bg(Color::Cyan)
    } else {
        Style::default().fg(Color::White)
    };
    let format_field = Paragraph::new(Line::from(vec![
        Span::styled(" Formato: ", Style::default().fg(Color::Green)),
        Span::styled("◀ ", if selected_field == 1 { Style::default().fg(Color::Cyan) } else { Style::default().fg(Color::DarkGray) }),
        Span::styled(format_text, format_style),
        Span::styled(" ▶", if selected_field == 1 { Style::default().fg(Color::Cyan) } else { Style::default().fg(Color::DarkGray) }),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(
        if selected_field == 1 {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    ));
    frame.render_widget(format_field, chunks[3]);
    
    // T932: Compression level selector
    let level_text = match level {
        crate::archive::compressor::CompressionLevel::Fast => "Rápido (1)",
        crate::archive::compressor::CompressionLevel::Normal => "Normal (6)",
        crate::archive::compressor::CompressionLevel::Maximum => "Máximo (9)",
    };
    let level_style = if selected_field == 2 {
        Style::default().fg(Color::Black).bg(Color::Cyan)
    } else {
        Style::default().fg(Color::White)
    };
    let level_field = Paragraph::new(Line::from(vec![
        Span::styled(" Nivel: ", Style::default().fg(Color::Green)),
        Span::styled("◀ ", if selected_field == 2 { Style::default().fg(Color::Cyan) } else { Style::default().fg(Color::DarkGray) }),
        Span::styled(level_text, level_style),
        Span::styled(" ▶", if selected_field == 2 { Style::default().fg(Color::Cyan) } else { Style::default().fg(Color::DarkGray) }),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(
        if selected_field == 2 {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    ));
    frame.render_widget(level_field, chunks[4]);
    
    // T933: Password checkbox
    let checkbox_style = if selected_field == 3 {
        Style::default().fg(Color::Black).bg(Color::Cyan)
    } else {
        Style::default().fg(Color::White)
    };
    let checkbox_text = if use_password { "[✓]" } else { "[ ]" };
    let password_checkbox = Paragraph::new(Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(checkbox_text, checkbox_style),
        Span::styled(" Proteger con contraseña", checkbox_style),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(
        if selected_field == 3 {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    ));
    frame.render_widget(password_checkbox, chunks[5]);
    
    // T934: Password fields (only if checkbox enabled)
    if use_password {
        let pass_style = if selected_field == 4 {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        let masked = "*".repeat(password.len());
        let password_field = Paragraph::new(Line::from(vec![
            Span::styled(" Contraseña: ", Style::default().fg(Color::Green)),
            Span::styled(&masked, pass_style),
            Span::styled("_", pass_style),
        ]))
        .block(Block::default().borders(Borders::ALL).border_style(
            if selected_field == 4 {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            }
        ));
        frame.render_widget(password_field, chunks[6]);
        
        let confirm_style = if selected_field == 5 {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        let masked_confirm = "*".repeat(confirm_password.len());
        let confirm_field = Paragraph::new(Line::from(vec![
            Span::styled(" Confirmar: ", Style::default().fg(Color::Green)),
            Span::styled(&masked_confirm, confirm_style),
            Span::styled("_", confirm_style),
        ]))
        .block(Block::default().borders(Borders::ALL).border_style(
            if selected_field == 5 {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            }
        ));
        frame.render_widget(confirm_field, chunks[7]);
    }
    
    // T935: Estimated size (placeholder for now)
    let estimated_text = "Tamaño estimado: calculando...";
    let estimate_info = Paragraph::new(estimated_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(estimate_info, chunks[8]);
    
    // Instructions
    let instructions = Paragraph::new(Line::from(vec![
        Span::styled("Tab/↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(": Navegar | "),
        Span::styled("←→", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(": Cambiar | "),
        Span::styled("Space", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(": Toggle | "),
        Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(": Comprimir | "),
        Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw(": Cancelar"),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(instructions, chunks[10]);
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
