use crate::search::{RecursiveSearcher, SearchResult};
use crate::ui::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use std::path::PathBuf;

pub enum DialogAction {
    Continue,
    Close,
    Navigate(SearchResult),
}

pub struct SearchDialog {
    input: String,
    results: Vec<SearchResult>,
    selected_index: usize,
    scroll_offset: usize,
    is_searching: bool,
    files_scanned: usize,
    searcher: Option<RecursiveSearcher>,
    root_path: PathBuf,
}

impl SearchDialog {
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            input: String::new(),
            results: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            is_searching: false,
            files_scanned: 0,
            searcher: None,
            root_path,
        }
    }
    
    pub fn handle_key(&mut self, key: KeyEvent) -> DialogAction {
        match (key.code, key.modifiers) {
            // Close dialog
            (KeyCode::Esc, _) => {
                if let Some(ref searcher) = self.searcher {
                    searcher.cancel();
                }
                return DialogAction::Close;
            }
            
            // Navigate to selected result
            (KeyCode::Enter, _) => {
                if !self.results.is_empty() && self.selected_index < self.results.len() {
                    let result = self.results[self.selected_index].clone();
                    if let Some(ref searcher) = self.searcher {
                        searcher.cancel();
                    }
                    return DialogAction::Navigate(result);
                }
            }
            
            // Navigate results
            (KeyCode::Up, _) => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    // Adjust scroll if needed
                    if self.selected_index < self.scroll_offset {
                        self.scroll_offset = self.selected_index;
                    }
                }
            }
            
            (KeyCode::Down, _) => {
                if self.selected_index + 1 < self.results.len() {
                    self.selected_index += 1;
                }
            }
            
            // Edit input
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                self.input.push(c);
                self.start_search();
            }
            
            (KeyCode::Backspace, _) => {
                self.input.pop();
                if !self.input.is_empty() {
                    self.start_search();
                } else {
                    // Clear results if input is empty
                    self.results.clear();
                    self.selected_index = 0;
                    self.scroll_offset = 0;
                    if let Some(ref searcher) = self.searcher {
                        searcher.cancel();
                    }
                }
            }
            
            _ => {}
        }
        
        // Update results if search is running
        self.update_results();
        
        DialogAction::Continue
    }
    
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Center the dialog
        let dialog_width = area.width.saturating_sub(10).min(100);
        let dialog_height = area.height.saturating_sub(6).min(30);
        
        let dialog_area = Rect {
            x: (area.width.saturating_sub(dialog_width)) / 2,
            y: (area.height.saturating_sub(dialog_height)) / 2,
            width: dialog_width,
            height: dialog_height,
        };
        
        // Clear area behind dialog
        frame.render_widget(Clear, dialog_area);
        
        // Split into sections: title + input + results + status
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Input field
                Constraint::Min(5),     // Results list
                Constraint::Length(2),  // Status bar
            ])
            .split(dialog_area);
        
        // Render input field
        self.render_input_field(frame, chunks[0], theme);
        
        // Render results list
        self.render_results_list(frame, chunks[1], theme);
        
        // Render status/progress
        self.render_progress(frame, chunks[2], theme);
    }
    
    fn render_input_field(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let input_text = format!("Search: {}", self.input);
        let cursor_pos = if self.input.is_empty() { "█" } else { "" };
        
        let input = Paragraph::new(format!("{}{}", input_text, cursor_pos))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Recursive Search (Ctrl+F)")
                    .border_style(Style::default().fg(theme.active_border))
            )
            .style(Style::default().fg(theme.dialog_fg).bg(theme.dialog_bg));
        
        frame.render_widget(input, area);
    }
    
    fn render_results_list(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if self.results.is_empty() {
            let message = if self.input.is_empty() {
                "Type to search recursively through subdirectories..."
            } else if self.is_searching {
                "Searching..."
            } else {
                "No results found"
            };
            
            let empty = Paragraph::new(message)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(theme.footer_fg).bg(theme.dialog_bg))
                .alignment(Alignment::Center);
            
            frame.render_widget(empty, area);
            return;
        }
        
        // Calculate visible range
        let list_height = area.height.saturating_sub(2) as usize; // Account for borders
        let visible_start = self.scroll_offset;
        let visible_end = (visible_start + list_height).min(self.results.len());
        
        // Adjust scroll offset if selection is out of view
        let _scroll_offset = if self.selected_index >= visible_end {
            self.selected_index.saturating_sub(list_height.saturating_sub(1))
        } else if self.selected_index < visible_start {
            self.selected_index
        } else {
            self.scroll_offset
        };
        
        // Create list items
        let items: Vec<ListItem> = self.results[visible_start..visible_end]
            .iter()
            .enumerate()
            .map(|(idx, result)| {
                let actual_idx = visible_start + idx;
                let is_selected = actual_idx == self.selected_index;
                
                // Format: "  relative/path/to/file.txt    1.2 KB  Today"
                let path_str = result.relative_path.display().to_string();
                let size_str = format_size(result.file_size);
                let date_str = format_date(&result.modified_time);
                
                let line = format!("{:<50} {:>10}  {}", path_str, size_str, date_str);
                
                let style = if is_selected {
                    Style::default()
                        .fg(theme.highlight_fg)
                        .bg(theme.highlight_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.dialog_fg).bg(theme.dialog_bg)
                };
                
                ListItem::new(Line::from(Span::styled(line, style)))
            })
            .collect();
        
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(
                        "Results ({} found, {} files scanned)",
                        self.results.len(),
                        self.files_scanned
                    ))
            )
            .style(Style::default().bg(theme.dialog_bg));
        
        frame.render_widget(list, area);
    }
    
    fn render_progress(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let status_text = if self.is_searching {
            "Searching... Press Esc to cancel"
        } else if !self.results.is_empty() {
            "↑↓ Navigate | Enter Select | Esc Close"
        } else {
            "Press Esc to close"
        };
        
        let status = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(theme.footer_fg).bg(theme.dialog_bg))
            .alignment(Alignment::Center);
        
        frame.render_widget(status, area);
    }
    
    fn start_search(&mut self) {
        if self.input.is_empty() {
            return;
        }
        
        // Cancel previous search
        if let Some(ref searcher) = self.searcher {
            searcher.cancel();
        }
        
        // Start new search
        let searcher = RecursiveSearcher::new(self.input.clone(), self.root_path.clone());
        searcher.start_search();
        
        self.searcher = Some(searcher);
        self.is_searching = true;
        self.results.clear();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }
    
    fn update_results(&mut self) {
        if let Some(ref searcher) = self.searcher {
            self.results = searcher.get_results();
            self.files_scanned = searcher.files_scanned();
            self.is_searching = searcher.is_running();
        }
    }
}

// Helper functions
fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn format_date(time: &std::time::SystemTime) -> String {
    use chrono::{DateTime, Local};
    
    let datetime: DateTime<Local> = (*time).into();
    let now = Local::now();
    
    if datetime.date_naive() == now.date_naive() {
        "Today".to_string()
    } else if datetime.date_naive() == now.date_naive() - chrono::Days::new(1) {
        "Yesterday".to_string()
    } else {
        datetime.format("%b %d").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dialog_creation() {
        let dialog = SearchDialog::new(PathBuf::from("/test"));
        assert_eq!(dialog.input, "");
        assert_eq!(dialog.results.len(), 0);
        assert!(!dialog.is_searching);
    }
    
    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1024 * 1024 * 2), "2.0 MB");
    }
}
