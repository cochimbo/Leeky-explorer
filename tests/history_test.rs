// Integration tests for navigation history
use leeky_explorer::models::panel::{Panel, NavigationHistory};
use std::path::PathBuf;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_history_creation() {
    let history = NavigationHistory::new(20);
    assert_eq!(history.count(), 0);
    assert_eq!(history.get_all().len(), 0);
}

#[test]
fn test_history_push() {
    let mut history = NavigationHistory::new(20);
    let path1 = PathBuf::from("/tmp/dir1");
    let path2 = PathBuf::from("/tmp/dir2");
    
    history.push(path1.clone());
    assert_eq!(history.count(), 1);
    assert_eq!(history.get_all()[0], path1);
    
    history.push(path2.clone());
    assert_eq!(history.count(), 2);
    assert_eq!(history.get_all()[1], path2);
}

#[test]
fn test_history_avoids_consecutive_duplicates() {
    let mut history = NavigationHistory::new(20);
    let path = PathBuf::from("/tmp/dir1");
    
    history.push(path.clone());
    history.push(path.clone()); // Should be ignored
    history.push(path.clone()); // Should be ignored
    
    assert_eq!(history.count(), 1);
}

#[test]
fn test_history_allows_non_consecutive_duplicates() {
    let mut history = NavigationHistory::new(20);
    let path1 = PathBuf::from("/tmp/dir1");
    let path2 = PathBuf::from("/tmp/dir2");
    
    history.push(path1.clone());
    history.push(path2.clone());
    history.push(path1.clone()); // Different from last, should be added
    
    assert_eq!(history.count(), 3);
    assert_eq!(history.get_all()[0], path1);
    assert_eq!(history.get_all()[1], path2);
    assert_eq!(history.get_all()[2], path1);
}

#[test]
fn test_history_size_limit() {
    let mut history = NavigationHistory::new(5);
    
    for i in 0..10 {
        let path = PathBuf::from(format!("/tmp/dir{}", i));
        history.push(path);
    }
    
    // Should only keep last 5
    assert_eq!(history.count(), 5);
    
    // Should have dirs 5-9
    let entries = history.get_all();
    assert_eq!(entries[0], PathBuf::from("/tmp/dir5"));
    assert_eq!(entries[4], PathBuf::from("/tmp/dir9"));
}

#[test]
fn test_history_clear() {
    let mut history = NavigationHistory::new(20);
    
    history.push(PathBuf::from("/tmp/dir1"));
    history.push(PathBuf::from("/tmp/dir2"));
    history.push(PathBuf::from("/tmp/dir3"));
    
    assert_eq!(history.count(), 3);
    
    history.clear();
    assert_eq!(history.count(), 0);
}

#[test]
fn test_navigation_adds_to_history() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let base_path = temp_dir.path();
    
    // Create test directories
    let dir1 = base_path.join("dir1");
    let dir2 = base_path.join("dir2");
    fs::create_dir(&dir1)?;
    fs::create_dir(&dir2)?;
    
    let mut panel = Panel::new(base_path.to_path_buf());
    panel.refresh_entries()?;
    
    // Initial path should be in history
    assert_eq!(panel.history.count(), 1);
    assert_eq!(panel.history.get_all()[0], base_path);
    
    // Navigate to dir1
    panel.current_path = dir1.clone();
    panel.history.push(dir1.clone());
    
    assert_eq!(panel.history.count(), 2);
    assert_eq!(panel.history.get_all()[1], dir1);
    
    // Navigate to dir2
    panel.current_path = dir2.clone();
    panel.history.push(dir2.clone());
    
    assert_eq!(panel.history.count(), 3);
    assert_eq!(panel.history.get_all()[2], dir2);
    
    Ok(())
}

#[test]
fn test_independent_panel_history() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let base_path = temp_dir.path();
    
    let dir1 = base_path.join("dir1");
    let dir2 = base_path.join("dir2");
    fs::create_dir(&dir1)?;
    fs::create_dir(&dir2)?;
    
    // Create two independent panels
    let mut panel_left = Panel::new(base_path.to_path_buf());
    let mut panel_right = Panel::new(base_path.to_path_buf());
    
    // Navigate left panel
    panel_left.current_path = dir1.clone();
    panel_left.history.push(dir1.clone());
    
    // Navigate right panel
    panel_right.current_path = dir2.clone();
    panel_right.history.push(dir2.clone());
    
    // Check they have independent histories
    assert_eq!(panel_left.history.count(), 2);
    assert_eq!(panel_right.history.count(), 2);
    
    assert_eq!(panel_left.history.get_all()[1], dir1);
    assert_eq!(panel_right.history.get_all()[1], dir2);
    
    Ok(())
}

#[test]
fn test_clean_invalid_paths() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let base_path = temp_dir.path();
    
    // Create some directories
    let dir1 = base_path.join("dir1");
    let dir2 = base_path.join("dir2");
    let dir3 = base_path.join("dir3");
    fs::create_dir(&dir1)?;
    fs::create_dir(&dir2)?;
    fs::create_dir(&dir3)?;
    
    let mut panel = Panel::new(base_path.to_path_buf());
    
    // Add all dirs to history
    panel.history.push(dir1.clone());
    panel.history.push(dir2.clone());
    panel.history.push(dir3.clone());
    
    assert_eq!(panel.history.count(), 4); // base + 3 dirs
    
    // Delete dir2
    fs::remove_dir(&dir2)?;
    
    // Clean invalid paths
    let removed = panel.history.clean_invalid();
    
    assert_eq!(removed, 1);
    assert_eq!(panel.history.count(), 3);
    
    // Verify dir2 is gone but others remain
    let entries = panel.history.get_all();
    assert!(entries.contains(&base_path.to_path_buf()));
    assert!(entries.contains(&dir1));
    assert!(!entries.contains(&dir2));
    assert!(entries.contains(&dir3));
    
    Ok(())
}

#[test]
fn test_clean_invalid_paths_all_valid() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let base_path = temp_dir.path();
    
    let dir1 = base_path.join("dir1");
    fs::create_dir(&dir1)?;
    
    let mut panel = Panel::new(base_path.to_path_buf());
    panel.history.push(dir1.clone());
    
    let removed = panel.history.clean_invalid();
    
    assert_eq!(removed, 0);
    assert_eq!(panel.history.count(), 2);
    
    Ok(())
}

#[test]
fn test_clean_invalid_paths_all_invalid() {
    let mut history = NavigationHistory::new(20);
    
    // Add non-existent paths
    history.push(PathBuf::from("/nonexistent/path1"));
    history.push(PathBuf::from("/nonexistent/path2"));
    history.push(PathBuf::from("/nonexistent/path3"));
    
    assert_eq!(history.count(), 3);
    
    let removed = history.clean_invalid();
    
    assert_eq!(removed, 3);
    assert_eq!(history.count(), 0);
}

#[test]
fn test_history_with_enter_dir() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let base_path = temp_dir.path();
    
    // Create test directory structure
    let subdir = base_path.join("subdir");
    fs::create_dir(&subdir)?;
    fs::File::create(subdir.join("file.txt"))?;
    
    let mut panel = Panel::new(base_path.to_path_buf());
    panel.refresh_entries()?;
    
    assert_eq!(panel.history.count(), 1);
    
    // Select the subdirectory (should be first entry)
    if !panel.entries.is_empty() {
        panel.cursor = 0;
        panel.enter_dir()?;
        panel.refresh_entries()?;
        
        // Should have added subdir to history
        assert_eq!(panel.history.count(), 2);
        assert_eq!(panel.current_path, subdir);
    }
    
    Ok(())
}

#[test]
fn test_history_with_go_up() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let base_path = temp_dir.path();
    
    let subdir = base_path.join("subdir");
    fs::create_dir(&subdir)?;
    
    // Start in subdir
    let mut panel = Panel::new(subdir.clone());
    panel.refresh_entries()?;
    
    assert_eq!(panel.history.count(), 1);
    assert_eq!(panel.current_path, subdir);
    
    // Go up to parent
    panel.go_up()?;
    panel.refresh_entries()?;
    
    // Should have added parent to history
    assert_eq!(panel.history.count(), 2);
    assert_eq!(panel.current_path, base_path);
    
    Ok(())
}

#[test]
fn test_history_performance_large_buffer() {
    use std::time::Instant;
    
    let mut history = NavigationHistory::new(20);
    
    let start = Instant::now();
    
    // Add 100 entries (should keep only last 20)
    for i in 0..100 {
        let path = PathBuf::from(format!("/tmp/dir{}", i));
        history.push(path);
    }
    
    let duration = start.elapsed();
    
    // Should complete in reasonable time
    assert!(duration.as_millis() < 100, "History operations too slow: {:?}", duration);
    assert_eq!(history.count(), 20);
}
