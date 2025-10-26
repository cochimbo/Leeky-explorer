use crate::models::bookmark::Bookmark;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Maximum number of bookmarks allowed (TASK-009)
const MAX_BOOKMARKS: usize = 50;

/// Manages a collection of bookmarks with JSON persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkManager {
    bookmarks: Vec<Bookmark>,
    #[serde(skip)]
    file_path: PathBuf,
}

impl BookmarkManager {
    /// Create a new BookmarkManager with the specified storage path
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            bookmarks: Vec::new(),
            file_path,
        }
    }

    /// Load bookmarks from the JSON file, creating it if it doesn't exist
    pub fn load(file_path: PathBuf) -> Result<Self> {
        if !file_path.exists() {
            // Create parent directory if needed
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)
                    .context("Failed to create bookmarks directory")?;
            }
            
            // Create empty bookmarks file
            let empty_manager = Self::new(file_path.clone());
            empty_manager.save()?;
            return Ok(empty_manager);
        }

        let content = fs::read_to_string(&file_path)
            .context("Failed to read bookmarks file")?;
        
        let mut manager: BookmarkManager = serde_json::from_str(&content)
            .context("Failed to parse bookmarks file")?;
        
        manager.file_path = file_path;
        Ok(manager)
    }

    /// Save bookmarks to the JSON file
    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self)
            .context("Failed to serialize bookmarks")?;
        
        fs::write(&self.file_path, json)
            .context("Failed to write bookmarks file")?;
        
        Ok(())
    }

    /// Add a new bookmark, checking for duplicates
    pub fn add(&mut self, name: String, path: PathBuf) -> Result<()> {
        // TASK-009: Check maximum bookmarks limit
        if self.bookmarks.len() >= MAX_BOOKMARKS {
            anyhow::bail!("Maximum number of bookmarks ({}) reached", MAX_BOOKMARKS);
        }
        
        // Check for duplicate names
        if self.bookmarks.iter().any(|b| b.name == name) {
            anyhow::bail!("A bookmark with the name '{}' already exists", name);
        }

        // Check for duplicate paths
        if self.bookmarks.iter().any(|b| b.path == path) {
            anyhow::bail!("This path is already bookmarked");
        }

        let bookmark = Bookmark::new(name, path);
        self.bookmarks.push(bookmark);
        self.save()?;
        
        Ok(())
    }

    /// Remove a bookmark by name
    pub fn remove(&mut self, name: &str) -> Result<()> {
        let initial_len = self.bookmarks.len();
        self.bookmarks.retain(|b| b.name != name);
        
        if self.bookmarks.len() == initial_len {
            anyhow::bail!("Bookmark '{}' not found", name);
        }
        
        self.save()?;
        Ok(())
    }

    /// Rename a bookmark
    pub fn rename(&mut self, old_name: &str, new_name: String) -> Result<()> {
        // Check if new name already exists
        if old_name != new_name && self.bookmarks.iter().any(|b| b.name == new_name) {
            anyhow::bail!("A bookmark with the name '{}' already exists", new_name);
        }

        let bookmark = self.bookmarks.iter_mut()
            .find(|b| b.name == old_name)
            .ok_or_else(|| anyhow::anyhow!("Bookmark '{}' not found", old_name))?;
        
        bookmark.name = new_name;
        self.save()?;
        
        Ok(())
    }

    /// Get all bookmarks, sorted by name
    pub fn get_all(&self) -> Vec<&Bookmark> {
        let mut bookmarks: Vec<&Bookmark> = self.bookmarks.iter().collect();
        bookmarks.sort_by(|a, b| a.name.cmp(&b.name));
        bookmarks
    }

    /// Get a bookmark by name
    pub fn get(&self, name: &str) -> Option<&Bookmark> {
        self.bookmarks.iter().find(|b| b.name == name)
    }

    /// Get a mutable reference to a bookmark by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Bookmark> {
        self.bookmarks.iter_mut().find(|b| b.name == name)
    }

    /// Update the last accessed timestamp for a bookmark and save
    pub fn access(&mut self, name: &str) -> Result<()> {
        let bookmark = self.get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Bookmark '{}' not found", name))?;
        
        bookmark.access();
        self.save()?;
        
        Ok(())
    }

    /// Count total bookmarks
    pub fn count(&self) -> usize {
        self.bookmarks.len()
    }

    /// Remove bookmarks with non-existent paths
    pub fn clean_invalid(&mut self) -> Result<usize> {
        let initial_len = self.bookmarks.len();
        self.bookmarks.retain(|b| b.path_exists());
        let removed = initial_len - self.bookmarks.len();
        
        if removed > 0 {
            self.save()?;
        }
        
        Ok(removed)
    }
}

/// TASK-009: Sanitize bookmark name by removing invalid characters
/// Replaces invalid characters with underscores and trims whitespace
pub fn sanitize_bookmark_name(name: &str) -> String {
    // Invalid characters for filenames/bookmarks: \ / : * ? " < > |
    // Note: \n, \r, \t are treated as whitespace and removed by trim
    let invalid_chars = ['\\', '/', ':', '*', '?', '"', '<', '>', '|'];
    
    // First trim whitespace (including \n, \r, \t)
    let trimmed = name.trim();
    
    // Then replace invalid characters
    let sanitized: String = trimmed
        .chars()
        .map(|c| if invalid_chars.contains(&c) { '_' } else { c })
        .collect();
    
    // Limit length to 100 characters
    sanitized.chars().take(100).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_bookmark_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bookmarks.json");
        
        let manager = BookmarkManager::load(file_path.clone()).unwrap();
        assert_eq!(manager.count(), 0);
        assert!(file_path.exists());
    }

    #[test]
    fn test_add_and_get_bookmark() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bookmarks.json");
        
        let mut manager = BookmarkManager::load(file_path).unwrap();
        
        manager.add("Home".to_string(), PathBuf::from("/home/user")).unwrap();
        assert_eq!(manager.count(), 1);
        
        let bookmark = manager.get("Home").unwrap();
        assert_eq!(bookmark.name, "Home");
        assert_eq!(bookmark.path, PathBuf::from("/home/user"));
    }

    #[test]
    fn test_duplicate_name() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bookmarks.json");
        
        let mut manager = BookmarkManager::load(file_path).unwrap();
        
        manager.add("Test".to_string(), PathBuf::from("/path1")).unwrap();
        let result = manager.add("Test".to_string(), PathBuf::from("/path2"));
        
        assert!(result.is_err());
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_duplicate_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bookmarks.json");
        
        let mut manager = BookmarkManager::load(file_path).unwrap();
        
        manager.add("Test1".to_string(), PathBuf::from("/same/path")).unwrap();
        let result = manager.add("Test2".to_string(), PathBuf::from("/same/path"));
        
        assert!(result.is_err());
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_remove_bookmark() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bookmarks.json");
        
        let mut manager = BookmarkManager::load(file_path).unwrap();
        
        manager.add("Test".to_string(), PathBuf::from("/test")).unwrap();
        assert_eq!(manager.count(), 1);
        
        manager.remove("Test").unwrap();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_remove_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bookmarks.json");
        
        let mut manager = BookmarkManager::load(file_path).unwrap();
        let result = manager.remove("NonExistent");
        
        assert!(result.is_err());
    }

    #[test]
    fn test_rename_bookmark() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bookmarks.json");
        
        let mut manager = BookmarkManager::load(file_path).unwrap();
        
        manager.add("OldName".to_string(), PathBuf::from("/test")).unwrap();
        manager.rename("OldName", "NewName".to_string()).unwrap();
        
        assert!(manager.get("OldName").is_none());
        assert!(manager.get("NewName").is_some());
    }

    #[test]
    fn test_rename_to_existing_name() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bookmarks.json");
        
        let mut manager = BookmarkManager::load(file_path).unwrap();
        
        manager.add("Name1".to_string(), PathBuf::from("/path1")).unwrap();
        manager.add("Name2".to_string(), PathBuf::from("/path2")).unwrap();
        
        let result = manager.rename("Name1", "Name2".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_sorted() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bookmarks.json");
        
        let mut manager = BookmarkManager::load(file_path).unwrap();
        
        manager.add("Charlie".to_string(), PathBuf::from("/c")).unwrap();
        manager.add("Alice".to_string(), PathBuf::from("/a")).unwrap();
        manager.add("Bob".to_string(), PathBuf::from("/b")).unwrap();
        
        let all = manager.get_all();
        assert_eq!(all[0].name, "Alice");
        assert_eq!(all[1].name, "Bob");
        assert_eq!(all[2].name, "Charlie");
    }

    #[test]
    fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bookmarks.json");
        
        {
            let mut manager = BookmarkManager::load(file_path.clone()).unwrap();
            manager.add("Test".to_string(), PathBuf::from("/test")).unwrap();
        }
        
        // Load again and verify persistence
        let manager = BookmarkManager::load(file_path).unwrap();
        assert_eq!(manager.count(), 1);
        assert!(manager.get("Test").is_some());
    }

    #[test]
    fn test_access_updates_timestamp() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bookmarks.json");
        
        let mut manager = BookmarkManager::load(file_path).unwrap();
        manager.add("Test".to_string(), PathBuf::from("/test")).unwrap();
        
        let original_time = manager.get("Test").unwrap().last_accessed;
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        manager.access("Test").unwrap();
        let updated_time = manager.get("Test").unwrap().last_accessed;
        
        assert!(updated_time > original_time);
    }

    // TASK-009: Edge case tests
    #[test]
    fn test_max_bookmarks_limit() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bookmarks.json");
        
        let mut manager = BookmarkManager::load(file_path).unwrap();
        
        // Add maximum allowed bookmarks
        for i in 0..50 {
            manager.add(format!("Bookmark{}", i), PathBuf::from(format!("/path{}", i))).unwrap();
        }
        
        assert_eq!(manager.count(), 50);
        
        // Try to add one more - should fail
        let result = manager.add("ExtraBookmark".to_string(), PathBuf::from("/extra"));
        assert!(result.is_err());
        assert_eq!(manager.count(), 50);
    }

    #[test]
    fn test_sanitize_bookmark_name() {
        assert_eq!(sanitize_bookmark_name("Normal Name"), "Normal Name");
        assert_eq!(sanitize_bookmark_name("Name/With\\Slash"), "Name_With_Slash");
        assert_eq!(sanitize_bookmark_name("Name:With*Invalid?Chars"), "Name_With_Invalid_Chars");
        assert_eq!(sanitize_bookmark_name("  Spaces  "), "Spaces");
        assert_eq!(sanitize_bookmark_name("Name<With>Pipes|"), "Name_With_Pipes_");
        assert_eq!(sanitize_bookmark_name("Name\"With\"Quotes"), "Name_With_Quotes");
        
        // Test length limit
        let long_name = "a".repeat(150);
        assert_eq!(sanitize_bookmark_name(&long_name).len(), 100);
    }

    #[test]
    fn test_empty_name_sanitization() {
        assert_eq!(sanitize_bookmark_name(""), "");
        assert_eq!(sanitize_bookmark_name("   "), "");
        assert_eq!(sanitize_bookmark_name("\n\t"), "");
    }

    #[test]
    fn test_corrupt_file_recovery() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bookmarks.json");
        
        // Create a corrupt JSON file
        std::fs::write(&file_path, "{ invalid json ").unwrap();
        
        // Load should fail gracefully and not panic
        let result = BookmarkManager::load(file_path.clone());
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_directory_for_bookmark_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("deep/nested/path/bookmarks.json");
        
        // Should create directories automatically
        let manager = BookmarkManager::load(file_path.clone()).unwrap();
        assert_eq!(manager.count(), 0);
        assert!(file_path.exists());
    }
}
