# Implementation Tasks: Quick Wins Features

**Branch**: `004-quick-wins-bookmarks`  
**Plan**: [plan.md](./plan.md) | **Spec**: [spec.md](./spec.md)

## Task Organization

Tasks are organized by implementation phase. Each task includes:
- **ID**: Unique task identifier
- **Phase**: Implementation phase (0-4)
- **Priority**: P1 (critical), P2 (high), P3 (medium), P4 (low)
- **Estimated Time**: Hours to complete
- **Dependencies**: Task IDs that must be completed first
- **Files**: Files to create or modify
- **Status**: ⬜ Not Started | 🔄 In Progress | ✅ Complete

---

## Phase 0: Foundation (Shared Infrastructure)

### TASK-001: Add chrono dependency ⬜
**Priority**: P1 | **Time**: 0.25h | **Dependencies**: None

**Description**: Add chrono crate for timestamp management in bookmarks

**Files**:
- `Cargo.toml` - Add `chrono = "0.4"` to dependencies

**Acceptance**:
- [ ] chrono added to Cargo.toml
- [ ] `cargo build` succeeds
- [ ] No version conflicts

---

### TASK-002: Create Bookmark model ⬜
**Priority**: P1 | **Time**: 1h | **Dependencies**: TASK-001

**Description**: Create core Bookmark struct with serialization

**Files**:
- `src/models/bookmark.rs` - NEW
- `src/models/mod.rs` - Add `pub mod bookmark;`

**Implementation**:
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bookmark {
    pub name: String,
    pub path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

impl Bookmark {
    pub fn new(name: String, path: PathBuf) -> Self { ... }
    pub fn access(&mut self) { ... }
}
```

**Acceptance**:
- [ ] Bookmark struct compiles
- [ ] Serialization/deserialization works
- [ ] Unit test for creation
- [ ] Unit test for access update

---

### TASK-003: Create BookmarkManager ⬜
**Priority**: P1 | **Time**: 2h | **Dependencies**: TASK-002

**Description**: Manager for bookmark collection with file persistence

**Files**:
- `src/config/bookmarks.rs` - NEW
- `src/config/mod.rs` - Add `pub mod bookmarks;`

**Implementation**:
```rust
use crate::models::bookmark::Bookmark;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkManager {
    bookmarks: Vec<Bookmark>,
    config_path: PathBuf,
}

impl BookmarkManager {
    pub fn load() -> Result<Self> { ... }
    pub fn save(&self) -> Result<()> { ... }
    pub fn add(&mut self, bookmark: Bookmark) -> Result<()> { ... }
    pub fn remove(&mut self, index: usize) -> Result<()> { ... }
    pub fn rename(&mut self, index: usize, new_name: String) -> Result<()> { ... }
    pub fn get_all(&self) -> &[Bookmark] { ... }
    pub fn find_by_path(&self, path: &Path) -> Option<usize> { ... }
}
```

**Acceptance**:
- [ ] BookmarkManager compiles
- [ ] load() creates file if not exists
- [ ] save() persists to JSON correctly
- [ ] add() prevents duplicates
- [ ] remove() works correctly
- [ ] rename() validates names
- [ ] Unit tests for all methods
- [ ] Cross-platform path handling works

---

### TASK-004: Integrate bookmarks into AppState ⬜
**Priority**: P1 | **Time**: 0.5h | **Dependencies**: TASK-003

**Description**: Add bookmark manager to application state

**Files**:
- `src/config/state.rs` - MODIFY

**Changes**:
```rust
pub struct AppState {
    // ... existing fields
    pub bookmarks: BookmarkManager,
}

impl AppState {
    pub fn load() -> Result<Self> {
        // ... existing code
        let bookmarks = BookmarkManager::load()?;
        Ok(Self {
            // ... existing fields
            bookmarks,
        })
    }
}
```

**Acceptance**:
- [ ] AppState includes bookmarks field
- [ ] Bookmarks load on app startup
- [ ] Bookmarks save on app exit
- [ ] Tests updated for new field

---

### TASK-005: Add bookmark keybindings ⬜
**Priority**: P1 | **Time**: 0.5h | **Dependencies**: None

**Description**: Define keybindings for bookmark operations

**Files**:
- `src/events/keybindings.rs` - MODIFY

**Changes**:
```rust
pub enum Action {
    // ... existing actions
    OpenBookmarkManager,
    AddBookmark,
    DeleteBookmark,
    RenameBookmark,
    NavigateToBookmark(usize),
    // For history
    NavigateBack,
    NavigateForward,
    // For editor
    OpenEditor,
    SaveFile,
}

// In key mapping
KeyEvent { code: KeyCode::Char('b'), modifiers: KeyModifiers::CONTROL, .. } => Some(Action::OpenBookmarkManager),
KeyEvent { code: KeyCode::Left, modifiers: KeyModifiers::ALT, .. } => Some(Action::NavigateBack),
KeyEvent { code: KeyCode::Right, modifiers: KeyModifiers::ALT, .. } => Some(Action::NavigateForward),
KeyCode::Char('e') => Some(Action::OpenEditor),
KeyCode::F(4) => Some(Action::OpenEditor),
```

**Acceptance**:
- [ ] All new actions defined
- [ ] Ctrl+B mapped to bookmark manager
- [ ] Alt+Left/Right mapped
- [ ] 'e' and F4 mapped to editor
- [ ] No keybinding conflicts (F5 already used for Copy)

---

## Phase 1: Bookmarks Feature (P1) - MVP

### TASK-006: Create bookmark manager UI widget ⬜
**Priority**: P1 | **Time**: 3h | **Dependencies**: TASK-002, TASK-003, TASK-005

**Description**: Modal widget for displaying and managing bookmarks

**Files**:
- `src/ui/bookmark_manager.rs` - NEW
- `src/ui/mod.rs` - Add `pub mod bookmark_manager;`

**Implementation**:
```rust
use crate::config::bookmarks::BookmarkManager;
use crate::models::bookmark::Bookmark;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub struct BookmarkManagerWidget {
    selected_index: Option<usize>,
    mode: BookmarkMode, // List, Add, Rename, Confirm
    input_buffer: String,
}

pub enum BookmarkMode {
    List,
    AddingName,
    Renaming(usize),
    ConfirmDelete(usize),
}

impl BookmarkManagerWidget {
    pub fn new() -> Self { ... }
    pub fn render(&mut self, frame: &mut Frame, area: Rect, bookmarks: &BookmarkManager) { ... }
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<BookmarkAction> { ... }
}

pub enum BookmarkAction {
    Navigate(PathBuf),
    Add(String, PathBuf),
    Delete(usize),
    Rename(usize, String),
    Close,
}
```

**UI Layout**:
```
┌─ Bookmarks (Ctrl+B) ─────────────────────────┐
│ ↓ Projects (/home/user/projects)             │
│   Downloads (/home/user/Downloads)           │
│   Documents (/home/user/Documents)           │
│                                               │
│ [a] Add current directory                    │
│ [d] Delete | [r] Rename | [Enter] Navigate   │
│ [Esc] Close                                   │
└───────────────────────────────────────────────┘
```

**Acceptance**:
- [ ] Modal renders centered on screen
- [ ] Bookmark list displays with paths
- [ ] Selected item highlighted
- [ ] Arrow keys navigate list
- [ ] 'a' prompts for name
- [ ] 'd' shows delete confirmation
- [ ] 'r' prompts for new name
- [ ] Enter navigates to bookmark
- [ ] Esc closes modal
- [ ] Empty state shows helpful message
- [ ] Ctrl+B toggles bookmark manager

---

### TASK-007: Implement name input dialog ⬜
**Priority**: P1 | **Time**: 1.5h | **Dependencies**: TASK-006

**Description**: Reusable text input widget for bookmark names

**Files**:
- `src/ui/text_input.rs` - NEW (or extend existing dialog)
- `src/ui/mod.rs` - Update exports

**Implementation**:
```rust
pub struct TextInputDialog {
    title: String,
    prompt: String,
    input: String,
    cursor_position: usize,
}

impl TextInputDialog {
    pub fn new(title: String, prompt: String) -> Self { ... }
    pub fn render(&self, frame: &mut Frame, area: Rect) { ... }
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<String> { ... }
}
```

**Acceptance**:
- [ ] Dialog renders with title and prompt
- [ ] Text input works with cursor
- [ ] Backspace deletes characters
- [ ] Enter returns input string
- [ ] Esc returns None (cancel)
- [ ] Input validation (non-empty, max length)

---

### TASK-008: Integrate bookmarks into event handler ⬜
**Priority**: P1 | **Time**: 2h | **Dependencies**: TASK-006, TASK-007

**Description**: Handle bookmark actions in main event loop

**Files**:
- `src/events/handler.rs` - MODIFY
- `src/app.rs` - MODIFY

**Changes in handler.rs**:
```rust
Action::OpenBookmarkManager => {
    // Toggle bookmark manager visibility
    app.show_bookmark_manager = !app.show_bookmark_manager;
}
Action::AddBookmark => {
    if let Some(name) = app.get_bookmark_name_input() {
        let path = app.active_panel().current_path.clone();
        app.state.bookmarks.add(Bookmark::new(name, path))?;
        app.state.bookmarks.save()?;
    }
}
Action::DeleteBookmark => {
    if let Some(index) = app.bookmark_manager.selected_index {
        app.state.bookmarks.remove(index)?;
        app.state.bookmarks.save()?;
    }
}
Action::NavigateToBookmark(index) => {
    if let Some(bookmark) = app.state.bookmarks.get_all().get(index) {
        if bookmark.path.exists() {
            app.active_panel_mut().enter_dir(bookmark.path.clone())?;
        } else {
            app.show_error("Bookmarked directory no longer exists");
        }
    }
}
```

**Acceptance**:
- [ ] Ctrl+B toggles bookmark manager
- [ ] Add bookmark creates entry
- [ ] Delete removes bookmark
- [ ] Navigate changes active panel path
- [ ] Non-existent paths show error
- [ ] All operations persist to disk
- [ ] Integration test covers full workflow

---

### TASK-009: Add bookmark edge case handling ⬜
**Priority**: P2 | **Time**: 1h | **Dependencies**: TASK-008

**Description**: Handle edge cases and error conditions

**Files**:
- `src/config/bookmarks.rs` - MODIFY
- `src/ui/bookmark_manager.rs` - MODIFY

**Edge Cases**:
1. Duplicate bookmark names - append number
2. Non-existent directory - show warning, offer to remove
3. Empty bookmark list - show "No bookmarks yet" message
4. Maximum bookmarks (50) - show warning
5. Invalid characters in name - sanitize or reject
6. Bookmark file corruption - fallback to empty list

**Acceptance**:
- [ ] All edge cases have tests
- [ ] User sees helpful error messages
- [ ] App never crashes from bookmark errors
- [ ] File corruption recovers gracefully

---

### TASK-010: Write bookmark integration tests ⬜
**Priority**: P1 | **Time**: 1.5h | **Dependencies**: TASK-008

**Description**: End-to-end tests for bookmark workflow

**Files**:
- `tests/bookmark_test.rs` - NEW

**Test Cases**:
```rust
#[test]
fn test_create_and_persist_bookmark() { ... }

#[test]
fn test_navigate_to_bookmark() { ... }

#[test]
fn test_delete_bookmark() { ... }

#[test]
fn test_rename_bookmark() { ... }

#[test]
fn test_bookmark_to_deleted_directory() { ... }

#[test]
fn test_50_bookmarks_performance() { ... }

#[test]
fn test_duplicate_bookmark_names() { ... }
```

**Acceptance**:
- [ ] All tests pass
- [ ] Coverage includes FR-001 through FR-008
- [ ] Cross-platform path handling tested
- [ ] Performance test validates <100ms operations

---

## Phase 2: Disk Usage Indicators (P2)

### TASK-011: Add disk usage percentage calculation ⬜
**Priority**: P2 | **Time**: 1h | **Dependencies**: None

**Description**: Extend disk_info module with usage calculations

**Files**:
- `src/fs/disk_info.rs` - MODIFY

**Changes**:
```rust
impl DiskInfo {
    pub fn usage_percentage(&self) -> f64 {
        if self.total_space == 0 {
            return 0.0;
        }
        (self.used_space as f64 / self.total_space as f64) * 100.0
    }
    
    pub fn warning_level(&self) -> UsageLevel {
        let usage = self.usage_percentage();
        if usage >= 90.0 {
            UsageLevel::Critical
        } else if usage >= 80.0 {
            UsageLevel::Warning
        } else {
            UsageLevel::Normal
        }
    }
}

pub enum UsageLevel {
    Normal,
    Warning,
    Critical,
}
```

**Acceptance**:
- [ ] usage_percentage() returns correct values
- [ ] warning_level() categorizes correctly
- [ ] Zero division handled
- [ ] Unit tests for various scenarios

---

### TASK-012: Add progress bar to drive selector ⬜
**Priority**: P2 | **Time**: 2h | **Dependencies**: TASK-011

**Description**: Render visual disk usage bars in drive selector

**Files**:
- `src/ui/drive_selector.rs` - MODIFY

**UI Enhancement**:
```
┌─ Select Drive (F9) ───────────────────────┐
│ > C:\ [████████████████░░░░] 80% (50GB)   │
│   D:\ [███████░░░░░░░░░░░░░] 35% (200GB)  │
│   E:\ [███████████████████░] 95% (10GB)   │
└───────────────────────────────────────────┘
         Normal          Warning   Critical
```

**Implementation**:
```rust
fn render_drive_with_usage(&self, drive: &DiskInfo, is_selected: bool) -> ListItem {
    let usage_pct = drive.usage_percentage();
    let level = drive.warning_level();
    
    let bar = create_usage_bar(usage_pct, level);
    let color = match level {
        UsageLevel::Normal => Color::Green,
        UsageLevel::Warning => Color::Yellow,
        UsageLevel::Critical => Color::Red,
    };
    
    let line = Line::from(vec![
        Span::raw(format!("{} ", drive.mount_point)),
        Span::styled(bar, Style::default().fg(color)),
        Span::raw(format!(" {:.0}% ", usage_pct)),
        Span::raw(format!("({})", format_size(drive.available_space))),
    ]);
    
    ListItem::new(line)
}
```

**Acceptance**:
- [ ] All drives show usage bars
- [ ] Colors match usage levels
- [ ] Percentage and free space displayed
- [ ] Layout doesn't break with long paths
- [ ] Performance acceptable (<500ms)

---

### TASK-013: Add disk space to status bar ⬜
**Priority**: P2 | **Time**: 0.5h | **Dependencies**: TASK-011

**Description**: Show current drive space in panel status bar

**Files**:
- `src/ui/panel_widget.rs` - MODIFY

**Changes**:
```rust
// In render_status_bar()
let current_drive = get_drive_for_path(&panel.current_path);
if let Some(disk_info) = current_drive {
    let free_space = format_size(disk_info.available_space);
    status_line.push(Span::raw(format!(" | Free: {}", free_space)));
}
```

**Acceptance**:
- [ ] Status bar shows free space
- [ ] Updates when changing drives
- [ ] Format is readable (e.g., "50.2 GB")
- [ ] No performance impact

---

### TASK-014: Handle disk usage edge cases ⬜
**Priority**: P2 | **Time**: 1h | **Dependencies**: TASK-012

**Description**: Handle unmounted drives and errors

**Files**:
- `src/fs/disk_info.rs` - MODIFY
- `src/ui/drive_selector.rs` - MODIFY

**Edge Cases**:
1. Unmounted drive - show "N/A" or skip
2. Network drive timeout - show "Calculating..."
3. Permission denied - show "Access Denied"
4. Drive removed while running - graceful handling

**Implementation**:
```rust
pub fn get_disk_usage_with_timeout(path: &Path, timeout: Duration) -> Option<DiskInfo> {
    // Use channel with timeout for network drives
}
```

**Acceptance**:
- [ ] Unmounted drives don't crash app
- [ ] Network drives timeout after 500ms
- [ ] Error states display user-friendly messages
- [ ] All edge cases have tests

---

### TASK-015: Write disk usage tests ⬜
**Priority**: P2 | **Time**: 1h | **Dependencies**: TASK-011, TASK-014

**Description**: Unit tests for disk usage calculations

**Files**:
- `tests/disk_usage_test.rs` - NEW

**Test Cases**:
```rust
#[test]
fn test_usage_percentage_calculation() { ... }

#[test]
fn test_warning_level_thresholds() { ... }

#[test]
fn test_zero_space_handling() { ... }

#[test]
fn test_unmounted_drive_handling() { ... }

#[test]
fn test_cross_platform_disk_info() { ... }
```

**Acceptance**:
- [ ] All tests pass
- [ ] Coverage includes FR-009 through FR-014
- [ ] Tested on Windows, Linux, macOS

---

## Phase 3: Navigation History (P3)

### TASK-016: Create NavigationHistory struct ⬜
**Priority**: P3 | **Time**: 1.5h | **Dependencies**: None

**Description**: History stack data structure for panel navigation

**Files**:
- `src/models/panel.rs` - MODIFY

**Implementation**:
```rust
#[derive(Debug, Clone)]
pub struct NavigationHistory {
    entries: Vec<PathBuf>,
    current_index: usize,
    max_size: usize,
}

impl NavigationHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: Vec::new(),
            current_index: 0,
            max_size,
        }
    }
    
    pub fn push(&mut self, path: PathBuf) {
        // Truncate forward history
        self.entries.truncate(self.current_index + 1);
        
        // Add new entry
        self.entries.push(path);
        
        // Enforce max size
        if self.entries.len() > self.max_size {
            self.entries.remove(0);
        } else {
            self.current_index += 1;
        }
    }
    
    pub fn back(&mut self) -> Option<&PathBuf> {
        if self.current_index > 0 {
            self.current_index -= 1;
            Some(&self.entries[self.current_index])
        } else {
            None
        }
    }
    
    pub fn forward(&mut self) -> Option<&PathBuf> {
        if self.current_index + 1 < self.entries.len() {
            self.current_index += 1;
            Some(&self.entries[self.current_index])
        } else {
            None
        }
    }
    
    pub fn can_go_back(&self) -> bool {
        self.current_index > 0
    }
    
    pub fn can_go_forward(&self) -> bool {
        self.current_index + 1 < self.entries.len()
    }
}
```

**Acceptance**:
- [ ] NavigationHistory compiles
- [ ] push() adds entries correctly
- [ ] back() returns previous path
- [ ] forward() returns next path
- [ ] Max size enforced (100 entries)
- [ ] Unit tests for all methods

---

### TASK-017: Integrate history into Panel ⬜
**Priority**: P3 | **Time**: 1h | **Dependencies**: TASK-016

**Description**: Add history field to Panel and track navigation

**Files**:
- `src/models/panel.rs` - MODIFY

**Changes**:
```rust
pub struct Panel {
    // ... existing fields
    pub history: NavigationHistory,
}

impl Panel {
    pub fn new(path: PathBuf) -> Self {
        let mut history = NavigationHistory::new(100);
        history.push(path.clone());
        
        Self {
            // ... existing fields
            history,
        }
    }
    
    pub fn enter_dir(&mut self, path: PathBuf) -> Result<()> {
        // ... existing code
        self.history.push(path.clone());
        Ok(())
    }
    
    pub fn go_up(&mut self) -> Result<()> {
        // ... existing code
        if let Some(parent) = parent {
            self.history.push(parent.clone());
        }
        Ok(())
    }
    
    pub fn navigate_back(&mut self) -> Result<()> {
        if let Some(path) = self.history.back() {
            let path = path.clone();
            self.navigate_without_history(path)?;
        }
        Ok(())
    }
    
    pub fn navigate_forward(&mut self) -> Result<()> {
        if let Some(path) = self.history.forward() {
            let path = path.clone();
            self.navigate_without_history(path)?;
        }
        Ok(())
    }
    
    fn navigate_without_history(&mut self, path: PathBuf) -> Result<()> {
        // Navigate without adding to history (for back/forward)
        self.current_path = path;
        self.load_entries()?;
        Ok(())
    }
}
```

**Acceptance**:
- [ ] Panel includes history field
- [ ] Navigation adds to history
- [ ] Back/forward don't re-add to history
- [ ] Each panel has independent history
- [ ] Tests updated for new field

---

### TASK-018: Implement history navigation handlers ⬜
**Priority**: P3 | **Time**: 0.5h | **Dependencies**: TASK-017, TASK-005

**Description**: Handle Alt+Left/Right in event handler

**Files**:
- `src/events/handler.rs` - MODIFY

**Changes**:
```rust
Action::NavigateBack => {
    app.active_panel_mut().navigate_back()?;
}
Action::NavigateForward => {
    app.active_panel_mut().navigate_forward()?;
}
```

**Acceptance**:
- [ ] Alt+Left goes back
- [ ] Alt+Right goes forward
- [ ] Inactive panel not affected
- [ ] Visual feedback when at history bounds (optional)

---

### TASK-019: Handle history edge cases ⬜
**Priority**: P3 | **Time**: 1h | **Dependencies**: TASK-018

**Description**: Handle non-existent paths and edge cases

**Files**:
- `src/models/panel.rs` - MODIFY

**Edge Cases**:
1. Navigate to deleted directory in history - show error, skip
2. At beginning of history - Alt+Left no-op
3. At end of history - Alt+Right no-op
4. History overflow >100 entries - remove oldest
5. New navigation from middle - clear forward history

**Acceptance**:
- [ ] Deleted paths handled gracefully
- [ ] Boundary conditions don't crash
- [ ] History size limited correctly
- [ ] All edge cases have tests

---

### TASK-020: Write history navigation tests ⬜
**Priority**: P3 | **Time**: 1h | **Dependencies**: TASK-017, TASK-019

**Description**: Unit tests for navigation history

**Files**:
- `tests/history_test.rs` - NEW

**Test Cases**:
```rust
#[test]
fn test_navigation_adds_to_history() { ... }

#[test]
fn test_back_navigation() { ... }

#[test]
fn test_forward_navigation() { ... }

#[test]
fn test_forward_history_cleared_on_new_navigation() { ... }

#[test]
fn test_independent_panel_history() { ... }

#[test]
fn test_history_size_limit() { ... }

#[test]
fn test_deleted_path_in_history() { ... }
```

**Acceptance**:
- [ ] All tests pass
- [ ] Coverage includes FR-015 through FR-020
- [ ] Performance validated (<50ms per operation)

---

## Phase 4: Text Editor (P4) - Optional/Deferred

### TASK-021: Create EditorBuffer struct ⬜
**Priority**: P4 | **Time**: 2h | **Dependencies**: None

**Description**: Buffer for text file content and cursor state

**Files**:
- `src/ui/text_editor.rs` - NEW
- `src/ui/mod.rs` - Add `pub mod text_editor;`

**Implementation**:
```rust
pub struct EditorBuffer {
    lines: Vec<String>,
    file_path: PathBuf,
    modified: bool,
    cursor: (usize, usize), // (line, col)
    scroll_offset: usize,
}

impl EditorBuffer {
    pub fn from_file(path: PathBuf) -> Result<Self> { ... }
    pub fn save(&mut self) -> Result<()> { ... }
    pub fn insert_char(&mut self, ch: char) { ... }
    pub fn delete_char(&mut self) { ... }
    pub fn move_cursor(&mut self, direction: Direction) { ... }
    pub fn is_modified(&self) -> bool { ... }
}
```

**Acceptance**:
- [ ] EditorBuffer compiles
- [ ] Can load text files
- [ ] Cursor movement works
- [ ] Insert/delete operations work
- [ ] Save writes to disk correctly
- [ ] UTF-8 handling correct

---

### TASK-022: Create text editor UI widget ⬜
**Priority**: P4 | **Time**: 3h | **Dependencies**: TASK-021

**Description**: Modal editor widget with rendering

**Files**:
- `src/ui/text_editor.rs` - MODIFY

**Implementation**:
```rust
pub struct TextEditorWidget {
    buffer: EditorBuffer,
    show_unsaved_warning: bool,
}

impl TextEditorWidget {
    pub fn new(buffer: EditorBuffer) -> Self { ... }
    
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Render line numbers
        // Render text content
        // Render cursor
        // Render status bar (file path, modified indicator, line/col)
    }
    
    pub fn handle_input(&mut self, key: KeyEvent) -> EditorAction {
        match key.code {
            KeyCode::Char(ch) => {
                self.buffer.insert_char(ch);
                EditorAction::Continue
            }
            KeyCode::Backspace => {
                self.buffer.delete_char();
                EditorAction::Continue
            }
            KeyEvent { code: KeyCode::Char('s'), modifiers: KeyModifiers::CONTROL, .. } => {
                EditorAction::Save
            }
            KeyCode::Esc => {
                if self.buffer.is_modified() {
                    EditorAction::ConfirmClose
                } else {
                    EditorAction::Close
                }
            }
            _ => EditorAction::Continue,
        }
    }
}

pub enum EditorAction {
    Continue,
    Save,
    Close,
    ConfirmClose,
}
```

**UI Layout**:
```
┌─ Editor: config.toml ──────────────────────────────── [Modified] ─┐
│  1 | [package]                                                     │
│  2 | name = "leeky-explorer"                                       │
│  3 | version = "0.4.0"                                             │
│  4 | edition = "2024"                                              │
│  5 | █                                                             │
│ ...                                                                │
├────────────────────────────────────────────────────────────────────┤
│ Ctrl+S: Save | Esc: Close | Line 5, Col 1                         │
└────────────────────────────────────────────────────────────────────┘
```

**Acceptance**:
- [ ] Editor renders full screen
- [ ] Line numbers display
- [ ] Cursor visible and positioned correctly
- [ ] Text scrolls when cursor moves off screen
- [ ] Status bar shows file info
- [ ] Modified indicator works

---

### TASK-023: Add file type validation ⬜
**Priority**: P4 | **Time**: 1h | **Dependencies**: TASK-022

**Description**: Prevent editing binary files and large files

**Files**:
- `src/ui/text_editor.rs` - MODIFY
- `src/events/handler.rs` - MODIFY

**Implementation**:
```rust
fn is_text_file(path: &Path) -> bool {
    // Check extension
    // Check for null bytes in first 8192 bytes
}

fn is_file_too_large(path: &Path, max_size: u64) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        metadata.len() > max_size
    } else {
        false
    }
}

// In handler
Action::OpenEditor => {
    let file = app.active_panel().selected_file()?;
    
    if !is_text_file(&file.path) {
        app.show_error("Cannot edit binary file");
        return Ok(());
    }
    
    if is_file_too_large(&file.path, 1_048_576) {
        app.show_warning("File is large (>1MB). Use external editor?");
        return Ok(());
    }
    
    let buffer = EditorBuffer::from_file(file.path.clone())?;
    app.editor = Some(TextEditorWidget::new(buffer));
}
```

**Acceptance**:
- [ ] Binary files rejected
- [ ] Large files warned
- [ ] Text files open successfully
- [ ] User sees clear error messages

---

### TASK-024: Implement save and close handlers ⬜
**Priority**: P4 | **Time**: 1h | **Dependencies**: TASK-023

**Description**: Handle save operations and unsaved changes

**Files**:
- `src/events/handler.rs` - MODIFY

**Changes**:
```rust
Action::SaveFile => {
    if let Some(editor) = &mut app.editor {
        match editor.buffer.save() {
            Ok(_) => app.show_message("File saved"),
            Err(e) => app.show_error(&format!("Save failed: {}", e)),
        }
    }
}

// In editor input handling
EditorAction::ConfirmClose => {
    app.show_confirm_dialog(
        "Unsaved changes",
        "Save before closing?",
        |response| {
            match response {
                ConfirmResponse::Yes => {
                    // Save and close
                }
                ConfirmResponse::No => {
                    // Close without saving
                }
                ConfirmResponse::Cancel => {
                    // Return to editor
                }
            }
        }
    );
}
```

**Acceptance**:
- [ ] Ctrl+S saves file
- [ ] Save success shows message
- [ ] Save errors show helpful message
- [ ] Esc with unsaved changes shows warning
- [ ] Confirm dialog works correctly

---

### TASK-025: Add editor edge case handling ⬜
**Priority**: P4 | **Time**: 1h | **Dependencies**: TASK-024

**Description**: Handle file permissions and concurrent modifications

**Files**:
- `src/ui/text_editor.rs` - MODIFY

**Edge Cases**:
1. Read-only file - allow viewing, prevent saving
2. File deleted while editing - show error on save
3. File modified externally - detect and warn
4. Invalid UTF-8 - reject or show as hex
5. Permission denied on save - show error

**Implementation**:
```rust
impl EditorBuffer {
    pub fn check_external_modifications(&self) -> bool {
        // Compare file mtime with when we loaded it
    }
    
    pub fn is_read_only(&self) -> bool {
        // Check file permissions
    }
}
```

**Acceptance**:
- [ ] Read-only files can't be saved
- [ ] External modifications detected
- [ ] Permission errors handled
- [ ] All edge cases have tests

---

### TASK-026: Write text editor tests ⬜
**Priority**: P4 | **Time**: 2h | **Dependencies**: TASK-025

**Description**: Integration tests for editor functionality

**Files**:
- `tests/text_editor_test.rs` - NEW

**Test Cases**:
```rust
#[test]
fn test_open_and_edit_text_file() { ... }

#[test]
fn test_save_modifications() { ... }

#[test]
fn test_unsaved_changes_warning() { ... }

#[test]
fn test_binary_file_rejection() { ... }

#[test]
fn test_large_file_warning() { ... }

#[test]
fn test_read_only_file() { ... }

#[test]
fn test_cursor_movement() { ... }

#[test]
fn test_insert_and_delete() { ... }
```

**Acceptance**:
- [ ] All tests pass
- [ ] Coverage includes FR-021 through FR-030
- [ ] Editor doesn't corrupt files
- [ ] Performance acceptable for <100KB files

---

## Documentation and Final Tasks

### TASK-027: Update README with new features ⬜
**Priority**: P1 | **Time**: 0.5h | **Dependencies**: Completed phases

**Description**: Document new keybindings and features

**Files**:
- `README.md` - MODIFY

**Sections to update**:
- Features list
- Keybindings table (Ctrl+B, Alt+Left/Right, 'e', F4)
- Configuration section (bookmarks.json)
- Screenshots (if applicable)

**Acceptance**:
- [ ] All new features documented
- [ ] Keybindings table updated
- [ ] Examples provided
- [ ] Configuration files documented

---

### TASK-028: Update CHANGELOG ⬜
**Priority**: P1 | **Time**: 0.25h | **Dependencies**: TASK-027

**Description**: Add v0.4.0 entry to CHANGELOG

**Files**:
- `CHANGELOG.md` - MODIFY

**Entry**:
```markdown
## [0.4.0] - 2025-MM-DD

### Added
- Bookmarks system with Ctrl+B keybinding for quick directory access
- Persistent bookmark storage in configuration
- Visual disk usage indicators in drive selector (F9)
- Disk space display in panel status bar
- Navigation history with Alt+Left (back) and Alt+Right (forward)
- [If included] Simple text editor with 'e' or F4 keybinding

### Changed
- Drive selector now shows visual usage bars with color coding
- Status bar displays current drive free space

### Fixed
- [Any bugs discovered during implementation]
```

**Acceptance**:
- [ ] CHANGELOG follows Keep a Changelog format
- [ ] All features listed
- [ ] Version and date correct

---

### TASK-029: Run full test suite ⬜
**Priority**: P1 | **Time**: 0.5h | **Dependencies**: All implementation tasks

**Description**: Verify all tests pass including new features

**Commands**:
```bash
cargo test --all-targets
cargo clippy -- -D warnings
cargo build --release
```

**Acceptance**:
- [ ] All unit tests pass (expected ~103 tests)
- [ ] All integration tests pass
- [ ] Clippy reports no warnings
- [ ] Release build succeeds
- [ ] Binary size increase <500KB
- [ ] Manual smoke testing on all platforms

---

### TASK-030: Create checklist for requirements ⬜
**Priority**: P2 | **Time**: 0.5h | **Dependencies**: Spec complete

**Description**: Verification checklist for all functional requirements

**Files**:
- `specs/004-quick-wins-bookmarks/checklists/requirements.md` - NEW

**Content**: Map each FR-XXX to test case and manual verification step

**Acceptance**:
- [ ] All 30 FRs have checklist items
- [ ] Each item has clear pass/fail criteria
- [ ] Organized by feature
- [ ] Can be used for final validation

---

## Task Summary

**Total Tasks**: 30
**Estimated Time**: 32.5 hours (~4-5 work days)

### By Phase:
- **Phase 0** (Foundation): 5 tasks, 4.75h
- **Phase 1** (Bookmarks): 5 tasks, 9.5h
- **Phase 2** (Disk Usage): 5 tasks, 5.5h
- **Phase 3** (History): 5 tasks, 5h
- **Phase 4** (Editor): 6 tasks, 10h *(optional/deferred)*
- **Documentation**: 4 tasks, 1.75h

### By Priority:
- **P1** (Critical): 15 tasks - Bookmarks, Foundation, Docs
- **P2** (High): 9 tasks - Disk Usage, Edge Cases
- **P3** (Medium): 5 tasks - Navigation History
- **P4** (Low): 6 tasks - Text Editor *(can be deferred)*

### Recommended Execution:
1. **Day 1**: TASK-001 through TASK-010 (Foundation + Bookmarks MVP) - 12h
2. **Day 2**: TASK-011 through TASK-015 (Disk Usage) + TASK-027-029 (Docs) - 7.75h
3. **Day 3**: TASK-016 through TASK-020 (Navigation History) - 5h
4. **Day 4-5**: TASK-021 through TASK-026 (Editor) *(optional)* - 10h

**MVP Release** (v0.4.0): Complete Phase 0-2 = 20h (2.5 days)  
**Enhanced Release** (v0.4.0): Add Phase 3 = 25h (3 days)  
**Full Release** (v0.4.1): Add Phase 4 = 35h (4.5 days)
