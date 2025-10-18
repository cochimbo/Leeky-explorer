// Panel data structure
use std::path::PathBuf;
use super::file_entry::FileEntry;
use anyhow::Result;
use glob::Pattern;

#[derive(Debug, Clone)]
pub struct Panel {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub cursor: usize,
    pub scroll_offset: usize,
    pub filter: Option<String>,
    pub last_quick_jump_char: Option<char>,  // T128c: Track last character for cyclic navigation
    pub last_quick_jump_index: usize,        // T128d: Track last position for cycling
}

impl Panel {
    pub fn new(path: PathBuf) -> Self {
        Self {
            current_path: path,
            entries: Vec::new(),
            cursor: 0,
            scroll_offset: 0,
            filter: None,
            last_quick_jump_char: None,
            last_quick_jump_index: 0,
        }
    }

    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.cursor)
    }

    pub fn has_entries(&self) -> bool {
        !self.entries.is_empty()
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor < self.entries.len().saturating_sub(1) {
            self.cursor += 1;
        }
    }

    pub fn move_cursor_to_top(&mut self) {
        self.cursor = 0;
    }

    pub fn move_cursor_to_bottom(&mut self) {
        self.cursor = self.entries.len().saturating_sub(1);
    }

    // T128f: Page Down - move 5 positions down
    pub fn page_down(&mut self) {
        let max_index = self.entries.len().saturating_sub(1);
        self.cursor = std::cmp::min(self.cursor + 5, max_index);
        self.adjust_scroll_for_cursor();
    }

    // T128g: Page Up - move 5 positions up
    pub fn page_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(5);
        self.adjust_scroll_for_cursor();
    }

    // T128c-d: Quick jump to files starting with character (cyclic navigation)
    pub fn quick_jump(&mut self, c: char) {
        if self.entries.is_empty() {
            return;
        }

        let c_lower = c.to_ascii_lowercase();
        
        // Check if same character pressed consecutively (for cycling)
        let start_index = if self.last_quick_jump_char == Some(c_lower) {
            // Cycle to next match after current position
            (self.last_quick_jump_index + 1) % self.entries.len()
        } else {
            // New character, start from beginning
            0
        };

        // Search for matching entries
        let mut found_index = None;
        
        // First search from start_index to end
        for i in start_index..self.entries.len() {
            if self.entries[i].name.to_ascii_lowercase().starts_with(c_lower) {
                found_index = Some(i);
                break;
            }
        }
        
        // If not found and we didn't start from 0, wrap around and search from beginning
        if found_index.is_none() && start_index > 0 {
            for i in 0..start_index {
                if self.entries[i].name.to_ascii_lowercase().starts_with(c_lower) {
                    found_index = Some(i);
                    break;
                }
            }
        }

        // If found, move cursor
        if let Some(index) = found_index {
            self.cursor = index;
            self.last_quick_jump_char = Some(c_lower);
            self.last_quick_jump_index = index;
            self.adjust_scroll_for_cursor();
        }
    }

    pub fn enter_dir(&mut self) -> Result<()> {
        if let Some(entry) = self.selected_entry()
            && entry.is_dir() {
                let new_path = entry.path.clone();
                self.current_path = new_path;
                self.cursor = 0;
                self.scroll_offset = 0;
            }
        Ok(())
    }

    pub fn refresh_entries(&mut self) -> Result<()> {
        let entries = crate::fs::navigator::read_dir(&self.current_path)?;
        self.entries = entries;
        
        // Ensure cursor is within bounds after refresh
        if self.cursor >= self.entries.len() && !self.entries.is_empty() {
            self.cursor = self.entries.len() - 1;
        }
        
        Ok(())
    }

    // T112b: Navigate up and position cursor on previous directory
    pub fn go_up(&mut self) -> Result<()> {
        if let Some(parent) = self.current_path.parent() {
            // Remember the current directory name before going up
            let previous_dir_name = self.current_path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string());
            
            self.current_path = parent.to_path_buf();
            
            // Refresh entries to load parent directory contents
            let entries = crate::fs::navigator::read_dir(&self.current_path)?;
            self.entries = entries;
            
            // Reset cursor and scroll initially
            self.cursor = 0;
            self.scroll_offset = 0;
            
            // T112b: Position cursor on the directory we came from
            if let Some(dir_name) = previous_dir_name
                && let Some(index) = self.entries.iter().position(|entry| entry.name == dir_name) {
                    self.cursor = index;
                    // Adjust scroll if needed to ensure the cursor is visible
                    self.adjust_scroll_for_cursor();
                }
        }
        Ok(())
    }
    
    // T112b: Helper to adjust scroll offset to keep cursor visible
    fn adjust_scroll_for_cursor(&mut self) {
        let visible_height = 20; // Approximate, will be adjusted by UI
        
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor >= self.scroll_offset + visible_height {
            self.scroll_offset = self.cursor.saturating_sub(visible_height - 1);
        }
    }

    // T402: Apply filter to entries list
    pub fn apply_filter(&mut self, pattern: &str, all_entries: &[FileEntry]) {
        self.filter = Some(pattern.to_string());
        
        if pattern.is_empty() {
            self.entries = all_entries.to_vec();
            return;
        }
        
        // Try glob pattern first
        if let Ok(glob_pattern) = Pattern::new(pattern) {
            self.entries = all_entries
                .iter()
                .filter(|entry| {
                    let name = entry.name.to_lowercase();
                    let pattern_lower = pattern.to_lowercase();
                    
                    // Check if it's a glob pattern (contains *, ?, [, ])
                    if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
                        glob_pattern.matches(&entry.name)
                    } else {
                        // Simple text matching (case-insensitive contains)
                        name.contains(&pattern_lower)
                    }
                })
                .cloned()
                .collect();
        } else {
            // Fallback to simple text matching if glob pattern is invalid
            let pattern_lower = pattern.to_lowercase();
            self.entries = all_entries
                .iter()
                .filter(|entry| entry.name.to_lowercase().contains(&pattern_lower))
                .cloned()
                .collect();
        }
        
        // Reset cursor to top after filtering
        self.cursor = 0;
        self.scroll_offset = 0;
    }

    // T405: Clear filter and restore full list
    pub fn clear_filter(&mut self, all_entries: &[FileEntry]) {
        self.filter = None;
        self.entries = all_entries.to_vec();
        self.cursor = 0;
        self.scroll_offset = 0;
    }

    // T406: Check if filter is active
    pub fn has_filter(&self) -> bool {
        self.filter.is_some()
    }

    // Get the current filter pattern
    pub fn get_filter(&self) -> Option<&str> {
        self.filter.as_deref()
    }
}

