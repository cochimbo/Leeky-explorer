# Release Process for Leeky Explorer

This document describes how to create a new release with pre-built binaries for multiple platforms.

## Prerequisites

- GitHub repository with Actions enabled
- Git configured with push access
- Version number decided (semantic versioning: MAJOR.MINOR.PATCH)

## Supported Platforms

The GitHub Actions workflow automatically builds for 6 platforms:

| Platform | Architecture | Target | Status |
|----------|-------------|--------|--------|
| Linux | x86_64 | `x86_64-unknown-linux-gnu` | ✅ |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` | ✅ |
| Linux | ARMv7 | `armv7-unknown-linux-gnueabihf` | ✅ |
| Windows | x86_64 | `x86_64-pc-windows-msvc` | ✅ |
| macOS | Intel | `x86_64-apple-darwin` | ✅ |
| macOS | Apple Silicon | `aarch64-apple-darwin` | ✅ |

## Release Checklist

### 1. Pre-Release Testing

```bash
# Run full test suite
cargo test --all-targets

# Run clippy
cargo clippy -- -D warnings

# Build release locally to verify
cargo build --release

# Test the binary
./target/release/leeky-explorer
```

### 2. Update Version

Update version in `Cargo.toml`:
```toml
[package]
name = "leeky-explorer"
version = "0.3.0"  # ← Update this
```

Update version references in README.md and CHANGELOG.md.

### 3. Update CHANGELOG

Update `CHANGELOG.md` with the new version:

```markdown
# Changelog

## [0.3.0] - 2025-10-26

### Added
- Complete theme system with 8 built-in themes
- Theme selector with live preview (F12)
- Enhanced search UX with Shift+F3 clear
- ARM platform support (Raspberry Pi)
- Search bug fixes (truncation, persistence)

### Changed
- Theme keybinding from F11 to F12
- Search pattern clears when changing directories

### Fixed
- Search text truncation issue
- Search pattern persistence bug
```

### 4. Commit Changes

```bash
# Commit version bump
git add Cargo.toml CHANGELOG.md README.md
git commit -m "chore: bump version to 0.3.0"
git push origin main
```

### 5. Create Git Tag

```bash
# Create annotated tag
git tag -a v0.3.0 -m "Release v0.3.0 - Theme system and ARM support"

# Push tag to GitHub (this triggers GitHub Actions build)
git push origin v0.3.0
```

### 6. Monitor GitHub Actions

1. Go to GitHub → Actions
2. Find the "Release" workflow triggered by your tag
3. Wait for all jobs to complete:
   - ✅ test
   - ✅ build-linux
   - ✅ build-linux-arm64
   - ✅ build-linux-armv7
   - ✅ build-windows
   - ✅ build-macos-intel
   - ✅ build-macos-arm
   - ✅ release

Build time: ~10-15 minutes for all platforms.

### 7. Verify Release

1. Go to GitHub → Releases
2. Find your release (v0.3.0)
3. Verify all artifacts are present:
   - leeky-explorer-v0.3.0-linux-x86_64.tar.gz
   - leeky-explorer-v0.3.0-linux-arm64.tar.gz
   - leeky-explorer-v0.3.0-linux-armv7.tar.gz
   - leeky-explorer-v0.3.0-windows-x86_64.zip
   - leeky-explorer-v0.3.0-macos-x86_64.tar.gz
   - leeky-explorer-v0.3.0-macos-arm64.tar.gz
   - SHA256 checksums for each (*.sha256 files)

### 8. Test Downloads

Download and test binaries on each platform:

**Linux x86_64**:
```bash
wget https://github.com/cochimbo/Leeky-explorer/releases/download/v0.3.0/leeky-explorer-v0.3.0-linux-x86_64.tar.gz
tar -xzf leeky-explorer-v0.3.0-linux-x86_64.tar.gz
./leeky-explorer
```

**Linux ARM64 (Raspberry Pi 64-bit)**:
```bash
wget https://github.com/cochimbo/Leeky-explorer/releases/download/v0.3.0/leeky-explorer-v0.3.0-linux-arm64.tar.gz
tar -xzf leeky-explorer-v0.3.0-linux-arm64.tar.gz
./leeky-explorer
```

**Linux ARMv7 (Raspberry Pi 32-bit)**:
```bash
wget https://github.com/cochimbo/Leeky-explorer/releases/download/v0.3.0/leeky-explorer-v0.3.0-linux-armv7.tar.gz
tar -xzf leeky-explorer-v0.3.0-linux-armv7.tar.gz
./leeky-explorer
```

**Windows**:
```powershell
Invoke-WebRequest -Uri "https://github.com/cochimbo/Leeky-explorer/releases/download/v0.3.0/leeky-explorer-v0.3.0-windows-x86_64.zip" -OutFile leeky-explorer.zip
Expand-Archive leeky-explorer.zip
.\leeky-explorer\leeky-explorer.exe
```

**macOS Intel**:
```bash
curl -L -o leeky-explorer.tar.gz https://github.com/cochimbo/Leeky-explorer/releases/download/v0.3.0/leeky-explorer-v0.3.0-macos-x86_64.tar.gz
tar -xzf leeky-explorer.tar.gz
./leeky-explorer
```

**macOS Apple Silicon**:
```bash
curl -L -o leeky-explorer.tar.gz https://github.com/cochimbo/Leeky-explorer/releases/download/v0.3.0/leeky-explorer-v0.3.0-macos-arm64.tar.gz
tar -xzf leeky-explorer.tar.gz
./leeky-explorer
```

### 9. Announce Release

- Update README.md with new version number
- Post announcement (if applicable)
- Update documentation links

## Building Binaries Locally

If GitHub Actions fails or you need to build manually:

### Linux x86_64

```bash
cargo build --release --target x86_64-unknown-linux-gnu
cd target/x86_64-unknown-linux-gnu/release
tar -czf leeky-explorer-v0.3.0-linux-x86_64.tar.gz leeky-explorer
sha256sum leeky-explorer-v0.3.0-linux-x86_64.tar.gz > leeky-explorer-v0.3.0-linux-x86_64.tar.gz.sha256
```

### Linux ARM64 (Cross-compile)

```bash
# Install cross-compiler
sudo apt-get install gcc-aarch64-linux-gnu

# Add Rust target
rustup target add aarch64-unknown-linux-gnu

# Build
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
  cargo build --release --target aarch64-unknown-linux-gnu

cd target/aarch64-unknown-linux-gnu/release
tar -czf leeky-explorer-v0.3.0-linux-arm64.tar.gz leeky-explorer
sha256sum leeky-explorer-v0.3.0-linux-arm64.tar.gz > leeky-explorer-v0.3.0-linux-arm64.tar.gz.sha256
```

### Linux ARMv7 (Cross-compile)

```bash
# Install cross-compiler
sudo apt-get install gcc-arm-linux-gnueabihf

# Add Rust target
rustup target add armv7-unknown-linux-gnueabihf

# Build
CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc \
  cargo build --release --target armv7-unknown-linux-gnueabihf

cd target/armv7-unknown-linux-gnueabihf/release
tar -czf leeky-explorer-v0.3.0-linux-armv7.tar.gz leeky-explorer
sha256sum leeky-explorer-v0.3.0-linux-armv7.tar.gz > leeky-explorer-v0.3.0-linux-armv7.tar.gz.sha256
```

### Windows (native)

```powershell
cargo build --release --target x86_64-pc-windows-msvc
cd target\release
Compress-Archive leeky-explorer.exe leeky-explorer-v0.3.0-windows-x86_64.zip
Get-FileHash leeky-explorer-v0.3.0-windows-x86_64.zip -Algorithm SHA256 | Select-Object -ExpandProperty Hash | Out-File leeky-explorer-v0.3.0-windows-x86_64.zip.sha256 -NoNewline
```

### macOS (Intel)

```bash
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
cd target/x86_64-apple-darwin/release
tar -czf leeky-explorer-v0.3.0-macos-x86_64.tar.gz leeky-explorer
shasum -a 256 leeky-explorer-v0.3.0-macos-x86_64.tar.gz > leeky-explorer-v0.3.0-macos-x86_64.tar.gz.sha256
```

### macOS (Apple Silicon)

```bash
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin
cd target/aarch64-apple-darwin/release
tar -czf leeky-explorer-v0.3.0-macos-arm64.tar.gz leeky-explorer
shasum -a 256 leeky-explorer-v0.3.0-macos-arm64.tar.gz > leeky-explorer-v0.3.0-macos-arm64.tar.gz.sha256
```

## Manual Release Creation

If you need to upload binaries manually:

1. Go to GitHub → Releases
2. Click "Draft a new release"
3. Choose tag (v0.3.0) or create new tag
4. Enter release title: "Leeky Explorer v0.3.0"
5. Add release notes (copy from CHANGELOG.md)
6. Upload artifacts:
   - Drag and drop or click to upload each binary file
   - Include .sha256 checksum files
   - Files will be available as downloadable assets

## Troubleshooting

### ARM Cross-Compilation Issues

If ARM builds fail, ensure cross-compilation tools are installed:
```bash
# For ARM64
sudo apt-get install gcc-aarch64-linux-gnu

# For ARMv7
sudo apt-get install gcc-arm-linux-gnueabihf
```

### macOS Builds Fail

macOS builds require a macOS runner (GitHub provides these). If building locally:
- macOS 13 runner for Intel builds
- macOS 14 runner for Apple Silicon builds

### Windows Build Fails

Ensure you're using MSVC target (not MinGW):
```bash
rustup target add x86_64-pc-windows-msvc
```

### Permission Denied on Binary

After downloading, you may need to make it executable:
```bash
chmod +x leeky-explorer
```

On macOS, you may need to allow it in System Preferences:
```bash
xattr -d com.apple.quarantine leeky-explorer
```

## Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0): Incompatible API changes
- **MINOR** (0.3.0): New features, backwards compatible
- **PATCH** (0.3.1): Bug fixes, backwards compatible

Examples:
- v0.1.0 - Initial release (dual-pane, file ops, multi-select, preview, search)
- v0.2.0 - Archive support (extract, compress, password protection)
- v0.3.0 - Theme system, ARM support, search improvements
- v0.3.1 - Bug fix release (if needed)
- v1.0.0 - Stable release, production ready

## Post-Release

After successful release:

1. ✅ Verify downloads work
2. ✅ Update README with new version
3. ✅ Close related issues/milestones
4. ✅ Start work on next version

## Automation Future

Consider adding:
- Automated changelog generation from commits
- Release notes from issue tracker
- Homebrew formula update
- AUR package update
- Notification webhooks
