//! Event handlers organized by functionality

pub mod collision;
pub mod dialogs;
pub mod file_operations;
pub mod modes;
pub mod navigation;

// Re-export commonly used functions
pub use collision::{
    continue_batch_operation,
    process_batch_without_collision_check,
    process_single_file_operation,
};

pub use file_operations::{
    start_copy_operation,
    start_copy_operation_skip_check,
    start_copy_operation_with_rename,
    start_move_operation,
    start_move_operation_skip_check,
    start_move_operation_with_rename,
    start_delete_operation,
    handle_copy_request,
    handle_move_request,
    handle_delete_request,
    handle_create_folder_request,
    handle_rename_request,
};

pub use modes::{
    handle_search_mode,
    handle_preview_mode,
    handle_editor_mode,
};
