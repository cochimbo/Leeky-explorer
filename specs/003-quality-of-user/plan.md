# Implementation Plan: Quality of User Experience Improvements

**Branch**: `003-quality-of-user` | **Date**: 2025-01-20 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/003-quality-of-user/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

**User Story 1 (Welcome Screen)**: Implement a welcome screen that displays on application launch, featuring ASCII art branding and the current version number. User presses Enter to proceed to the main file manager interface.

**User Story 2 (Disk Space Info)**: Replace redundant path display in header with actionable disk space information showing used/total space and percentage free for each panel's current filesystem/drive.

Technical approach uses existing Ratatui rendering capabilities, crossterm event handling, and integrates with current application state management. For disk space, we'll use the fs2 crate (already a dependency) to query filesystem statistics.

## Technical Context

**Language/Version**: Rust (edition = "2024")  
**Primary Dependencies**: 
- ratatui 0.29.0 (TUI framework)
- crossterm 0.29.0 (terminal handling)
- tokio 1.35 (async runtime with "full" features)
- anyhow 1.0 (error handling)
- fs2 3.0 (filesystem utilities - ALREADY DEPENDENCY, used for disk space queries)

**Storage**: Static ASCII art file in assets/images/, version from Cargo.toml  
**Testing**: cargo test with tempfile 3.8 (dev dependency), manual terminal testing  
**Target Platform**: Cross-platform terminal (Linux, Windows, macOS)
**Project Type**: Single project - dual-pane TUI file manager  
**Performance Goals**: <1 second welcome screen display time, instant Enter key response  
**Constraints**: Must work in terminals as small as 80x24, graceful fallback for missing assets  
**Scale/Scope**: 
- User Story 1: Single welcome screen, ~100 lines of code, 1 ASCII art asset file
- User Story 2: Header modification, ~150 lines of code (disk space module + header update), cross-platform filesystem detection

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

[Gates determined based on constitution file]

## Project Structure

### Documentation (this feature)

```
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)
### Source Code (repository root)

```
src/
├── main.rs              # Modified: Show welcome screen before initializing panels
├── ui/
│   ├── mod.rs           # Modified US1: Export welcome_screen module, add render_welcome function
│                        # Modified US2: Update render_header to show disk space instead of paths
│   ├── welcome_screen.rs # New US1: Welcome screen rendering and logic
│   ├── layout.rs        # Existing: May need welcome screen layout
│   └── [other ui files] # Existing: dialog.rs, panel_widget.rs, etc.
├── app.rs               # Modified US1: Add show_welcome: bool field to AppState
│                        # Modified US2: Add disk_space_cache: HashMap<PathBuf, DiskSpaceInfo>
├── fs/
│   ├── mod.rs           # Existing: Filesystem operations module
│   └── disk_info.rs     # New US2: Disk space query utilities
├── event_loop.rs        # Modified US1: Handle welcome screen state in main loop
└── events/
    └── handler.rs       # Modified US1: Handle Enter key when show_welcome is true

assets/
└── images/
    ├── leekpc.png       # Existing: Project image asset (used for dynamic ASCII conversion)

tests/
├── unit/
│   ├── welcome_screen_test.rs # New US1: Unit tests for welcome screen
│   └── disk_info_test.rs      # New US2: Unit tests for disk space queries
└── [other test files]   # Existing: file_operations_test.rs, config_test.rs, etc.

Cargo.toml               # Modified later: Update to version 0.3.0 when ready
```

**Structure Decision US1**: Single project structure with welcome screen as a new UI module in `src/ui/`. The project does NOT use an AppScreen enum - instead it uses `AppState` with various state flags and dialog states. The welcome screen will use a simple `show_welcome: bool` flag in AppState. When true, render welcome screen instead of panels. When user presses Enter, set to false and proceed to normal file manager UI.

**Structure Decision US2**: Disk space information will be queried using fs2 crate (already a dependency for disk operations). A new `src/fs/disk_info.rs` module will encapsulate cross-platform disk space queries. The header in `src/ui/mod.rs` will be updated to show disk space for each panel's current path instead of the full path (which is already visible in panel borders).

## Complexity Tracking

*No constitution violations - this is a simple, focused feature addition*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |

## Implementation Phases

### User Story 1: Welcome Screen

#### Phase 0: Setup and Asset Creation
**Goal**: Prepare welcome screen infrastructure

1. **Create ASCII Art Logo**
   - ~~Create `assets/images/logo.txt` with ASCII art~~ CHANGED: Use dynamic PNG→ASCII conversion
   - Use existing `assets/images/leekpc.png` with `image_to_ascii()` function (from preview module)
   - Test that art displays correctly in different terminal sizes
   - Create fallback message if PNG is missing or conversion fails: ASCII borders with "Leeky File Manager"

2. **Add show_welcome Flag to AppState**
   - Modify `src/app.rs` to add `pub show_welcome: bool` field to AppState struct
   - Initialize to `true` in `AppState::new()` method
   - This flag controls whether to show welcome screen or normal UI

#### Phase 1: Welcome Screen Module
**Goal**: Implement welcome screen rendering logic

1. **Create `src/ui/welcome_screen.rs`**
   ```rust
   // Core structure:
   - pub fn render(frame: &mut Frame, area: Rect, version: &str)
   - fn load_logo(terminal_area: Rect) -> String // Load PNG and convert to ASCII
   - fn fallback_logo() -> String // Fallback ASCII art with borders
   - pub fn render_minimal(frame: &mut Frame, area: Rect, version: &str) // For small terminals
   ```

2. **Rendering Logic**
   - Use `image::open()` to load PNG synchronously
   - Calculate target dimensions: (area.width - 4, area.height - 8)
   - Call `image_to_ascii()` from preview module with calculated dimensions
   - Parse ANSI color codes using shared `parse_ansi_line()` function
   - Use Ratatui's `Paragraph` widget for ASCII art with colored Spans
   - Use `Block` with centered text for version display (cyan)
   - Show "Press Enter to continue" instruction (green)
   - Handle small terminals: show simplified version if height < 40 or width < 10

3. **Export Module**
   - Add `pub mod welcome_screen;` to `src/ui/mod.rs`
   - Add `pub fn render_welcome(frame: &mut Frame, version: &str)` wrapper

#### Phase 2: Event Handling
**Goal**: Detect Enter key and transition to main interface

1. **Modify `src/events/handler.rs`**
   - Add early check at beginning of `handle_key` function
   - If `app.show_welcome == true` and key is Enter: set `app.show_welcome = false`
   - If `app.show_welcome == true` and key is NOT Enter: return early (ignore all other keys)
   - This prevents any file manager actions during welcome screen

2. **KeyEvent Filtering**
   - Filter KeyEventKind::Press only to prevent double events from Press/Release/Repeat
   - Add check in event_loop.rs: `if key.kind != KeyEventKind::Press { continue; }`

3. **State Transition**
   - Setting `app.show_welcome = false` causes next render to show normal panels
   - Panels are already initialized in main.rs before event loop starts
   - No additional initialization needed

#### Phase 3: Rendering Integration
**Goal**: Show welcome screen on application startup

1. **Modify `src/event_loop.rs`**
   - In the `run()` function's render loop, check `app.show_welcome`
   - If true: call `ui::render_welcome()` instead of normal UI rendering
   - If false: proceed with existing render logic (panels, dialogs, etc.)
   - Pass version string from `env!("CARGO_PKG_VERSION")` to render_welcome

2. **Modify `src/ui/mod.rs`**
   - Add public function `pub fn render_welcome(frame: &mut Frame, version: &str)`
   - This function calls `welcome_screen::render()` with full terminal area
   - Refactor `parse_ansi_line()` to shared function (DRY principle)

#### Phase 4: ASCII Conversion Optimization
**Goal**: Improve image quality for welcome screen

1. **Optimize `src/preview/image_viewer.rs`**
   - Adjust brightness thresholds from [64,128,192,255] to [32,96,160,220]
   - Remove light shade (`░`) character to reduce background noise
   - Remove dark shade (`▓`) character, use full block (`█`) instead
   - Keep medium shade (`▒`) for necessary mid-tone transitions

#### Phase 5: Testing and Edge Cases
**Goal**: Verify all scenarios from spec

1. **Unit Tests** (SKIPPED - decided not to create)
   
2. **Manual Testing**
   - Test on Windows PowerShell, ~~Linux terminals, macOS Terminal.app~~ (cross-platform pending)
   - Test terminal resize during welcome screen (pending)
   - Test with missing PNG file (pending)
   - Test in very small terminal (80x24, 60x20 minimum) ✅
   - Test in large terminal (200x60) ✅
   - Verify Enter key transitions correctly ✅
   - Verify all other keys are ignored during welcome screen ✅

---

### User Story 2: Disk Space Information

#### Phase 6: Disk Space Module
**Goal**: Create utilities to query filesystem space

1. **Create `src/fs/disk_info.rs`**
   ```rust
   // Core structure:
   - pub struct DiskSpaceInfo {
       pub used_bytes: u64,
       pub total_bytes: u64,
       pub free_bytes: u64,
       pub drive_label: String, // "C:", "/dev/sda1", "/", etc.
   }
   
   - pub fn get_disk_space(path: &Path) -> Result<DiskSpaceInfo>
   - fn format_disk_space(info: &DiskSpaceInfo) -> String // "C: 45.2GB / 120GB (62% free)"
   - fn format_size(bytes: u64) -> String // Convert to KB/MB/GB/TB
   - fn get_drive_label(path: &Path) -> String // Extract drive letter or partition name
   ```

2. **Cross-Platform Implementation**
   - Use `fs2::statvfs()` for Linux/macOS
   - Use `fs2::available_space()` and `fs2::total_space()` for Windows
   - Detect drive letter on Windows: extract first component of path (C:, D:, etc.)
   - Detect mount point on Linux: use `path.canonicalize()` and match against /proc/mounts
   - macOS volume detection: similar to Linux but check /Volumes

3. **Error Handling**
   - Return `Result` with descriptive errors
   - Fallback to "N/A" if space cannot be determined
   - Handle network drives, virtual filesystems, special devices gracefully

#### Phase 7: Header Modification
**Goal**: Update header to show disk space instead of paths

1. **Modify `src/ui/mod.rs` - `render_header()` function**
   - Remove current path display logic (lines 141-142)
   - Query disk space for left panel: `get_disk_space(&app.left_panel.current_path)`
   - Query disk space for right panel: `get_disk_space(&app.right_panel.current_path)`
   - Format disk space strings: "C: 45.2GB / 120GB (62% free)"
   - Render formatted strings in header with cyan color
   - Handle errors: show "Space: N/A" if query fails

2. **Layout Adjustments**
   - Keep existing header height (3 lines)
   - Use same Block with borders structure
   - Center disk space info or align left/right for each panel
   - Ensure text fits within terminal width (truncate if needed)

#### Phase 8: Performance Optimization
**Goal**: Avoid lag from repeated filesystem queries

1. **Add Caching to AppState**
   - Add `pub disk_space_cache: HashMap<PathBuf, (DiskSpaceInfo, Instant)>` to AppState
   - Cache disk space results for 5 seconds
   - Query filesystem only when:
     - Panel navigates to new path
     - Cache entry expired (>5 seconds old)
     - Cache miss for current path

2. **Update Logic**
   - In `Panel::enter_dir()` and `Panel::go_up()`: invalidate cache entry for new path
   - In `render_header()`: check cache before calling `get_disk_space()`
   - Add `fn get_cached_disk_space(app: &mut AppState, path: &Path) -> DiskSpaceInfo` helper

#### Phase 9: Testing
**Goal**: Verify all scenarios from spec

1. **Unit Tests** (`tests/unit/disk_info_test.rs`)
   - Test `format_size()` with various byte values (KB, MB, GB, TB ranges)
   - Test `get_drive_label()` on Windows paths (C:\, D:\Users\, etc.)
   - Test `get_drive_label()` on Linux paths (/, /home, /mnt/data, etc.)
   - Test error handling for inaccessible paths
   - Mock filesystem calls for deterministic testing

2. **Manual Testing**
   - Test on Windows with multiple drives (C:, D:, E:)
   - Test navigation between drives updates header correctly
   - Test on Linux with multiple mount points (/, /home, /mnt)
   - Test with network drives (show "N/A" gracefully)
   - Test caching: verify no lag when navigating within same drive
   - Test cache expiration: verify updates after 5 seconds
   - Test very long drive labels (truncate properly)
   - Test terminal resize with disk space info visible

## Technical Decisions

### Architecture

**User Story 1 - Welcome Screen**:

**State Flag Approach**: Welcome screen uses a `show_welcome: bool` field in AppState instead of an enum-based screen system (which doesn't exist in this codebase). When `show_welcome` is true, the event loop renders the welcome screen instead of the normal dual-pane interface. This is the simplest integration pattern for this architecture.

**Rendering Strategy**: Use standard Ratatui widgets (Paragraph with colored Spans, Block). Parse ANSI color codes from ASCII art using shared `parse_ansi_line()` function. Center content using Layout calculations based on terminal dimensions. The welcome screen gets the full terminal area and renders independently.

**Asset Loading**: Use dynamic PNG→ASCII conversion. Load `assets/images/leekpc.png` synchronously with `image::open()`, calculate target dimensions based on terminal size, call `image_to_ascii()` from preview module. If file missing or unreadable, use hardcoded fallback ASCII art with borders.

**Version Source**: Use Rust's `env!("CARGO_PKG_VERSION")` macro at compile time. This reads version "0.2.0" (soon 0.3.0) from Cargo.toml. No runtime file reading needed.

**Event Flow**: The existing event_loop.rs module handles the main loop. We add a check early in the render cycle and in event handling to intercept when welcome screen is active. Filter KeyEventKind::Press only to prevent double key events.

---

**User Story 2 - Disk Space Information**:

**Filesystem Query Strategy**: Use fs2 crate (already a dependency) for cross-platform disk space queries. On Windows, use drive letters (C:, D:) extracted from path. On Linux/macOS, resolve canonical path and match against mount points.

**Caching Strategy**: Cache disk space results in AppState using HashMap<PathBuf, (DiskSpaceInfo, Instant)>. Cache lifetime: 5 seconds. This prevents lag from repeated filesystem queries while keeping data reasonably fresh. Invalidate cache when panel navigates to new path.

**Header Update**: Replace current redundant path display in header (already visible in panel borders) with compact disk space info. Format: "Drive: UsedGB / TotalGB (XX% free)". Use cyan color for consistency with existing header style.

**Unit Selection**: Automatically select appropriate units (KB, MB, GB, TB) based on byte magnitude. Thresholds: <1024 bytes = B, <1MB = KB, <1GB = MB, <1TB = GB, ≥1TB = TB. Always show 1 decimal place for readability.

**Error Handling**: Graceful fallback to "Space: N/A" for:
- Network drives / unmounted volumes
- Virtual filesystems (proc, sys, dev)
- Access denied / permission errors
- Unknown filesystem types

**Cross-Platform Differences**:
- Windows: Extract drive letter from path start (C:\, D:\Users\, etc.)
- Linux: Match against /proc/mounts for mount point detection
- macOS: Similar to Linux but also check /Volumes for external drives

### File Changes

| File | Change Type | Purpose | User Story |
|------|-------------|---------|------------|
| `src/app.rs` | Modified | Add `pub show_welcome: bool` field, init to true | US1 |
| `src/ui/mod.rs` | Modified | Export welcome_screen, add render_welcome(), refactor parse_ansi_line(), update render_header() for disk space | US1 + US2 |
| `src/ui/welcome_screen.rs` | New | Welcome screen rendering logic with PNG→ASCII conversion | US1 |
| `src/events/handler.rs` | Modified | Check show_welcome flag, handle Enter key to dismiss | US1 |
| `src/event_loop.rs` | Modified | Conditional rendering, KeyEventKind::Press filter | US1 |
| `src/preview/image_viewer.rs` | Modified | Optimize ASCII conversion thresholds and shading characters | US1 |
| `src/fs/disk_info.rs` | New | Disk space query utilities with cross-platform support | US2 |
| `src/fs/mod.rs` | Modified | Export disk_info module | US2 |
| `assets/images/leekpc.png` | Existing | Project image (768x768 PNG, used for dynamic ASCII) | US1 |
| `tests/unit/disk_info_test.rs` | New | Unit tests for disk space formatting and queries | US2 |

### Dependencies

**No new dependencies required**. All functionality uses existing crates:
- Ratatui: Already used for all UI rendering
- Crossterm: Already used for key event handling
- std::fs: For reading logo file
- env!() macro: Built into Rust
- fs2: Already a dependency for disk operations (used for disk space queries in US2)
- image: Already a dependency for image preview (used for PNG loading in US1)

## Success Criteria Mapping

### User Story 1 (Welcome Screen)

| Success Criterion | Implementation | Verification |
|-------------------|----------------|--------------|
| SC-001: 100% launch display | show_welcome initialized to true in AppState | Manual test: launches every time ✅ |
| SC-002: <1s transition | Direct state flag change on Enter, no async | Manual test: measured <1s ✅ |
| SC-003: Clear version display | env!("CARGO_PKG_VERSION") in centered cyan text | Visual inspection ✅ |
| SC-004: Graceful fallbacks | PNG load error → hardcoded ASCII borders fallback | Test with deleted leekpc.png (pending) |
| SC-005: 95% terminal compatibility | Tested on Windows PowerShell, Linux/macOS pending | Test matrix: 1/5+ terminals ⏳ |

### User Story 2 (Disk Space Information)

| Success Criterion | Implementation | Verification |
|-------------------|----------------|--------------|
| SC-006: ±1% accuracy | fs2 crate filesystem queries (kernel-level data) | Manual test: compare with df/dir output |
| SC-007: <50ms render | Cached results (5s lifetime), query only on navigation | Measure with Instant::now() timestamps |
| SC-008: 100% Windows drives | Extract drive letter from path.components()[0] | Test on C:\, D:\, E:\ |
| SC-009: 95% Linux/macOS mounts | Canonical path matching against /proc/mounts | Test on /, /home, /mnt/data |
| SC-010: <100ms update | Cache invalidation on path change, query async if needed | Measure navigation transition time |
| SC-011: 100% graceful fallback | Result → "Space: N/A" on error | Test network drives, virtual fs |

## Open Questions

**User Story 1 (Welcome Screen)**: None - all technical decisions confirmed with user. Implementation complete.

**User Story 2 (Disk Space Information)**: 
- Should cache be per-drive or per-path? (Recommendation: per-drive, more efficient)
- What format for very large drives (10TB+)? (Recommendation: show as TB with 1 decimal: "8.5TB / 12.0TB")
- Should we show filesystem type (NTFS, ext4, APFS)? (Recommendation: No, space info is sufficient)
- How to handle btrfs/ZFS with complex volume management? (Recommendation: Show space for current mount point)

---

### User Story 3: Detailed Column View

#### Phase 10: Data Model Enhancement
**Goal**: Add missing data to FileEntry model

1. **Modify `src/models/file_entry.rs`**
   - Add `pub created: Option<SystemTime>` field to FileEntry struct
   - Update constructor to initialize created field
   - Add `pub extension: Option<String>` field (extracted from name)
   
2. **Modify `src/fs/navigator.rs`**
   - In `read_dir()` function, query creation time from metadata
   - Windows: Use `metadata.created()` (available)
   - Unix: Use `metadata.created()` where available, fallback to `modified` if not
   - Extract file extension: split on last '.', handle special cases (.tar.gz, no extension, dotfiles)

3. **Error Handling**
   - If creation time unavailable: use modified time as fallback
   - If modified time also unavailable: use None
   - Document platform limitations in code comments

#### Phase 11: Column Layout Module
**Goal**: Calculate dynamic column widths and alignment

1. **Create `src/ui/column_layout.rs`**
   ```rust
   // Core structure:
   - pub struct ColumnLayout {
       pub icon_width: u16,     // Fixed: 2 chars
       pub mark_width: u16,     // Fixed: 1 char
       pub name_width: u16,     // Dynamic: remaining space
       pub ext_width: u16,      // Fixed or dynamic: 8 chars
       pub size_width: u16,     // Fixed: 10 chars (e.g. "1.23 GB")
       pub modified_width: u16, // Fixed: 16 chars (YYYY-MM-DD HH:MM)
       pub created_width: u16,  // Fixed: 16 chars
       pub perms_width: u16,    // Fixed: 10 chars (Windows) or 9 chars (Unix)
   }
   
   - pub fn calculate_layout(available_width: u16, entries: &[FileEntry]) -> ColumnLayout
   - fn calculate_name_width(total: u16, fixed_cols: u16, max_name_len: usize) -> u16
   - fn should_hide_column(available: u16, min_required: u16) -> bool
   ```

2. **Dynamic Width Logic**
   - Minimum terminal width: 80 columns
   - Priority order (hide columns if space limited):
     1. Icon, Mark, Name (always visible)
     2. Size, Modified (hide if <100 cols)
     3. Extension, Created, Permissions (hide if <120 cols)
   - Name column gets remaining space after fixed columns
   - Truncate with "..." if name exceeds available width

3. **Column Alignment**
   - Left-aligned: Icon, Mark, Name, Extension
   - Right-aligned: Size
   - Center-aligned: Modified, Created, Permissions

#### Phase 12: Formatting Utilities
**Goal**: Format file metadata for display

1. **Create `src/ui/formatters.rs`**
   ```rust
   // Core structure:
   - pub fn format_extension(name: &str) -> String // Extract and format extension
   - pub fn format_size(bytes: u64) -> String // "1.23 GB", "456 KB", etc.
   - pub fn format_date(time: Option<SystemTime>) -> String // "2025-01-26 14:30" or "N/A"
   - pub fn format_permissions(entry: &FileEntry) -> String // Platform-specific
   ```

2. **Extension Extraction**
   - Split on last '.' character
   - Handle multi-part extensions (.tar.gz → "tar.gz")
   - Dotfiles without extension (.gitignore → show as name, no ext)
   - Directories: show empty string or "DIR"

3. **Date Formatting**
   - ISO format: "YYYY-MM-DD HH:MM"
   - Use local timezone
   - Handle Option<SystemTime>: show "N/A" if None
   - Consider chrono crate for formatting (optional, can use manual formatting)

4. **Permissions Formatting**
   - Windows: "RHSA" format (Readonly, Hidden, System, Archive)
     - Example: "R--A" (readonly and archive), "RH--" (readonly and hidden)
     - Use 4 characters: R/-, H/-, S/-, A/-
   - Unix: "rwxr-xr-x" format (user, group, other)
     - Example: "rwxr-xr-x", "rw-r--r--", "drwxr-xr-x" (directory)
     - Prefix: d (directory), l (symlink), - (file)
     - Use 10 characters total

#### Phase 13: Panel Rendering Update
**Goal**: Replace simple list with columnar view

1. **Modify `src/ui/panel_widget.rs`**
   - Replace current rendering loop with column-based rendering
   - Use `ColumnLayout::calculate_layout()` to get widths
   - Render header row with column titles:
     - "Icon | Name | Ext | Size | Modified | Created | Perms"
   - Render each file entry as a row with formatted columns
   - Apply correct alignment for each column (left/right/center)
   - Highlight selected row with different background color

2. **Header Row**
   - Separate header row with border (─ characters)
   - Use bold or different color for column titles
   - Align titles to match data alignment

3. **Data Rows**
   - Use padding/spacing between columns (1-2 spaces)
   - Truncate long names with "..." if needed
   - Use icon from existing `get_icon()` function
   - Show mark indicator (space or '*' for marked files)
   - Apply selection highlighting

#### Phase 14: Testing
**Goal**: Verify all scenarios from spec

1. **Unit Tests** (`tests/unit/column_layout_test.rs`)
   - Test `calculate_layout()` with various terminal widths
   - Test column hiding logic (<80, <100, <120 cols)
   - Test `format_extension()` with various filenames
   - Test `format_date()` with valid and None SystemTime
   - Test `format_permissions()` on Windows and Unix

2. **Manual Testing**
   - Test with files of various name lengths
   - Test with files with multiple extensions (.tar.gz)
   - Test with dotfiles (.gitignore, .bashrc)
   - Test with very long filenames (>50 chars)
   - Test with directories vs regular files
   - Test on Windows: verify RHSA permissions
   - Test on Unix: verify rwx permissions
   - Test terminal resize: verify columns adapt
   - Test in minimum width (80 cols): verify essential columns visible
   - Test in wide terminal (200+ cols): verify all columns visible
