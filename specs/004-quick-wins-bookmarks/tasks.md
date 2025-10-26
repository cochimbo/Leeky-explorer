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
    // For editor (enhance existing F4 preview)
    EditFile,
    SaveFile,
}

// In key mapping
KeyEvent { code: KeyCode::Char('b'), modifiers: KeyModifiers::CONTROL, .. } => Some(Action::OpenBookmarkManager),
KeyEvent { code: KeyCode::Left, modifiers: KeyModifiers::ALT, .. } => Some(Action::NavigateBack),
KeyEvent { code: KeyCode::Right, modifiers: KeyModifiers::ALT, .. } => Some(Action::NavigateForward),
// F4 already opens preview (Action::OpenPreview), will add edit mode detection
```

**Note**: F4 is already mapped to `OpenPreview`. The editor will be an enhancement to the preview system, where pressing 'e' while in preview mode switches to edit mode for text files.

**Acceptance**:
- [ ] All new actions defined
- [ ] Ctrl+B mapped to bookmark manager
- [ ] Alt+Left/Right mapped
- [ ] F4 already mapped to preview (edit mode will be added)
- [ ] No keybinding conflicts (F5 for Copy, 'e' for QuickJump)

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

## Phase 4: Go To Path (Ctrl+G) (P3)

**Total Estimated Time**: 2-3 hours  
**Priority**: P3 - High productivity value, low complexity  
**Dependencies**: None (independent feature)

### TASK-021: Add Go To Path dialog state ⬜
**Priority**: P3 | **Time**: 30min | **Dependencies**: None

**Description**: Add dialog state for Go To Path input

**Files**:
- `src/app.rs` - MODIFY (add GoToPath variant to DialogState)
- `src/events/keybindings.rs` - MODIFY (add ToggleGoToPath action)

**Implementation**:
```rust
// In src/app.rs DialogState enum
GoToPath {
    input: String,
    error_message: Option<String>,
},

// In src/events/keybindings.rs
pub enum Action {
    // ... existing actions
    ToggleGoToPath, // Ctrl+G to open/close Go To Path dialog
}
```

**Acceptance**:
- [ ] DialogState::GoToPath variant added
- [ ] ToggleGoToPath action added
- [ ] Ctrl+G keybinding mapped
- [ ] Compiles without errors

---

### TASK-022: Create Go To Path dialog UI ⬜
**Priority**: P3 | **Time**: 1h | **Dependencies**: TASK-021

**Description**: Modal dialog for path input

**Files**:
- `src/ui/goto_dialog.rs` - NEW
- `src/ui/mod.rs` - MODIFY (add pub mod goto_dialog)
- `src/ui/dialog.rs` - MODIFY (add render case for GoToPath)

**Implementation**:
```rust
pub fn render(
    frame: &mut Frame,
    input: &str,
    error: &Option<String>,
    theme: &Theme,
) {
    // Centered modal dialog (50x30% of screen)
    // Title: "Go To Path (Ctrl+G)"
    // Input field with current input
    // Show error message if present
    // Footer: "Enter: Navigate | Esc: Cancel | Ctrl+V: Paste"
}
```

**UI Design**:
- Simple input box centered on screen
- Real-time path validation visual feedback
- Error messages in red below input
- Current directory hint

**Acceptance**:
- [ ] Dialog renders correctly
- [ ] Input text displayed
- [ ] Error messages shown when present
- [ ] Theme colors applied
- [ ] Footer with instructions

---

### TASK-023: Implement path validation and navigation ⬜
**Priority**: P3 | **Time**: 1h | **Dependencies**: TASK-022

**Description**: Handle path input, validation, and navigation

**Files**:
- `src/events/handler.rs` - MODIFY (add handle_goto_dialog function)
- `src/fs/path_utils.rs` - NEW or MODIFY (path expansion utilities)

**Implementation**:
```rust
fn handle_goto_dialog(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    // Handle text input
    // Handle Enter: validate and navigate
    // Handle Ctrl+V: paste from clipboard
    // Handle Esc: close dialog
}

fn expand_path(input: &str, current_dir: &Path) -> Result<PathBuf> {
    // Expand ~ to home directory
    // Expand %VAR% or $VAR environment variables
    // Resolve relative paths
    // Validate path exists and is directory
}
```

**Path Validation**:
1. Trim whitespace
2. Expand environment variables
3. Resolve relative paths
4. Check if path exists
5. Check if path is directory
6. Check read permissions

**Acceptance**:
- [ ] Absolute paths work (C:\Users\, /home/user/)
- [ ] Relative paths work (../, ./subdir)
- [ ] Environment variables expanded (~, %USERPROFILE%, $HOME)
- [ ] Invalid paths show clear error
- [ ] File paths rejected with error
- [ ] Permission errors handled gracefully
- [ ] Successful navigation adds to history
- [ ] Panel refreshes after navigation

---

### TASK-024: Write Go To Path tests ⬜
**Priority**: P3 | **Time**: 30min | **Dependencies**: TASK-023

**Description**: Unit and integration tests for Go To Path

**Files**:
- `tests/goto_test.rs` - NEW

**Test Cases**:
```rust
#[test]
fn test_absolute_path_navigation() { ... }

#[test]
fn test_relative_path_navigation() { ... }

#[test]
fn test_environment_variable_expansion() { ... }

#[test]
fn test_invalid_path_shows_error() { ... }

#[test]
fn test_file_path_rejected() { ... }

#[test]
fn test_whitespace_trimmed() { ... }

#[test]
fn test_adds_to_history() { ... }
```

**Acceptance**:
- [ ] All test cases pass
- [ ] Coverage for FR-021 through FR-030
- [ ] Edge cases tested (permissions, invalid chars, long paths)

---

### TASK-025: Update footer with Ctrl+G shortcut ✅
**Priority**: P3 | **Time**: 5min | **Dependencies**: TASK-023 | **Completed**: Phase 4

**Description**: Add Ctrl+G shortcut to footer help

**Files**:
- `src/ui/mod.rs` - MODIFY (render_footer function)

**Change**:
```rust
// Add to line 2 or 3 of footer shortcuts
("Ctrl+G", "Go To", Color::Yellow),
```

**Acceptance**:
- [x] Ctrl+G appears in footer
- [x] No layout issues
- [x] Visible in all themes

---

### TASK-026: Add shell-like autocomplete to Go To Path ✅
**Priority**: P2 | **Time**: 2h | **Dependencies**: TASK-025 | **Completed**: Phase 4

**Description**: Implement bash/zsh-style autocomplete with real-time suggestion list

**Features**:
- Real-time suggestion list showing subdirectories
- Tab: autocomplete common prefix or unique match (adds trailing separator)
- Enter: select highlighted suggestion and update input
- Up/Down: navigate suggestions with circular wrap
- Suggestions update dynamically as user types or deletes
- Initialize input with current directory path
- Fix Windows UNC path display (remove \\?\ prefix)

**Files**:
- `src/app.rs` - MODIFY (DialogState::GoToPath: add suggestions tracking)
- `src/events/handler.rs` - MODIFY (Tab/Up/Down handlers, helper functions)
- `src/ui/goto_dialog.rs` - MODIFY (render suggestions list)
- `src/ui/dialog.rs` - MODIFY (pass suggestions to render)

**Implementation**:
```rust
// In app.rs - DialogState::GoToPath
suggestions: Vec<PathBuf>,
selected_suggestion: usize,

// New helper functions in handler.rs
fn get_directory_children(path: &Path) -> Vec<PathBuf>
fn get_suggestions_for_input(input: &str, current_dir: &Path) -> Vec<PathBuf>
fn autocomplete_path(input: &str, suggestions: &[PathBuf]) -> String
fn expand_path_variables_only(input: &str) -> String
fn clean_windows_path(path: PathBuf) -> PathBuf  // Windows UNC fix
```

**UI Enhancements**:
- Suggestion list widget below input with scroll
- Highlight selected item with "►" indicator
- Dynamic footer messages based on state
- Green hint showing suggestion count and instructions
- "No subdirectories" message when list is empty

**Acceptance**:
- [x] Tab autocompletes common prefix or full match
- [x] Enter selects highlighted suggestion
- [x] Up/Down navigate with wrap-around
- [x] Suggestions update in real-time while typing
- [x] Input initialized with current directory
- [x] Windows UNC prefix (\\?\) removed from display
- [x] Visual feedback with highlight and arrows
- [x] Dynamic help messages
- [x] All existing tests pass (111 tests)
- [x] Code compiles without warnings

**Commit**: `980b3d4` - "feat: Add shell-like autocomplete to Go To Path dialog (Ctrl+G)"

---

## Phase 5: Text Editor (P4) - Optional/Deferred

### TASK-027: Create EditorBuffer struct ✅
**Priority**: P4 | **Time**: 2h | **Dependencies**: None
**Completed**: Used `tui-textarea` crate instead of custom implementation

**Description**: Buffer for text file content and cursor state. This will enhance the existing preview system to support editing.

**Files**:
- `src/ui/text_editor.rs` - NEW (or enhance `src/preview/text_viewer.rs`)
- `src/ui/mod.rs` - Add `pub mod text_editor;` if new

**Note**: Consider enhancing the existing preview system rather than creating a completely separate editor. The preview modal already opens with F4, we can add an 'e' key handler within preview mode to switch to edit mode for text files.

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

### TASK-028: Create text editor UI widget ✅
**Priority**: P4 | **Time**: 3h | **Dependencies**: TASK-027
**Completed**: Full modal editor with tui-textarea, theme integration, status bar

**Description**: Modal editor widget with rendering (enhancement of preview modal)

**Files**:
- `src/ui/text_editor.rs` - MODIFY (or `src/preview/text_viewer.rs`)

**Implementation Strategy**: Enhance the existing preview modal to support an "edit mode" for text files:
1. When preview is open (F4) for a text file, show hint: "Press 'e' to edit"
2. Pressing 'e' in preview switches to edit mode
3. Edit mode allows modifications
4. Ctrl+S saves, Esc returns to preview or closes

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
│ Ctrl+S: Save | Esc: Back to Preview | Line 5, Col 1               │
└────────────────────────────────────────────────────────────────────┘
```

**Note**: This reuses the preview infrastructure (F4), just adding edit capabilities.

**Acceptance**:
- [ ] Editor renders full screen
- [ ] Line numbers display
- [ ] Cursor visible and positioned correctly
- [ ] Text scrolls when cursor moves off screen
- [ ] Status bar shows file info
- [ ] Modified indicator works

---

### TASK-029: Add file type validation ✅
**Priority**: P4 | **Time**: 1h | **Dependencies**: TASK-028
**Completed**: Binary file detection (extensions + null bytes), 2MB size limit

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

// In preview modal handler (when 'e' pressed in preview)
Action::EditFile => {
    let file = app.active_panel().selected_file()?;
    
    if !is_text_file(&file.path) {
        app.show_error("Cannot edit binary file");
        return Ok(());
    }
    
    if is_file_too_large(&file.path, 1_048_576) {
        app.show_warning("File is large (>1MB). Use external editor?");
        return Ok(());
    }
    
    // Switch preview to edit mode
    if let Some(preview) = &mut app.preview_modal {
        preview.enable_edit_mode()?;
    }
}
```

**Note**: This assumes the user has already opened preview with F4, then presses 'e' within the preview to enter edit mode.

**Acceptance**:
- [ ] Binary files rejected
- [ ] Large files warned
- [ ] Text files open successfully
- [ ] User sees clear error messages

---

### TASK-030: Implement save and close handlers ✅
**Priority**: P4 | **Time**: 1h | **Dependencies**: TASK-029
**Completed**: Ctrl+S save, Esc close, unsaved changes confirmation dialog

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

### TASK-031: Add editor edge case handling ✅
**Priority**: P4 | **Time**: 1h | **Dependencies**: TASK-030  
**Status**: ✅ Complete  
**Completion Note**: Added file_mtime tracking, check_external_modifications() method, file existence check before save, and proper error messages for all edge cases.

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

### TASK-032: Write text editor tests ⬜
**Priority**: P4 | **Time**: 2h | **Dependencies**: TASK-031

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

## Phase 6: Recursive Deep Search (Ctrl+F)

### TASK-037: Create SearchResult and SearchState structs ⬜
**Priority**: P2 | **Time**: 1.5h | **Dependencies**: None

**Description**: Core data structures for recursive file search

**Files**:
- `src/search/mod.rs` - NEW (module declaration)
- `src/search/recursive.rs` - NEW
- `src/lib.rs` - MODIFY (add search module)

**SearchResult struct**:
```rust
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_name: String,
    pub full_path: PathBuf,
    pub relative_path: PathBuf,
    pub file_size: u64,
    pub modified_time: SystemTime,
}

impl SearchResult {
    pub fn new(full_path: PathBuf, root: &Path) -> Result<Self>;
    pub fn matches_pattern(&self, pattern: &str, case_sensitive: bool) -> bool;
}
```

**SearchState struct**:
```rust
pub struct SearchState {
    query: String,
    root_path: PathBuf,
    results: Vec<SearchResult>,
    is_running: bool,
    files_scanned: usize,
    use_glob: bool,
    max_depth: usize,
}

impl SearchState {
    pub fn new(query: String, root_path: PathBuf) -> Self;
    pub fn is_glob_pattern(query: &str) -> bool;
}
```

**Implementation Notes**:
- `is_glob_pattern()`: Check if query contains `*`, `?`, `[`, `]`
- `matches_pattern()`: Use `glob` crate or simple string matching
- `relative_path`: Use `pathdiff` or manual calculation from root

**Tests**:
```rust
#[test]
fn test_search_result_creation() { ... }

#[test]
fn test_glob_pattern_detection() { ... }

#[test]
fn test_matches_pattern_simple() { ... }

#[test]
fn test_matches_pattern_glob() { ... }
```

**Acceptance**:
- [ ] SearchResult stores file metadata correctly
- [ ] SearchState initializes with proper defaults
- [ ] Glob pattern detection works (*.rs, file?.txt)
- [ ] Unit tests pass

---

### TASK-038: Implement recursive search engine ⬜
**Priority**: P2 | **Time**: 2.5h | **Dependencies**: TASK-037

**Description**: Core search logic with recursive directory traversal

**Files**:
- `src/search/recursive.rs` - MODIFY
- `Cargo.toml` - MODIFY (add `glob` and `ignore` crates)

**Dependencies**:
```toml
glob = "0.3"
ignore = "0.4"  # Respects .gitignore, handles symlinks
```

**RecursiveSearcher implementation**:
```rust
pub struct RecursiveSearcher {
    state: Arc<Mutex<SearchState>>,
    cancel_flag: Arc<AtomicBool>,
}

impl RecursiveSearcher {
    pub fn new(query: String, root_path: PathBuf) -> Self;
    
    /// Start search in background thread
    pub fn start_search(&self) -> JoinHandle<()>;
    
    /// Recursive search implementation
    fn search_directory(
        &self,
        dir: &Path,
        current_depth: usize,
    ) -> Result<()>;
    
    /// Cancel ongoing search
    pub fn cancel(&self);
    
    /// Get current results (non-blocking)
    pub fn get_results(&self) -> Vec<SearchResult>;
    
    /// Check if search is still running
    pub fn is_running(&self) -> bool;
    
    /// Get progress (files scanned)
    pub fn files_scanned(&self) -> usize;
}
```

**Key Features**:
- Use `ignore::WalkBuilder` for efficient traversal
- Respect `.gitignore` files
- Handle permission errors gracefully (skip and continue)
- Max depth limit (default 20 levels)
- Detect circular symlinks
- Stream results as found (don't wait for completion)

**Algorithm**:
```rust
fn search_directory(&self, dir: &Path, depth: usize) -> Result<()> {
    // Check cancel flag
    if self.cancel_flag.load(Ordering::Relaxed) {
        return Ok(());
    }
    
    // Check max depth
    if depth > self.state.lock().unwrap().max_depth {
        return Ok(());
    }
    
    // Read directory entries
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        // Update scanned count
        self.state.lock().unwrap().files_scanned += 1;
        
        if path.is_dir() {
            // Recurse into subdirectory
            self.search_directory(&path, depth + 1)?;
        } else {
            // Check if file matches query
            if self.matches_query(&path) {
                let result = SearchResult::new(path, &self.root)?;
                self.state.lock().unwrap().results.push(result);
            }
        }
    }
    
    Ok(())
}
```

**Tests**:
```rust
#[test]
fn test_search_current_directory() { ... }

#[test]
fn test_search_recursive() { ... }

#[test]
fn test_glob_pattern_matching() { ... }

#[test]
fn test_cancel_search() { ... }

#[test]
fn test_max_depth_limit() { ... }

#[test]
fn test_permission_errors_handled() { ... }
```

**Acceptance**:
- [ ] Searches recursively through subdirectories
- [ ] Glob patterns work correctly
- [ ] Can cancel search mid-execution
- [ ] Handles permission errors without crashing
- [ ] Max depth prevents infinite recursion
- [ ] Tests pass

---

### TASK-039: Create search dialog UI component ⬜
**Priority**: P2 | **Time**: 2h | **Dependencies**: TASK-037

**Description**: Modal dialog for recursive search

**Files**:
- `src/ui/search_dialog.rs` - NEW
- `src/ui/mod.rs` - MODIFY (export search_dialog)

**SearchDialog struct**:
```rust
pub struct SearchDialog {
    input: String,
    results: Vec<SearchResult>,
    selected_index: usize,
    scroll_offset: usize,
    is_searching: bool,
    files_scanned: usize,
    searcher: Option<RecursiveSearcher>,
}

impl SearchDialog {
    pub fn new(root_path: PathBuf) -> Self;
    
    pub fn handle_key(&mut self, key: KeyEvent) -> DialogAction;
    
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme);
    
    fn start_search(&mut self);
    
    fn update_results(&mut self);
    
    fn render_input_field(&self, frame: &mut Frame, area: Rect, theme: &Theme);
    
    fn render_results_list(&self, frame: &mut Frame, area: Rect, theme: &Theme);
    
    fn render_progress(&self, frame: &mut Frame, area: Rect, theme: &Theme);
}
```

**UI Layout**:
```
┌─ Recursive Search (Ctrl+F) ──────────────────────────┐
│ Search: *.rs█                                         │
│ ────────────────────────────────────────────────────  │
│ Results (42 found, 1,234 files scanned):             │
│                                                       │
│ > src/main.rs                      2.3 KB  Today     │
│   src/app.rs                       5.1 KB  Yesterday │
│   src/ui/panel.rs                 12.4 KB  Oct 20    │
│   tests/integration_test.rs        1.8 KB  Oct 15    │
│   ...                                                 │
│                                                       │
│ [Searching... Press Esc to cancel]                   │
└───────────────────────────────────────────────────────┘
```

**Visual Differentiation from F3 Filter**:
- Title: "Recursive Search (Ctrl+F)" vs "Filter (F3)"
- Border color: Different from filter (use search_border theme color)
- Shows full paths vs just filenames
- Shows progress indicator
- Shows files scanned count

**Key Handling**:
- Any char: Add to input and restart search
- Backspace: Remove from input and restart search
- Up/Down: Navigate results
- Enter: Select result and close dialog
- Esc: Cancel search and close dialog
- Ctrl+C: Copy selected path to clipboard (optional)

**Acceptance**:
- [ ] Dialog opens centered on screen
- [ ] Input field accepts text
- [ ] Results update as search progresses
- [ ] Progress indicator shows during search
- [ ] Can navigate results with arrows
- [ ] Enter selects result
- [ ] Esc cancels and closes

---

### TASK-040: Integrate search dialog with AppState ⬜
**Priority**: P2 | **Time**: 1.5h | **Dependencies**: TASK-039

**Description**: Wire search dialog into main application

**Files**:
- `src/app.rs` - MODIFY
- `src/events/keybindings.rs` - MODIFY
- `src/events/handler.rs` - MODIFY

**AppState modifications**:
```rust
pub struct AppState {
    // ... existing fields
    pub search_dialog: Option<SearchDialog>,
}

impl AppState {
    pub fn open_search_dialog(&mut self) {
        let root = self.get_active_panel().get_path();
        self.search_dialog = Some(SearchDialog::new(root));
    }
    
    pub fn close_search_dialog(&mut self) {
        self.search_dialog = None;
    }
    
    pub fn has_search_dialog(&self) -> bool {
        self.search_dialog.is_some()
    }
    
    pub fn navigate_to_search_result(&mut self, result: &SearchResult) {
        // Navigate to parent directory
        let parent = result.full_path.parent().unwrap();
        self.get_active_panel_mut().set_path(parent.to_path_buf());
        
        // Select the file
        self.get_active_panel_mut().select_file(&result.file_name);
        
        self.close_search_dialog();
    }
}
```

**Keybinding**:
```rust
// In keybindings.rs
pub enum Action {
    // ... existing actions
    OpenRecursiveSearch,
}

// In get_action()
(KeyCode::Char('f'), KeyModifiers::CONTROL) => Action::OpenRecursiveSearch,
```

**Event Handler**:
```rust
// In handler.rs - handle_key()
pub fn handle_key(app: &mut AppState, key: KeyEvent) -> Result<bool> {
    // Priority 1: Search dialog (if open)
    if app.has_search_dialog() {
        if let Some(dialog) = app.search_dialog.as_mut() {
            match dialog.handle_key(key) {
                DialogAction::Close => {
                    app.close_search_dialog();
                    return Ok(true);
                }
                DialogAction::Navigate(result) => {
                    app.navigate_to_search_result(&result);
                    return Ok(true);
                }
                DialogAction::Continue => return Ok(true),
            }
        }
    }
    
    // Handle OpenRecursiveSearch action
    match action {
        Action::OpenRecursiveSearch => {
            app.open_search_dialog();
            Ok(true)
        }
        // ... other actions
    }
}
```

**Rendering Priority** (in event_loop.rs):
```rust
// Render order (highest to lowest):
// 1. Welcome screen
// 2. Dialogs (Input, Password, Confirm, DriveSelector)
// 3. Preview modal
// 4. Editor
// 5. Search dialog  <-- Add here
// 6. Panels + footer
```

**Acceptance**:
- [ ] Ctrl+F opens search dialog
- [ ] Search dialog has higher z-index than panels
- [ ] Esc closes search dialog
- [ ] Enter navigates to selected result
- [ ] Search works in both left and right panel
- [ ] Integration with existing app flow works

---

### TASK-041: Add search performance optimizations ⬜
**Priority**: P3 | **Time**: 1.5h | **Dependencies**: TASK-038, TASK-039

**Description**: Optimize search for large directory trees

**Files**:
- `src/search/recursive.rs` - MODIFY
- `src/ui/search_dialog.rs` - MODIFY

**Optimizations**:

1. **Debounced Search**:
```rust
// Don't restart search on every keystroke
// Wait 300ms after last keypress
use std::time::{Duration, Instant};

struct SearchDialog {
    last_input_time: Instant,
    debounce_duration: Duration,
}

// In update loop:
if self.last_input_time.elapsed() > self.debounce_duration {
    self.start_search();
}
```

2. **Result Limit**:
```rust
// Stop search after finding N results
const MAX_RESULTS: usize = 500;

if self.state.lock().unwrap().results.len() >= MAX_RESULTS {
    self.cancel();
}
```

3. **Cache Recent Searches**:
```rust
// Store last 10 searches to avoid re-scanning
use lru::LruCache;

struct SearchCache {
    cache: LruCache<String, Vec<SearchResult>>,
}
```

4. **Skip Large Directories**:
```rust
// Skip common large dirs: node_modules, target, .git
const SKIP_DIRS: &[&str] = &["node_modules", "target", ".git", "dist", "build"];

if SKIP_DIRS.contains(&dir_name.to_str().unwrap()) {
    continue;
}
```

5. **Progress Throttling**:
```rust
// Update UI every N files instead of every file
if files_scanned % 100 == 0 {
    // Trigger UI update
}
```

**Acceptance**:
- [ ] Search doesn't freeze UI with 10,000+ files
- [ ] Debouncing prevents excessive searches
- [ ] Result limit prevents memory issues
- [ ] Common large directories skipped
- [ ] Progress updates don't spam UI

---

### TASK-042: Add integration tests for recursive search ⬜
**Priority**: P2 | **Time**: 2h | **Dependencies**: TASK-037 through TASK-041

**Description**: Comprehensive testing of search functionality

**Files**:
- `tests/recursive_search_test.rs` - NEW

**Test Scenarios**:

```rust
#[test]
fn test_simple_recursive_search() {
    // Create temp directory structure:
    // root/
    //   file1.txt
    //   subdir/
    //     file2.txt
    //     nested/
    //       file3.txt
    
    let searcher = RecursiveSearcher::new("file".into(), root);
    searcher.start_search().join().unwrap();
    
    let results = searcher.get_results();
    assert_eq!(results.len(), 3);
}

#[test]
fn test_glob_pattern_search() {
    // Search for "*.txt"
    let searcher = RecursiveSearcher::new("*.txt".into(), root);
    // Assert only .txt files found
}

#[test]
fn test_case_insensitive_search() {
    // Search for "FILE" should find "file1.txt"
}

#[test]
fn test_max_depth_limit() {
    // Create deeply nested structure
    // Verify search stops at max_depth
}

#[test]
fn test_permission_denied_handling() {
    // Create directory without read permissions
    // Verify search continues without crashing
}

#[test]
fn test_cancel_mid_search() {
    // Start search on large directory
    // Cancel after 100ms
    // Verify it stops quickly
}

#[test]
fn test_empty_results() {
    // Search for non-existent pattern
    // Verify empty results, no crash
}

#[test]
fn test_special_characters_in_query() {
    // Search for files with spaces, unicode, etc.
}

#[test]
fn test_symlink_handling() {
    // Create circular symlink
    // Verify search doesn't infinite loop
}

#[test]
fn test_navigate_to_result() {
    // Simulate selecting search result
    // Verify panel navigates to correct directory
}
```

**Performance Tests**:
```rust
#[test]
fn test_search_performance_1000_files() {
    // Create 1000 files across 100 directories
    let start = Instant::now();
    let searcher = RecursiveSearcher::new("test".into(), root);
    searcher.start_search().join().unwrap();
    let elapsed = start.elapsed();
    
    assert!(elapsed < Duration::from_secs(2));
}

#[test]
fn test_ui_responsiveness() {
    // Verify UI updates don't block
    // Check that results stream in incrementally
}
```

**Acceptance**:
- [ ] All integration tests pass
- [ ] Coverage includes FR-041 through FR-055
- [ ] Edge cases handled correctly
- [ ] Performance acceptable (<2s for 1000 files)

---

## Documentation and Final Tasks

### TASK-033: Update README with new features ⬜
**Priority**: P1 | **Time**: 0.5h | **Dependencies**: Completed phases

**Description**: Document new keybindings and features

**Files**:
- `README.md` - MODIFY

**Sections to update**:
- Features list
- Keybindings table (Ctrl+B, Alt+Left/Right, F4 edit mode)
- Configuration section (bookmarks.json)
- Screenshots (if applicable)

**Keybinding Documentation**:
- Ctrl+B: Open/close bookmark manager
- Alt+Left: Navigate backward in history
- Alt+Right: Navigate forward in history  
- F4: Open preview (press 'e' in preview to edit text files)
- Ctrl+S: Save file (when in edit mode)

**Acceptance**:
- [ ] All new features documented
- [ ] Keybindings table updated
- [ ] Examples provided
- [ ] Configuration files documented

---

### TASK-034: Update CHANGELOG ⬜
**Priority**: P1 | **Time**: 0.25h | **Dependencies**: TASK-033

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
- [If included] Simple text editor accessible from preview (F4 then 'e')

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

### TASK-035: Run full test suite ⬜
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

### TASK-036: Create checklist for requirements ⬜
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

**Total Tasks**: 42  
**Completed**: 31 tasks (TASK-001 through TASK-031) ✅  
**Remaining**: 11 tasks (TASK-032 through TASK-042)  
**Estimated Time Remaining**: ~14.75 hours

### By Phase:
- **Phase 0** (Foundation): 5 tasks, 4.75h ✅
- **Phase 1** (Bookmarks): 5 tasks, 9.5h ✅
- **Phase 2** (Disk Usage): 3 tasks, 5.5h ✅
- **Phase 3** (Navigation History): 5 tasks, 5h ✅
- **Phase 4** (Go To Path): 6 tasks, 4.5h ✅
- **Phase 5** (Text Editor): 5 tasks, 8h ✅
- **Phase 6** (Recursive Search): 6 tasks, 11h ⬜ *(NEW!)*
- **Documentation**: 5 tasks, 2.25h ⬜

### By Priority:
- **P1** (Critical): 18 tasks - All core features complete ✅
- **P2** (High): 16 tasks - 10 complete, 6 remaining (search feature) ⬜
- **P3** (Medium): 7 tasks - 6 complete, 1 remaining (optimizations)
- **P4** (Low): 6 tasks - 5 complete, 1 remaining (integration tests)

### Completed Work:
1. ✅ **Phase 0-1**: TASK-001 through TASK-010 (Foundation + Bookmarks) - 14.25h
2. ✅ **Phase 2**: TASK-011 through TASK-015 (Disk Usage) - 5.5h
3. ✅ **Phase 3**: TASK-016 through TASK-020 (Navigation History) - 5h
4. ✅ **Phase 4**: TASK-021 through TASK-026 (Go To Path + Autocomplete) - 4.5h
5. ✅ **Phase 5**: TASK-027 through TASK-031 (Text Editor + Edge Cases) - 8h

**Current Status**: 
- 31+ commits on branch `004-quick-wins-bookmarks`
- 81+ tests passing
- All Phase 0-5 features fully functional
- Ready to implement Phase 6 (Recursive Search)

### Remaining Work:

**Phase 6 - Recursive Search (11h)**:
- **TASK-037**: SearchResult/SearchState structs - 1.5h
- **TASK-038**: Recursive search engine - 2.5h
- **TASK-039**: Search dialog UI - 2h
- **TASK-040**: Integration with AppState - 1.5h
- **TASK-041**: Performance optimizations - 1.5h
- **TASK-042**: Integration tests - 2h

**Testing & Documentation (3.75h)**:
- **TASK-032**: Editor integration tests - 2h
- **TASK-033**: Update README - 0.5h
- **TASK-034**: Update CHANGELOG - 0.25h
- **TASK-035**: Full test suite validation - 0.5h
- **TASK-036**: Requirements checklist - 0.5h

**Next Steps**:
1. Implement Phase 6 (Recursive Search)
2. Complete TASK-032 (editor tests)
3. Documentation (TASK-033 to 036)
4. Release v0.5.0 with 6 major features!
4. Release v0.4.0
