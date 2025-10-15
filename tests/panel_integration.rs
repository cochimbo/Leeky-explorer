// Integration test for basic panel operations
use leeky_explorer::models::panel::Panel;
use std::path::PathBuf;

#[test]
fn test_panel_creation() {
    let home = dirs::home_dir().unwrap();
    let panel = Panel::new(home.clone());
    
    assert_eq!(panel.current_path, home);
    assert_eq!(panel.cursor, 0);
    assert_eq!(panel.scroll_offset, 0);
}

#[test]
fn test_cursor_movement() {
    let mut panel = Panel::new(PathBuf::from("."));
    
    // Test move up at top
    panel.cursor = 0;
    panel.move_cursor_up();
    assert_eq!(panel.cursor, 0);
    
    // Test move to top
    panel.cursor = 5;
    panel.move_cursor_to_top();
    assert_eq!(panel.cursor, 0);
}
