# Release Process for Leeky Explorer

This document describes how to create a new release with pre-built binaries for multiple platforms.

## Prerequisites

- GitLab repository with CI/CD enabled
- Git configured with push access
- Version number decided (semantic versioning: MAJOR.MINOR.PATCH)

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
version = "0.1.0"  # ← Update this
```

Update version references in README.md if any.

### 3. Update CHANGELOG

Create or update `CHANGELOG.md`:

```markdown
# Changelog

## [0.1.0] - 2025-10-18

### Added
- Initial release
- Dual-pane file navigation
- File operations (copy, move, delete)
- Archive support (ZIP, TAR.GZ, etc.)
- File preview (text, images)
- Multi-selection
- Search and filter

### Fixed
- (list any bug fixes)

### Changed
- (list any changes)
```

### 4. Commit Changes

```bash
# Commit version bump
git add Cargo.toml Cargo.lock CHANGELOG.md README.md
git commit -m "chore: Bump version to 0.1.0"
git push origin main
```

### 5. Create Git Tag

```bash
# Create annotated tag
git tag -a v0.1.0 -m "Release v0.1.0 - Initial release with full feature set"

# Push tag to GitLab (this triggers CI/CD build)
git push origin v0.1.0
```

### 6. Monitor CI/CD Pipeline

1. Go to GitHub → Actions
2. Find the "Release" workflow triggered by your tag
3. Wait for all jobs to complete:
   - ✅ test
   - ✅ build-linux
   - ✅ build-windows
   - ✅ build-macos-intel
   - ✅ build-macos-arm
   - ✅ release

### 7. Verify Release

1. Go to GitHub → Releases
2. Find your release (v0.1.0)
3. Verify all artifacts are present:
   - leeky-explorer-v0.1.0-linux-x86_64.tar.gz
   - leeky-explorer-v0.1.0-windows-x86_64.zip
   - leeky-explorer-v0.1.0-macos-x86_64.tar.gz (if available)
   - leeky-explorer-v0.1.0-macos-arm64.tar.gz (if available)
   - SHA256 checksums for each

### 8. Test Downloads

Download and test binaries on each platform:

**Linux**:
```bash
wget <download-url>/leeky-explorer-v0.1.0-linux-x86_64.tar.gz
tar -xzf leeky-explorer-v0.1.0-linux-x86_64.tar.gz
./leeky-explorer --version
```

**Windows**:
```powershell
Invoke-WebRequest -Uri <download-url> -OutFile leeky-explorer.zip
Expand-Archive leeky-explorer.zip
.\leeky-explorer\leeky-explorer.exe
```

**macOS**:
```bash
curl -L -o leeky-explorer.tar.gz <download-url>
tar -xzf leeky-explorer.tar.gz
./leeky-explorer --version
```

### 9. Announce Release

- Update README.md with new version number
- Post announcement (if applicable)
- Update documentation links

## Building Binaries Locally

If CI/CD fails or you need to build manually:

### Linux

```bash
cargo build --release --target x86_64-unknown-linux-gnu
cd target/x86_64-unknown-linux-gnu/release
tar -czf leeky-explorer-v0.1.0-linux-x86_64.tar.gz leeky-explorer
sha256sum leeky-explorer-v0.1.0-linux-x86_64.tar.gz > leeky-explorer-v0.1.0-linux-x86_64.tar.gz.sha256
```

### Windows (on Linux with MinGW)

```bash
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
cd target/x86_64-pc-windows-gnu/release
zip leeky-explorer-v0.1.0-windows-x86_64.zip leeky-explorer.exe
sha256sum leeky-explorer-v0.1.0-windows-x86_64.zip > leeky-explorer-v0.1.0-windows-x86_64.zip.sha256
```

### Windows (native)

```powershell
cargo build --release
cd target\release
Compress-Archive leeky-explorer.exe leeky-explorer-v0.1.0-windows-x86_64.zip
Get-FileHash leeky-explorer-v0.1.0-windows-x86_64.zip -Algorithm SHA256 > leeky-explorer-v0.1.0-windows-x86_64.zip.sha256
```

### macOS (Intel)

```bash
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
cd target/x86_64-apple-darwin/release
tar -czf leeky-explorer-v0.1.0-macos-x86_64.tar.gz leeky-explorer
shasum -a 256 leeky-explorer-v0.1.0-macos-x86_64.tar.gz > leeky-explorer-v0.1.0-macos-x86_64.tar.gz.sha256
```

### macOS (ARM/M1/M2)

```bash
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin
cd target/aarch64-apple-darwin/release
tar -czf leeky-explorer-v0.1.0-macos-arm64.tar.gz leeky-explorer
shasum -a 256 leeky-explorer-v0.1.0-macos-arm64.tar.gz > leeky-explorer-v0.1.0-macos-arm64.tar.gz.sha256
```

## Manual Release Creation

If you need to upload binaries manually:

1. Go to GitHub → Releases
2. Click "Draft a new release"
3. Choose tag (v0.1.0) or create new tag
4. Enter release title: "Leeky Explorer v0.1.0"
5. Add release notes (copy from CHANGELOG.md)
6. Upload artifacts:
   - Drag and drop or click to upload each binary file
   - Include .sha256 checksum files
   - Files will be available as downloadable assets

## Troubleshooting

### macOS Builds Fail

macOS builds require a macOS runner. If you don't have one:
- Build locally on macOS and upload manually
- Or mark `build:macos-*` jobs as `allow_failure: true` in `.gitlab-ci.yml`

### Windows Build Fails on Linux

Ensure MinGW is installed:
```bash
# Debian/Ubuntu
sudo apt-get install mingw-w64

# Add Rust target
rustup target add x86_64-pc-windows-gnu
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
- **MINOR** (0.1.0): New features, backwards compatible
- **PATCH** (0.1.1): Bug fixes, backwards compatible

Examples:
- v0.1.0 - Initial release
- v0.1.1 - Bug fix release
- v0.2.0 - New features (search, preview, etc.)
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
