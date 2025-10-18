# Build script for Leeky Explorer on Windows
# Usage: .\build.ps1 [command]

param(
    [Parameter(Position=0)]
    [ValidateSet("help", "build", "release", "test", "clean", "package", "all")]
    [string]$Command = "help"
)

$ErrorActionPreference = "Stop"
$BinaryName = "leeky-explorer"
$BuildDir = "builds"
$Version = $(git describe --tags --always --dirty 2>$null)
if (-not $Version) { $Version = "dev" }

function Show-Help {
    Write-Host "Leeky Explorer - Build Script for Windows" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage: .\build.ps1 [command]" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Commands:" -ForegroundColor Green
    Write-Host "  help     - Show this help message"
    Write-Host "  build    - Build debug binary"
    Write-Host "  release  - Build optimized release binary"
    Write-Host "  test     - Run all tests and clippy"
    Write-Host "  clean    - Clean build artifacts"
    Write-Host "  package  - Create distribution ZIP with checksums"
    Write-Host "  all      - Run tests + build release + package"
    Write-Host ""
}

function Build-Debug {
    Write-Host "Building debug binary..." -ForegroundColor Yellow
    cargo build
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ Debug build complete" -ForegroundColor Green
        Write-Host "Binary: target\debug\$BinaryName.exe" -ForegroundColor Cyan
    }
}

function Build-Release {
    Write-Host "Building release binary..." -ForegroundColor Yellow
    cargo build --release
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ Release build complete" -ForegroundColor Green
        Write-Host "Binary: target\release\$BinaryName.exe" -ForegroundColor Cyan
        
        # Show binary size
        $size = (Get-Item "target\release\$BinaryName.exe").Length / 1MB
        Write-Host "Size: $([math]::Round($size, 2)) MB" -ForegroundColor Cyan
    }
}

function Run-Tests {
    Write-Host "Running tests..." -ForegroundColor Yellow
    cargo test --all-targets
    if ($LASTEXITCODE -ne 0) {
        Write-Host "✗ Tests failed" -ForegroundColor Red
        exit 1
    }
    
    Write-Host ""
    Write-Host "Running clippy..." -ForegroundColor Yellow
    cargo clippy -- -D warnings
    if ($LASTEXITCODE -ne 0) {
        Write-Host "✗ Clippy found issues" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "✓ All tests passed" -ForegroundColor Green
}

function Clean-Build {
    Write-Host "Cleaning build artifacts..." -ForegroundColor Yellow
    cargo clean
    if (Test-Path $BuildDir) {
        Remove-Item -Recurse -Force $BuildDir
    }
    Write-Host "✓ Clean complete" -ForegroundColor Green
}

function Create-Package {
    Write-Host "Creating distribution package..." -ForegroundColor Yellow
    
    # Build release first
    Build-Release
    if ($LASTEXITCODE -ne 0) { exit 1 }
    
    # Create distribution directory
    $distDir = "$BuildDir\dist"
    New-Item -ItemType Directory -Force -Path $distDir | Out-Null
    
    # Copy binary
    $binaryPath = "target\release\$BinaryName.exe"
    Copy-Item $binaryPath $distDir
    
    # Create ZIP
    $zipName = "$BinaryName-$Version-windows-x86_64.zip"
    $zipPath = "$distDir\$zipName"
    
    if (Test-Path $zipPath) {
        Remove-Item $zipPath
    }
    
    Compress-Archive -Path "$distDir\$BinaryName.exe" -DestinationPath $zipPath
    
    # Create SHA256 checksum
    $hash = (Get-FileHash -Path $zipPath -Algorithm SHA256).Hash
    "$hash  $zipName" | Out-File -FilePath "$zipPath.sha256" -Encoding ASCII
    
    Write-Host ""
    Write-Host "✓ Package created successfully" -ForegroundColor Green
    Write-Host "Location: $zipPath" -ForegroundColor Cyan
    Write-Host "SHA256: $zipPath.sha256" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Package contents:" -ForegroundColor Yellow
    Get-ChildItem $distDir | Format-Table Name, Length, LastWriteTime
}

function Build-All {
    Run-Tests
    if ($LASTEXITCODE -ne 0) { exit 1 }
    
    Build-Release
    if ($LASTEXITCODE -ne 0) { exit 1 }
    
    Create-Package
}

# Execute command
switch ($Command) {
    "help" { Show-Help }
    "build" { Build-Debug }
    "release" { Build-Release }
    "test" { Run-Tests }
    "clean" { Clean-Build }
    "package" { Create-Package }
    "all" { Build-All }
    default { Show-Help }
}
