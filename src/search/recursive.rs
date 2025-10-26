use std::path::{Path, PathBuf};
use std::time::SystemTime;
use anyhow::Result;

/// Represents a single search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_name: String,
    pub full_path: PathBuf,
    pub relative_path: PathBuf,
    pub file_size: u64,
    pub modified_time: SystemTime,
}

impl SearchResult {
    /// Create a new search result from a full path and root directory
    pub fn new(full_path: PathBuf, root: &Path) -> Result<Self> {
        let metadata = std::fs::metadata(&full_path)?;
        
        let file_name = full_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        
        let relative_path = full_path
            .strip_prefix(root)
            .unwrap_or(&full_path)
            .to_path_buf();
        
        let file_size = metadata.len();
        let modified_time = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        
        Ok(Self {
            file_name,
            full_path,
            relative_path,
            file_size,
            modified_time,
        })
    }
    
    /// Check if this result matches a search pattern
    pub fn matches_pattern(&self, pattern: &str, case_sensitive: bool) -> bool {
        let file_name = if case_sensitive {
            self.file_name.clone()
        } else {
            self.file_name.to_lowercase()
        };
        
        let pattern = if case_sensitive {
            pattern.to_string()
        } else {
            pattern.to_lowercase()
        };
        
        // Check if it's a glob pattern
        if SearchState::is_glob_pattern(&pattern) {
            // Use glob matching
            if let Ok(glob_pattern) = glob::Pattern::new(&pattern) {
                return glob_pattern.matches(&file_name);
            }
        }
        
        // Simple substring match
        file_name.contains(&pattern)
    }
}

/// Represents the state of a search operation
pub struct SearchState {
    pub query: String,
    pub root_path: PathBuf,
    pub results: Vec<SearchResult>,
    pub is_running: bool,
    pub files_scanned: usize,
    pub use_glob: bool,
    pub max_depth: usize,
}

impl SearchState {
    /// Create a new search state
    pub fn new(query: String, root_path: PathBuf) -> Self {
        let use_glob = Self::is_glob_pattern(&query);
        
        Self {
            query,
            root_path,
            results: Vec::new(),
            is_running: false,
            files_scanned: 0,
            use_glob,
            max_depth: 20, // Default max depth
        }
    }
    
    /// Check if a query string is a glob pattern
    pub fn is_glob_pattern(query: &str) -> bool {
        query.contains('*') || query.contains('?') || query.contains('[') || query.contains(']')
    }
}

/// Main recursive search engine
pub struct RecursiveSearcher {
    // Placeholder for now - will implement in TASK-038
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_search_result_creation() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();
        
        let result = SearchResult::new(file_path.clone(), temp.path()).unwrap();
        
        assert_eq!(result.file_name, "test.txt");
        assert_eq!(result.full_path, file_path);
        assert_eq!(result.relative_path, PathBuf::from("test.txt"));
        assert_eq!(result.file_size, 12); // "test content" = 12 bytes
    }
    
    #[test]
    fn test_glob_pattern_detection() {
        assert!(SearchState::is_glob_pattern("*.rs"));
        assert!(SearchState::is_glob_pattern("file?.txt"));
        assert!(SearchState::is_glob_pattern("test[123].log"));
        assert!(!SearchState::is_glob_pattern("simple"));
        assert!(!SearchState::is_glob_pattern("file.txt"));
    }
    
    #[test]
    fn test_matches_pattern_simple() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test_file.txt");
        fs::write(&file_path, "content").unwrap();
        
        let result = SearchResult::new(file_path, temp.path()).unwrap();
        
        // Case insensitive by default
        assert!(result.matches_pattern("test", false));
        assert!(result.matches_pattern("TEST", false));
        assert!(result.matches_pattern("file", false));
        assert!(!result.matches_pattern("missing", false));
        
        // Case sensitive
        assert!(result.matches_pattern("test", true));
        assert!(!result.matches_pattern("TEST", true));
    }
    
    #[test]
    fn test_matches_pattern_glob() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.rs");
        fs::write(&file_path, "content").unwrap();
        
        let result = SearchResult::new(file_path, temp.path()).unwrap();
        
        assert!(result.matches_pattern("*.rs", false));
        assert!(result.matches_pattern("test.*", false));
        assert!(result.matches_pattern("test.?s", false));
        assert!(!result.matches_pattern("*.txt", false));
    }
    
    #[test]
    fn test_search_state_initialization() {
        let state = SearchState::new("*.rs".to_string(), PathBuf::from("/test"));
        
        assert_eq!(state.query, "*.rs");
        assert_eq!(state.root_path, PathBuf::from("/test"));
        assert_eq!(state.results.len(), 0);
        assert!(!state.is_running);
        assert_eq!(state.files_scanned, 0);
        assert!(state.use_glob);
        assert_eq!(state.max_depth, 20);
    }
    
    #[test]
    fn test_search_state_simple_query() {
        let state = SearchState::new("simple".to_string(), PathBuf::from("/test"));
        
        assert!(!state.use_glob);
    }
}
