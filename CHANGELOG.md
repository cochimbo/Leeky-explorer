# Changelog

All notable changes to Leeky Explorer will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
