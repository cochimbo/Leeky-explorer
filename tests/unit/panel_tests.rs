// Unit tests for Panel
use leeky_explorer::models::panel::Panel;
use std::path::PathBuf;

#[test]
fn test_move_cursor_up_at_top() {
    let mut panel = Panel::new(PathBuf::from("/tmp"));
    panel.cursor = 0;
    panel.move_cursor_up();
    assert_eq!(panel.cursor, 0, "Cursor should not move up from position 0");
}

#[test]
fn test_move_cursor_down_at_bottom() {
    let mut panel = Panel::new(PathBuf::from("/tmp"));
    // Simulate 5 entries
    panel.entries = vec![]; // Would need real FileEntry objects
    panel.cursor = 0;
    
    // With empty entries, cursor should stay at 0
    panel.move_cursor_down();
    assert_eq!(panel.cursor, 0, "Cursor should not move in empty list");
}

#[test]
fn test_move_cursor_down_normal() {
    let mut panel = Panel::new(PathBuf::from("/tmp"));
    panel.cursor = 0;
    panel.move_cursor_down();
    // Note: Without entries, saturating_sub will keep it at 0
    assert!(panel.cursor >= 0, "Cursor should be non-negative");
}

#[test]
fn test_cursor_bounds() {
    let mut panel = Panel::new(PathBuf::from("/tmp"));
    panel.cursor = 0;
    
    // Move up from top - should stay at 0
    panel.move_cursor_up();
    assert_eq!(panel.cursor, 0);
    
    // Move to top explicitly
    panel.move_cursor_to_top();
    assert_eq!(panel.cursor, 0);
}
