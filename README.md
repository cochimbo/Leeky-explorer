# 🗂️ Leeky Explorer

A fast, dual-pane terminal file manager built with Rust and Ratatui.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## ✨ Features

- **Dual-Pane Navigation**: Classic two-panel interface for efficient file management
- **Fast & Responsive**: Built with Rust for maximum performance
- **Archive Support**: Extract and create ZIP, TAR.GZ, TAR.BZ2, TAR.XZ, and 7Z archives
- **Password Protection**: Encrypt/decrypt ZIP archives with passwords
- **File Preview**: View text files and images directly in the terminal (ASCII art)
- **Multi-Selection**: Select multiple files for batch operations
- **Smart Search**: Quick filter with glob pattern support (`*.txt`, `test*`, etc.)
- **Progress Tracking**: Real-time progress bars for long operations
- **Operation Cancellation**: Press ESC to cancel ongoing operations
- **Safe Operations**: Collision detection, disk space validation, permission checks
- **Session Persistence**: Remembers panel positions and active panel between sessions

## 📦 Installation

### Pre-built Binaries (Recommended)

Download the latest release for your platform from [GitHub Releases](https://github.com/cochimbo/Leeky-explorer/releases):

**Linux (x86_64)**:
```bash
# Download and extract
wget https://github.com/cochimbo/Leeky-explorer/releases/download/v0.1.0/leeky-explorer-v0.1.0-linux-x86_64.tar.gz
tar -xzf leeky-explorer-v0.1.0-linux-x86_64.tar.gz

# Verify checksum (optional)
sha256sum -c leeky-explorer-v0.1.0-linux-x86_64.tar.gz.sha256

# Move to PATH
sudo mv leeky-explorer /usr/local/bin/

# Run
leeky-explorer
```

**Windows (x86_64)**:
```powershell
# Download from GitHub Releases page or use PowerShell
Invoke-WebRequest -Uri "https://github.com/cochimbo/Leeky-explorer/releases/download/v0.1.0/leeky-explorer-v0.1.0-windows-x86_64.zip" -OutFile "leeky-explorer.zip"

# Extract
Expand-Archive leeky-explorer.zip -DestinationPath .

# Verify checksum (optional)
Get-FileHash leeky-explorer.exe -Algorithm SHA256

# Add to PATH or run directly
.\leeky-explorer.exe
```

**macOS (Intel)**:
```bash
# Download and extract
curl -L -o leeky-explorer.tar.gz https://github.com/cochimbo/Leeky-explorer/releases/download/v0.1.0/leeky-explorer-v0.1.0-macos-x86_64.tar.gz
tar -xzf leeky-explorer.tar.gz

# Verify checksum (optional)
shasum -a 256 -c leeky-explorer-v0.1.0-macos-x86_64.tar.gz.sha256

# Move to PATH
sudo mv leeky-explorer /usr/local/bin/

# Run (may need to allow in System Preferences > Security)
leeky-explorer
```

**macOS (M1/M2 ARM)**:
```bash
# Download and extract
curl -L -o leeky-explorer.tar.gz https://github.com/cochimbo/Leeky-explorer/releases/download/v0.1.0/leeky-explorer-v0.1.0-macos-arm64.tar.gz
tar -xzf leeky-explorer.tar.gz

# Move to PATH
sudo mv leeky-explorer /usr/local/bin/

# Run
leeky-explorer
```

### From Source

```bash
# Clone the repository
git clone https://gitlab.com/cochimbo/Leeky-explorer.git
cd Leeky-explorer

# Build and install
cargo install --path .

# Run
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
| `↑` / `k` | Move cursor up |
| `↓` / `j` | Move cursor down |
| `←` / `h` | Switch to left panel |
| `→` / `l` | Switch to right panel |
| `Enter` | Enter directory / Open file preview |
| `Backspace` | Go to parent directory |
| `Tab` | Switch active panel |
| `Home` | Jump to first item |
| `End` | Jump to last item |
| `Page Up` | Scroll up one page |
| `Page Down` | Scroll down one page |

### File Operations

| Key | Action |
|-----|--------|
| `F5` | Copy file(s) to opposite panel |
| `F6` | Move/Rename file(s) to opposite panel |
| `F7` | Create new folder |
| `F8` / `Delete` | Delete selected file(s) |
| `Ctrl+A` | Select/Deselect all files |
| `Insert` / `Space` | Mark/Unmark file for batch operations |

### Archive Operations

| Key | Action |
|-----|--------|
| `F9` | Extract archive to current panel |
| `F10` | Compress selected files |

**Compression Options**:
- Format: ZIP, TAR.GZ, TAR.BZ2, TAR.XZ
- Compression Level: Fast, Normal, Maximum
- Password Protection (ZIP only)

### Search & Filter

| Key | Action |
|-----|--------|
| `/` | Start search/filter mode |
| `Esc` | Clear filter |
| Any character | Filter files by pattern |

**Search supports**:
- Glob patterns: `*.rs`, `test*`, `file?.txt`
- Case-insensitive matching
- Real-time filtering

### Preview Mode

| Key | Action |
|-----|--------|
| `Enter` | Open file preview (on file) |
| `Esc` | Close preview |
| `↑` / `k` | Scroll up |
| `↓` / `j` | Scroll down |
| `Page Up` | Scroll up one page |
| `Page Down` | Scroll down one page |
| `Home` | Jump to start |
| `End` | Jump to end |

**Supported Formats**:
- **Text**: .txt, .md, .rs, .json, .toml, .yml, README, LICENSE, etc.
- **Images**: .png, .jpg, .jpeg, .gif, .bmp (rendered as ASCII art)
- **Archives**: Shows format and compression info

### Other

| Key | Action |
|-----|--------|
| `F1` / `?` | Help screen (keybindings) |
| `Ctrl+C` / `q` | Quit application |
| `Esc` | Cancel operation / Close dialog |

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
 F1 Help  F5 Copy  F6 Move  F7 MkDir  F8 Delete  F9 Extract  F10 Compress  / Search  q Quit
```

## 🚀 Usage Examples

### Basic Navigation

1. **Switch between panels**: Press `Tab` or `→` / `←`
2. **Enter directory**: Press `Enter` on a folder
3. **Go up**: Press `Backspace`
4. **Jump to location**: Use `Home` / `End` for first/last item

### Copy Files

1. Navigate to source file in one panel
2. Navigate to destination directory in other panel
3. Press `F5` to copy
4. Confirm in dialog

### Multi-Select Operations

1. Mark files with `Insert` or `Space`
2. Or press `Ctrl+A` to select all
3. Press `F5` (copy), `F6` (move), or `F8` (delete)
4. All marked files will be processed

### Search/Filter

1. Press `/` to start search
2. Type pattern (e.g., `*.rs` for Rust files)
3. Results update in real-time
4. Press `Esc` to clear filter

### Create Archive

1. Select files to compress (use `Insert` for multiple)
2. Press `F10`
3. Choose format (ZIP, TAR.GZ, etc.)
4. Choose compression level
5. Optionally add password (ZIP only)
6. Confirm to create

### Extract Archive

1. Navigate to `.zip`, `.tar.gz`, `.7z`, etc.
2. Press `F9`
3. Choose extraction option:
   - Extract here
   - Extract to new folder
4. Archive contents appear in current panel

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
│   │   └── preview_modal.rs # File preview window
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

# Generate code coverage
cargo tarpaulin --out Html
```

**Test Coverage**: 58 tests across 8 test suites
- Unit tests for core functionality
- Integration tests for file operations
- Compression/extraction tests
- UI state management tests

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
