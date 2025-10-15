// State persistence
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use crate::app::PanelSide;
use crate::config::paths::{get_config_dir, get_state_file_path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedState {
    pub left_panel_path: PathBuf,
    pub right_panel_path: PathBuf,
    pub active_panel: ActivePanel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ActivePanel {
    Left,
    Right,
}

impl From<PanelSide> for ActivePanel {
    fn from(side: PanelSide) -> Self {
        match side {
            PanelSide::Left => ActivePanel::Left,
            PanelSide::Right => ActivePanel::Right,
        }
    }
}

impl From<ActivePanel> for PanelSide {
    fn from(panel: ActivePanel) -> Self {
        match panel {
            ActivePanel::Left => PanelSide::Left,
            ActivePanel::Right => PanelSide::Right,
        }
    }
}

impl Default for PersistedState {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            left_panel_path: home_dir.clone(),
            right_panel_path: home_dir,
            active_panel: ActivePanel::Left,
        }
    }
}

impl PersistedState {
    /// Load state from JSON file
    /// Returns default state if file doesn't exist or can't be read
    pub fn load() -> Result<Self> {
        let state_path = get_state_file_path()?;
        
        // If file doesn't exist, return default
        if !state_path.exists() {
            return Ok(Self::default());
        }
        
        // Read and deserialize
        let contents = fs::read_to_string(&state_path)
            .context("Failed to read state file")?;
        
        let state: PersistedState = serde_json::from_str(&contents)
            .context("Failed to parse state file")?;
        
        Ok(state)
    }
    
    /// Save state to JSON file
    /// Creates config directory if it doesn't exist
    pub fn save(&self) -> Result<()> {
        let config_dir = get_config_dir()?;
        
        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .context("Failed to create config directory")?;
        }
        
        let state_path = get_state_file_path()?;
        
        // Serialize to JSON
        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize state")?;
        
        // Write to file
        fs::write(&state_path, json)
            .context("Failed to write state file")?;
        
        Ok(())
    }
}
