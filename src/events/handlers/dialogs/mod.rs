//! Dialog event handlers
//! 
//! This module contains handlers for all dialog types in the application,
//! organized by functional domain.

// Sub-modules organized by functionality
pub mod file_operation_dialogs;  // File operation dialogs (collision, rename, compress, passwords)
pub mod navigation_dialogs;      // Navigation dialogs (drives, bookmarks, history, goto)
pub mod connection_dialogs;      // Connection dialogs (SFTP/SMB, search)
pub mod ui_dialogs;              // UI dialogs (theme selector, generic input)

// Re-export all public handler functions for backward compatibility
pub use file_operation_dialogs::{
    handle_collision,
    handle_rename_dialog,
    handle_password_input_dialog,
    handle_compress_options_dialog,
};

pub use navigation_dialogs::{
    handle_drive_selector_dialog,
    handle_bookmark_manager_dialog,
    handle_history_viewer_dialog,
    handle_goto_dialog,
};

pub use connection_dialogs::{
    handle_connection_dialog,
    handle_search_dialog,
};

pub use ui_dialogs::{
    handle_input_dialog,
    handle_theme_selector_dialog,
};

// Re-export helper functions used by other modules
pub use navigation_dialogs::get_directory_children;
