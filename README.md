# 🗂️ Leeky Explorer

![Leeky Explorer Logo](assets/images/leekpc.png)

A fast, dual-pane terminal file manager built with Rust and Ratatui.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/cochimbo/Leeky-explorer)](https://github.com/cochimbo/Leeky-explorer/releases)

## ✨ Features

- **Dual-Pane Navigation**: Classic two-panel interface for efficient file management
- **Fast & Responsive**: Built with Rust for maximum performance
 **🎨 Theme System**: 8 built-in themes with live preview (`Ctrl+W`)
 **📚 Bookmarks**: Save and manage up to 50 favorite directories (`Ctrl+Shift+D` to add, `Ctrl+B` to manage)
 **🔍 Text Editor**: Built-in editor with syntax highlighting, undo/redo, and UTF-8/emoji support (`Ctrl+E` to open, `Ctrl+S` to save)
 **🔎 Recursive Search**: Search files with glob patterns and real-time results (`Ctrl+F`)
 **⏱️ Navigation History**: Per-panel history with dialogs and keyboard navigation (`Alt+Left/Right`, `Ctrl+H`)
 **↗️ Go To Path**: Jump to any directory, with tilde/env expansion (`Ctrl+G`)
 **Drive Selector**: Quick drive switching on Windows (`Ctrl+D`)
- **Archive Support**: Extract and create ZIP, TAR.GZ, TAR.BZ2, TAR.XZ, and 7Z archives
- **Password Protection**: Encrypt/decrypt ZIP archives with passwords
- **File Preview**: View text files and images directly in the terminal (ASCII art)
- **Multi-Selection**: Select multiple files for batch operations
- **Smart Search**: Quick filter with glob pattern support (`*.txt`, `test*`, etc.)
- **Drive Selector**: Quick drive switching on Windows (F10)
- **Progress Tracking**: Real-time progress bars for long operations
- **Operation Cancellation**: Press ESC to cancel ongoing operations
- **Safe Operations**: Collision detection, disk space validation, permission checks
- **Session Persistence**: Remembers panel positions, active panel, and theme between sessions
- **Cross-Platform**: Full support for Windows, Linux, macOS, and Raspberry Pi (ARM)

## 📦 Installation

### Pre-built Binaries (Recommended)


Download the latest release for your platform:

- [Windows x86_64 (ZIP)](https://github.com/cochimbo/Leeky-explorer/releases/download/v0.4.0/leeky-explorer-v0.4.0-windows-x86_64.zip)
- [Linux x86_64 (tar.gz)](https://github.com/cochimbo/Leeky-explorer/releases/download/v0.4.0/leeky-explorer-v0.4.0-linux-x86_64.tar.gz)
- [Linux ARM64 (Raspberry Pi)](https://github.com/cochimbo/Leeky-explorer/releases/download/v0.4.0/leeky-explorer-v0.4.0-linux-arm64.tar.gz)

Or visit [GitHub Releases](https://github.com/cochimbo/Leeky-explorer/releases/latest).


#### 🐧 Linux (x86_64)

## ✨ Features

- **Dual-Pane Navigation**: Classic two-panel interface for efficient file management
- **Fast & Responsive**: Built with Rust for maximum performance
- **🎨 Theme System**: 8 built-in themes with live preview (F12)
- **📚 Bookmarks**: Save and manage up to 50 favorite directories (Ctrl+D/Ctrl+B)
- **🔍 Text Editor**: Built-in editor with syntax highlighting, undo/redo, and UTF-8/emoji support (F4)
- **🔎 Recursive Search**: Search files with glob patterns and real-time results (Ctrl+F)
- **⏱️ Navigation History**: Per-panel history with dialogs and keyboard navigation (Alt+Left/Right, Ctrl+H)
- **↗️ Go To Path**: Jump to any directory, with tilde/env expansion (Ctrl+G)
- **🔄 Auto-Refresh**: Detects external directory changes every 5 seconds
- **Archive Support**: Extract/create ZIP, TAR.GZ, TAR.BZ2, TAR.XZ, 7Z (with password for ZIP)
- **Password Protection**: Encrypt/decrypt ZIP archives with passwords
- **File Preview**: View text files and images (ASCII art) in terminal
- **Multi-Selection**: Select multiple files for batch operations
- **Smart Search**: Quick filter with glob pattern support (`*.txt`, `test*`, etc.)
- **Drive Selector**: Quick drive switching on Windows (F10)
- **Progress Tracking**: Real-time progress bars for copy, move, extract, and delete
- **Operation Cancellation**: Press ESC to cancel ongoing operations
- **Safe Operations**: Collision detection, disk space validation, permission checks
- **Session Persistence**: Remembers panel positions, active panel, and theme between sessions
- **Remote Connections**:
   - **SFTP**: Secure file transfer (fully supported)
   - **SMB (experimental)**: Connect to Windows shares (direct connection)
   - **FTP/FTPS**: [Removed in 0.4.0]
- **Robustness**: Improved error handling, progress/cancel in all file operations
- **Cross-Platform**: Windows, Linux, macOS, Raspberry Pi (ARM)

> **Note:** FTP/FTPS support has been removed in 0.4.0. Only SFTP and SMB are supported for remote connections.
```bash
wget https://github.com/cochimbo/Leeky-explorer/releases/download/v0.3.0/leeky-explorer-v0.4.0-linux-armv7.tar.gz
tar -xzf leeky-explorer-v0.3.0-linux-armv7.tar.gz
leeky-explorer
```

#### 🪟 Windows (x86_64)
```powershell
# Download from GitHub Releases page or use PowerShell
Invoke-WebRequest -Uri "https://github.com/cochimbo/Leeky-explorer/releases/download/v0.4.0/leeky-explorer-v0.3.0-windows-x86_64.zip" -OutFile "leeky-explorer.zip"

# Extract
Expand-Archive leeky-explorer.zip -DestinationPath .

# Verify checksum (optional)
Get-FileHash leeky-explorer.exe -Algorithm SHA256

# run directly
.\leeky-explorer.exe
```

#### 🍎 macOS (Intel)
```bash
# Download and extract
curl -L -o leeky-explorer.tar.gz https://github.com/cochimbo/Leeky-explorer/releases/download/v0.4.0/leeky-explorer-v0.3.0-macos-x86_64.tar.gz
tar -xzf leeky-explorer.tar.gz

# Verify checksum (optional)
shasum -a 256 -c leeky-explorer-v0.3.0-macos-x86_64.tar.gz.sha256

# Run (may need to allow in System Preferences > Security)
leeky-explorer
```

#### 🍎 macOS (M1/M2/M3/M4 Apple Silicon)
```bash
# Download and extract
curl -L -o leeky-explorer.tar.gz https://github.com/cochimbo/Leeky-explorer/releases/download/v0.4.0/leeky-explorer-v0.3.0-macos-arm64.tar.gz
tar -xzf leeky-explorer.tar.gz

# Run
leeky-explorer
```

### Supported Platforms

| Platform | Architecture | Status |
|----------|-------------|--------|
| Linux | x86_64 | ✅ Fully Supported |
| Linux | ARM64 (aarch64) | ✅ Fully Supported |
| Linux | ARMv7 (32-bit) | ✅ Fully Supported |
| Windows | x86_64 | ✅ Fully Supported |
| macOS | Intel (x86_64) | ✅ Fully Supported |
| macOS | Apple Silicon (ARM64) | ✅ Fully Supported |

### From Source

```bash
# Clone the repository
git clone https://github.com/cochimbo/Leeky-explorer.git
cd Leeky-explorer

# Build and install
cargo install --path .

# Run
leeky-explorer
```
leeky-explorer
```

### Build Locally

**Using Make** (Linux/macOS):
```bash
# Build release binary
make build-release

# Or build for all platforms
make build-all

# Create distribution packages
make package
```

**Using PowerShell** (Windows):
```powershell
# Build release binary
.\build.ps1 release

# Run tests and create package
.\build.ps1 all
```

### Prerequisites

- Rust 1.70 or higher
- A terminal with Unicode and color support

## 🎮 Key Bindings

### Navigation

| Key | Action |
|-----|--------|
| `↑` / `↓` | Move cursor up/down |
| `Enter` | Enter directory / Preview file |
| `Backspace` | Go to parent directory |
| `Tab` | Switch active panel |
| `Home` | Jump to first item |
| `End` | Jump to last item |
| `Page Up` / `Page Down` | Scroll by page |
| `Ctrl+Q` | Quit application |

### File Operations

| Key | Action |
|-----|--------|
| `Ctrl+C` | Copy file(s) |
| `Ctrl+X` | Move file(s) |
| `Delete` | Delete file(s) |
| `Ctrl+Shift+N` | New folder |
| `Ctrl+R` | Rename file/directory |
| `Space` | Mark/Unmark file |
| `Ctrl+A` | Select all |
| `Esc` | Cancel selection / operation |

### Archive Operations

| Key | Action |
|-----|--------|
| `Ctrl+Shift+E` | Extract archive |
| `Ctrl+Shift+A` | Compress file(s) |

**Compression options:**
- Formats: ZIP, TAR.GZ, TAR.BZ2, TAR.XZ
- Levels: Fast, Normal, Maximum
- Password protection (ZIP only)

### Search & Filter



| Key | Action |
|-----|--------|
| `Ctrl+F` | Recursive search in current directory |
| Any character | Filter files by pattern (in filter mode) |
| `Esc` | Clear filter / cancel search |

**Recursive search (Ctrl+F):**
- Search through entire directory tree
- Supports glob patterns (`*.txt`, `test*`, `**/*.rs`)
- Real-time results
- Enter to go to found file
- Esc to cancel search



### System & Customization

| Key | Action |
|-----|--------|
| `Ctrl+E` | Open integrated text editor |
| `Ctrl+S` | Save in text editor |
| `Ctrl+D` | Drive selector (Windows) |
| `Ctrl+Shift+D` | Add directory to bookmarks |
| `Ctrl+B` | Bookmark manager |
| `Ctrl+G` | Go to path |
| `Ctrl+H` | Navigation history |
| `Alt+Left` / `Alt+Right` | Back / Forward in history |
| `Ctrl+W` | Theme selector |

**Built-in themes:**
- 8 themes: Classic, Light, Dark, High Contrast, Nord, Dracula, Solarized Dark, Solarized Light

**Bookmarks:**
- Up to 50 favorite directories
- Quick and persistent access



### Preview & Editor Mode

| Key | Action |
|-----|--------|
| `Ctrl+E` | Open text editor (on text file) |
| `Ctrl+S` | Save changes in editor |
| `Esc` / `q` | Close preview/editor |
| `↑` / `↓` | Scroll up/down |
| `Page Up` / `Page Down` | Scroll by page |
| `Home` / `End` | Jump to start/end |

**Supported formats:**
- **Text**: .txt, .md, .rs, .json, .toml, .yml, README, LICENSE, etc.
- **Images**: .png, .jpg, .jpeg, .gif, .bmp (ASCII art)
- **Archives**: shows format and compression info



### Other

| Key | Action |
|-----|--------|
| `Ctrl+Q` | Quit application |
| `Esc` | Cancel operation / close dialog |

## 🖼️ Screenshot

```
┌─ /home/user/projects ──────────────────┐┌─ /home/user/documents ─────────────────┐
│  📁 ..                                  ││  📁 ..                                  │
│  📁 leeky-explorer/                     ││  📄 notes.txt                    2.4 KB │
│▸ 📁 rust-project/                       ││  📄 report.pdf                 145.2 KB │
│  📁 website/                             ││  📁 images/                             │
│  📄 README.md                     5.2 KB ││  📄 todo.md                      1.8 KB │
│  📄 Cargo.toml                    0.8 KB ││                                         │
│                                          ││                                         │
│                                          ││                                         │
│                                          ││                                         │
└──────────────────────────────────────────┘└──────────────────────────────────────────┘
 F1 :Help  ↑↓ :Nav  Tab :Switch  Enter :Open/Preview  Space :Select  Ctrl+Q :Quit
```

## 🚀 Usage Examples

### Basic Navigation

1. **Switch panels**: Press `Tab`
2. **Enter directory**: Press `Enter` on a folder
3. **Go up**: Press `Backspace`
4. **Jump to first/last item**: Use `Home` / `End`

### Copy/Move Files

1. Navigate to the source file in one panel
2. Navigate to the destination directory in the other panel
3. Press `Ctrl+C` to copy or `Ctrl+X` to move
4. Confirm in the dialog

### Multi-Select

1. Mark files with `Space`
2. Or press `Ctrl+A` to select all
3. Press `Ctrl+C` (copy), `Ctrl+X` (move), or `Delete` (delete)
4. All marked files will be processed

### Search/Filter

**Local filter:**
1. Type any character to filter files in the panel
2. Results update in real time
3. Press `Esc` to clear the filter

**Recursive search (Ctrl+F):**
1. Press `Ctrl+F` to start recursive search
2. Type the search pattern (e.g., `*.txt` or `test*`)
3. Real-time results with location and size
4. Press `Enter` on a result to navigate
5. Press `Esc` to close the search

### Bookmarks

**Add bookmark:**
1. Navigate to the directory you want to save
2. Press `Ctrl+Shift+D`
3. Enter a name
4. Bookmark saved!

**Use bookmarks:**
1. Press `Ctrl+B` to open the bookmark manager
2. Use arrow keys to select
3. Press `Enter` to navigate
4. Press `d` to delete, `r` to rename

### Navigation History

1. Navigate normally
2. Press `Alt+Left` to go back
3. Press `Alt+Right` to go forward
4. Press `Ctrl+H` to see the full history

### Go To Path

1. Press `Ctrl+G`
2. Type the absolute or relative path
3. Supports `~` for home and environment variables
4. Press `Enter` to navigate

### Edit Text Files

1. Navigate to a text file (`.txt`, `.md`, `.rs`, etc.)
2. Press `Ctrl+E`
3. Edit with the keyboard
4. Press `Ctrl+S` to save
5. Press `Esc` to exit (warns if there are unsaved changes)

### Compress Files

1. Select files (use `Space` for multiple)
2. Press `Ctrl+Shift+A`
3. Choose format and compression level
4. Optionally add password (ZIP)
5. Confirm to create the archive

### Extract Archives

1. Navigate to a compressed file (`.zip`, `.tar.gz`, etc.)
2. Press `Ctrl+Shift+E`
3. Choose extraction option
4. Contents will appear in the current panel

## 🛠️ Architecture

### Project Structure

```
leeky-explorer/
├── src/
│   ├── main.rs              # Entry point
│   ├── app.rs               # Application state
│   ├── event_loop.rs        # Main event loop & operation execution
│   ├── models/              # Data structures
│   │   ├── panel.rs         # Panel state & navigation
│   │   ├── file_entry.rs    # File metadata
│   │   ├── operation.rs     # Background operations
│   │   └── selection.rs     # Multi-select state
│   ├── ui/                  # User interface
│   │   ├── panel_widget.rs  # Dual-pane rendering
│   │   ├── dialog.rs        # Dialogs (confirm, input, progress)
│   │   ├── preview_modal.rs # File preview window
│   │   ├── theme.rs         # Theme system (8 built-in themes)
│   │   └── theme_selector.rs # Theme picker with live preview
│   ├── events/              # Event handling
│   │   ├── handler.rs       # Key event dispatcher
│   │   └── keybindings.rs   # Key mappings
│   ├── fs/                  # Filesystem operations
│   │   ├── navigator.rs     # Directory reading
│   │   ├── operations.rs    # Copy/move/delete
│   │   └── search.rs        # File filtering
│   ├── archive/             # Archive support
│   │   ├── extractor.rs     # Extract archives
│   │   ├── compressor.rs    # Create archives
│   │   ├── formats.rs       # Format detection
│   │   └── password.rs      # Password handling
│   ├── preview/             # File preview
│   │   ├── text_viewer.rs   # Text files
│   │   ├── image_viewer.rs  # Images (ASCII art)
│   │   └── encoding.rs      # UTF-8/Latin1 detection
│   └── config/              # Configuration
│       └── state.rs         # Session persistence
└── tests/                   # Integration tests
```

### Key Technologies

- **Ratatui**: Terminal UI framework
- **Crossterm**: Cross-platform terminal manipulation
- **Tokio**: Async runtime for background operations
- **Zip/Tar**: Archive handling
- **Image**: Image decoding for ASCII conversion

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_copy_file

# Run clippy for code quality
cargo clippy --all-targets --all-features -- -D warnings
```

**Test Coverage**: 83 tests across 8 test suites
- Unit tests for core functionality
- Integration tests for file operations
- Compression/extraction tests
- UI state management tests
- Configuration persistence tests
- Search and filter tests

## 🐛 Error Handling

The application includes comprehensive error handling:

- **File Not Found (T951)**: Validates files exist before operations
- **Permission Denied (T952)**: Detects and reports permission errors
- **Disk Space (T953)**: Checks available space before extraction/copy
- **Collisions (T954)**: Prompts user before overwriting files
- **Operation Cancellation (T955)**: Press ESC to cancel long operations
- **Large Operations (T956)**: Warns for operations >1GB or >1000 files

## 📝 Configuration

Configuration is automatically saved in:
- **Linux/macOS**: `~/.config/leeky-explorer/state.json`
- **Windows**: `%APPDATA%\leeky-explorer\state.json`

Stored data:
- Last panel paths (left/right)
- Active panel selection
- Selected theme name

The theme preference persists between sessions, and you can switch themes anytime with `F12`.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- Built with [Ratatui](https://github.com/ratatui-org/ratatui)
- Inspired by classic file managers like Midnight Commander and Norton Commander

## 📧 Contact

- Author: cochimbo
- Repository: [github.com/cochimbo/Leeky-explorer](https://github.com/cochimbo/Leeky-explorer)

---

Made with ❤️ and 🦀 Rust
