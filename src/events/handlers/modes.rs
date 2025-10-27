//! Special mode handlers
//! 
//! This module contains handlers for special interaction modes (search, preview, editor).

use anyhow::Result;
use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState};

use crate::app::{AppState, DialogState, ConfirmAction, JumpTarget};
use crate::events::keybindings::Action;

/// Handle search mode key events (T411-T415)
pub fn handle_search_mode(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    if key.kind != crossterm::event::KeyEventKind::Press {
        return Ok(Action::None);
    }

    match key.code {
        crossterm::event::KeyCode::Esc => {
            // T414: Deactivate search and clear filter
            app.deactivate_search();
            Ok(Action::Cancel)
        }
        crossterm::event::KeyCode::Enter => {
            // T415: Finalize filter and return to navigation
            // Keep the filtered results but exit search mode
            app.search_mode = false;
            Ok(Action::None)
        }
        crossterm::event::KeyCode::Backspace => {
            // T412: Remove last character
            app.search_backspace();
            Ok(Action::None)
        }
        crossterm::event::KeyCode::Char(c) => {
            // T412-T413: Append character and apply filter in real-time
            app.search_append(c);
            Ok(Action::None)
        }
        _ => Ok(Action::None),
    }
}

/// Handle preview mode key events (T627-T630)
pub fn handle_preview_mode(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    use crate::events::keybindings::map_key_to_preview_action;
    
    if key.kind != crossterm::event::KeyEventKind::Press {
        return Ok(Action::None);
    }

    let action = map_key_to_preview_action(key);

    match action {
        Action::ClosePreview => {
            // T628: Close preview with Esc or Q
            app.close_preview();
            Ok(Action::None)
        }
        Action::ScrollPreviewUp => {
            // Scroll up by 1 line
            app.scroll_preview(-1);
            Ok(Action::None)
        }
        Action::ScrollPreviewDown => {
            // Scroll down by 1 line
            app.scroll_preview(1);
            Ok(Action::None)
        }
        Action::PagePreviewUp => {
            // T630: Scroll up by page (20 lines)
            app.scroll_preview(-20);
            Ok(Action::None)
        }
        Action::PagePreviewDown => {
            // T630: Scroll down by page (20 lines)
            app.scroll_preview(20);
            Ok(Action::None)
        }
        Action::JumpPreviewStart => {
            // T629: Jump to start of file
            app.jump_preview(JumpTarget::Start);
            Ok(Action::None)
        }
        Action::JumpPreviewEnd => {
            // T629: Jump to end of file
            app.jump_preview(JumpTarget::End);
            Ok(Action::None)
        }
        _ => Ok(Action::None),
    }
}

/// Handle editor mode - TASK-030
pub fn handle_editor_mode(app: &mut AppState, key: KeyEvent) -> Result<Action> {
    use crossterm::event::{KeyCode, KeyModifiers};
    use crate::ui::text_editor::EditorAction;
    
    if let Some(ref mut editor) = app.editor_state {
        // Check if this is a special action (Save, Close)
        let action = editor.handle_key(key.code, key.modifiers);
        
        match action {
            EditorAction::Save => {
                // Try to save
                match editor.save() {
                    Ok(_) => {
                        // Clear any error message
                        app.error_message = None;
                    }
                    Err(e) => {
                        app.error_message = Some(format!("Failed to save: {}", e));
                    }
                }
                Ok(Action::None)
            }
            EditorAction::Close => {
                // Close editor
                app.close_editor();
                Ok(Action::None)
            }
            EditorAction::ConfirmClose => {
                // Show unsaved changes dialog
                app.dialog_state = Some(DialogState::Confirm {
                    message: "File has unsaved changes. Close anyway?".to_string(),
                    confirm_action: ConfirmAction::CloseEditor,
                });
                Ok(Action::None)
            }
            EditorAction::Continue => {
                // Pass the key to textarea (only if not Ctrl+S or Esc)
                if !matches!((key.code, key.modifiers), 
                            (KeyCode::Char('s'), KeyModifiers::CONTROL) | 
                            (KeyCode::Char('S'), KeyModifiers::CONTROL) |
                            (KeyCode::Esc, _)) {
                    // Convert our KeyEvent to ratatui's crossterm KeyEvent
                    // We need to manually convert since they're different versions
                    use ratatui::crossterm::event::{
                        KeyCode as RKeyCode,
                        KeyModifiers as RKeyModifiers,
                        KeyEventKind as RKeyEventKind,
                        KeyEventState as RKeyEventState,
                    };
                    
                    // Convert KeyCode
                    let rcode = match key.code {
                        KeyCode::Backspace => RKeyCode::Backspace,
                        KeyCode::Enter => RKeyCode::Enter,
                        KeyCode::Left => RKeyCode::Left,
                        KeyCode::Right => RKeyCode::Right,
                        KeyCode::Up => RKeyCode::Up,
                        KeyCode::Down => RKeyCode::Down,
                        KeyCode::Home => RKeyCode::Home,
                        KeyCode::End => RKeyCode::End,
                        KeyCode::PageUp => RKeyCode::PageUp,
                        KeyCode::PageDown => RKeyCode::PageDown,
                        KeyCode::Tab => RKeyCode::Tab,
                        KeyCode::BackTab => RKeyCode::BackTab,
                        KeyCode::Delete => RKeyCode::Delete,
                        KeyCode::Insert => RKeyCode::Insert,
                        KeyCode::F(n) => RKeyCode::F(n),
                        KeyCode::Char(c) => RKeyCode::Char(c),
                        KeyCode::Null => RKeyCode::Null,
                        KeyCode::Esc => RKeyCode::Esc,
                        _ => RKeyCode::Null, // Default for unhandled keys
                    };
                    
                    // Convert KeyModifiers
                    let mut rmod = RKeyModifiers::empty();
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        rmod |= RKeyModifiers::SHIFT;
                    }
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        rmod |= RKeyModifiers::CONTROL;
                    }
                    if key.modifiers.contains(KeyModifiers::ALT) {
                        rmod |= RKeyModifiers::ALT;
                    }
                    
                    // Convert KeyEventKind
                    let rkind = match key.kind {
                        KeyEventKind::Press => RKeyEventKind::Press,
                        KeyEventKind::Repeat => RKeyEventKind::Repeat,
                        KeyEventKind::Release => RKeyEventKind::Release,
                    };
                    
                    // Convert KeyEventState
                    let mut rstate = RKeyEventState::empty();
                    if key.state.contains(KeyEventState::KEYPAD) {
                        rstate |= RKeyEventState::KEYPAD;
                    }
                    if key.state.contains(KeyEventState::CAPS_LOCK) {
                        rstate |= RKeyEventState::CAPS_LOCK;
                    }
                    if key.state.contains(KeyEventState::NUM_LOCK) {
                        rstate |= RKeyEventState::NUM_LOCK;
                    }
                    
                    let ratatui_key = ratatui::crossterm::event::KeyEvent {
                        code: rcode,
                        modifiers: rmod,
                        kind: rkind,
                        state: rstate,
                    };
                    editor.input_key(ratatui_key);
                }
                Ok(Action::None)
            }
        }
    } else {
        Ok(Action::None)
    }
}
