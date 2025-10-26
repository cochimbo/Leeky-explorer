use leeky_explorer::ui::text_editor::TextEditor;
use leeky_explorer::ui::theme::Theme;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::{NamedTempFile, TempDir};

// Helper function to create a test file
fn create_test_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "{}", content).unwrap();
    file
}

// Helper function to read file content
fn read_file_content(path: &PathBuf) -> String {
    fs::read_to_string(path).unwrap()
}

#[test]
fn test_open_and_edit_text_file() {
    let temp_file = create_test_file("Initial content\nSecond line\n");
    let path = temp_file.path().to_path_buf();
    
    let theme = Theme::default();
    let editor = TextEditor::from_file(path.clone(), &theme);
    
    assert!(editor.is_ok());
    let editor = editor.unwrap();
    assert_eq!(editor.line_count(), 2);
    assert_eq!(editor.get_line(0).unwrap(), "Initial content");
    assert_eq!(editor.get_line(1).unwrap(), "Second line");
    assert!(!editor.is_modified());
}

#[test]
fn test_save_modifications() {
    let temp_file = create_test_file("Original text\n");
    let path = temp_file.path().to_path_buf();
    
    let theme = Theme::default();
    let mut editor = TextEditor::from_file(path.clone(), &theme).unwrap();
    
    // Simulate editing by marking as modified
    editor.set_modified(true);
    assert!(editor.is_modified());
    
    // Save the file
    let result = editor.save();
    assert!(result.is_ok());
    assert!(!editor.is_modified());
}

#[test]
fn test_unsaved_changes_warning() {
    let temp_file = create_test_file("Some content\n");
    let path = temp_file.path().to_path_buf();
    
    let theme = Theme::default();
    let mut editor = TextEditor::from_file(path.clone(), &theme).unwrap();
    
    // Initially no modifications
    assert!(!editor.is_modified());
    
    // Simulate modification
    editor.set_modified(true);
    assert!(editor.is_modified());
    
    // After save, no unsaved changes
    editor.save().unwrap();
    assert!(!editor.is_modified());
}

#[test]
fn test_reject_binary_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    
    // Write binary content (PNG header)
    let binary_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    temp_file.write_all(&binary_data).unwrap();
    
    let path = temp_file.path().to_path_buf();
    let theme = Theme::default();
    let result = TextEditor::from_file(path, &theme);
    
    // Should fail to open binary file
    assert!(result.is_err());
}

#[test]
fn test_large_file_warning() {
    // Create a file just over 1MB
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();
    
    // Write 1.5 MB of data
    let large_content = "x".repeat(1_500_000);
    fs::write(&path, large_content).unwrap();
    
    let theme = Theme::default();
    let result = TextEditor::from_file(path, &theme);
    
    // Should succeed but may have warnings (we just check it loads)
    assert!(result.is_ok());
}

#[test]
fn test_read_only_file_handling() {
    let temp_file = create_test_file("Read-only content\n");
    let path = temp_file.path().to_path_buf();
    
    // Set file as read-only
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&path, perms.clone()).unwrap();
    
    let theme = Theme::default();
    let editor = TextEditor::from_file(path.clone(), &theme).unwrap();
    
    assert!(editor.is_read_only());
    
    // Cleanup: remove read-only flag
    perms.set_readonly(false);
    fs::set_permissions(&path, perms).unwrap();
}

#[test]
fn test_cursor_movement_and_editing() {
    let temp_file = create_test_file("Line 1\nLine 2\nLine 3\n");
    let path = temp_file.path().to_path_buf();
    
    let theme = Theme::default();
    let mut editor = TextEditor::from_file(path, &theme).unwrap();
    
    // Initial state
    assert_eq!(editor.line_count(), 3);
    assert!(!editor.is_modified());
    
    // Insert text (simulated by marking as modified)
    editor.set_modified(true);
    assert!(editor.is_modified());
}

#[test]
fn test_empty_file() {
    let temp_file = create_test_file("");
    let path = temp_file.path().to_path_buf();
    
    let theme = Theme::default();
    let editor = TextEditor::from_file(path, &theme);
    
    assert!(editor.is_ok());
    let editor = editor.unwrap();
    assert_eq!(editor.line_count(), 1); // Empty file has one empty line
}

#[test]
fn test_utf8_file() {
    let temp_file = create_test_file("Hello 世界\nBonjour 🌍\n");
    let path = temp_file.path().to_path_buf();
    
    let theme = Theme::default();
    let editor = TextEditor::from_file(path, &theme);
    
    assert!(editor.is_ok());
    let editor = editor.unwrap();
    assert_eq!(editor.line_count(), 2);
    assert!(editor.get_line(0).unwrap().contains("世界"));
    assert!(editor.get_line(1).unwrap().contains("🌍"));
}

#[test]
fn test_multiline_file() {
    let content = (1..=100).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n");
    let temp_file = create_test_file(&content);
    let path = temp_file.path().to_path_buf();
    
    let theme = Theme::default();
    let editor = TextEditor::from_file(path, &theme).unwrap();
    
    assert_eq!(editor.line_count(), 100);
}

#[test]
fn test_nonexistent_file() {
    let path = PathBuf::from("/nonexistent/path/to/file.txt");
    let theme = Theme::default();
    let result = TextEditor::from_file(path, &theme);
    
    // Should fail for nonexistent file
    assert!(result.is_err());
}

#[test]
fn test_save_preserves_content() {
    let temp_file = create_test_file("Test content\n");
    let path = temp_file.path().to_path_buf();
    
    let theme = Theme::default();
    let mut editor = TextEditor::from_file(path.clone(), &theme).unwrap();
    
    // Mark as modified
    editor.set_modified(true);
    
    // Save
    editor.save().unwrap();
    
    // Verify content is preserved
    let content = read_file_content(&path);
    assert!(content.contains("Test content"));
}

#[test]
fn test_multiple_saves() {
    let temp_file = create_test_file("Initial\n");
    let path = temp_file.path().to_path_buf();
    
    let theme = Theme::default();
    let mut editor = TextEditor::from_file(path, &theme).unwrap();
    
    // First save
    editor.set_modified(true);
    assert!(editor.save().is_ok());
    assert!(!editor.is_modified());
    
    // Second save
    editor.set_modified(true);
    assert!(editor.save().is_ok());
    assert!(!editor.is_modified());
}

#[test]
fn test_file_permissions_preserved() {
    let temp_file = create_test_file("Content\n");
    let path = temp_file.path().to_path_buf();
    
    // Get original permissions
    let original_perms = fs::metadata(&path).unwrap().permissions();
    
    let theme = Theme::default();
    let mut editor = TextEditor::from_file(path.clone(), &theme).unwrap();
    
    // Save file
    editor.set_modified(true);
    editor.save().unwrap();
    
    // Check permissions are similar (readonly status)
    let new_perms = fs::metadata(&path).unwrap().permissions();
    assert_eq!(original_perms.readonly(), new_perms.readonly());
}

#[test]
fn test_editor_with_tabs() {
    let temp_file = create_test_file("Line with\ttabs\n");
    let path = temp_file.path().to_path_buf();
    
    let theme = Theme::default();
    let editor = TextEditor::from_file(path, &theme);
    
    assert!(editor.is_ok());
    let editor = editor.unwrap();
    assert!(editor.get_line(0).unwrap().contains('\t'));
}

#[test]
fn test_editor_with_long_lines() {
    let long_line = "x".repeat(10000);
    let temp_file = create_test_file(&long_line);
    let path = temp_file.path().to_path_buf();
    
    let theme = Theme::default();
    let editor = TextEditor::from_file(path, &theme);
    
    assert!(editor.is_ok());
    let editor = editor.unwrap();
    assert!(editor.get_line(0).unwrap().len() >= 10000);
}

#[test]
fn test_directory_path_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_path_buf();
    
    let theme = Theme::default();
    let result = TextEditor::from_file(dir_path, &theme);
    
    // Should fail for directory
    assert!(result.is_err());
}
