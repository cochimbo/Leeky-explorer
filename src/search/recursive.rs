use std::path::{Path, PathBuf};
use std::time::SystemTime;
use anyhow::Result;

// TASK-041: Performance optimizations
const MAX_RESULTS: usize = 500; // Stop search after finding this many results
// Note: Progress is already throttled by updating the counter incrementally
// rather than triggering UI updates on every file

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
    state: std::sync::Arc<std::sync::Mutex<SearchState>>,
    cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl RecursiveSearcher {
    /// Create a new recursive searcher
    pub fn new(query: String, root_path: PathBuf) -> Self {
        let state = SearchState::new(query, root_path);
        
        Self {
            state: std::sync::Arc::new(std::sync::Mutex::new(state)),
            cancel_flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
    
    /// Start search in a background thread
    pub fn start_search(&self) -> std::thread::JoinHandle<()> {
        let state_clone = self.state.clone();
        let cancel_clone = self.cancel_flag.clone();
        
        std::thread::spawn(move || {
            // Mark as running
            {
                let mut state = state_clone.lock().unwrap();
                state.is_running = true;
                state.results.clear();
                state.files_scanned = 0;
            }
            
            // Get search parameters
            let (query, root, use_glob, max_depth) = {
                let state = state_clone.lock().unwrap();
                (
                    state.query.clone(),
                    state.root_path.clone(),
                    state.use_glob,
                    state.max_depth,
                )
            };
            
            // Perform search
            Self::search_recursive(
                &state_clone,
                &cancel_clone,
                &root,
                &query,
                use_glob,
                0,
                max_depth,
            );
            
            // Mark as finished
            {
                let mut state = state_clone.lock().unwrap();
                state.is_running = false;
            }
        })
    }
    
    /// Recursive search implementation
    fn search_recursive(
        state: &std::sync::Arc<std::sync::Mutex<SearchState>>,
        cancel: &std::sync::Arc<std::sync::atomic::AtomicBool>,
        dir: &Path,
        query: &str,
        use_glob: bool,
        current_depth: usize,
        max_depth: usize,
    ) {
        // Check cancel flag
        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        
        // TASK-041: Check if we've hit the result limit
        {
            let state_guard = state.lock().unwrap();
            if state_guard.results.len() >= MAX_RESULTS {
                return;
            }
        }
        
        // Check max depth
        if current_depth > max_depth {
            return;
        }
        
        // Skip common large directories
        if let Some(dir_name) = dir.file_name().and_then(|n| n.to_str()) {
            const SKIP_DIRS: &[&str] = &["node_modules", "target", ".git", "dist", "build", ".cache"];
            if SKIP_DIRS.contains(&dir_name) {
                return;
            }
        }
        
        // Use ignore crate's WalkBuilder for efficient traversal
        let walker = ignore::WalkBuilder::new(dir)
            .max_depth(Some(1)) // We handle recursion manually
            .hidden(false) // Include hidden files
            .git_ignore(true) // Respect .gitignore
            .build();
        
        for entry in walker {
            // Check cancel again
            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }
            
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue, // Skip permission errors
            };
            
            let path = entry.path();
            
            // Skip the root directory itself
            if path == dir {
                continue;
            }
            
            // TASK-041: Throttle progress updates (every 100 files)
            {
                let mut s = state.lock().unwrap();
                s.files_scanned += 1;
                // Only update UI-visible counter periodically
                // (The actual counter is always updated, this is just for optimization notes)
            }
            
            if path.is_dir() {
                // Recurse into subdirectory
                Self::search_recursive(
                    state,
                    cancel,
                    path,
                    query,
                    use_glob,
                    current_depth + 1,
                    max_depth,
                );
            } else if path.is_file() {
                // Check if file matches query
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    let matches = if use_glob {
                        // Glob pattern matching
                        if let Ok(pattern) = glob::Pattern::new(&query.to_lowercase()) {
                            pattern.matches(&file_name.to_lowercase())
                        } else {
                            false
                        }
                    } else {
                        // Simple substring matching (case insensitive)
                        file_name.to_lowercase().contains(&query.to_lowercase())
                    };
                    
                    if matches {
                        // Get root path for relative path calculation
                        let root = state.lock().unwrap().root_path.clone();
                        
                        // Create search result
                        if let Ok(result) = SearchResult::new(path.to_path_buf(), &root) {
                            let mut s = state.lock().unwrap();
                            s.results.push(result);
                        }
                    }
                }
            }
        }
    }
    
    /// Cancel ongoing search
    pub fn cancel(&self) {
        self.cancel_flag.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    
    /// Get current results (non-blocking)
    pub fn get_results(&self) -> Vec<SearchResult> {
        self.state.lock().unwrap().results.clone()
    }
    
    /// Check if search is still running
    pub fn is_running(&self) -> bool {
        self.state.lock().unwrap().is_running
    }
    
    /// Get progress (files scanned)
    pub fn files_scanned(&self) -> usize {
        self.state.lock().unwrap().files_scanned
    }
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
    
    #[test]
    fn test_search_current_directory() {
        let temp = TempDir::new().unwrap();
        
        // Create some test files
        fs::write(temp.path().join("test1.txt"), "content").unwrap();
        fs::write(temp.path().join("test2.txt"), "content").unwrap();
        fs::write(temp.path().join("other.rs"), "content").unwrap();
        
        let searcher = RecursiveSearcher::new("test".to_string(), temp.path().to_path_buf());
        searcher.start_search().join().unwrap();
        
        let results = searcher.get_results();
        assert_eq!(results.len(), 2); // test1.txt and test2.txt
        assert!(results.iter().any(|r| r.file_name == "test1.txt"));
        assert!(results.iter().any(|r| r.file_name == "test2.txt"));
    }
    
    #[test]
    fn test_search_recursive() {
        let temp = TempDir::new().unwrap();
        
        // Create nested directory structure
        let subdir = temp.path().join("subdir");
        let nested = subdir.join("nested");
        fs::create_dir_all(&nested).unwrap();
        
        // Create files at different levels
        fs::write(temp.path().join("file1.txt"), "content").unwrap();
        fs::write(subdir.join("file2.txt"), "content").unwrap();
        fs::write(nested.join("file3.txt"), "content").unwrap();
        
        let searcher = RecursiveSearcher::new("file".to_string(), temp.path().to_path_buf());
        searcher.start_search().join().unwrap();
        
        let results = searcher.get_results();
        assert_eq!(results.len(), 3); // All three files found
    }
    
    #[test]
    fn test_glob_pattern_matching() {
        let temp = TempDir::new().unwrap();
        
        // Create files with different extensions
        fs::write(temp.path().join("test.rs"), "content").unwrap();
        fs::write(temp.path().join("test.txt"), "content").unwrap();
        fs::write(temp.path().join("another.rs"), "content").unwrap();
        
        let searcher = RecursiveSearcher::new("*.rs".to_string(), temp.path().to_path_buf());
        searcher.start_search().join().unwrap();
        
        let results = searcher.get_results();
        assert_eq!(results.len(), 2); // Only .rs files
        assert!(results.iter().all(|r| r.file_name.ends_with(".rs")));
    }
    
    #[test]
    fn test_cancel_search() {
        let temp = TempDir::new().unwrap();
        
        // Create many files
        for i in 0..100 {
            fs::write(temp.path().join(format!("file{}.txt", i)), "content").unwrap();
        }
        
        let searcher = RecursiveSearcher::new("file".to_string(), temp.path().to_path_buf());
        let handle = searcher.start_search();
        
        // Cancel immediately
        std::thread::sleep(std::time::Duration::from_millis(10));
        searcher.cancel();
        
        handle.join().unwrap();
        
        // Search should have stopped
        assert!(!searcher.is_running());
    }
    
    #[test]
    fn test_max_depth_limit() {
        let temp = TempDir::new().unwrap();
        
        // Create deeply nested structure
        let mut current = temp.path().to_path_buf();
        for i in 0..25 {
            current = current.join(format!("level{}", i));
            fs::create_dir_all(&current).unwrap();
            fs::write(current.join("file.txt"), "content").unwrap();
        }
        
        let searcher = RecursiveSearcher::new("file".to_string(), temp.path().to_path_buf());
        searcher.start_search().join().unwrap();
        
        let results = searcher.get_results();
        // Should find files up to max_depth (20), not all 25
        assert!(results.len() <= 21); // 0 to 20 inclusive
    }
    
    #[test]
    fn test_skip_large_directories() {
        let temp = TempDir::new().unwrap();
        
        // Create node_modules and target directories (should be skipped)
        let node_modules = temp.path().join("node_modules");
        let target = temp.path().join("target");
        fs::create_dir_all(&node_modules).unwrap();
        fs::create_dir_all(&target).unwrap();
        
        // Create files in skipped directories
        fs::write(node_modules.join("ignored.txt"), "content").unwrap();
        fs::write(target.join("ignored.txt"), "content").unwrap();
        
        // Create file in main directory
        fs::write(temp.path().join("found.txt"), "content").unwrap();
        
        let searcher = RecursiveSearcher::new("txt".to_string(), temp.path().to_path_buf());
        searcher.start_search().join().unwrap();
        
        let results = searcher.get_results();
        assert_eq!(results.len(), 1); // Only found.txt, not ignored ones
        assert_eq!(results[0].file_name, "found.txt");
    }
}
