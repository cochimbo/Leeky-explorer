use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a bookmarked directory for quick access
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bookmark {
    /// User-friendly display name for the bookmark
    pub name: String,
    /// Absolute path to the bookmarked directory
    pub path: PathBuf,
    /// Timestamp when the bookmark was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the bookmark was last accessed
    pub last_accessed: DateTime<Utc>,
}

impl Bookmark {
    /// Create a new bookmark with the given name and path
    pub fn new(name: String, path: PathBuf) -> Self {
        let now = Utc::now();
        Self {
            name,
            path,
            created_at: now,
            last_accessed: now,
        }
    }

    /// Update the last_accessed timestamp to now
    pub fn access(&mut self) {
        self.last_accessed = Utc::now();
    }

    /// Check if the bookmarked path still exists
    pub fn path_exists(&self) -> bool {
        self.path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmark_creation() {
        let name = "Test Bookmark".to_string();
        let path = PathBuf::from("/test/path");
        
        let bookmark = Bookmark::new(name.clone(), path.clone());
        
        assert_eq!(bookmark.name, name);
        assert_eq!(bookmark.path, path);
        assert_eq!(bookmark.created_at, bookmark.last_accessed);
    }

    #[test]
    fn test_bookmark_access() {
        let mut bookmark = Bookmark::new("Test".to_string(), PathBuf::from("/test"));
        let original_access = bookmark.last_accessed;
        
        // Sleep briefly to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(10));
        bookmark.access();
        
        assert!(bookmark.last_accessed > original_access);
    }

    #[test]
    fn test_bookmark_serialization() {
        let bookmark = Bookmark::new("Test".to_string(), PathBuf::from("/test/path"));
        
        // Serialize to JSON
        let json = serde_json::to_string(&bookmark).unwrap();
        
        // Deserialize back
        let deserialized: Bookmark = serde_json::from_str(&json).unwrap();
        
        assert_eq!(bookmark, deserialized);
    }

    #[test]
    fn test_bookmark_path_exists() {
        // Test with non-existent path
        let bookmark = Bookmark::new("Test".to_string(), PathBuf::from("/nonexistent/path"));
        assert!(!bookmark.path_exists());
        
        // Test with existing path (current directory should exist)
        let current_dir = std::env::current_dir().unwrap();
        let bookmark_existing = Bookmark::new("Current".to_string(), current_dir);
        assert!(bookmark_existing.path_exists());
    }
}
