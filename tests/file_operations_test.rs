// File operations tests
use anyhow::Result;
use leeky_explorer::fs::operations;
use std::fs;
use tempfile::TempDir;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_copy_file_with_progress() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let source_path = temp_dir.path().join("source.txt");
    let dest_path = temp_dir.path().join("dest.txt");
    
    // Create source file with test content
    let test_content = "Hello, World! This is a test file for copy operations.";
    fs::write(&source_path, test_content)?;
    
    // Create progress channel
    let (progress_tx, mut progress_rx) = mpsc::channel(100);
    
    // Clone paths for async task
    let src = source_path.clone();
    let dst = dest_path.clone();
    
    // Spawn copy operation
    let copy_task = tokio::spawn(async move {
        operations::copy_file_with_progress(&src, &dst, progress_tx).await
    });
    
    // Collect progress updates
    let mut progress_updates = Vec::new();
    while let Some(progress) = progress_rx.recv().await {
        let is_complete = progress.is_complete();
        progress_updates.push(progress);
        if is_complete {
            break;
        }
    }
    
    // Wait for copy to complete
    copy_task.await??;
    
    // Verify file was copied
    assert!(dest_path.exists());
    let dest_content = fs::read_to_string(&dest_path)?;
    assert_eq!(dest_content, test_content);
    
    // Verify we got progress updates
    assert!(!progress_updates.is_empty());
    let final_progress = progress_updates.last().unwrap();
    assert!(final_progress.is_complete());
    assert_eq!(final_progress.percentage(), 100.0);
    
    Ok(())
}

#[tokio::test]
async fn test_copy_dir_recursive() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let source_dir = temp_dir.path().join("source_dir");
    let dest_dir = temp_dir.path().join("dest_dir");
    
    // Create source directory structure
    fs::create_dir(&source_dir)?;
    fs::write(source_dir.join("file1.txt"), "Content 1")?;
    fs::write(source_dir.join("file2.txt"), "Content 2")?;
    
    let subdir = source_dir.join("subdir");
    fs::create_dir(&subdir)?;
    fs::write(subdir.join("file3.txt"), "Content 3")?;
    
    // Calculate total size
    let total_size = operations::get_total_size(&source_dir).await?;
    
    // Create progress channel
    let (progress_tx, mut progress_rx) = mpsc::channel(100);
    
    // Clone paths
    let src = source_dir.clone();
    let dst = dest_dir.clone();
    
    // Spawn copy operation
    let copy_task = tokio::spawn(async move {
        operations::copy_dir_recursive(&src, &dst, progress_tx, total_size).await
    });
    
    // Collect progress updates
    let mut final_progress = None;
    while let Some(progress) = progress_rx.recv().await {
        if progress.is_complete() {
            final_progress = Some(progress);
            break;
        }
    }
    
    // Wait for copy to complete
    copy_task.await??;
    
    // Verify directory structure was copied
    assert!(dest_dir.exists());
    assert!(dest_dir.join("file1.txt").exists());
    assert!(dest_dir.join("file2.txt").exists());
    assert!(dest_dir.join("subdir").exists());
    assert!(dest_dir.join("subdir").join("file3.txt").exists());
    
    // Verify content
    assert_eq!(fs::read_to_string(dest_dir.join("file1.txt"))?, "Content 1");
    assert_eq!(fs::read_to_string(dest_dir.join("file2.txt"))?, "Content 2");
    assert_eq!(fs::read_to_string(dest_dir.join("subdir").join("file3.txt"))?, "Content 3");
    
    // Verify progress tracking
    assert!(final_progress.is_some());
    let progress = final_progress.unwrap();
    assert_eq!(progress.files_done, progress.files_total);
    assert_eq!(progress.files_total, 3); // 3 files copied
    
    Ok(())
}

#[tokio::test]
async fn test_move_item() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let source_path = temp_dir.path().join("source.txt");
    let dest_path = temp_dir.path().join("dest.txt");
    
    // Create source file
    let test_content = "File to be moved";
    fs::write(&source_path, test_content)?;
    
    // Create progress channel
    let (progress_tx, mut progress_rx) = mpsc::channel(100);
    
    // Clone paths
    let src = source_path.clone();
    let dst = dest_path.clone();
    
    // Spawn move operation
    let move_task = tokio::spawn(async move {
        operations::move_item(&src, &dst, progress_tx).await
    });
    
    // Wait for completion
    while let Some(progress) = progress_rx.recv().await {
        if progress.is_complete() {
            break;
        }
    }
    
    move_task.await??;
    
    // Verify file was moved (source gone, dest exists)
    assert!(!source_path.exists());
    assert!(dest_path.exists());
    assert_eq!(fs::read_to_string(&dest_path)?, test_content);
    
    Ok(())
}

#[tokio::test]
async fn test_copy_handles_error_nonexistent_source() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let source_path = temp_dir.path().join("nonexistent.txt");
    let dest_path = temp_dir.path().join("dest.txt");
    
    let (progress_tx, _progress_rx) = mpsc::channel(100);
    
    let result = operations::copy_file_with_progress(
        &source_path,
        &dest_path,
        progress_tx,
    ).await;
    
    // Should return error
    assert!(result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_copy_overwrites_existing_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let source_path = temp_dir.path().join("source.txt");
    let dest_path = temp_dir.path().join("dest.txt");
    
    // Create both files
    fs::write(&source_path, "New content")?;
    fs::write(&dest_path, "Old content")?;
    
    let (progress_tx, mut progress_rx) = mpsc::channel(100);
    
    // Clone paths
    let src = source_path.clone();
    let dst = dest_path.clone();
    
    let copy_task = tokio::spawn(async move {
        operations::copy_file_with_progress(&src, &dst, progress_tx).await
    });
    
    while let Some(progress) = progress_rx.recv().await {
        if progress.is_complete() {
            break;
        }
    }
    
    copy_task.await??;
    
    // Verify destination has new content
    assert_eq!(fs::read_to_string(&dest_path)?, "New content");
    
    Ok(())
}
