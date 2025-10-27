//! Event handlers organized by functionality

pub mod collision;
pub mod dialogs;
pub mod file_operations;
pub mod navigation;

// Re-export commonly used functions
pub use collision::{
    continue_batch_operation,
    process_batch_without_collision_check,
    process_single_file_operation,
};
