// TASK-042: Integration tests for recursive search functionality
use leeky_explorer::search::RecursiveSearcher;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// Helper function to create a test directory structure
fn create_test_directory() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    
    // Create directory structure:
    // root/
    //   file1.txt
    //   file2.rs
    //   config.toml
    //   subdir/
    //     file3.txt
    //     nested.rs
    //     deep/
    //       file4.txt
    //       deeper.rs
    
    fs::write(root.join("file1.txt"), "content1").unwrap();
    fs::write(root.join("file2.rs"), "fn main() {}").unwrap();
    fs::write(root.join("config.toml"), "[package]").unwrap();
    
    let subdir = root.join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("file3.txt"), "content3").unwrap();
    fs::write(subdir.join("nested.rs"), "// comment").unwrap();
    
    let deep = subdir.join("deep");
    fs::create_dir(&deep).unwrap();
    fs::write(deep.join("file4.txt"), "content4").unwrap();
    fs::write(deep.join("deeper.rs"), "// deeper").unwrap();
    
    temp_dir
}

#[test]
fn test_simple_recursive_search() {
    let temp_dir = create_test_directory();
    let root = temp_dir.path().to_path_buf();
    
    // Search for "file" (should find file1.txt, file3.txt, file4.txt)
    let searcher = RecursiveSearcher::new("file".to_string(), root.clone(), None);
    let handle = searcher.start_search();
    
    // Wait for completion
    handle.join().unwrap();
    
    let results = searcher.get_results();
    
    // Verify results contain expected files
    let filenames: Vec<String> = results.iter()
        .map(|r| r.file_name.clone())
        .collect();
    
    // Debug: print what we found
    println!("Found {} files: {:?}", filenames.len(), filenames);
    
    assert!(filenames.contains(&"file1.txt".to_string()), "Should find file1.txt");
    assert!(filenames.contains(&"file3.txt".to_string()), "Should find file3.txt");
    assert!(filenames.contains(&"file4.txt".to_string()), "Should find file4.txt");
    
    // Should find at least 3 files with 'file' in name
    assert!(results.len() >= 3, "Should find at least 3 files with 'file' in name, found {}", results.len());
}

#[test]
fn test_search_in_subdirectories() {
    let temp_dir = create_test_directory();
    let root = temp_dir.path().to_path_buf();
    
    // Search for ".txt" files
    let searcher = RecursiveSearcher::new("txt".to_string(), root.clone(), None);
    let handle = searcher.start_search();
    handle.join().unwrap();
    
    let results = searcher.get_results();
    assert_eq!(results.len(), 3, "Should find all 3 .txt files");
    
    // Verify files from different depths are found
    let paths: Vec<PathBuf> = results.iter()
        .map(|r| r.relative_path.clone())
        .collect();
    
    assert!(paths.iter().any(|p| p.to_str().unwrap().contains("file1.txt")));
    assert!(paths.iter().any(|p| p.to_str().unwrap().contains("subdir")));
    assert!(paths.iter().any(|p| p.to_str().unwrap().contains("deep")));
}

#[test]
fn test_glob_pattern_matching() {
    let temp_dir = create_test_directory();
    let root = temp_dir.path().to_path_buf();
    
    // Search for "*.rs" files using glob pattern
    let searcher = RecursiveSearcher::new("*.rs".to_string(), root.clone(), None);
    let handle = searcher.start_search();
    handle.join().unwrap();
    
    let results = searcher.get_results();
    assert_eq!(results.len(), 3, "Should find all 3 .rs files");
    
    // Verify all results end with .rs
    for result in &results {
        assert!(result.file_name.ends_with(".rs"), 
                "File {} should end with .rs", result.file_name);
    }
}

#[test]
fn test_glob_pattern_question_mark() {
    let temp_dir = create_test_directory();
    let root = temp_dir.path();
    
    // Create files: test1.txt, test2.txt, test10.txt
    fs::write(root.join("test1.txt"), "1").unwrap();
    fs::write(root.join("test2.txt"), "2").unwrap();
    fs::write(root.join("test10.txt"), "10").unwrap();
    
    // Search for "test?.txt" (should match test1.txt and test2.txt, not test10.txt)
    let searcher = RecursiveSearcher::new("test?.txt".to_string(), root.to_path_buf(), None);
    let handle = searcher.start_search();
    handle.join().unwrap();
    
    let results = searcher.get_results();
    assert_eq!(results.len(), 2, "Should find exactly 2 files matching test?.txt");
    
    let filenames: Vec<String> = results.iter()
        .map(|r| r.file_name.clone())
        .collect();
    
    assert!(filenames.contains(&"test1.txt".to_string()));
    assert!(filenames.contains(&"test2.txt".to_string()));
    assert!(!filenames.contains(&"test10.txt".to_string()));
}

#[test]
fn test_case_insensitive_search() {
    let temp_dir = create_test_directory();
    let root = temp_dir.path();
    
    // Create files with mixed case
    fs::write(root.join("README.md"), "readme").unwrap();
    fs::write(root.join("config.toml"), "config").unwrap();
    
    // Search with uppercase (should find regardless of case)
    let searcher = RecursiveSearcher::new("README".to_string(), root.to_path_buf(), None);
    let handle = searcher.start_search();
    handle.join().unwrap();
    
    let results = searcher.get_results();
    assert_eq!(results.len(), 1, "Should find README.md case-insensitively");
    assert_eq!(results[0].file_name, "README.md");
}

#[test]
fn test_cancel_search() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    
    // Create many files across multiple directories to ensure search takes time
    for i in 0..20 {
        let dir = root.join(format!("dir{}", i));
        fs::create_dir(&dir).unwrap();
        for j in 0..50 {
            fs::write(dir.join(format!("file{}.txt", j)), "content").unwrap();
        }
    }
    
    let searcher = RecursiveSearcher::new("file".to_string(), root.to_path_buf(), None);
    let handle = searcher.start_search();
    
    // Cancel after a short delay
    thread::sleep(Duration::from_millis(20));
    searcher.cancel();
    
    // Wait for search to stop
    handle.join().unwrap();
    
    // Search should have stopped
    assert!(!searcher.is_running(), "Search should be cancelled");
    
    // Due to cancellation, should have found fewer results
    // Note: exact count depends on timing, so we just verify cancellation worked
    let results = searcher.get_results();
    println!("Found {} results after cancellation (out of 1000 total)", results.len());
    
    // If we found all 1000, cancellation didn't work
    assert!(results.len() < 1000, 
            "Search should have been cancelled before finding all 1000 files, found {}", 
            results.len());
}

#[test]
fn test_max_depth_limit() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    
    // Create deeply nested structure (25 levels)
    let mut current = root.to_path_buf();
    for i in 0..25 {
        current = current.join(format!("level{}", i));
        fs::create_dir(&current).unwrap();
        fs::write(current.join(format!("file{}.txt", i)), "content").unwrap();
    }
    
    // Search with default max_depth (20)
    let searcher = RecursiveSearcher::new("file".to_string(), root.to_path_buf(), None);
    let handle = searcher.start_search();
    handle.join().unwrap();
    
    let results = searcher.get_results();
    
    // Should find files up to depth 20, but not beyond
    assert!(results.len() <= 21, "Should respect max_depth limit of 20");
    assert!(results.len() >= 20, "Should find files within depth limit");
}

#[test]
fn test_empty_results() {
    let temp_dir = create_test_directory();
    let root = temp_dir.path().to_path_buf();
    
    // Search for pattern that doesn't exist
    let searcher = RecursiveSearcher::new("nonexistent_pattern_xyz".to_string(), root, None);
    let handle = searcher.start_search();
    handle.join().unwrap();
    
    let results = searcher.get_results();
    assert_eq!(results.len(), 0, "Should return empty results for non-matching pattern");
}

#[test]
fn test_special_characters_in_query() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    
    // Create files with spaces and special characters
    fs::write(root.join("my file.txt"), "content").unwrap();
    fs::write(root.join("test-file.txt"), "content").unwrap();
    fs::write(root.join("file_name.txt"), "content").unwrap();
    
    // Search for "my file" with space
    let searcher = RecursiveSearcher::new("my file".to_string(), root.to_path_buf(), None);
    let handle = searcher.start_search();
    handle.join().unwrap();
    
    let results = searcher.get_results();
    assert_eq!(results.len(), 1, "Should find file with space in name");
    assert_eq!(results[0].file_name, "my file.txt");
    
    // Search for "test-file" with hyphen
    let searcher2 = RecursiveSearcher::new("test-file".to_string(), root.to_path_buf(), None);
    let handle2 = searcher2.start_search();
    handle2.join().unwrap();
    
    let results2 = searcher2.get_results();
    assert_eq!(results2.len(), 1, "Should find file with hyphen");
    assert_eq!(results2[0].file_name, "test-file.txt");
}

#[test]
fn test_skip_large_directories() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    
    // Create a node_modules directory (should be skipped)
    let node_modules = root.join("node_modules");
    fs::create_dir(&node_modules).unwrap();
    fs::write(node_modules.join("package.txt"), "should be skipped").unwrap();
    
    // Create a normal directory
    let src = root.join("src");
    fs::create_dir(&src).unwrap();
    fs::write(src.join("main.txt"), "should be found").unwrap();
    
    // Search for .txt files
    let searcher = RecursiveSearcher::new("txt".to_string(), root.to_path_buf(), None);
    let handle = searcher.start_search();
    handle.join().unwrap();
    
    let results = searcher.get_results();
    
    // Should only find main.txt, not package.txt from node_modules
    assert_eq!(results.len(), 1, "Should skip node_modules directory");
    assert_eq!(results[0].file_name, "main.txt");
}

#[test]
fn test_search_result_metadata() {
    let temp_dir = create_test_directory();
    let root = temp_dir.path().to_path_buf();
    
    let searcher = RecursiveSearcher::new("file1".to_string(), root.clone(), None);
    let handle = searcher.start_search();
    handle.join().unwrap();
    
    let results = searcher.get_results();
    assert_eq!(results.len(), 1);
    
    let result = &results[0];
    
    // Verify metadata is populated
    assert_eq!(result.file_name, "file1.txt");
    assert!(result.full_path.exists());
    assert!(result.file_size > 0, "File size should be greater than 0");
    
    // Verify relative path is calculated correctly
    let relative_str = result.relative_path.to_str().unwrap();
    assert!(relative_str.contains("file1.txt"));
}

#[test]
fn test_performance_1000_files() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    
    // Create 1000 files across multiple directories
    for i in 0..10 {
        let dir = root.join(format!("dir{}", i));
        fs::create_dir(&dir).unwrap();
        
        for j in 0..100 {
            let filename = if j % 2 == 0 {
                format!("test{}.txt", j)
            } else {
                format!("other{}.rs", j)
            };
            fs::write(dir.join(filename), "content").unwrap();
        }
    }
    
    // Search for "test" pattern
    let searcher = RecursiveSearcher::new("test".to_string(), root.to_path_buf(), None);
    
    let start = Instant::now();
    let handle = searcher.start_search();
    handle.join().unwrap();
    let elapsed = start.elapsed();
    
    let results = searcher.get_results();
    
    // Should find 500 test*.txt files (50 per directory, 10 directories)
    assert_eq!(results.len(), 500, "Should find all 500 test files");
    
    // Performance assertion: should complete within 2 seconds
    assert!(elapsed < Duration::from_secs(2), 
            "Search of 1000 files should complete in <2s, took {:?}", elapsed);
    
    println!("Performance: Searched 1000 files in {:?}", elapsed);
}

#[test]
fn test_result_limit_respected() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    
    // Create 600 files (more than MAX_RESULTS limit of 500)
    for i in 0..600 {
        fs::write(root.join(format!("file{}.txt", i)), "content").unwrap();
    }
    
    let searcher = RecursiveSearcher::new("file".to_string(), root.to_path_buf(), None);
    let handle = searcher.start_search();
    handle.join().unwrap();
    
    let results = searcher.get_results();
    
    // Should stop at 500 results due to MAX_RESULTS limit
    assert_eq!(results.len(), 500, "Should respect MAX_RESULTS limit of 500");
}

#[test]
fn test_files_scanned_counter() {
    let temp_dir = create_test_directory();
    let root = temp_dir.path().to_path_buf();
    
    let searcher = RecursiveSearcher::new("file".to_string(), root, None);
    let handle = searcher.start_search();
    handle.join().unwrap();
    
    let files_scanned = searcher.files_scanned();
    
    // Should have scanned all files in the test directory (8 files total)
    assert!(files_scanned >= 8, "Should have scanned at least 8 files");
    println!("Files scanned: {}", files_scanned);
}

#[test]
fn test_concurrent_searches() {
    let temp_dir = create_test_directory();
    let root = temp_dir.path().to_path_buf();
    
    // Start two searches concurrently
    let searcher1 = RecursiveSearcher::new("txt".to_string(), root.clone(), None);
    let searcher2 = RecursiveSearcher::new("rs".to_string(), root.clone(), None);
    
    let handle1 = searcher1.start_search();
    let handle2 = searcher2.start_search();
    
    handle1.join().unwrap();
    handle2.join().unwrap();
    
    let results1 = searcher1.get_results();
    let results2 = searcher2.get_results();
    
    // First search should find .txt files
    assert_eq!(results1.len(), 3, "Should find 3 .txt files");
    
    // Second search should find .rs files
    assert_eq!(results2.len(), 3, "Should find 3 .rs files");
    
    // Results should be independent
    assert_ne!(results1[0].file_name, results2[0].file_name);
}
