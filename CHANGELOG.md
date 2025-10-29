# [0.4.0] - 2025-10-29

### Changed
- Refactor: Modularización de archivos grandes (`dialogs.rs`, `event_loop.rs`, `extractor.rs`) en submódulos mantenibles.
- Refactor: Limpieza de código, eliminación de archivos de refactoring y warnings.

### Fixed
- El diálogo de extracción ahora captura correctamente el foco del teclado.
- Restaurado el progreso incremental en operaciones de copiar/mover locales (barra de progreso en tiempo real).
- Progreso de extracción de archivos sigue funcionando correctamente.

### Removed
- Eliminado soporte para FTP/FTPS (solo SFTP y SMB).

### Added
- Soporte básico de conexión SMB (solo conexión directa, sin discovery).
- Mejoras de robustez y testeo en operaciones de archivos.

### Download
- [Windows x86_64 (ZIP)](https://github.com/cochimbo/Leeky-explorer/releases/download/v0.4.0/leeky-explorer-v0.4.0-windows-x86_64.zip)
- [Linux x86_64 (tar.gz)](https://github.com/cochimbo/Leeky-explorer/releases/download/v0.4.0/leeky-explorer-v0.4.0-linux-x86_64.tar.gz)
- [Linux ARM64 (Raspberry Pi)](https://github.com/cochimbo/Leeky-explorer/releases/download/v0.4.0/leeky-explorer-v0.4.0-linux-arm64.tar.gz)

# Changelog

All notable changes to Leeky Explorer will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2025-10-27

### Added
- **Bookmark System (TASK-001 to TASK-011)**:
  - Save up to 50 favorite directories with custom names
  - Ctrl+D to quickly bookmark current directory
  - Ctrl+B to open bookmark manager dialog
  - Navigate, rename, and delete bookmarks
  - Sorted by most recently accessed
  - Persistent storage in config file
  - Visual indicators (⭐) for bookmarked directories

- **Text Editor (TASK-026 to TASK-028, TASK-032)**:
  - Built-in text editor with line numbers
  - Ctrl+S to save changes
  - Full cursor navigation and editing support
  - Undo/Redo functionality
  - Unsaved changes warning on exit
  - Read-only mode for protected files
  - External modification detection
  - Binary file rejection with warnings
  - Large file (>1MB) warning
  - UTF-8 support with emoji and unicode
  - 17 comprehensive integration tests

- **Recursive Search (TASK-029 to TASK-031, TASK-042)**:
  - Ctrl+F to search files across entire directory trees
  - Glob pattern support (`*.txt`, `test*`, `**/*.rs`)
  - Real-time results as search progresses
  - Shows file size, location, and files scanned count
  - Press Enter on result to navigate to file
  - Press Esc to cancel ongoing search
  - Background thread execution for non-blocking UI
  - Respects max depth (10 levels) and result limits (1000 files)
  - Auto-scroll in search results list
  - 15 integration tests for search functionality

- **Navigation History (TASK-014 to TASK-019)**:
  - Alt+Left to go back through visited directories
  - Alt+Right to go forward in history
  - Ctrl+H to view full navigation history dialog
  - Independent history per panel (up to 50 entries each)
  - Avoids consecutive duplicates
  - Cleans up invalid/deleted paths automatically
  - 14 integration tests for history tracking

- **Go To Path Dialog (TASK-020 to TASK-025)**:
  - Ctrl+G to quickly jump to any directory
  - Supports absolute and relative paths
  - Tilde (`~`) expansion for home directory
  - Environment variable expansion (`%USERPROFILE%`, `$HOME`)
  - Path validation and error messages
  - Adds visited paths to navigation history
  - Cross-platform path handling
  - 14 integration tests for path navigation

- **Disk Usage Indicator (TASK-012 to TASK-013)**:
  - Shows available/total disk space in footer
  - Visual progress bar with color coding
  - Warning colors for low disk space (<10% = red, <20% = yellow)
  - Per-panel disk info display
  - Drive label on Windows (C:, D:, etc.)

- **Auto-Refresh**:
  - Automatically detects external directory changes every 5 seconds
  - Polling-based implementation with minimal performance impact
  - Preserves cursor position when file still exists after refresh
  - Clears selection state when active panel refreshes
  - Directory timestamp tracking for change detection

### Changed
- **Search Dialog (TASK-040)**: Fixed auto-scroll when navigating beyond visible items
- **Footer Layout**: Updated to show Ctrl+B (Bookmarks) and Ctrl+F (Search) keybindings
- **Config File**: Extended to store bookmarks, history, and editor state

### Technical
- Added `SelectionState` model for independent panel selection tracking
- Added `BookmarkManager` with LRU-style access tracking
- Added `NavigationHistory` with bidirectional navigation
- Added `TextEditor` widget with tui-textarea integration
- Added `RecursiveSearch` with background thread execution
- Added `GotoPathDialog` with path validation and expansion
- Refactored `Panel` to track directory modification timestamps
- Total test count increased to **199 tests** (95 unit + 104 integration)
- Comprehensive integration tests for all Phase 6 features
- All tests passing with zero failures

### Performance
- Recursive search uses background threads to avoid blocking UI
- Auto-refresh checks only every 5 seconds for minimal overhead
- Bookmark access tracking uses efficient timestamp comparison
- Navigation history limited to 50 entries per panel

## [0.3.0] - 2025-10-26

### Added
- **Theme System (US5)**: Complete theming support with 8 built-in themes
  - Classic (default): Traditional blue/gray look
  - Light: High-contrast light theme
  - Dark: High-contrast dark theme
  - High Contrast: Maximum visibility
  - Nord: Cool northern color palette
  - Dracula: Popular dark theme
  - Solarized Dark: Low-contrast dark
  - Solarized Light: Low-contrast light
- **Theme Selector Dialog**: Interactive theme picker with F12 hotkey
  - Live preview box showing all UI elements
  - Cursor starts on currently active theme
  - Visual checkmark (✓) indicator for active theme
  - Hot-swap themes without restart
- **Theme Persistence**: Selected theme saved to config.json
- **ARM Platform Support**: Added Raspberry Pi builds
  - Linux ARM64 (aarch64) for Raspberry Pi 3/4/5 (64-bit OS)
  - Linux ARMv7 (32-bit) for Raspberry Pi 2/3 (32-bit OS)
- **Enhanced Search UX**:
  - Shift+F3 to explicitly clear search pattern and filter
  - F3 now toggles search mode (press again to clear and exit)
  - Search pattern visual clears when changing directories

### Changed
- **Theme Selector Keybinding**: Changed from F11 to F12 (F11 conflicts with PowerShell fullscreen)
- **Search Behavior**: Pattern and filter automatically clear when navigating to different directories
- Footer updated to show new keybindings: F12 for Theme, Shift+F3 for Clear Search

### Fixed
- **Search Text Truncation**: Fixed item names being cut off during search display
  - List area now properly reserves space for search bar
  - No more overlap between list items and search bar
- **Search Pattern Persistence**: Pattern now clears when entering/exiting directories
- **Search Visual State**: Search bar visibility correctly syncs with search mode

### Technical
- Refactored all UI components to use theme system:
  - Panel widgets (borders, backgrounds, highlights)
  - Footer with themed keybinding display
  - All dialog types (confirm, input, progress, error, extract, password, collision, compress)
  - Welcome screen with themed logo
  - Drive selector dialog
  - Theme selector dialog
  - Preview modals (text and image)
- Fixed all Clippy warnings for cleaner code:
  - Collapsed nested if statements using let-chains
  - Reduced function parameters with ScrollOffsets struct
  - Optimized vector initialization
  - Removed needless borrows and redundant branches
- Updated test fixtures for theme support
- GitHub Actions workflow now builds for 6 platforms

## [0.2.0] - 2025-10-20

### Added
- **Archive Operations (US4)**:
  - Extract archives with F9 (ZIP, TAR, GZ, BZ2, XZ, 7Z, RAR)
  - Compress files/folders with Shift+F9
  - Password-protected archive support
  - Progress dialogs for long operations
  - Collision detection with overwrite/skip/rename options
  - Automatic format detection from file extension
- **Drive Selector (US4)**: Press F10 to quickly switch between drives (Windows)
- **Multi-format Support**:
  - ZIP (create/extract)
  - TAR formats: .tar, .tar.gz, .tar.bz2, .tar.xz
  - Compressed formats: .gz, .bz2, .xz
  - Read-only: .7z, .rar
- **Compression Options Dialog**:
  - Choose compression level (Store, Fast, Default, Best)
  - Optional password protection
  - Format selection for directories (ZIP or TAR.GZ)

### Changed
- Enhanced error handling for archive operations
- Improved collision resolution workflow
- Better progress tracking for archive extraction

### Fixed
- Archive path sanitization to prevent directory traversal
- Proper handling of nested archives
- Memory-efficient streaming for large archives

## [0.1.0] - 2025-10-15

### Added
- **Dual-Pane Navigation**: Independent left and right panels with Tab switching
- **File Operations**:
  - Copy (F5) with progress tracking
  - Move (F6) with progress tracking
  - Delete (F8) with confirmation
  - Create directory (F7)
  - Rename (F2) name only, (Shift+F2) with extension
- **Multi-Selection System (US3)**:
  - Space to mark/unmark individual files
  - Ctrl+A to select all
  - Visual indicators for marked items
  - Operations work on marked items or current item
- **Preview System (US3)**:
  - F4 to preview files
  - Text file preview with syntax awareness
  - Image preview (PNG, JPG, GIF, BMP, WebP, ICO)
  - Encoding detection (UTF-8, UTF-16, Latin-1)
  - Scroll support (arrows, PgUp/PgDn, Home/End)
  - Q or Esc to close preview
- **Search and Filter (US3)**:
  - F3 to activate search mode
  - Real-time filtering as you type
  - Case-insensitive search
  - Glob pattern support (*.txt, ?.log)
  - Enter to finalize, Esc to cancel
- **Navigation Enhancements**:
  - Vim-like keybindings (j/k for up/down)
  - Quick jump: Press letter to jump to files starting with that letter
  - Page navigation (PgUp/PgDn moves 5 items)
  - Home/End to jump to first/last item
  - Backspace to go up one directory
- **Display Features**:
  - File size formatting (B, KB, MB, GB)
  - File type icons (directories, archives, executables, etc.)
  - Sortable columns: Name, Extension, Size, Modified, Created, Permissions
  - Disk space info in header with usage bar
  - Selection counter in panel headers
- **Configuration**:
  - Persistent state (panel paths, active panel)
  - Config saved to `~/.config/leeky-explorer/config.json`
  - Welcome screen on first launch

### Technical
- Built with Rust 🦀
- TUI framework: Ratatui 0.29
- Terminal handling: Crossterm 0.29
- Async runtime: Tokio 1.35
- Archive handling: zip, tar, flate2, bzip2, xz2, sevenz-rust, unrar
- Image processing: image, ratatui-image
- Cross-platform: Windows, macOS (Intel & Apple Silicon), Linux (x86_64, ARM64, ARMv7)

## Release Information

### Platform Support
- **Windows**: x86_64
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon M1/M2)
- **Linux**: x86_64, aarch64 (ARM 64-bit), armv7 (ARM 32-bit)

### Installation
Download the appropriate binary for your platform from the [Releases](https://github.com/cochimbo/Leeky-explorer/releases) page.

### System Requirements
- Terminal with UTF-8 support
- Minimum terminal size: 80x24
- Recommended: 120x30 or larger for optimal experience

---

[0.3.0]: https://github.com/cochimbo/Leeky-explorer/releases/tag/v0.3.0
[0.2.0]: https://github.com/cochimbo/Leeky-explorer/releases/tag/v0.2.0
[0.1.0]: https://github.com/cochimbo/Leeky-explorer/releases/tag/v0.1.0
