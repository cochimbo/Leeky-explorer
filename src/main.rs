// Leeky Explorer - Dual-pane TUI file explorer
pub mod models;
pub mod ui;
pub mod fs;
pub mod events;
pub mod config;
pub mod app;
pub mod preview;
pub mod archive;
pub mod event_loop;
pub mod search;  // TASK-040: Recursive search module

use anyhow::Result;
use app::AppState;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

#[tokio::main]
async fn main() -> Result<()> {
    // T851: Initialize logging system
    init_logging()?;
    log::info!("Leeky Explorer starting");
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize app state
    let mut app = AppState::new()?;
    
    // Load initial directory contents and store for filtering
    app.left_panel.refresh_entries()?;
    app.left_all_entries = app.left_panel.entries.clone();
    app.right_panel.refresh_entries()?;
    app.right_all_entries = app.right_panel.entries.clone();

    // Run the application using the event loop module
    let result = event_loop::run(&mut terminal, &mut app).await;

    // T508: Save state on exit
    if result.is_ok() {
        let _ = app.save_state(); // Ignore errors during cleanup
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

// T851: Initialize logging system with file output and rotation
fn init_logging() -> Result<()> {
    use std::io::Write;
    
    // Get log directory (use %APPDATA%/leeky-explorer on Windows)
    let log_dir = if cfg!(target_os = "windows") {
        std::env::var("APPDATA")
            .map(|appdata| std::path::PathBuf::from(appdata).join("leeky-explorer"))
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
    } else {
        dirs::data_local_dir()
            .map(|dir| dir.join("leeky-explorer"))
            .unwrap_or_else(|| std::path::PathBuf::from("."))
    };
    
    // Create log directory if it doesn't exist
    std::fs::create_dir_all(&log_dir)?;
    
    let log_file_path = log_dir.join("leeky.log");
    
    // T851b: Basic rotation - check if file > 10MB and rotate
    if let Ok(metadata) = std::fs::metadata(&log_file_path)
        && metadata.len() > 10 * 1024 * 1024 { // 10MB
            let backup_path = log_dir.join("leeky.log.old");
            let _ = std::fs::rename(&log_file_path, &backup_path); // Rotate to .old
        }
    
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)?;
    
    // Configure env_logger to write to both file and stderr
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug) // Changed from Info to Debug for progress debugging
        .format(|buf, record| {
            // T851b: Structured format with timestamp, level, module, message
            writeln!(
                buf,
                "{} [{}] {} - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .init();
    
    Ok(())
}
