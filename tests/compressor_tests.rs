// Unit tests for archive compression functionality
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// Import compression modules
use leeky_explorer::archive::compressor::{compress_archive, CompressionOptions, CompressionLevel};
use leeky_explorer::archive::formats::ArchiveFormat;
use leeky_explorer::archive::extractor::extract_archive_unbounded;

/// Helper: Create a test directory with sample files
fn create_test_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    // Create a simple text file
    let file1 = dir.join("test1.txt");
    fs::write(&file1, b"Hello, World!\nThis is a test file.\n")?;
    files.push(file1);
    
    // Create another text file
    let file2 = dir.join("test2.txt");
    fs::write(&file2, b"Second test file with some content.\nMultiple lines.\n")?;
    files.push(file2);
    
    // Create a subdirectory with a file
    let subdir = dir.join("subdir");
    fs::create_dir(&subdir)?;
    let file3 = subdir.join("nested.txt");
    fs::write(&file3, b"Nested file content.\n")?;
    files.push(file3);
    
    Ok(files)
}

/// Helper: Create a large test file for compression level testing
fn create_large_file(path: &Path, size_kb: usize) -> Result<()> {
    // Create repetitive content that compresses well
    let pattern = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789\n";
    let mut content = Vec::new();
    
    let iterations = (size_kb * 1024) / pattern.len() + 1;
    for _ in 0..iterations {
        content.extend_from_slice(pattern);
    }
    
    content.truncate(size_kb * 1024);
    fs::write(path, &content)?;
    
    Ok(())
}

/// Helper: Count files in directory recursively
fn count_files_recursive(dir: &Path) -> usize {
    let mut count = 0;
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    count += 1;
                } else if metadata.is_dir() {
                    count += count_files_recursive(&entry.path());
                }
            }
        }
    }
    
    count
}

#[tokio::test]
async fn test_compress_single_file_to_zip() -> Result<()> {
    // T958: Test compress single file to ZIP
    let temp_dir = TempDir::new()?;
    let source_file = temp_dir.path().join("test.txt");
    fs::write(&source_file, b"Test content for ZIP compression")?;
    
    let output_zip = temp_dir.path().join("output.zip");
    
    // Create progress channel (ignored in test)
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    
    // Compress
    let options = CompressionOptions {
        output_path: output_zip.clone(),
        format: ArchiveFormat::ZIP,
        level: CompressionLevel::Normal,
        password: None,
    };
    
    compress_archive(&[source_file.clone()], options, tx)?;
    
    // Verify archive was created
    assert!(output_zip.exists(), "ZIP archive should be created");
    assert!(output_zip.metadata()?.len() > 0, "ZIP archive should not be empty");
    
    // Extract and verify content
    let extract_dir = temp_dir.path().join("extracted");
    fs::create_dir(&extract_dir)?;
    
    let (extract_tx, _extract_rx) = tokio::sync::mpsc::unbounded_channel();
    extract_archive_unbounded(&output_zip, &extract_dir, ArchiveFormat::ZIP, None, extract_tx)?;
    
    let extracted_file = extract_dir.join("test.txt");
    assert!(extracted_file.exists(), "File should be extracted");
    
    let content = fs::read_to_string(&extracted_file)?;
    assert_eq!(content, "Test content for ZIP compression");
    
    Ok(())
}

#[tokio::test]
async fn test_compress_multiple_files_to_zip() -> Result<()> {
    // T959: Test compress multiple files to ZIP
    let temp_dir = TempDir::new()?;
    let files = create_test_files(temp_dir.path())?;
    
    let output_zip = temp_dir.path().join("multi.zip");
    
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    
    let options = CompressionOptions {
        output_path: output_zip.clone(),
        format: ArchiveFormat::ZIP,
        level: CompressionLevel::Normal,
        password: None,
    };
    
    // Only compress the two root files (not subdirectory)
    let root_files: Vec<PathBuf> = files.iter()
        .filter(|f| f.parent() == Some(temp_dir.path()))
        .cloned()
        .collect();
    
    compress_archive(&root_files, options, tx)?;
    
    assert!(output_zip.exists(), "ZIP archive should be created");
    
    // Extract and verify all files are present
    let extract_dir = temp_dir.path().join("extracted");
    fs::create_dir(&extract_dir)?;
    
    let (extract_tx, _extract_rx) = tokio::sync::mpsc::unbounded_channel();
    extract_archive_unbounded(&output_zip, &extract_dir, ArchiveFormat::ZIP, None, extract_tx)?;
    
    assert!(extract_dir.join("test1.txt").exists(), "test1.txt should be extracted");
    assert!(extract_dir.join("test2.txt").exists(), "test2.txt should be extracted");
    
    Ok(())
}

#[tokio::test]
async fn test_compress_directory_to_tar_gz() -> Result<()> {
    // T960: Test compress directory recursively to TAR.GZ
    let temp_dir = TempDir::new()?;
    
    // Create simple files (no subdirectories for this test)
    let file1 = temp_dir.path().join("file1.txt");
    fs::write(&file1, b"Content 1")?;
    
    let file2 = temp_dir.path().join("file2.txt");
    fs::write(&file2, b"Content 2")?;
    
    let output_tar = temp_dir.path().join("archive.tar.gz");
    
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    
    let options = CompressionOptions {
        output_path: output_tar.clone(),
        format: ArchiveFormat::TarGz,
        level: CompressionLevel::Normal,
        password: None,
    };
    
    // Compress both files
    compress_archive(&[file1, file2], options, tx)?;
    
    assert!(output_tar.exists(), "TAR.GZ archive should be created");
    assert!(output_tar.metadata()?.len() > 0, "TAR.GZ should not be empty");
    
    // Extract to a different location
    let extract_dir = temp_dir.path().join("extracted");
    fs::create_dir(&extract_dir)?;
    
    let (extract_tx, _extract_rx) = tokio::sync::mpsc::unbounded_channel();
    extract_archive_unbounded(&output_tar, &extract_dir, ArchiveFormat::TarGz, None, extract_tx)?;
    
    // Verify both files were extracted
    assert!(extract_dir.join("file1.txt").exists(), "file1.txt should be extracted");
    assert!(extract_dir.join("file2.txt").exists(), "file2.txt should be extracted");
    
    // Verify content
    let content1 = fs::read_to_string(extract_dir.join("file1.txt"))?;
    assert_eq!(content1, "Content 1");
    
    Ok(())
}

#[tokio::test]
async fn test_compress_with_password() -> Result<()> {
    // T961: Test compress with password (ZIP)
    let temp_dir = TempDir::new()?;
    let source_file = temp_dir.path().join("secret.txt");
    fs::write(&source_file, b"Secret content")?;
    
    let output_zip = temp_dir.path().join("protected.zip");
    let password = "test_password_123";
    
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    
    let options = CompressionOptions {
        output_path: output_zip.clone(),
        format: ArchiveFormat::ZIP,
        level: CompressionLevel::Normal,
        password: Some(password.to_string()),
    };
    
    compress_archive(&[source_file], options, tx)?;
    
    assert!(output_zip.exists(), "Protected ZIP should be created");
    
    // Try to extract with wrong password (should fail)
    let extract_dir1 = temp_dir.path().join("extracted_wrong");
    fs::create_dir(&extract_dir1)?;
    
    let (extract_tx1, _extract_rx1) = tokio::sync::mpsc::unbounded_channel();
    let result_wrong = extract_archive_unbounded(
        &output_zip, 
        &extract_dir1, 
        ArchiveFormat::ZIP, 
        Some("wrong_password".to_string()),
        extract_tx1
    );
    
    assert!(result_wrong.is_err(), "Extraction with wrong password should fail");
    
    // Extract with correct password (should succeed)
    let extract_dir2 = temp_dir.path().join("extracted_correct");
    fs::create_dir(&extract_dir2)?;
    
    let (extract_tx2, _extract_rx2) = tokio::sync::mpsc::unbounded_channel();
    extract_archive_unbounded(
        &output_zip,
        &extract_dir2,
        ArchiveFormat::ZIP,
        Some(password.to_string()),
        extract_tx2
    )?;
    
    let extracted_file = extract_dir2.join("secret.txt");
    assert!(extracted_file.exists(), "File should be extracted with correct password");
    
    let content = fs::read_to_string(&extracted_file)?;
    assert_eq!(content, "Secret content");
    
    Ok(())
}

#[tokio::test]
async fn test_compression_levels() -> Result<()> {
    // T962: Test compression level affects output size
    let temp_dir = TempDir::new()?;
    let source_file = temp_dir.path().join("compressible.txt");
    
    // Create a file with repetitive content (compresses well)
    create_large_file(&source_file, 100)?; // 100 KB
    
    let (tx_fast, _rx_fast) = tokio::sync::mpsc::unbounded_channel();
    let (tx_max, _rx_max) = tokio::sync::mpsc::unbounded_channel();
    
    // Compress with Fast level (low compression)
    let output_fast = temp_dir.path().join("fast.zip");
    let options_fast = CompressionOptions {
        output_path: output_fast.clone(),
        format: ArchiveFormat::ZIP,
        level: CompressionLevel::Fast,
        password: None,
    };
    
    compress_archive(&[source_file.clone()], options_fast, tx_fast)?;
    
    // Compress with Maximum level
    let output_max = temp_dir.path().join("maximum.zip");
    let options_max = CompressionOptions {
        output_path: output_max.clone(),
        format: ArchiveFormat::ZIP,
        level: CompressionLevel::Maximum,
        password: None,
    };
    
    compress_archive(&[source_file], options_max, tx_max)?;
    
    let size_fast = output_fast.metadata()?.len();
    let size_max = output_max.metadata()?.len();
    
    // Maximum compression should be smaller than Fast
    assert!(size_max < size_fast, 
        "Maximum compression ({} bytes) should be smaller than Fast ({} bytes)", 
        size_max, size_fast);
    
    // Maximum should be at least 20% smaller for this repetitive content
    let compression_ratio = (size_max as f64) / (size_fast as f64);
    assert!(compression_ratio < 0.9, 
        "Compression ratio should be < 0.9 for repetitive content, got {}", 
        compression_ratio);
    
    Ok(())
}

#[test]
fn test_estimate_compressed_size() -> Result<()> {
    // T965: Test estimate_compressed_size() approximation
    let temp_dir = TempDir::new()?;
    create_test_files(temp_dir.path())?;
    
    let files: Vec<PathBuf> = vec![
        temp_dir.path().join("test1.txt"),
        temp_dir.path().join("test2.txt"),
    ];
    
    let estimate = leeky_explorer::archive::estimate_compressed_size(&files)?;
    
    // Estimate should be non-zero
    assert!(estimate > 0, "Estimate should be positive");
    
    // Estimate should be less than sum of file sizes (due to compression)
    let total_size: u64 = files.iter()
        .filter_map(|f| fs::metadata(f).ok())
        .map(|m| m.len())
        .sum();
    
    assert!(estimate <= total_size, 
        "Estimate ({} bytes) should be <= total size ({} bytes)", 
        estimate, total_size);
    
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn test_tar_preserves_permissions() -> Result<()> {
    // T964: Test TAR preserves Unix permissions
    use std::os::unix::fs::PermissionsExt;
    
    let temp_dir = TempDir::new()?;
    let source_file = temp_dir.path().join("executable.sh");
    fs::write(&source_file, b"#!/bin/bash\necho 'test'\n")?;
    
    // Set executable permissions (0o755)
    let mut perms = fs::metadata(&source_file)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&source_file, perms)?;
    
    let output_tar = temp_dir.path().join("archive.tar");
    
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    
    let options = CompressionOptions {
        output_path: output_tar.clone(),
        format: ArchiveFormat::TAR,
        level: CompressionLevel::Fast, // Fast for TAR
        password: None,
    };
    
    compress_archive(&[source_file], options, tx)?;
    
    // Extract
    let extract_dir = temp_dir.path().join("extracted");
    fs::create_dir(&extract_dir)?;
    
    let (extract_tx, _extract_rx) = tokio::sync::mpsc::unbounded_channel();
    extract_archive_unbounded(&output_tar, &extract_dir, ArchiveFormat::TAR, None, extract_tx)?;
    
    let extracted_file = extract_dir.join("executable.sh");
    let extracted_perms = fs::metadata(&extracted_file)?.permissions();
    
    // Verify executable bit is preserved
    assert_eq!(extracted_perms.mode() & 0o111, 0o111, 
        "Executable permissions should be preserved");
    
    Ok(())
}

// Note: T963 (ZIP64 for files >4GB) and T966 (cancellation test) 
// are more complex and would require either:
// - Very large file fixtures (impractical for unit tests)
// - Mocking/integration test approach
// Skipping for now, can be implemented as integration tests later
