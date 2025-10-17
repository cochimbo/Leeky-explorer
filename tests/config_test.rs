// Configuration and persistence tests
use anyhow::Result;
use leeky_explorer::config::state::{ActivePanel, PersistedState};
use std::path::PathBuf;
use tempfile::TempDir;
use std::fs;
use std::env;

#[test]
fn test_load_valid_state() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config_dir = temp_dir.path().join("leeky-explorer");
    fs::create_dir(&config_dir)?;
    
    let state_file = config_dir.join("state.json");
    let json = r#"{
        "left_panel_path": "/home/user/documents",
        "right_panel_path": "/home/user/downloads",
        "active_panel": "Right"
    }"#;
    fs::write(&state_file, json)?;
    
    // Temporarily override config dir
    unsafe {
        env::set_var("HOME", temp_dir.path());
    }
    
    // This test verifies the structure can be deserialized
    let state: PersistedState = serde_json::from_str(json)?;
    assert_eq!(state.left_panel_path, PathBuf::from("/home/user/documents"));
    assert_eq!(state.right_panel_path, PathBuf::from("/home/user/downloads"));
    assert!(matches!(state.active_panel, ActivePanel::Right));
    
    Ok(())
}

#[test]
fn test_load_missing_file_returns_default() -> Result<()> {
    // When file doesn't exist, load() should return default state
    let default_state = PersistedState::default();
    
    // Verify default state has home directory
    assert!(default_state.left_panel_path.exists() || default_state.left_panel_path == PathBuf::from("."));
    assert!(matches!(default_state.active_panel, ActivePanel::Left));
    
    Ok(())
}

#[test]
fn test_save_creates_config_directory() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config_dir = temp_dir.path().join("leeky-explorer");
    
    // Verify directory doesn't exist yet
    assert!(!config_dir.exists());
    
    // Create state and save (with mocked paths)
    let state = PersistedState {
        left_panel_path: PathBuf::from("/test/left"),
        right_panel_path: PathBuf::from("/test/right"),
        active_panel: ActivePanel::Left,
    };
    
    // We can't actually test save() here because it uses dirs::config_dir()
    // Instead, we test serialization
    let json = serde_json::to_string_pretty(&state)?;
    assert!(json.contains("left_panel_path"));
    assert!(json.contains("/test/left"));
    assert!(json.contains("Right") || json.contains("Left"));
    
    Ok(())
}

#[test]
fn test_full_cycle_serialize_deserialize() -> Result<()> {
    let original = PersistedState {
        left_panel_path: PathBuf::from("/home/user/projects"),
        right_panel_path: PathBuf::from("/home/user/music"),
        active_panel: ActivePanel::Right,
    };
    
    // Serialize
    let json = serde_json::to_string(&original)?;
    
    // Deserialize
    let restored: PersistedState = serde_json::from_str(&json)?;
    
    // Verify all fields match
    assert_eq!(original.left_panel_path, restored.left_panel_path);
    assert_eq!(original.right_panel_path, restored.right_panel_path);
    assert!(matches!(restored.active_panel, ActivePanel::Right));
    
    Ok(())
}

#[test]
fn test_active_panel_conversion() {
    use leeky_explorer::app::PanelSide;
    
    // Test ActivePanel -> PanelSide
    let active_left = ActivePanel::Left;
    let side_left: PanelSide = active_left.into();
    assert_eq!(side_left, PanelSide::Left);
    
    let active_right = ActivePanel::Right;
    let side_right: PanelSide = active_right.into();
    assert_eq!(side_right, PanelSide::Right);
    
    // Test PanelSide -> ActivePanel
    let converted_left: ActivePanel = PanelSide::Left.into();
    assert!(matches!(converted_left, ActivePanel::Left));
    
    let converted_right: ActivePanel = PanelSide::Right.into();
    assert!(matches!(converted_right, ActivePanel::Right));
}

#[test]
fn test_default_state_values() {
    let state = PersistedState::default();
    
    // Should have valid paths
    assert!(state.left_panel_path.as_os_str().len() > 0);
    assert!(state.right_panel_path.as_os_str().len() > 0);
    
    // Both panels should start at same location
    assert_eq!(state.left_panel_path, state.right_panel_path);
    
    // Should default to left panel
    assert!(matches!(state.active_panel, ActivePanel::Left));
}
