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

### TASK-027: Create EditorBuffer struct ⬜
**Priority**: P4 | **Time**: 2h | **Dependencies**: None

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

### TASK-028: Create text editor UI widget ⬜
**Priority**: P4 | **Time**: 3h | **Dependencies**: TASK-027

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

### TASK-029: Add file type validation ⬜
**Priority**: P4 | **Time**: 1h | **Dependencies**: TASK-028

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

### TASK-030: Implement save and close handlers ⬜
**Priority**: P4 | **Time**: 1h | **Dependencies**: TASK-029

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

### TASK-031: Add editor edge case handling ⬜
**Priority**: P4 | **Time**: 1h | **Dependencies**: TASK-030

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

**Total Tasks**: 36  
**Completed**: 26 tasks (TASK-001 through TASK-026) ✅  
**Remaining**: 10 tasks (TASK-027 through TASK-036)  
**Estimated Time**: 34.5 hours (~4-5 work days)

### By Phase:
- **Phase 0** (Foundation): 5 tasks, 4.75h ✅
- **Phase 1** (Bookmarks): 5 tasks, 9.5h ✅
- **Phase 2** (Disk Usage): 3 tasks, 5.5h ✅
- **Phase 3** (Navigation History): 5 tasks, 5h ✅
- **Phase 4** (Go To Path): 6 tasks, 4.5h ✅
- **Phase 5** (Text Editor): 6 tasks, 10h ⬜ *(optional/deferred)*
- **Documentation**: 4 tasks, 1.75h ⬜

### By Priority:
- **P1** (Critical): 18 tasks - Bookmarks, Foundation, Go To Path, Docs
- **P2** (High): 10 tasks - Disk Usage, Edge Cases, Autocomplete
- **P3** (Medium): 6 tasks - Navigation History, Go To Path
- **P4** (Low): 6 tasks - Text Editor *(can be deferred)*

### Completed Work:
1. ✅ **Phase 0-1**: TASK-001 through TASK-010 (Foundation + Bookmarks) - 12h
2. ✅ **Phase 2**: TASK-011 through TASK-015 (Disk Usage) - 5.5h
3. ✅ **Phase 3**: TASK-016 through TASK-020 (Navigation History) - 5h
4. ✅ **Phase 4**: TASK-021 through TASK-026 (Go To Path + Autocomplete) - 4.5h

**Current Status**: 
- 27 commits on branch `004-quick-wins-bookmarks`
- 111 tests passing (78 library + 33 integration)
- All features fully functional

### Remaining Work:
- **Phase 5** (Text Editor): TASK-027 through TASK-032 - 10h *(optional)*
- **Documentation**: TASK-033 through TASK-036 - 1.75h

**Release Options**:
- **MVP Release** (v0.4.0): Complete Phases 0-4 + Docs = 27h ✅ (CURRENT)
- **Full Release** (v0.4.1): Add Phase 5 (Text Editor) = 37h
