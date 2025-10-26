// Integration tests for bookmark functionality
use anyhow::Result;
use leeky_explorer::config::bookmarks::BookmarkManager;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_create_and_persist_bookmark() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bookmark_file = temp_dir.path().join("bookmarks.json");
    
    // Create a bookmark manager
    let mut manager = BookmarkManager::new(bookmark_file.clone());
    
    // Add a bookmark
    let test_path = PathBuf::from("/home/user/documents");
    manager.add("Documents".to_string(), test_path.clone())?;
    
    // Save to disk
    manager.save()?;
    
    // Load a new manager from the same file
    let loaded_manager = BookmarkManager::load(bookmark_file)?;
    
    // Verify the bookmark was persisted
    let bookmarks = loaded_manager.get_all();
    assert_eq!(bookmarks.len(), 1);
    assert_eq!(bookmarks[0].name, "Documents");
    assert_eq!(bookmarks[0].path, test_path);
    
    Ok(())
}

#[test]
fn test_navigate_to_bookmark() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bookmark_file = temp_dir.path().join("bookmarks.json");
    
    let mut manager = BookmarkManager::new(bookmark_file);
    
    // Add multiple bookmarks
    let path1 = PathBuf::from("/home/user/projects");
    let path2 = PathBuf::from("/home/user/downloads");
    
    manager.add("Projects".to_string(), path1.clone())?;
    manager.add("Downloads".to_string(), path2.clone())?;
    
    // Get bookmark by name
    let bookmark = manager.get("Downloads");
    assert!(bookmark.is_some());
    let bookmark_path = bookmark.unwrap().path.clone();
    let bookmark_timestamp = bookmark.unwrap().last_accessed;
    assert_eq!(bookmark_path, path2);
    
    // Access bookmark (updates timestamp)
    manager.access("Downloads")?;
    
    // Verify access count increased
    let bookmark_after = manager.get("Downloads").unwrap();
    assert!(bookmark_after.last_accessed > bookmark_timestamp);
    
    Ok(())
}

#[test]
fn test_delete_bookmark() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bookmark_file = temp_dir.path().join("bookmarks.json");
    
    let mut manager = BookmarkManager::new(bookmark_file);
    
    // Add bookmarks
    manager.add("Test1".to_string(), PathBuf::from("/path1"))?;
    manager.add("Test2".to_string(), PathBuf::from("/path2"))?;
    manager.add("Test3".to_string(), PathBuf::from("/path3"))?;
    
    assert_eq!(manager.get_all().len(), 3);
    
    // Delete a bookmark
    manager.remove("Test2")?;
    
    assert_eq!(manager.get_all().len(), 2);
    assert!(manager.get("Test1").is_some());
    assert!(manager.get("Test2").is_none());
    assert!(manager.get("Test3").is_some());
    
    Ok(())
}

#[test]
fn test_rename_bookmark() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bookmark_file = temp_dir.path().join("bookmarks.json");
    
    let mut manager = BookmarkManager::new(bookmark_file);
    
    let test_path = PathBuf::from("/home/user/projects");
    manager.add("OldName".to_string(), test_path.clone())?;
    
    // Rename the bookmark
    manager.rename("OldName", "NewName".to_string())?;
    
    // Verify old name doesn't exist
    assert!(manager.get("OldName").is_none());
    
    // Verify new name exists with same path
    let renamed = manager.get("NewName");
    assert!(renamed.is_some());
    assert_eq!(renamed.unwrap().path, test_path);
    
    Ok(())
}

#[test]
fn test_bookmark_to_deleted_directory() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bookmark_file = temp_dir.path().join("bookmarks.json");
    
    // Create a real directory
    let real_dir = temp_dir.path().join("real_directory");
    std::fs::create_dir(&real_dir)?;
    
    let mut manager = BookmarkManager::new(bookmark_file);
    
    // Add bookmark to real directory
    manager.add("RealDir".to_string(), real_dir.clone())?;
    
    // Add bookmark to non-existent directory
    let fake_path = PathBuf::from("/this/path/does/not/exist");
    manager.add("FakeDir".to_string(), fake_path.clone())?;
    
    // Verify both bookmarks exist
    assert_eq!(manager.get_all().len(), 2);
    
    // Check which ones are valid
    let real_bookmark = manager.get("RealDir").unwrap();
    let fake_bookmark = manager.get("FakeDir").unwrap();
    
    assert!(real_bookmark.path_exists());
    assert!(!fake_bookmark.path_exists());
    
    // Clean invalid bookmarks
    let removed = manager.clean_invalid()?;
    assert_eq!(removed, 1);
    assert_eq!(manager.get_all().len(), 1);
    
    // Only the real directory bookmark should remain
    assert!(manager.get("RealDir").is_some());
    assert!(manager.get("FakeDir").is_none());
    
    Ok(())
}

#[test]
fn test_50_bookmarks_performance() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bookmark_file = temp_dir.path().join("bookmarks.json");
    
    let mut manager = BookmarkManager::new(bookmark_file.clone());
    
    // Add 50 bookmarks (the maximum)
    let start = std::time::Instant::now();
    for i in 0..50 {
        let name = format!("Bookmark{:02}", i);
        let path = PathBuf::from(format!("/path/to/bookmark/{}", i));
        manager.add(name, path)?;
    }
    let add_duration = start.elapsed();
    
    // Verify all were added
    assert_eq!(manager.get_all().len(), 50);
    
    // Test that adding one more fails
    let result = manager.add("TooMany".to_string(), PathBuf::from("/path/51"));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Maximum number of bookmarks"));
    
    // Test save/load performance
    let start = std::time::Instant::now();
    manager.save()?;
    let save_duration = start.elapsed();
    
    let start = std::time::Instant::now();
    let loaded = BookmarkManager::load(bookmark_file)?;
    let load_duration = start.elapsed();
    
    assert_eq!(loaded.get_all().len(), 50);
    
    // Verify operations are reasonably fast
    // Note: Each add() calls save(), so 50 adds = 50 file writes
    assert!(add_duration < std::time::Duration::from_millis(250), 
        "Adding 50 bookmarks took {:?}, expected < 250ms", add_duration);
    assert!(save_duration < std::time::Duration::from_millis(100),
        "Saving 50 bookmarks took {:?}, expected < 100ms", save_duration);
    assert!(load_duration < std::time::Duration::from_millis(100),
        "Loading 50 bookmarks took {:?}, expected < 100ms", load_duration);
    
    Ok(())
}

#[test]
fn test_duplicate_bookmark_names() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bookmark_file = temp_dir.path().join("bookmarks.json");
    
    let mut manager = BookmarkManager::new(bookmark_file);
    
    // Add a bookmark
    manager.add("MyBookmark".to_string(), PathBuf::from("/path1"))?;
    
    // Try to add another with the same name (should fail)
    let result = manager.add("MyBookmark".to_string(), PathBuf::from("/path2"));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
    
    // Verify only one bookmark exists
    assert_eq!(manager.get_all().len(), 1);
    
    Ok(())
}

#[test]
fn test_cross_platform_path_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bookmark_file = temp_dir.path().join("bookmarks.json");
    
    let mut manager = BookmarkManager::new(bookmark_file.clone());
    
    // Add a bookmark with a real temp directory path
    let real_path = temp_dir.path().to_path_buf();
    manager.add("TempDir".to_string(), real_path.clone())?;
    
    // Save and reload
    manager.save()?;
    let loaded = BookmarkManager::load(bookmark_file)?;
    
    // Verify path is correctly loaded
    let bookmark = loaded.get("TempDir").unwrap();
    assert_eq!(bookmark.path, real_path);
    
    // On Windows, verify backslashes are handled
    #[cfg(windows)]
    {
        let windows_path = PathBuf::from(r"C:\Users\Test\Documents");
        manager.add("WinPath".to_string(), windows_path.clone())?;
        let win_bookmark = manager.get("WinPath").unwrap();
        assert_eq!(win_bookmark.path, windows_path);
    }
    
    // On Unix, verify forward slashes
    #[cfg(unix)]
    {
        let unix_path = PathBuf::from("/home/user/documents");
        manager.add("UnixPath".to_string(), unix_path.clone())?;
        let unix_bookmark = manager.get("UnixPath").unwrap();
        assert_eq!(unix_bookmark.path, unix_path);
    }
    
    Ok(())
}

#[test]
fn test_bookmark_sorting() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bookmark_file = temp_dir.path().join("bookmarks.json");
    
    let mut manager = BookmarkManager::new(bookmark_file);
    
    // Add bookmarks in random order
    manager.add("Zebra".to_string(), PathBuf::from("/z"))?;
    manager.add("Apple".to_string(), PathBuf::from("/a"))?;
    manager.add("Mango".to_string(), PathBuf::from("/m"))?;
    
    // Get all bookmarks (should be sorted alphabetically)
    let bookmarks = manager.get_all();
    assert_eq!(bookmarks.len(), 3);
    assert_eq!(bookmarks[0].name, "Apple");
    assert_eq!(bookmarks[1].name, "Mango");
    assert_eq!(bookmarks[2].name, "Zebra");
    
    Ok(())
}
