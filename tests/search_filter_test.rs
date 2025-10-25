// Search and filtering tests
use anyhow::Result;
use leeky_explorer::models::file_entry::{EntryType, FileEntry};
use leeky_explorer::models::panel::Panel;
use std::path::PathBuf;
use tempfile::TempDir;
use std::fs;

// Helper to create test permissions
#[cfg(unix)]
fn test_permissions() -> fs::Permissions {
    use std::os::unix::fs::PermissionsExt;
    fs::Permissions::from_mode(0o644)
}

#[cfg(windows)]
fn test_permissions() -> fs::Permissions {
    // On Windows, just create default permissions
    let temp = tempfile::NamedTempFile::new().unwrap();
    temp.as_file().metadata().unwrap().permissions()
}

// Helper to create test entries
fn create_test_entries() -> Vec<FileEntry> {
    let perms = test_permissions();
    let now = std::time::SystemTime::now();
    
    vec![
        FileEntry::new(
            "document.txt".to_string(),
            EntryType::File,
            1024,
            now,
            Some(now),
            Some("txt".to_string()),
            perms.clone(),
            PathBuf::from("/test/document.txt"),
            #[cfg(windows)]
            Some(0x00000020), // FILE_ATTRIBUTE_ARCHIVE
        ),
        FileEntry::new(
            "image.png".to_string(),
            EntryType::File,
            2048,
            now,
            Some(now),
            Some("png".to_string()),
            perms.clone(),
            PathBuf::from("/test/image.png"),
            #[cfg(windows)]
            Some(0x00000020),
        ),
        FileEntry::new(
            "test_file.rs".to_string(),
            EntryType::File,
            512,
            now,
            Some(now),
            Some("rs".to_string()),
            perms.clone(),
            PathBuf::from("/test/test_file.rs"),
            #[cfg(windows)]
            Some(0x00000020),
        ),
        FileEntry::new(
            "main.rs".to_string(),
            EntryType::File,
            4096,
            now,
            Some(now),
            Some("rs".to_string()),
            perms.clone(),
            PathBuf::from("/test/main.rs"),
            #[cfg(windows)]
            Some(0x00000020),
        ),
        FileEntry::new(
            "README.md".to_string(),
            EntryType::File,
            256,
            now,
            Some(now),
            Some("md".to_string()),
            perms,
            PathBuf::from("/test/README.md"),
            #[cfg(windows)]
            Some(0x00000020),
        ),
    ]
}

#[test]
fn test_apply_filter_simple_text() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut panel = Panel::new(temp_dir.path().to_path_buf());
    let all_entries = create_test_entries();
    
    // Apply filter for "test"
    panel.apply_filter("test", &all_entries);
    
    // Should match "test_file.rs" (case-insensitive contains)
    assert_eq!(panel.entries.len(), 1);
    assert_eq!(panel.entries[0].name, "test_file.rs");
    assert!(panel.has_filter());
    assert_eq!(panel.get_filter(), Some("test"));
    
    Ok(())
}

#[test]
fn test_apply_filter_case_insensitive() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut panel = Panel::new(temp_dir.path().to_path_buf());
    let all_entries = create_test_entries();
    
    // Apply filter for "READ" (should match "README.md")
    panel.apply_filter("READ", &all_entries);
    
    assert_eq!(panel.entries.len(), 1);
    assert_eq!(panel.entries[0].name, "README.md");
    
    Ok(())
}

#[test]
fn test_apply_filter_glob_pattern() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut panel = Panel::new(temp_dir.path().to_path_buf());
    let all_entries = create_test_entries();
    
    // Apply glob filter for "*.rs"
    panel.apply_filter("*.rs", &all_entries);
    
    // Should match "test_file.rs" and "main.rs"
    assert_eq!(panel.entries.len(), 2);
    assert!(panel.entries.iter().any(|e| e.name == "test_file.rs"));
    assert!(panel.entries.iter().any(|e| e.name == "main.rs"));
    
    Ok(())
}

#[test]
fn test_apply_filter_glob_pattern_txt() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut panel = Panel::new(temp_dir.path().to_path_buf());
    let all_entries = create_test_entries();
    
    // Apply glob filter for "*.txt"
    panel.apply_filter("*.txt", &all_entries);
    
    // Should match "document.txt"
    assert_eq!(panel.entries.len(), 1);
    assert_eq!(panel.entries[0].name, "document.txt");
    
    Ok(())
}

#[test]
fn test_apply_filter_no_matches() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut panel = Panel::new(temp_dir.path().to_path_buf());
    let all_entries = create_test_entries();
    
    // Apply filter that matches nothing
    panel.apply_filter("nonexistent", &all_entries);
    
    // Should return empty list
    assert_eq!(panel.entries.len(), 0);
    assert!(panel.has_filter());
    
    Ok(())
}

#[test]
fn test_clear_filter() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut panel = Panel::new(temp_dir.path().to_path_buf());
    let all_entries = create_test_entries();
    
    // Apply filter
    panel.apply_filter("test", &all_entries);
    assert_eq!(panel.entries.len(), 1);
    assert!(panel.has_filter());
    
    // Clear filter
    panel.clear_filter(&all_entries);
    
    // Should restore full list
    assert_eq!(panel.entries.len(), 5);
    assert!(!panel.has_filter());
    assert_eq!(panel.get_filter(), None);
    assert_eq!(panel.cursor, 0); // Cursor should reset
    
    Ok(())
}

#[test]
fn test_filter_resets_cursor() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut panel = Panel::new(temp_dir.path().to_path_buf());
    let all_entries = create_test_entries();
    
    // Set cursor to middle
    panel.cursor = 2;
    
    // Apply filter
    panel.apply_filter("test", &all_entries);
    
    // Cursor should reset to 0
    assert_eq!(panel.cursor, 0);
    
    Ok(())
}

#[test]
fn test_empty_filter_returns_all() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut panel = Panel::new(temp_dir.path().to_path_buf());
    let all_entries = create_test_entries();
    
    // Apply empty filter
    panel.apply_filter("", &all_entries);
    
    // Should return all entries
    assert_eq!(panel.entries.len(), 5);
    
    Ok(())
}

#[tokio::test]
async fn test_search_workflow_integration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create some test files
    fs::write(temp_dir.path().join("file1.txt"), "content")?;
    fs::write(temp_dir.path().join("file2.rs"), "code")?;
    fs::write(temp_dir.path().join("test_data.csv"), "data")?;
    
    let mut panel = Panel::new(temp_dir.path().to_path_buf());
    panel.refresh_entries()?;
    
    let all_entries = panel.entries.clone();
    let initial_count = all_entries.len();
    
    // User presses '/' to activate search mode
    // Simulated by calling apply_filter
    
    // User types "file"
    panel.apply_filter("file", &all_entries);
    assert_eq!(panel.entries.len(), 2); // file1.txt, file2.rs
    assert!(panel.has_filter());
    
    // User types more characters: "file1"
    panel.apply_filter("file1", &all_entries);
    assert_eq!(panel.entries.len(), 1); // Only file1.txt
    
    // User presses Esc to clear filter
    panel.clear_filter(&all_entries);
    assert_eq!(panel.entries.len(), initial_count);
    assert!(!panel.has_filter());
    
    Ok(())
}

#[test]
fn test_glob_pattern_with_question_mark() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut panel = Panel::new(temp_dir.path().to_path_buf());
    let perms = test_permissions();
    let now = std::time::SystemTime::now();
    
    let all_entries = vec![
        FileEntry::new(
            "file1.txt".to_string(),
            EntryType::File,
            100,
            now,
            Some(now),
            Some("txt".to_string()),
            perms.clone(),
            PathBuf::from("/test/file1.txt"),
            #[cfg(windows)]
            Some(0x00000020),
        ),
        FileEntry::new(
            "file2.txt".to_string(),
            EntryType::File,
            100,
            now,
            Some(now),
            Some("txt".to_string()),
            perms.clone(),
            PathBuf::from("/test/file2.txt"),
            #[cfg(windows)]
            Some(0x00000020),
        ),
        FileEntry::new(
            "file10.txt".to_string(),
            EntryType::File,
            100,
            now,
            Some(now),
            Some("txt".to_string()),
            perms,
            PathBuf::from("/test/file10.txt"),
            #[cfg(windows)]
            Some(0x00000020),
        ),
    ];
    
    // Apply glob filter "file?.txt" (matches single character)
    panel.apply_filter("file?.txt", &all_entries);
    
    // Should match file1.txt and file2.txt, but not file10.txt
    assert_eq!(panel.entries.len(), 2);
    assert!(panel.entries.iter().any(|e| e.name == "file1.txt"));
    assert!(panel.entries.iter().any(|e| e.name == "file2.txt"));
    
    Ok(())
}
