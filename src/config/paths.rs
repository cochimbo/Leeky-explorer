// Configuration paths
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Get the configuration directory for leeky-explorer
/// Returns ~/.config/leeky-explorer on Unix, %APPDATA%\leeky-explorer on Windows
pub fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Could not determine config directory")?
        .join("leeky-explorer");
    
    Ok(config_dir)
}

/// Get the full path to the state file
pub fn get_state_file_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("state.json"))
}
