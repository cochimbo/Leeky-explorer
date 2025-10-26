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

### User Story 4 - Go To Path (Ctrl+G) (Priority: P3)

Users need to quickly navigate to a known directory path without manually clicking through the directory tree, especially for deeply nested or frequently accessed locations.

**Why this priority**: High productivity boost for power users. Simple implementation with immediate value. Complements bookmarks by handling one-off navigation needs.

**Independent Test**: Can be tested by pressing Ctrl+G, typing a path, and verifying navigation to that directory in the active panel.

**Acceptance Scenarios**:

1. **Given** I'm in any directory, **When** I press Ctrl+G, **Then** a "Go To Path" dialog opens with an input field
2. **Given** the Go To dialog is open, **When** I type a valid absolute path and press Enter, **Then** the active panel navigates to that directory
3. **Given** the Go To dialog is open, **When** I type a valid relative path and press Enter, **Then** the active panel navigates to that path relative to current directory
4. **Given** the Go To dialog is open, **When** I type an invalid or non-existent path, **Then** an error message is shown and the dialog remains open
5. **Given** the Go To dialog is open, **When** I press Esc, **Then** the dialog closes without navigation
6. **Given** the Go To dialog is open, **When** I type a path to a file (not a directory), **Then** an error indicates only directories are valid
7. **Given** I have a path in my clipboard, **When** I open Go To dialog, **Then** I can paste the path (Ctrl+V)
8. **Given** I'm typing a path, **When** the path contains environment variables like %USERPROFILE% or ~, **Then** the system expands them correctly

---

### User Story 5 - Simple Text File Editor (Priority: P4)

Users want to quickly edit configuration files, notes, or small text files without leaving the file explorer and opening external editors.

**Why this priority**: Lower priority due to complexity and potential scope creep. Most users have preferred external editors. Can be deferred to later iteration.

**Independent Test**: Can be tested by selecting a text file, pressing F4, making edits, saving, and verifying changes persist.

**Note**: Uses F4 keybinding (already assigned to preview, will be enhanced to support editing mode).

**Acceptance Scenarios**:

1. **Given** I have a text file selected, **When** I press F4 and then 'e' for edit mode, **Then** a modal editor opens with the file content
2. **Given** the editor is open, **When** I make changes, **Then** the changes are reflected in the editor buffer
3. **Given** I've made changes, **When** I press Ctrl+S, **Then** the file is saved to disk
4. **Given** the editor is open, **When** I press Esc, **Then** the editor closes (with unsaved changes warning if applicable)
5. **Given** I try to edit a binary file, **When** I press F4 and 'e', **Then** an error message indicates the file is not editable
6. **Given** I try to edit a large file (>1MB), **When** I press F4 and 'e', **Then** a warning prompts to use external editor instead

---

### User Story 6 - Recursive Deep Search (Ctrl+F) (Priority: P2)

Users need to find files across entire directory trees, not just in the current folder, when they know part of a filename but not its exact location.

**Why this priority**: High-value feature for large codebases and nested directory structures. Distinguishes from F3 filter which only searches current directory. Similar to `fd` or `fzf` tools.

**Independent Test**: Can be tested by pressing Ctrl+F, typing a search term, and verifying results from all subdirectories appear with full paths.

**Key Difference from F3 Filter**:
- **F3 (Filter)**: Filters files in CURRENT directory only. Fast, instant results, no subdirectories.
- **Ctrl+F (Search)**: Searches RECURSIVELY through all subdirectories. Shows results with full paths. Can take time on large trees.

**Acceptance Scenarios**:

1. **Given** I'm in any directory, **When** I press Ctrl+F, **Then** a search dialog opens with an input field for search term
2. **Given** the search dialog is open, **When** I type a search term, **Then** results appear in real-time showing files matching the term from current directory and all subdirectories
3. **Given** search results are displayed, **When** I select a result and press Enter, **Then** the active panel navigates to the parent directory of that file and selects it
4. **Given** the search is running, **When** I press Esc, **Then** the search is cancelled and dialog closes
5. **Given** I have search results, **When** I press Up/Down arrows, **Then** I can navigate through the result list
6. **Given** search results are displayed, **When** there are many results, **Then** results are paginated or scrollable
7. **Given** I'm searching, **When** I type `*.rs` or `*.txt`, **Then** the search supports glob patterns
8. **Given** the search finds no matches, **When** the search completes, **Then** a "No results found" message is displayed
9. **Given** I start a search in a large directory tree, **When** the search takes time, **Then** a progress indicator shows search is ongoing
10. **Given** I'm viewing search results, **When** results show full paths, **Then** paths are displayed relative to the search root for readability

**Performance Considerations**:
- Search should be interruptible (cancel with Esc)
- Results should stream in as they're found (don't wait for full tree scan)
- Large directories (>10,000 files) should show progress indicator
- Option to limit search depth or file count

---

### Edge Cases

- **Bookmarks**: What happens when a bookmarked directory is deleted or moved?
- **Bookmarks**: How to handle duplicate bookmark names?
- **Bookmarks**: Maximum number of bookmarks (performance consideration)?
- **Disk Usage**: What happens when drive is unmounted while app is running?
- **Disk Usage**: How to handle network drives with slow response times?
- **History**: What happens when navigating to a directory that no longer exists?
- **History**: Maximum history size (memory consideration)?
- **Go To Path**: What happens when user enters a path with invalid characters?
- **Go To Path**: How to handle paths with insufficient permissions?
- **Go To Path**: How to handle very long paths (>260 chars on Windows)?
- **Go To Path**: Should the input support path auto-completion?
- **Editor**: What happens when file is modified externally while being edited?
- **Editor**: How to handle files without write permissions?
- **Editor**: Character encoding detection (UTF-8, UTF-16, ASCII)?
- **Recursive Search**: What happens when searching a directory tree with thousands of files?
- **Recursive Search**: How to handle permission denied errors on some subdirectories?
- **Recursive Search**: Should hidden files/folders be included in search results?
- **Recursive Search**: How to handle symbolic links that create circular references?
- **Recursive Search**: What's the maximum search depth to prevent infinite loops?
- **Recursive Search**: How to differentiate from F3 filter in the UI?

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

**Go To Path (P3):**
- **FR-021**: System MUST open Go To Path dialog via Ctrl+G keybinding
- **FR-022**: System MUST accept absolute paths (e.g., C:\Users\John\Documents, /home/user/documents)
- **FR-023**: System MUST accept relative paths from current directory (e.g., ../parent, ./subdir)
- **FR-024**: System MUST expand environment variables (%USERPROFILE%, $HOME, ~)
- **FR-025**: System MUST validate path before navigation and show error for invalid paths
- **FR-026**: System MUST reject paths that point to files (only directories allowed)
- **FR-027**: System MUST handle permission errors gracefully (show clear error message)
- **FR-028**: System MUST support clipboard paste in path input (Ctrl+V)
- **FR-029**: System MUST trim whitespace from input path
- **FR-030**: System MUST add successful navigation to history

**Text Editor (P4):**
- **FR-031**: System MUST open text editor via F4 keybinding for selected file (edit mode in preview)
- **FR-032**: System MUST support basic text editing (insert, delete, navigation)
- **FR-033**: System MUST save file via Ctrl+S keybinding
- **FR-034**: System MUST close editor via Esc keybinding
- **FR-035**: System MUST warn before closing editor with unsaved changes
- **FR-036**: System MUST prevent editing binary files (show error message)
- **FR-037**: System MUST warn when attempting to edit large files (>1MB)
- **FR-038**: System MUST handle UTF-8 encoded files
- **FR-039**: System MUST show line numbers in editor
- **FR-040**: System MUST support syntax highlighting [NICE TO HAVE - can defer]

**Recursive Search (P2):**
- **FR-041**: System MUST open search dialog via Ctrl+F keybinding
- **FR-042**: System MUST search recursively through all subdirectories from current directory
- **FR-043**: System MUST display results with relative paths from search root
- **FR-044**: System MUST support case-insensitive search by default
- **FR-045**: System MUST support glob patterns (*.rs, *.txt, file?.log)
- **FR-046**: System MUST allow cancelling search via Esc keybinding
- **FR-047**: System MUST navigate to selected result's parent directory on Enter
- **FR-048**: System MUST stream results as they are found (incremental display)
- **FR-049**: System MUST show progress indicator for searches taking >500ms
- **FR-050**: System MUST handle permission errors gracefully (skip and continue)
- **FR-051**: System MUST limit search depth to prevent stack overflow (default: 20 levels)
- **FR-052**: System MUST differentiate visually from F3 filter (different dialog title/style)
- **FR-053**: System MUST display "No results found" when search completes with no matches
- **FR-054**: System MUST support up/down navigation through results list
- **FR-055**: System MUST exclude hidden files/folders by default [CONFIGURABLE]

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

- **SearchResult**: Represents a file found by recursive search
  - `file_name`: String - Name of the file
  - `full_path`: PathBuf - Absolute path to file
  - `relative_path`: PathBuf - Path relative to search root
  - `file_size`: u64 - Size in bytes
  - `modified_time`: SystemTime - Last modification time
  - `match_score`: f32 - Relevance score (for fuzzy matching, optional)

- **SearchState**: Represents ongoing search operation
  - `query`: String - Search term or pattern
  - `root_path`: PathBuf - Starting directory for search
  - `results`: Vec<SearchResult> - Found files
  - `is_running`: bool - Whether search is in progress
  - `files_scanned`: usize - Progress counter
  - `use_glob`: bool - Whether query is a glob pattern

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

**Go To Path:**
- **SC-013**: Dialog opens and accepts input in <50ms
- **SC-014**: Path validation completes in <100ms for local paths
- **SC-015**: Users can navigate to any valid directory with 100% accuracy
- **SC-016**: Environment variable expansion works correctly on all supported platforms
- **SC-017**: Error messages clearly indicate why a path is invalid
- **SC-018**: 80% of power users adopt Ctrl+G as preferred navigation method for known paths

**Text Editor:**
- **SC-019**: Editor opens files <100KB in under 200ms
- **SC-020**: Users can make simple edits (config files, notes) without exiting application
- **SC-021**: 100% of file saves complete successfully or show clear error message
- **SC-022**: Zero data loss incidents due to editor bugs
- **SC-023**: Editor prevents editing binary files 100% of the time

**Recursive Search:**
- **SC-024**: Search dialog opens in <50ms
- **SC-025**: First results appear within 100ms for directories with <1000 files
- **SC-026**: Search can be cancelled instantly with Esc at any point
- **SC-027**: Users can find files 3x faster than manual navigation through subdirectories
- **SC-028**: Search correctly handles permission errors without crashing
- **SC-029**: Glob patterns work accurately for all common cases (*.ext, file?, prefix*)
- **SC-030**: Users clearly understand difference between F3 (filter) and Ctrl+F (search)
- **SC-031**: Search scales to directories with 10,000+ files without freezing UI
- **SC-032**: 90% of users discover and use recursive search within first 3 sessions

**Overall:**
- **SC-033**: All quick wins features combined add <500KB to binary size
- **SC-034**: No measurable performance degradation in existing features
- **SC-035**: Feature discoverability: 70% of users find at least 4 of 6 features in first 10 minutes
