// Remote connection dialog UI
use crate::remote::{AuthMethod, ConnectionConfig, ConnectionManager, ConnectionType};
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

#[derive(Debug, Clone)]
pub enum ConnectionDialogState {
    SavedConnections {
        selected: usize,  // Selected connection
        connections: Vec<ConnectionConfig>,
    },
    TypeSelection {
        selected: usize,  // 0=SFTP, 1=SMB
    },
    SftpForm {
        selected_field: usize,  // 0=name, 1=host, 2=port, 3=username, 4=auth_type, 5=password/key_path, 6=save_credentials
        name: String,
        host: String,
        port: String,
        username: String,
        auth_type: usize,  // 0=password, 1=key
        password: String,
        key_path: String,
        save_credentials: bool,
        error: Option<String>,
    },
    SmbForm {
        selected_field: usize,  // 0=name, 1=host, 2=share, 3=username, 4=password, 5=domain, 6=guest_mode, 7=save_credentials
        name: String,
        host: String,
        share: String,
        username: String,
        password: String,
        domain: String,
        guest_mode: bool,
        save_credentials: bool,
        error: Option<String>,
    },
}

impl ConnectionDialogState {
    pub fn new() -> Self {
        // Try to load saved connections first
        match ConnectionManager::load() {
            Ok(manager) if !manager.is_empty() => {
                Self::SavedConnections {
                    selected: 0,
                    connections: manager.list().to_vec(),
                }
            }
            _ => Self::TypeSelection { selected: 0 }
        }
    }
    
    pub fn select_type(&mut self, conn_type: usize) {
        match conn_type {
            0 => {
                // SFTP
                *self = Self::SftpForm {
                    selected_field: 0,
                    name: String::new(),
                    host: String::new(),
                    port: "22".to_string(),
                    username: String::new(),
                    auth_type: 0,
                    password: String::new(),
                    key_path: String::new(),
                    save_credentials: true,  // Default to saving
                    error: None,
                };
            }
            1 => {
                // SMB
                *self = Self::SmbForm {
                    selected_field: 0,
                    name: String::new(),
                    host: String::new(),
                    share: String::new(),
                    username: String::new(),
                    password: String::new(),
                    domain: String::new(),
                    guest_mode: false,
                    save_credentials: true,  // Default to saving
                    error: None,
                };
            }
            _ => {}
        }
    }
    
    pub fn to_config(&self) -> Option<ConnectionConfig> {
        match self {
            Self::SftpForm {
                name,
                host,
                port,
                username,
                auth_type,
                password,
                key_path,
                save_credentials,
                ..
            } => {
                if name.is_empty() || host.is_empty() || username.is_empty() {
                    return None;
                }
                
                let port = port.parse().unwrap_or(22);
                let auth = if *auth_type == 0 {
                    if password.is_empty() {
                        return None;
                    }
                    AuthMethod::Password {
                        password: Some(password.clone()),
                        stored: *save_credentials,
                    }
                } else {
                    if key_path.is_empty() {
                        return None;
                    }
                    AuthMethod::PublicKey {
                        key_path: std::path::PathBuf::from(key_path),
                        passphrase: None,
                        stored: false,
                    }
                };
                
                Some(ConnectionConfig {
                    name: name.clone(),
                    connection_type: ConnectionType::Sftp,
                    host: host.clone(),
                    port,
                    username: username.clone(),
                    auth,
                    initial_path: Some(std::path::PathBuf::from("/")),
                })
            }
            Self::SmbForm {
                name,
                host,
                share,
                username,
                password,
                guest_mode,
                save_credentials,
                ..
            } => {
                if name.is_empty() || host.is_empty() || share.is_empty() {
                    return None;
                }
                
                // If guest mode, allow empty username/password
                let final_username = if *guest_mode {
                    "guest".to_string()
                } else {
                    if username.is_empty() {
                        return None;
                    }
                    username.clone()
                };
                
                let final_password = if *guest_mode {
                    None
                } else {
                    Some(password.clone())
                };
                
                Some(ConnectionConfig {
                    name: name.clone(),
                    connection_type: ConnectionType::Smb,
                    host: host.clone(),
                    port: 445,  // Default SMB port
                    username: final_username,
                    auth: AuthMethod::Password {
                        password: final_password,
                        stored: *save_credentials,
                    },
                    initial_path: Some(std::path::PathBuf::from(format!("/{}", share))),
                })
            }
            Self::SavedConnections { selected, connections } => {
                connections.get(*selected).cloned()
            }
            _ => None,
        }
    }
}

pub fn render(frame: &mut Frame, state: &ConnectionDialogState, theme: &Theme) {
    let area = centered_rect(70, 70, frame.area());
    
    frame.render_widget(Clear, area);
    
    match state {
        ConnectionDialogState::SavedConnections { .. } => {
            render_saved_connections(frame, state, area, theme);
        }
        ConnectionDialogState::TypeSelection { selected } => {
            render_type_selection(frame, *selected, area, theme);
        }
        ConnectionDialogState::SftpForm { .. } => {
            render_sftp_form(frame, state, area, theme);
        }
        ConnectionDialogState::SmbForm { .. } => {
            render_smb_form(frame, state, area, theme);
        }
    }
}

fn render_saved_connections(frame: &mut Frame, state: &ConnectionDialogState, area: Rect, theme: &Theme) {
    let ConnectionDialogState::SavedConnections { selected, connections } = state else {
        return;
    };
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 📚 Saved Connections ")
        .style(Style::default().bg(theme.dialog_bg).fg(theme.dialog_fg));
    
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(inner);
    
    // Render connection list
    let items: Vec<ListItem> = connections
        .iter()
        .enumerate()
        .map(|(i, conn)| {
            let style = if i == *selected {
                Style::default().fg(theme.highlight_fg).bg(theme.highlight_bg)
            } else {
                Style::default().fg(theme.dialog_fg)
            };
            
            let content = format!(
                "{} - {}@{}:{}",
                conn.connection_type.as_str(),
                conn.username,
                conn.host,
                conn.port
            );
            
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", conn.name), style.add_modifier(Modifier::BOLD)),
                Span::styled(content, style),
            ]))
        })
        .collect();
    
    let list = List::new(items)
        .highlight_style(Style::default().fg(theme.highlight_fg).bg(theme.highlight_bg))
        .highlight_symbol("► ");
    
    frame.render_widget(list, chunks[0]);
    
    // Help text
    let help = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(theme.highlight_fg).add_modifier(Modifier::BOLD)),
            Span::raw(" Connect  "),
            Span::styled("N", Style::default().fg(theme.highlight_fg).add_modifier(Modifier::BOLD)),
            Span::raw(" New  "),
            Span::styled("Del", Style::default().fg(theme.highlight_fg).add_modifier(Modifier::BOLD)),
            Span::raw(" Delete  "),
            Span::styled("Esc", Style::default().fg(theme.highlight_fg).add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel"),
        ]),
    ])
    .alignment(Alignment::Center)
    .style(Style::default().fg(theme.dialog_fg));
    
    frame.render_widget(help, chunks[1]);
}

fn render_type_selection(frame: &mut Frame, selected: usize, area: Rect, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 🌐 New Remote Connection ")
        .style(Style::default().bg(theme.dialog_bg).fg(theme.dialog_fg));
    
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Title
            Constraint::Min(10),    // List
            Constraint::Length(2),  // Help
        ])
        .split(inner);
    
    // Title
    let title = Paragraph::new("Select connection type:")
        .style(Style::default().fg(theme.dialog_fg));
    frame.render_widget(title, chunks[0]);
    
    // Connection types
    let types = vec![
        ("SFTP", "Secure File Transfer Protocol (SSH)"),
        ("SMB", "Server Message Block (Windows shares)"),
    ];
    
    let items: Vec<ListItem> = types
        .iter()
        .enumerate()
        .map(|(i, (name, desc))| {
            let style = if i == selected {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            
            let prefix = if i == selected { "▶ " } else { "  " };
            
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(*name, style),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(*desc, Style::default().fg(theme.info_color)),
                ]),
            ])
        })
        .collect();
    
    let list = List::new(items).style(Style::default().bg(theme.dialog_bg));
    frame.render_widget(list, chunks[1]);
    
    // Help
    let help = Paragraph::new(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(": Select | "),
        Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(": Continue | "),
        Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw(": Cancel"),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}

fn render_sftp_form(frame: &mut Frame, state: &ConnectionDialogState, area: Rect, theme: &Theme) {
    let (selected_field, name, host, port, username, auth_type, password, key_path, save_credentials, error) = match state {
        ConnectionDialogState::SftpForm {
            selected_field,
            name,
            host,
            port,
            username,
            auth_type,
            password,
            key_path,
            save_credentials,
            error,
        } => (*selected_field, name, host, port, username, *auth_type, password, key_path, *save_credentials, error),
        _ => return,
    };
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 🔐 SFTP Connection ")
        .style(Style::default().bg(theme.dialog_bg).fg(theme.dialog_fg));
    
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Name
            Constraint::Length(2),  // Host
            Constraint::Length(2),  // Port
            Constraint::Length(2),  // Username
            Constraint::Length(2),  // Auth type
            Constraint::Length(2),  // Password/Key
            Constraint::Length(2),  // Save credentials checkbox
            Constraint::Length(2),  // Error
            Constraint::Length(2),  // Help
        ])
        .split(inner);
    
    render_field(frame, chunks[0], "Connection Name", name, selected_field == 0, theme);
    render_field(frame, chunks[1], "Host", host, selected_field == 1, theme);
    render_field(frame, chunks[2], "Port", port, selected_field == 2, theme);
    render_field(frame, chunks[3], "Username", username, selected_field == 3, theme);
    
    // Auth type selector
    let auth_text = if auth_type == 0 { "Password" } else { "Public Key" };
    let auth_style = if selected_field == 4 {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.dialog_fg)
    };
    let auth_line = Paragraph::new(format!("Auth Type: {} [Tab to change]", auth_text))
        .style(auth_style);
    frame.render_widget(auth_line, chunks[4]);
    
    // Password or key path
    if auth_type == 0 {
        render_password_field(frame, chunks[5], "Password", password, selected_field == 5, theme);
    } else {
        render_field(frame, chunks[5], "Key Path", key_path, selected_field == 5, theme);
    }
    
    // Save credentials checkbox
    let checkbox_icon = if save_credentials { "☑" } else { "☐" };
    let checkbox_style = if selected_field == 6 {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.dialog_fg)
    };
    let checkbox_line = Paragraph::new(format!("{} Guardar credenciales [Space to toggle]", checkbox_icon))
        .style(checkbox_style);
    frame.render_widget(checkbox_line, chunks[6]);
    
    // Error message
    if let Some(err) = error {
        let error_msg = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red));
        frame.render_widget(error_msg, chunks[7]);
    }
    
    // Help
    let help = Paragraph::new(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(": Navigate | "),
        Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(": Connect | "),
        Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw(": Cancel"),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(help, chunks[8]);
}

fn render_smb_form(frame: &mut Frame, state: &ConnectionDialogState, area: Rect, theme: &Theme) {
    let (selected_field, name, host, share, username, password, domain, guest_mode, save_credentials, error) = match state {
        ConnectionDialogState::SmbForm {
            selected_field,
            name,
            host,
            share,
            username,
            password,
            domain,
            guest_mode,
            save_credentials,
            error,
        } => (*selected_field, name, host, share, username, password, domain, *guest_mode, *save_credentials, error),
        _ => return,
    };
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 🖧 SMB/CIFS Connection ")
        .style(Style::default().bg(theme.dialog_bg).fg(theme.dialog_fg));
    
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Name
            Constraint::Length(2),  // Host
            Constraint::Length(2),  // Share
            Constraint::Length(2),  // Username
            Constraint::Length(2),  // Password
            Constraint::Length(2),  // Domain
            Constraint::Length(2),  // Guest mode checkbox
            Constraint::Length(2),  // Save credentials checkbox
            Constraint::Length(2),  // Error
            Constraint::Length(2),  // Help
        ])
        .split(inner);
    
    render_field(frame, chunks[0], "Connection Name", name, selected_field == 0, theme);
    render_field(frame, chunks[1], "Host/Server", host, selected_field == 1, theme);
    render_field(frame, chunks[2], "Share Name", share, selected_field == 2, theme);
    
    // If guest mode, disable username/password fields
    if guest_mode {
        let guest_style = Style::default().fg(Color::Gray);
        let guest_text = format!("Username: {} (guest mode)", username);
        let guest_field = Paragraph::new(guest_text).style(guest_style);
        frame.render_widget(guest_field, chunks[3]);
        
        let pwd_text = "Password: (not required in guest mode)";
        let pwd_field = Paragraph::new(pwd_text).style(guest_style);
        frame.render_widget(pwd_field, chunks[4]);
    } else {
        render_field(frame, chunks[3], "Username", username, selected_field == 3, theme);
        render_password_field(frame, chunks[4], "Password", password, selected_field == 4, theme);
    }
    
    render_field(frame, chunks[5], "Domain (optional)", domain, selected_field == 5, theme);
    
    // Guest mode checkbox
    let guest_checkbox_icon = if guest_mode { "☑" } else { "☐" };
    let guest_checkbox_style = if selected_field == 6 {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.dialog_fg)
    };
    let guest_checkbox_line = Paragraph::new(format!("{} Guest mode [Space to toggle]", guest_checkbox_icon))
        .style(guest_checkbox_style);
    frame.render_widget(guest_checkbox_line, chunks[6]);
    
    // Save credentials checkbox (disabled in guest mode)
    if guest_mode {
        let checkbox_line = Paragraph::new("☐ Guardar credenciales (disabled in guest mode)")
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(checkbox_line, chunks[7]);
    } else {
        let checkbox_icon = if save_credentials { "☑" } else { "☐" };
        let checkbox_style = if selected_field == 7 {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.dialog_fg)
        };
        let checkbox_line = Paragraph::new(format!("{} Guardar credenciales [Space to toggle]", checkbox_icon))
            .style(checkbox_style);
        frame.render_widget(checkbox_line, chunks[7]);
    }
    
    // Error message
    if let Some(err) = error {
        let error_msg = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red));
        frame.render_widget(error_msg, chunks[8]);
    }
    
    // Help
    let help = Paragraph::new(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(": Navigate | "),
        Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(": Connect | "),
        Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw(": Cancel"),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(help, chunks[9]);
}

fn render_field(frame: &mut Frame, area: Rect, label: &str, value: &str, selected: bool, theme: &Theme) {
    let style = if selected {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.dialog_fg)
    };
    
    let cursor = if selected { "█" } else { "" };
    let text = format!("{}: {}{}", label, value, cursor);
    let field = Paragraph::new(text).style(style);
    frame.render_widget(field, area);
}

fn render_password_field(frame: &mut Frame, area: Rect, label: &str, value: &str, selected: bool, theme: &Theme) {
    let style = if selected {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.dialog_fg)
    };
    
    let cursor = if selected { "█" } else { "" };
    let masked = "*".repeat(value.len());
    let text = format!("{}: {}{}", label, masked, cursor);
    let field = Paragraph::new(text).style(style);
    frame.render_widget(field, area);
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
