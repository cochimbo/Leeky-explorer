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
}

impl Panel {
    pub fn new(path: PathBuf) -> Self {
        Self {
            current_path: path,
            entries: Vec::new(),
            cursor: 0,
            scroll_offset: 0,
            filter: None,
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

    pub fn enter_dir(&mut self) -> Result<()> {
        if let Some(entry) = self.selected_entry() {
            if entry.is_dir() {
                let new_path = entry.path.clone();
                self.current_path = new_path;
                self.cursor = 0;
                self.scroll_offset = 0;
            }
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

    pub fn go_up(&mut self) -> Result<()> {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.cursor = 0;
            self.scroll_offset = 0;
        }
        Ok(())
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

