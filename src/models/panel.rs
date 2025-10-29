// Panel data structure
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Instant, SystemTime};
use super::file_entry::FileEntry;
use anyhow::Result;
use glob::Pattern;
use crate::remote::VirtualFileSystem;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Navigation history for a panel (last 20 visited directories)
#[derive(Debug, Clone)]
pub struct NavigationHistory {
    entries: Vec<PathBuf>,
    max_size: usize,
}

impl NavigationHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_size,
        }
    }
    
    /// Add a new path to history
    pub fn push(&mut self, path: PathBuf) {
        // Don't add duplicate of last entry
        if self.entries.last() == Some(&path) {
            return;
        }
        
        // Add new entry
        self.entries.push(path);
        
        // Enforce max size (keep most recent)
        if self.entries.len() > self.max_size {
            self.entries.remove(0);
        }
    }
    
    /// Get all history entries (most recent last)
    pub fn get_all(&self) -> &[PathBuf] {
        &self.entries
    }
    
    /// Get count of history entries
    pub fn count(&self) -> usize {
        self.entries.len()
    }
    
    /// Clear all history
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    
    /// Remove invalid paths (directories that no longer exist)
    pub fn clean_invalid(&mut self) -> usize {
        let before_count = self.entries.len();
        self.entries.retain(|path| path.exists());
        before_count - self.entries.len()
    }
}

#[derive(Clone)]
pub struct Panel {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub cursor: usize,
    pub scroll_offset: usize,
    pub filter: Option<String>,
    pub last_quick_jump_char: Option<char>,  // T128c: Track last character for cyclic navigation
    pub last_quick_jump_index: usize,        // T128d: Track last position for cycling
    // US3: Text scrolling for long fields in highlight
    pub text_scroll_offset: usize,           // Horizontal scroll offset for selected item's name
    pub ext_scroll_offset: usize,            // Horizontal scroll offset for selected item's extension
    pub size_scroll_offset: usize,           // Horizontal scroll offset for size
    pub modified_scroll_offset: usize,       // Horizontal scroll offset for modified date
    pub created_scroll_offset: usize,        // Horizontal scroll offset for created date
    pub perms_scroll_offset: usize,          // Horizontal scroll offset for permissions
    pub text_scroll_timer: Instant,          // Last time text scroll was updated
    pub scroll_pause_until: Option<Instant>, // Pause scrolling until this time (for loop restart delay)
    pub history: NavigationHistory,          // Navigation history (last 20 dirs)
    pub last_modified: Option<SystemTime>,   // Auto-refresh: Last modified time of current directory
    // Remote filesystem support
    pub vfs: Option<Arc<dyn VirtualFileSystem>>, // Virtual filesystem (None = local, Some = remote)
    pub connection_info: Option<String>,     // Display string for remote connection (e.g., "user@host")
}

impl Panel {
    pub fn new(path: PathBuf) -> Self {
        let mut history = NavigationHistory::new(20); // Keep last 20 directories
        history.push(path.clone());
        
        Self {
            current_path: path,
            entries: Vec::new(),
            cursor: 0,
            scroll_offset: 0,
            filter: None,
            last_quick_jump_char: None,
            last_quick_jump_index: 0,
            text_scroll_offset: 0,
            ext_scroll_offset: 0,
            size_scroll_offset: 0,
            modified_scroll_offset: 0,
            created_scroll_offset: 0,
            perms_scroll_offset: 0,
            text_scroll_timer: Instant::now(),
            scroll_pause_until: None,
            history,
            last_modified: None,
            vfs: None,              // Start with local filesystem
            connection_info: None,  // No remote connection initially
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
            self.reset_text_scroll();
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor < self.entries.len().saturating_sub(1) {
            self.cursor += 1;
            self.reset_text_scroll();
        }
    }

    pub fn move_cursor_to_top(&mut self) {
        self.cursor = 0;
        self.reset_text_scroll();
    }

    pub fn move_cursor_to_bottom(&mut self) {
        self.cursor = self.entries.len().saturating_sub(1);
        self.reset_text_scroll();
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
                self.current_path = new_path.clone();
                self.history.push(new_path); // Add to history
                self.cursor = 0;
                self.scroll_offset = 0;
                // Clear any active filter when entering a new directory
                self.filter = None;
            }
        Ok(())
    }

    pub fn refresh_entries(&mut self) -> Result<()> {
        // Use VFS if connected to remote, otherwise use local filesystem
        if let Some(vfs) = &self.vfs {
            // Remote filesystem - use VFS (sync operations)
            log::info!("Refreshing remote directory: {:?}", self.current_path);
            
            use crate::remote::VfsEntryType;
            use crate::models::file_entry::{FileEntry, EntryType};
            
            let vfs_entries = match vfs.list_dir(&self.current_path) {
                Ok(entries) => {
                    log::info!("Successfully got {} entries from VFS", entries.len());
                    entries
                }
                Err(e) => {
                    log::error!("Failed to list remote directory: {}", e);
                    return Err(e);
                }
            };
            
            // Convert VfsEntry to FileEntry
            self.entries = vfs_entries.into_iter().map(|ve| {
                let entry_type = match ve.entry_type {
                    VfsEntryType::Directory => EntryType::Dir,
                    VfsEntryType::File => EntryType::File,
                    VfsEntryType::Symlink => EntryType::Symlink,
                };
                
                let extension = if entry_type == EntryType::File {
                    ve.name.rfind('.').map(|i| ve.name[i+1..].to_string())
                } else {
                    None
                };
                
                // Convert Unix permissions to std::fs::Permissions
                let perms = {
                    #[cfg(unix)]
                    {
                        std::fs::Permissions::from_mode(ve.permissions)
                    }
                    #[cfg(windows)]
                    {
                        // On Windows, approximate: if writable bit set, not readonly
                        let temp_path = std::env::temp_dir();
                        let mut p = std::fs::metadata(&temp_path)
                            .map(|m| m.permissions())
                            .unwrap_or_else(|_| {
                                // Create a default writable permission
                                let perm = std::fs::OpenOptions::new()
                                    .write(true)
                                    .create(true)
                                    .truncate(true)
                                    .open(temp_path.join("_temp_perm_check"))
                                    .and_then(|f| f.metadata())
                                    .map(|m| m.permissions())
                                    .unwrap_or_else(|_| {
                                        // Ultimate fallback: use current dir
                                        std::fs::metadata(".")
                                            .map(|m| m.permissions())
                                            .unwrap()
                                    });
                                #[cfg(unix)]
                                {
                                    use std::os::unix::fs::PermissionsExt;
                                    perm.set_mode(0o644); // rw-r--r--
                                }
                                #[cfg(windows)]
                                // On Windows, do not set_readonly(false) as per Clippy recommendation.
                                perm
                            });
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let writable = (ve.permissions & 0o200) != 0;
                            let mut mode = p.mode();
                            if writable {
                                mode |= 0o200;
                            } else {
                                mode &= !0o200;
                            }
                            p.set_mode(mode);
                        }
                        #[cfg(windows)]
                        {
                            p.set_readonly((ve.permissions & 0o200) == 0); // Check owner write bit
                        }
                        p
                    }
                };
                
                #[cfg(windows)]
                {
                    FileEntry::new(
                        ve.name,
                        entry_type,
                        ve.size,
                        ve.modified,
                        Some(ve.modified),
                        extension,
                        perms,
                        ve.path,
                        None, // No Windows attributes for remote files
                    )
                }
                
                #[cfg(not(windows))]
                {
                    FileEntry::new(
                        ve.name,
                        entry_type,
                        ve.size,
                        ve.modified,
                        Some(ve.modified),
                        extension,
                        perms,
                        ve.path,
                    )
                }
            }).collect();
            
            // For remote, we can't track directory changes
            self.last_modified = None;
        } else {
            // Local filesystem
            let entries = crate::fs::navigator::read_dir(&self.current_path)?;
            self.entries = entries;
            
            // Update last_modified timestamp
            if let Ok(metadata) = std::fs::metadata(&self.current_path)
                && let Ok(modified) = metadata.modified() {
                    self.last_modified = Some(modified);
                }
        }
        
        // Ensure cursor is within bounds after refresh
        if self.cursor >= self.entries.len() && !self.entries.is_empty() {
            self.cursor = self.entries.len() - 1;
        }
        
        Ok(())
    }

    /// Auto-refresh: Check if directory has been modified externally
    pub fn has_directory_changed(&self) -> bool {
        if let Ok(metadata) = std::fs::metadata(&self.current_path)
            && let Ok(modified) = metadata.modified() {
                // Compare with stored timestamp
                if let Some(last_mod) = self.last_modified {
                    return modified > last_mod;
                }
                // No timestamp stored yet, consider it changed
                return true;
            }
        false
    }
    
    /// Auto-refresh: Refresh if directory changed, clearing cursor and selection
    pub fn auto_refresh_if_changed(&mut self) -> Result<bool> {
        if self.has_directory_changed() {
            // Save current filename to try to restore cursor position
            let current_file = self.selected_entry().map(|e| e.name.clone());
            
            // Refresh the entries
            self.refresh_entries()?;
            
            // Try to restore cursor to same file
            if let Some(filename) = current_file {
                if let Some(idx) = self.entries.iter().position(|e| e.name == filename) {
                    self.cursor = idx;
                } else {
                    // File no longer exists, reset cursor
                    self.cursor = 0;
                }
            } else {
                self.cursor = 0;
            }
            
            // Reset scroll offset
            self.scroll_offset = 0;
            self.reset_text_scroll();
            
            return Ok(true);
        }
        Ok(false)
    }

    // T112b: Navigate up and position cursor on previous directory
    pub fn go_up(&mut self) -> Result<()> {
        // For remote filesystems, handle path navigation carefully
        // to preserve Unix-style paths (forward slashes)
        if self.vfs.is_some() {
            // Remote filesystem: manually handle path navigation with forward slashes
            let current_str = self.current_path.to_string_lossy().to_string();
            
            // Split by forward slash and remove last component
            let mut parts: Vec<&str> = current_str.split('/').filter(|s| !s.is_empty()).collect();
            
            if parts.is_empty() {
                // Already at root
                return Ok(());
            }
            
            // Remember current directory name
            let previous_dir_name = parts.last().map(|s| s.to_string());
            
            // Remove last component to go up
            parts.pop();
            
            // Rebuild path with forward slashes
            let parent_path = if parts.is_empty() {
                PathBuf::from("/")
            } else {
                PathBuf::from(format!("/{}", parts.join("/")))
            };
            
            self.current_path = parent_path.clone();
            self.history.push(parent_path);
            
            // Clear any active filter when going up to parent directory
            self.filter = None;
            
            // Refresh entries to load parent directory contents (uses VFS)
            self.refresh_entries()?;
            
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
        } else {
            // Local filesystem: use standard PathBuf navigation
            if let Some(parent) = self.current_path.parent() {
                // Remember the current directory name before going up
                let previous_dir_name = self.current_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|s| s.to_string());
                
                let parent_path = parent.to_path_buf();
                self.current_path = parent_path.clone();
                self.history.push(parent_path); // Add to history
                
                // Clear any active filter when going up to parent directory
                self.filter = None;
                
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

    // US3: Reset text scroll when cursor changes
    pub fn reset_text_scroll(&mut self) {
        self.text_scroll_offset = 0;
        self.ext_scroll_offset = 0;
        self.size_scroll_offset = 0;
        self.modified_scroll_offset = 0;
        self.created_scroll_offset = 0;
        self.perms_scroll_offset = 0;
        self.text_scroll_timer = Instant::now();
        self.scroll_pause_until = None;
    }

    // US3: Update text scroll animation for selected item
    // Returns true if screen needs refresh
    pub fn update_text_scroll(
        &mut self,
        name_width: usize,
        ext_width: usize,
        size_width: usize,
        modified_width: usize,
        created_width: usize,
        perms_width: usize,
    ) -> bool {
        // First get data we need without borrowing self
        let (name_len, ext_len, size_len, modified_len, created_len, perms_len) = {
            let entry = match self.selected_entry() {
                Some(e) => e,
                None => return false,
            };

            let name_len = entry.name.chars().count();
            let ext_len = entry.extension.as_ref().map(|e| e.chars().count()).unwrap_or(0);
            let size_len = crate::ui::formatters::format_size(entry).chars().count();
            let modified_len = crate::ui::formatters::format_date(Some(entry.modified)).chars().count();
            let created_len = crate::ui::formatters::format_date(entry.created).chars().count();
            let perms_len = crate::ui::formatters::format_permissions(entry).chars().count();
            
            (name_len, ext_len, size_len, modified_len, created_len, perms_len)
        };

        let now = Instant::now();

        // Check if we're in pause period
        if let Some(pause_until) = self.scroll_pause_until {
            if now < pause_until {
                return false; // Still paused
            }
            // Pause ended, clear it
            self.scroll_pause_until = None;
        }

        // Check which columns need scrolling
        let needs_name_scroll = name_len > name_width;
        let needs_ext_scroll = ext_len > ext_width;
        let needs_size_scroll = size_len > size_width;
        let needs_modified_scroll = modified_len > modified_width;
        let needs_created_scroll = created_len > created_width;
        let needs_perms_scroll = perms_len > perms_width;

        if !needs_name_scroll && !needs_ext_scroll && !needs_size_scroll 
            && !needs_modified_scroll && !needs_created_scroll && !needs_perms_scroll {
            return false;
        }

        let elapsed = now.duration_since(self.text_scroll_timer).as_millis();

        // Update every 200ms for smooth scrolling
        if elapsed < 200 {
            return false;
        }

        self.text_scroll_timer = now;

        let mut should_pause = false;

        // Scroll each column independently
        if needs_name_scroll {
            let max_offset = name_len.saturating_sub(name_width);
            self.text_scroll_offset += 1;
            if self.text_scroll_offset > max_offset + 3 {
                self.text_scroll_offset = 0;
                should_pause = true;
            }
        }

        if needs_ext_scroll {
            let max_offset = ext_len.saturating_sub(ext_width);
            self.ext_scroll_offset += 1;
            if self.ext_scroll_offset > max_offset + 2 {
                self.ext_scroll_offset = 0;
                should_pause = true;
            }
        }

        if needs_size_scroll {
            let max_offset = size_len.saturating_sub(size_width);
            self.size_scroll_offset += 1;
            if self.size_scroll_offset > max_offset + 2 {
                self.size_scroll_offset = 0;
                should_pause = true;
            }
        }

        if needs_modified_scroll {
            let max_offset = modified_len.saturating_sub(modified_width);
            self.modified_scroll_offset += 1;
            if self.modified_scroll_offset > max_offset + 2 {
                self.modified_scroll_offset = 0;
                should_pause = true;
            }
        }

        if needs_created_scroll {
            let max_offset = created_len.saturating_sub(created_width);
            self.created_scroll_offset += 1;
            if self.created_scroll_offset > max_offset + 2 {
                self.created_scroll_offset = 0;
                should_pause = true;
            }
        }

        if needs_perms_scroll {
            let max_offset = perms_len.saturating_sub(perms_width);
            self.perms_scroll_offset += 1;
            if self.perms_scroll_offset > max_offset + 2 {
                self.perms_scroll_offset = 0;
                should_pause = true;
            }
        }

        // If any column looped back, pause for 2 seconds
        if should_pause {
            self.scroll_pause_until = Some(now + std::time::Duration::from_secs(2));
        }

        true // Need refresh
    }
    
    /// Check if this panel is connected to a remote filesystem
    pub fn is_remote(&self) -> bool {
        self.vfs.is_some()
    }
    
    /// Connect this panel to a remote filesystem
    pub fn connect_remote(&mut self, vfs: Arc<dyn VirtualFileSystem>, info: String, initial_path: PathBuf) {
        self.vfs = Some(vfs);
        self.connection_info = Some(info);
        self.current_path = initial_path;
        self.entries.clear();
        self.cursor = 0;
        self.scroll_offset = 0;
        self.history.clear();
        self.history.push(self.current_path.clone());
    }
    
    /// Disconnect from remote filesystem and return to local
    pub fn disconnect_remote(&mut self, fallback_path: PathBuf) {
        self.vfs = None;
        self.connection_info = None;
        self.current_path = fallback_path;
        self.entries.clear();
        self.cursor = 0;
        self.scroll_offset = 0;
        self.history.clear();
        self.history.push(self.current_path.clone());
    }
}
