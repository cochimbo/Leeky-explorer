// Keybindings
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, KeyEventKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    MoveUp,
    MoveDown,
    SwitchPanel,
    EnterDirectory,
    GoUp,
    Refresh,
    Copy,
    Move,
    Delete,
    CreateFolder,
    Search,
    ToggleSelection,  // T562: Space to mark/unmark
    SelectAll,        // T563: Ctrl+A to select all
    ClearSelection,   // T564: Esc to clear selection (when marks exist)
    ConfirmYes,
    ConfirmNo,
    ConfirmInput,
    Cancel,
    InputChar(char),
    InputBackspace,
    None,
}

pub fn map_key_to_action(key: KeyEvent) -> Action {
    // Only handle key press events, ignore release and repeat
    if key.kind != KeyEventKind::Press {
        return Action::None;
    }

    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), KeyModifiers::NONE) | (KeyCode::Char('Q'), KeyModifiers::NONE) => {
            Action::Quit
        }
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => Action::Quit,
        (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => Action::MoveUp,
        (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => Action::MoveDown,
        (KeyCode::Tab, _) => Action::SwitchPanel,
        (KeyCode::Enter, _) => Action::EnterDirectory,
        (KeyCode::Backspace, _) => Action::GoUp,
        (KeyCode::Char('r'), KeyModifiers::NONE) => Action::Refresh,
        (KeyCode::F(5), _) => Action::Copy,
        (KeyCode::F(6), _) => Action::Move,
        (KeyCode::F(7), _) => Action::CreateFolder,
        (KeyCode::F(8), _) => Action::Delete,
        // F3 for search (standard in file explorers)
        (KeyCode::F(3), _) => Action::Search,
        // T565-T566: Selection keybindings
        (KeyCode::Char(' '), KeyModifiers::NONE) => Action::ToggleSelection,
        (KeyCode::Char('a'), KeyModifiers::CONTROL) => Action::SelectAll,
        (KeyCode::Char('y'), KeyModifiers::NONE) | (KeyCode::Char('Y'), KeyModifiers::NONE) => {
            Action::ConfirmYes
        }
        (KeyCode::Char('n'), KeyModifiers::NONE) | (KeyCode::Char('N'), KeyModifiers::NONE) => {
            Action::ConfirmNo
        }
        (KeyCode::Esc, _) => Action::Cancel,
        _ => Action::None,
    }
}

// Special handling for input dialogs
pub fn map_key_to_input_action(key: KeyEvent) -> Action {
    if key.kind != KeyEventKind::Press {
        return Action::None;
    }

    match key.code {
        KeyCode::Enter => Action::ConfirmInput,
        KeyCode::Backspace => Action::InputBackspace,
        KeyCode::Esc => Action::Cancel,
        KeyCode::Char(c) => Action::InputChar(c),
        _ => Action::None,
    }
}
