// Application state management
use crate::models::panel::Panel;
use crate::models::operation::Operation;
use crate::models::file_entry::{FileEntry, EntryType};
use crate::models::selection::SelectionState;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpTarget {
    Start,
    End,
}

pub struct AppState {
    pub left_panel: Panel,
    pub right_panel: Panel,
    pub active_panel: PanelSide,
    pub current_operation: Option<Operation>,
    pub dialog_state: Option<DialogState>,
    pub error_message: Option<String>,
    pub search_mode: bool,
    pub search_pattern: String,
    pub left_all_entries: Vec<FileEntry>,
    pub right_all_entries: Vec<FileEntry>,
    pub selection_state: SelectionState,
    pub preview_state: Option<PreviewState>,
}

#[derive(Debug, Clone)]
pub enum DialogState {
    Confirm {
        message: String,
        confirm_action: ConfirmAction,
    },
    Input {
        prompt: String,
        value: String,
    },
    Progress {
        message: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone)]
pub enum PreviewState {
    Text {
        content: String,
        scroll_offset: usize,
        total_lines: usize,
        file_path: PathBuf,
        file_size: u64,
        warning: Option<String>,
    },
}

impl PreviewState {
    pub fn scroll_offset(&self) -> usize {
        match self {
            PreviewState::Text { scroll_offset, .. } => *scroll_offset,
        }
    }

    pub fn total_lines(&self) -> usize {
        match self {
            PreviewState::Text { total_lines, .. } => *total_lines,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmAction {
    Copy,
    Move,
    Delete,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        // T507: Load persisted state
        let persisted = crate::config::state::PersistedState::load()
            .unwrap_or_default();

        Ok(Self {
            left_panel: Panel::new(persisted.left_panel_path),
            right_panel: Panel::new(persisted.right_panel_path),
            active_panel: persisted.active_panel.into(),
            current_operation: None,
            dialog_state: None,
            error_message: None,
            search_mode: false,
            search_pattern: String::new(),
            left_all_entries: Vec::new(),
            right_all_entries: Vec::new(),
            selection_state: SelectionState::new(),
            preview_state: None,
        })
    }

    /// Save current state to config file
    pub fn save_state(&self) -> anyhow::Result<()> {
        let state = crate::config::state::PersistedState {
            left_panel_path: self.left_panel.current_path.clone(),
            right_panel_path: self.right_panel.current_path.clone(),
            active_panel: self.active_panel.into(),
        };
        
        state.save()
    }

    pub fn active_panel_mut(&mut self) -> &mut Panel {
        match self.active_panel {
            PanelSide::Left => &mut self.left_panel,
            PanelSide::Right => &mut self.right_panel,
        }
    }

    pub fn active_panel(&self) -> &Panel {
        match self.active_panel {
            PanelSide::Left => &self.left_panel,
            PanelSide::Right => &self.right_panel,
        }
    }

    pub fn inactive_panel(&self) -> &Panel {
        match self.active_panel {
            PanelSide::Left => &self.right_panel,
            PanelSide::Right => &self.left_panel,
        }
    }

    pub fn switch_panel(&mut self) {
        self.active_panel = match self.active_panel {
            PanelSide::Left => PanelSide::Right,
            PanelSide::Right => PanelSide::Left,
        };
    }
    
    pub fn show_confirm_dialog(&mut self, message: String, action: ConfirmAction) {
        self.dialog_state = Some(DialogState::Confirm {
            message,
            confirm_action: action,
        });
    }

    pub fn show_error(&mut self, message: String) {
        self.dialog_state = Some(DialogState::Error { message });
    }
    
    pub fn show_input_dialog(&mut self, prompt: String) {
        self.dialog_state = Some(DialogState::Input {
            prompt,
            value: String::new(),
        });
    }
    
    pub fn input_dialog_append(&mut self, ch: char) {
        if let Some(DialogState::Input { value, .. }) = &mut self.dialog_state {
            value.push(ch);
        }
    }
    
    pub fn input_dialog_backspace(&mut self) {
        if let Some(DialogState::Input { value, .. }) = &mut self.dialog_state {
            value.pop();
        }
    }
    
    pub fn get_input_value(&self) -> Option<String> {
        if let Some(DialogState::Input { value, .. }) = &self.dialog_state {
            Some(value.clone())
        } else {
            None
        }
    }

    // T568-T569: Selection methods
    pub fn toggle_selection(&mut self) {
        let panel = self.active_panel();
        if let Some(entry) = panel.entries.get(panel.cursor) {
            let is_marked = self.selection_state.toggle_mark(self.active_panel, entry.path.clone());
            
            // T568: Advance cursor to next item after marking
            if is_marked {
                self.active_panel_mut().move_cursor_down();
            }
        }
    }

    pub fn select_all(&mut self) {
        let panel = self.active_panel();
        let paths: Vec<PathBuf> = panel.entries.iter().map(|e| e.path.clone()).collect();
        self.selection_state.mark_all(self.active_panel, paths);
    }

    pub fn clear_selection(&mut self) {
        self.selection_state.clear(self.active_panel);
    }

    pub fn has_selection(&self) -> bool {
        self.selection_state.has_marked(self.active_panel)
    }

    pub fn close_dialog(&mut self) {
        self.dialog_state = None;
    }

    pub fn has_dialog(&self) -> bool {
        self.dialog_state.is_some()
    }

    // T612-T615: Preview management methods
    pub async fn open_text_preview(&mut self) -> anyhow::Result<()> {
        let panel = self.active_panel();
        if let Some(entry) = panel.entries.get(panel.cursor) {
            let path = &entry.path;

            // Check if it's a text file
            if !crate::preview::is_text_file(path) {
                self.show_error("Cannot preview: not a text file".to_string());
                return Ok(());
            }

            // Check if it's a file (not a directory)
            if entry.entry_type == EntryType::Dir {
                self.show_error("Cannot preview a directory".to_string());
                return Ok(());
            }

            // Load the file
            match crate::preview::load_text_file(path).await {
                Ok((content, warning)) => {
                    let total_lines = content.lines().count();
                    let file_size = entry.size;

                    self.preview_state = Some(PreviewState::Text {
                        content,
                        scroll_offset: 0,
                        total_lines,
                        file_path: path.clone(),
                        file_size,
                        warning,
                    });
                }
                Err(e) => {
                    self.show_error(format!("Failed to load file: {}", e));
                }
            }
        }

        Ok(())
    }

    pub fn close_preview(&mut self) {
        self.preview_state = None;
    }

    pub fn scroll_preview(&mut self, direction: i32) {
        if let Some(PreviewState::Text {
            scroll_offset,
            total_lines,
            ..
        }) = &mut self.preview_state
        {
            if direction > 0 {
                // Scroll down
                *scroll_offset = (*scroll_offset + direction as usize).min(*total_lines - 1);
            } else if direction < 0 {
                // Scroll up
                *scroll_offset = scroll_offset.saturating_sub((-direction) as usize);
            }
        }
    }

    pub fn jump_preview(&mut self, target: JumpTarget) {
        if let Some(PreviewState::Text {
            scroll_offset,
            total_lines,
            ..
        }) = &mut self.preview_state
        {
            match target {
                JumpTarget::Start => *scroll_offset = 0,
                JumpTarget::End => *scroll_offset = total_lines.saturating_sub(1),
            }
        }
    }

    pub fn has_preview(&self) -> bool {
        self.preview_state.is_some()
    }

    // T411: Activate search mode
    pub fn activate_search(&mut self) {
        self.search_mode = true;
        self.search_pattern.clear();
        
        // Ensure all_entries is populated for active panel
        if self.get_all_entries_for_active().is_empty() {
            let entries = self.active_panel().entries.clone();
            self.store_all_entries(entries);
        }
    }

    // T414: Deactivate search mode and clear filter
    pub fn deactivate_search(&mut self) {
        self.search_mode = false;
        self.search_pattern.clear();
        
        // Clear filter on active panel
        let mut all_entries = self.get_all_entries_for_active();
        
        // If all_entries is empty, refresh from panel
        if all_entries.is_empty() {
            all_entries = self.active_panel().entries.clone();
        }
        
        self.active_panel_mut().clear_filter(&all_entries);
    }

    // T412: Append character to search pattern
    pub fn search_append(&mut self, ch: char) {
        if self.search_mode {
            self.search_pattern.push(ch);
            self.apply_current_filter();
        }
    }

    // T412: Remove last character from search pattern
    pub fn search_backspace(&mut self) {
        if self.search_mode && !self.search_pattern.is_empty() {
            self.search_pattern.pop();
            self.apply_current_filter();
        }
    }

    // T413: Apply current search pattern as filter
    fn apply_current_filter(&mut self) {
        let mut all_entries = self.get_all_entries_for_active();
        
        // If all_entries is empty, use current panel entries
        if all_entries.is_empty() {
            all_entries = self.active_panel().entries.clone();
            self.store_all_entries(all_entries.clone());
        }
        
        let pattern = self.search_pattern.clone();
        self.active_panel_mut().apply_filter(&pattern, &all_entries);
    }

    // Get all entries for active panel
    fn get_all_entries_for_active(&self) -> Vec<FileEntry> {
        match self.active_panel {
            PanelSide::Left => self.left_all_entries.clone(),
            PanelSide::Right => self.right_all_entries.clone(),
        }
    }

    // Update all entries storage after directory refresh
    pub fn store_all_entries(&mut self, entries: Vec<FileEntry>) {
        match self.active_panel {
            PanelSide::Left => self.left_all_entries = entries,
            PanelSide::Right => self.right_all_entries = entries,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            Self {
                left_panel: Panel::new(PathBuf::from(".")),
                right_panel: Panel::new(PathBuf::from(".")),
                active_panel: PanelSide::Left,
                current_operation: None,
                dialog_state: None,
                error_message: None,
                search_mode: false,
                search_pattern: String::new(),
                left_all_entries: Vec::new(),
                right_all_entries: Vec::new(),
                selection_state: SelectionState::new(),
                preview_state: None,
            }
        })
    }
}
