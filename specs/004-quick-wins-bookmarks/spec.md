# Feature Specification: Quick Wins - Enhanced Navigation & Productivity

**Feature Branch**: `004-quick-wins-bookmarks`  
**Created**: 2025-10-26  
**Status**: Draft  
**Input**: User description: "Quick Wins - Bookmarks, Disk Usage, History Navigation and Text Editor"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Bookmark Favorite Directories (Priority: P1)

Users frequently work with the same directories (projects, downloads, documents) and need quick access to them without repetitive navigation.

**Why this priority**: Bookmarks are the most universally useful feature - every user will benefit immediately. High impact, moderate complexity.

**Independent Test**: Can be fully tested by creating bookmarks, reopening the app, and verifying persistence. Delivers instant value by reducing navigation time.

**Note**: Uses Ctrl+B keybinding (F5 is already assigned to Copy operation).

**Acceptance Scenarios**:

1. **Given** I'm in any directory, **When** I press Ctrl+B, **Then** a bookmark menu opens showing saved bookmarks and option to add current directory
2. **Given** the bookmark menu is open, **When** I select "Add Bookmark", **Then** the current directory is added to bookmarks with a custom name prompt
3. **Given** I have saved bookmarks, **When** I select a bookmark from the menu, **Then** the active panel navigates to that directory
4. **Given** I have bookmarks saved, **When** I restart leeky-explorer, **Then** all bookmarks persist and are available
5. **Given** I'm viewing the bookmark menu, **When** I press 'd' on a bookmark, **Then** that bookmark is deleted
6. **Given** I'm viewing the bookmark menu, **When** I press 'r' on a bookmark, **Then** I can rename the bookmark

---

### User Story 2 - Visual Disk Usage Indicators (Priority: P2)

Users need to quickly assess disk space availability across all drives without leaving the file explorer.

**Why this priority**: Highly visible quality-of-life improvement. Low complexity, high visual impact. Prevents "disk full" surprises.

**Independent Test**: Can be tested by viewing the drive selector and verifying correct usage percentages and visual bars for all mounted drives.

**Acceptance Scenarios**:

1. **Given** I open the drive selector (F9), **When** viewing the drive list, **Then** each drive shows a percentage bar of space used
2. **Given** a drive is nearly full (>90%), **When** viewing drive selector, **Then** the usage bar displays in warning color (red/yellow)
3. **Given** multiple drives exist, **When** viewing drive selector, **Then** all drives show accurate real-time usage information
4. **Given** I'm in any panel, **When** viewing the status bar, **Then** I see the current drive's available space

---

### User Story 3 - Navigation History (Back/Forward) (Priority: P3)

Users often need to return to previously visited directories or retrace their navigation path, similar to web browser behavior.

**Why this priority**: Nice-to-have feature that improves navigation fluidity. Familiar pattern from web browsers. Medium complexity.

**Independent Test**: Can be tested by navigating through several directories, pressing back/forward keys, and verifying correct path traversal.

**Acceptance Scenarios**:

1. **Given** I've navigated through multiple directories, **When** I press Alt+Left, **Then** I return to the previous directory in history
2. **Given** I've gone back in history, **When** I press Alt+Right, **Then** I move forward in history
3. **Given** I'm at the oldest point in history, **When** I press Alt+Left, **Then** nothing happens (or a subtle indicator shows no more history)
4. **Given** I've gone back and then navigate to a new directory, **When** I press Alt+Right, **Then** the forward history is cleared
5. **Given** I switch between panels, **When** I use history navigation, **Then** each panel maintains its own independent history

---

### User Story 4 - Simple Text File Editor (Priority: P4)

Users want to quickly edit configuration files, notes, or small text files without leaving the file explorer and opening external editors.

**Why this priority**: Lower priority due to complexity and potential scope creep. Most users have preferred external editors. Can be deferred to later iteration.

**Independent Test**: Can be tested by selecting a text file, pressing 'e', making edits, saving, and verifying changes persist.

**Acceptance Scenarios**:

1. **Given** I have a text file selected, **When** I press 'e' or F4, **Then** a modal editor opens with the file content
2. **Given** the editor is open, **When** I make changes, **Then** the changes are reflected in the editor buffer
3. **Given** I've made changes, **When** I press Ctrl+S, **Then** the file is saved to disk
4. **Given** the editor is open, **When** I press Esc, **Then** the editor closes (with unsaved changes warning if applicable)
5. **Given** I try to edit a binary file, **When** I press 'e', **Then** an error message indicates the file is not editable
6. **Given** I try to edit a large file (>1MB), **When** I press 'e', **Then** a warning prompts to use external editor instead

---

### Edge Cases

- **Bookmarks**: What happens when a bookmarked directory is deleted or moved?
- **Bookmarks**: How to handle duplicate bookmark names?
- **Bookmarks**: Maximum number of bookmarks (performance consideration)?
- **Disk Usage**: What happens when drive is unmounted while app is running?
- **Disk Usage**: How to handle network drives with slow response times?
- **History**: What happens when navigating to a directory that no longer exists?
- **History**: Maximum history size (memory consideration)?
- **Editor**: What happens when file is modified externally while being edited?
- **Editor**: How to handle files without write permissions?
- **Editor**: Character encoding detection (UTF-8, UTF-16, ASCII)?

## Requirements *(mandatory)*

### Functional Requirements

**Bookmarks (P1):**
- **FR-001**: System MUST persist bookmarks to disk in configuration file (~/.config/leeky/bookmarks.json or equivalent)
- **FR-002**: System MUST allow adding current directory as bookmark with custom name
- **FR-003**: System MUST allow removing existing bookmarks
- **FR-004**: System MUST allow renaming existing bookmarks
- **FR-005**: System MUST navigate to bookmarked directory when selected from menu
- **FR-006**: System MUST handle bookmarks to non-existent directories gracefully (show warning, offer to remove)
- **FR-007**: Bookmark menu MUST be accessible via Ctrl+B keybinding
- **FR-008**: System MUST support at least 50 bookmarks without performance degradation

**Disk Usage (P2):**
- **FR-009**: System MUST display disk usage percentage for all available drives
- **FR-010**: System MUST show visual progress bar for disk usage in drive selector
- **FR-011**: System MUST use warning colors (yellow >80%, red >90%) for high disk usage
- **FR-012**: System MUST display current drive's free space in status bar
- **FR-013**: System MUST refresh disk usage information when drive selector is opened
- **FR-014**: System MUST handle unmounted/unavailable drives without crashing

**Navigation History (P3):**
- **FR-015**: System MUST maintain separate navigation history for each panel
- **FR-016**: System MUST support backward navigation via Alt+Left keybinding
- **FR-017**: System MUST support forward navigation via Alt+Right keybinding
- **FR-018**: System MUST clear forward history when navigating to new directory from middle of history
- **FR-019**: System MUST persist history for at least 100 entries per panel
- **FR-020**: System MUST handle navigation to non-existent historical directories gracefully

**Text Editor (P4):**
- **FR-021**: System MUST open text editor via 'e' or F4 keybinding for selected file
- **FR-022**: System MUST support basic text editing (insert, delete, navigation)
- **FR-023**: System MUST save file via Ctrl+S keybinding
- **FR-024**: System MUST close editor via Esc keybinding
- **FR-025**: System MUST warn before closing editor with unsaved changes
- **FR-026**: System MUST prevent editing binary files (show error message)
- **FR-027**: System MUST warn when attempting to edit large files (>1MB)
- **FR-028**: System MUST handle UTF-8 encoded files
- **FR-029**: System MUST show line numbers in editor
- **FR-030**: System MUST support syntax highlighting [NICE TO HAVE - can defer]

### Key Entities

- **Bookmark**: Represents a saved directory location
  - `name`: String - User-friendly display name
  - `path`: PathBuf - Absolute path to directory
  - `created_at`: DateTime - When bookmark was created
  - `last_accessed`: DateTime - Last time bookmark was used (for sorting)

- **NavigationHistory**: Represents browsing history for a panel
  - `entries`: Vec<PathBuf> - Stack of visited directories
  - `current_index`: usize - Current position in history
  - `max_size`: usize - Maximum history size (default 100)

- **DiskUsageInfo**: Represents disk space information
  - `total_space`: u64 - Total disk capacity in bytes
  - `used_space`: u64 - Used space in bytes
  - `available_space`: u64 - Available space in bytes
  - `usage_percentage`: f64 - Calculated percentage (0-100)
  - `mount_point`: PathBuf - Drive/volume mount point

- **EditorBuffer**: Represents text file being edited
  - `content`: String - File content
  - `file_path`: PathBuf - Path to file
  - `modified`: bool - Whether buffer has unsaved changes
  - `cursor_position`: (usize, usize) - Line and column
  - `scroll_offset`: usize - Vertical scroll position

## Success Criteria *(mandatory)*

### Measurable Outcomes

**Bookmarks:**
- **SC-001**: Users can create and access bookmarks in under 5 seconds
- **SC-002**: Bookmarks persist across application restarts 100% of the time
- **SC-003**: Users report 50% reduction in time spent navigating to frequently used directories
- **SC-004**: Bookmark operations (add, remove, rename, navigate) complete in <100ms

**Disk Usage:**
- **SC-005**: Disk usage information displays within 500ms of opening drive selector
- **SC-006**: Users can identify nearly-full drives at a glance (without reading percentages)
- **SC-007**: Status bar shows current drive space without performance impact
- **SC-008**: 95% accuracy in disk usage calculations across all supported platforms

**Navigation History:**
- **SC-009**: Users can navigate backward/forward through history in <50ms per operation
- **SC-010**: History maintains accurate state for 100 navigation operations without memory issues
- **SC-011**: 90% of users discover and use history navigation within first session (with keybinding hints)
- **SC-012**: Each panel maintains independent history without cross-contamination

**Text Editor:**
- **SC-013**: Editor opens files <100KB in under 200ms
- **SC-014**: Users can make simple edits (config files, notes) without exiting application
- **SC-015**: 100% of file saves complete successfully or show clear error message
- **SC-016**: Zero data loss incidents due to editor bugs
- **SC-017**: Editor prevents editing binary files 100% of the time

**Overall:**
- **SC-018**: All quick wins features combined add <500KB to binary size
- **SC-019**: No measurable performance degradation in existing features
- **SC-020**: Feature discoverability: 70% of users find at least 2 of 4 features in first 10 minutes
