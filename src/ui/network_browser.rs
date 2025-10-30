// Network browser UI for SMB discovery
use crate::app::{NetworkBrowserState, NetworkBrowserView};
use crate::ui::theme::Theme;
use crate::ui::utils::centered_rect;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, state: &NetworkBrowserState, theme: &Theme) {
    let area = centered_rect(70, 70, frame.area());
    
    frame.render_widget(Clear, area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 🌐 Network Browser ")
        .style(Style::default().bg(theme.dialog_bg).fg(theme.dialog_fg));
    
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Title/breadcrumb
            Constraint::Min(10),    // List
            Constraint::Length(2),  // Status/error
            Constraint::Length(2),  // Help
        ])
        .split(inner);
    
    // Title/breadcrumb
    let title_text = match &state.view_mode {
        NetworkBrowserView::ServerList => "Network Servers",
        NetworkBrowserView::ShareList(server) => &format!("Shares on {}", server),
    };
    let title = Paragraph::new(title_text)
        .style(Style::default().fg(theme.dialog_fg).add_modifier(Modifier::BOLD));
    frame.render_widget(title, chunks[0]);
    
    // List
    match &state.view_mode {
        NetworkBrowserView::ServerList => {
            render_server_list(frame, state, chunks[1], theme);
        }
        NetworkBrowserView::ShareList(_) => {
            render_share_list(frame, state, chunks[1], theme);
        }
    }
    
    // Status/error
    if state.loading {
        let loading = Paragraph::new("⏳ Loading...")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        frame.render_widget(loading, chunks[2]);
    } else if let Some(err) = &state.error {
        let error = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        frame.render_widget(error, chunks[2]);
    }
    
    // Help
    let help_text = match &state.view_mode {
        NetworkBrowserView::ServerList => {
            vec![
                Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(": Navigate | "),
                Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(": View Shares | "),
                Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(": Cancel"),
            ]
        }
        NetworkBrowserView::ShareList(_) => {
            vec![
                Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(": Navigate | "),
                Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(": Connect | "),
                Span::styled("Backspace", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(": Back | "),
                Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(": Cancel"),
            ]
        }
    };
    
    let help = Paragraph::new(Line::from(help_text))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[3]);
}

fn render_server_list(frame: &mut Frame, state: &NetworkBrowserState, area: Rect, theme: &Theme) {
    if state.servers.is_empty() {
        let empty = Paragraph::new("No servers found on network")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }
    
    let items: Vec<ListItem> = state
        .servers
        .iter()
        .enumerate()
        .map(|(i, server)| {
            let style = if i == state.selected_server {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.dialog_fg)
            };
            
            let content = if let Some(ref comment) = server.comment {
                format!("🖥️  {}  ({})", server.name, comment)
            } else {
                format!("🖥️  {}", server.name)
            };
            
            ListItem::new(content).style(style)
        })
        .collect();
    
    let list = List::new(items)
        .style(Style::default().bg(theme.dialog_bg));
    
    frame.render_widget(list, area);
}

fn render_share_list(frame: &mut Frame, state: &NetworkBrowserState, area: Rect, theme: &Theme) {
    // For now, show a placeholder - we'll need to load shares when entering this view
    let placeholder = Paragraph::new("Loading shares...")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(placeholder, area);
}
