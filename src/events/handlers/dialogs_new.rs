//! Dialog event handlers
//! 
//! This module contains handlers for all dialog types in the application.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::path::PathBuf;

use crate::app::{AppState, DialogState, ConfirmAction};
use crate::events::keybindings::{Action, map_key_to_input_action};

// Re-export collision handlers
pub use super::collision::{continue_batch_operation, process_batch_without_collision_check, process_single_file_operation};

