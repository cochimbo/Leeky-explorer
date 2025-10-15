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
    }
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
