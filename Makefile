# Makefile for Leeky Explorer
# Cross-platform build automation

.PHONY: help build build-release build-all test clean install

BINARY_NAME := leeky-explorer
VERSION := $(shell git describe --tags --always --dirty 2>/dev/null || echo "dev")
BUILD_DIR := builds

help:
	@echo "Leeky Explorer - Build System"
	@echo ""
	@echo "Available targets:"
	@echo "  make build           - Build debug binary for current platform"
	@echo "  make build-release   - Build optimized release binary"
	@echo "  make build-all       - Build release for all platforms"
	@echo "  make test            - Run all tests"
	@echo "  make clean           - Clean build artifacts"
	@echo "  make install         - Install to ~/.cargo/bin"
	@echo "  make package         - Create distribution packages"
	@echo ""
	@echo "Platform-specific:"
	@echo "  make build-linux     - Build Linux x86_64"
	@echo "  make build-windows   - Build Windows x86_64"
	@echo "  make build-macos     - Build macOS x86_64"
	@echo "  make build-macos-arm - Build macOS ARM64"

# Development build
build:
	cargo build

# Release build for current platform
build-release:
	cargo build --release

# Run tests
test:
	cargo test --all-targets
	cargo clippy -- -D warnings

# Clean build artifacts
clean:
	cargo clean
	rm -rf $(BUILD_DIR)

# Install to user's cargo bin
install:
	cargo install --path .

# Build for Linux x86_64
build-linux:
	@echo "Building for Linux x86_64..."
	cargo build --release --target x86_64-unknown-linux-gnu
	@mkdir -p $(BUILD_DIR)/linux
	cp target/x86_64-unknown-linux-gnu/release/$(BINARY_NAME) $(BUILD_DIR)/linux/
	@echo "✓ Linux binary: $(BUILD_DIR)/linux/$(BINARY_NAME)"

# Build for Windows x86_64 (requires mingw-w64)
build-windows:
	@echo "Building for Windows x86_64..."
	rustup target add x86_64-pc-windows-gnu 2>/dev/null || true
	cargo build --release --target x86_64-pc-windows-gnu
	@mkdir -p $(BUILD_DIR)/windows
	cp target/x86_64-pc-windows-gnu/release/$(BINARY_NAME).exe $(BUILD_DIR)/windows/
	@echo "✓ Windows binary: $(BUILD_DIR)/windows/$(BINARY_NAME).exe"

# Build for macOS x86_64
build-macos:
	@echo "Building for macOS x86_64..."
	rustup target add x86_64-apple-darwin 2>/dev/null || true
	cargo build --release --target x86_64-apple-darwin
	@mkdir -p $(BUILD_DIR)/macos-x86
	cp target/x86_64-apple-darwin/release/$(BINARY_NAME) $(BUILD_DIR)/macos-x86/
	@echo "✓ macOS x86_64 binary: $(BUILD_DIR)/macos-x86/$(BINARY_NAME)"

# Build for macOS ARM64 (M1/M2)
build-macos-arm:
	@echo "Building for macOS ARM64..."
	rustup target add aarch64-apple-darwin 2>/dev/null || true
	cargo build --release --target aarch64-apple-darwin
	@mkdir -p $(BUILD_DIR)/macos-arm
	cp target/aarch64-apple-darwin/release/$(BINARY_NAME) $(BUILD_DIR)/macos-arm/
	@echo "✓ macOS ARM64 binary: $(BUILD_DIR)/macos-arm/$(BINARY_NAME)"

# Build all platforms (best effort)
build-all: build-linux build-windows
	@echo ""
	@echo "Note: macOS builds require macOS host or cross-compilation setup"
	@echo "Run 'make build-macos' or 'make build-macos-arm' on macOS"

# Create distribution packages with version tag
package: build-all
	@echo "Creating distribution packages..."
	@mkdir -p $(BUILD_DIR)/dist
	
	# Linux tarball
	cd $(BUILD_DIR)/linux && tar -czf ../dist/$(BINARY_NAME)-$(VERSION)-linux-x86_64.tar.gz $(BINARY_NAME)
	cd $(BUILD_DIR)/dist && sha256sum $(BINARY_NAME)-$(VERSION)-linux-x86_64.tar.gz > $(BINARY_NAME)-$(VERSION)-linux-x86_64.tar.gz.sha256
	
	# Windows zip
	cd $(BUILD_DIR)/windows && zip ../dist/$(BINARY_NAME)-$(VERSION)-windows-x86_64.zip $(BINARY_NAME).exe
	cd $(BUILD_DIR)/dist && sha256sum $(BINARY_NAME)-$(VERSION)-windows-x86_64.zip > $(BINARY_NAME)-$(VERSION)-windows-x86_64.zip.sha256
	
	@echo ""
	@echo "✓ Packages created in $(BUILD_DIR)/dist/"
	@ls -lh $(BUILD_DIR)/dist/

# Quick development cycle
dev: test build
	@echo "✓ Development build ready"

# Check if all targets are available
check-targets:
	@echo "Installed Rust targets:"
	@rustup target list --installed
	@echo ""
	@echo "Available targets for cross-compilation:"
	@rustup target list | grep -E "(linux-gnu|windows-gnu|apple-darwin)"
