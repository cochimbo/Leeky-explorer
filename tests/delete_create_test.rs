// Delete and create operations tests
use anyhow::Result;
use leeky_explorer::fs::operations;
use std::fs;
use tempfile::TempDir;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_delete_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test_file.txt");
    
    // Create test file
    fs::write(&file_path, "Test content")?;
    assert!(file_path.exists());
    
    // Delete file
    operations::delete_file(&file_path).await?;
    
    // Verify file is deleted
    assert!(!file_path.exists());
    
    Ok(())
}

#[tokio::test]
async fn test_delete_dir_recursive() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("test_dir");
    
    // Create directory structure
    fs::create_dir(&dir_path)?;
    fs::write(dir_path.join("file1.txt"), "Content 1")?;
    fs::write(dir_path.join("file2.txt"), "Content 2")?;
    
    let subdir = dir_path.join("subdir");
    fs::create_dir(&subdir)?;
    fs::write(subdir.join("file3.txt"), "Content 3")?;
    
    assert!(dir_path.exists());
    
    // Create progress channel
    let (progress_tx, mut progress_rx) = mpsc::channel(100);
    
    // Clone path
    let path = dir_path.clone();
    
    // Spawn delete operation
    let delete_task = tokio::spawn(async move {
        operations::delete_dir_recursive(&path, progress_tx).await
    });
    
    // Collect progress updates
    let mut final_progress = None;
    while let Some(progress) = progress_rx.recv().await {
        if progress.is_complete() {
            final_progress = Some(progress);
            break;
        }
    }
    
    // Wait for delete to complete
    delete_task.await??;
    
    // Verify directory is deleted
    assert!(!dir_path.exists());
    
    // Verify progress tracking
    assert!(final_progress.is_some());
    let progress = final_progress.unwrap();
    assert_eq!(progress.files_done, progress.files_total);
    assert_eq!(progress.files_total, 3); // 3 files deleted
    
    Ok(())
}

#[tokio::test]
async fn test_create_dir() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let new_dir = temp_dir.path().join("new_directory");
    
    assert!(!new_dir.exists());
    
    // Create directory
    operations::create_dir(&new_dir).await?;
    
    // Verify directory exists
    assert!(new_dir.exists());
    assert!(new_dir.is_dir());
    
    Ok(())
}

#[tokio::test]
async fn test_is_dir_empty() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let empty_dir = temp_dir.path().join("empty");
    let non_empty_dir = temp_dir.path().join("non_empty");
    
    // Create directories
    fs::create_dir(&empty_dir)?;
    fs::create_dir(&non_empty_dir)?;
    fs::write(non_empty_dir.join("file.txt"), "content")?;
    
    // Test empty directory
    assert!(operations::is_dir_empty(&empty_dir).await?);
    
    // Test non-empty directory
    assert!(!operations::is_dir_empty(&non_empty_dir).await?);
    
    Ok(())
}

#[tokio::test]
async fn test_delete_handles_error_nonexistent_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let nonexistent = temp_dir.path().join("nonexistent.txt");
    
    let result = operations::delete_file(&nonexistent).await;
    
    // Should return error
    assert!(result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_create_dir_fails_if_exists() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let existing_dir = temp_dir.path().join("existing");
    
    // Create directory
    fs::create_dir(&existing_dir)?;
    
    // Try to create again
    let result = operations::create_dir(&existing_dir).await;
    
    // Should return error
    assert!(result.is_err());
    
    Ok(())
}
