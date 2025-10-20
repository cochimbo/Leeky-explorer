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
    Rename,           // F2 to rename file/directory (name only, no extension)
    RenameWithExtension, // Shift+F2 to rename with extension
    Search,
    ToggleSelection,  // T562: Space to mark/unmark
    SelectAll,        // T563: Ctrl+A to select all
    ClearSelection,   // T564: Esc to clear selection (when marks exist)
    OpenPreview,      // T625: F4 to open preview
    ClosePreview,     // T628: Esc/Q to close preview
    ExtractArchive,   // T838: F9 to extract archive
    CompressArchive,  // T937: Shift+F9 to compress archive
    ScrollPreviewUp,
    ScrollPreviewDown,
    PagePreviewUp,
    PagePreviewDown,
    JumpPreviewStart,
    JumpPreviewEnd,
    ConfirmYes,
    ConfirmNo,
    ConfirmInput,
    Cancel,
    InputChar(char),
    InputBackspace,
    QuickJump(char),  // T128c: Jump to file starting with character
    PageDown,         // T128f: Page Down - move 5 positions down
    PageUp,           // T128g: Page Up - move 5 positions up
    JumpToStart,      // T128h: Home key - jump to first entry
    JumpToEnd,        // T128i: End key - jump to last entry
    None,
}

pub fn map_key_to_action(key: KeyEvent) -> Action {
    // Only handle key press events, ignore release and repeat
    if key.kind != KeyEventKind::Press {
        return Action::None;
    }

    match (key.code, key.modifiers) {
        // T128b: Changed from 'q'/'Q' to Ctrl+Q to free up alphanumeric keys for navigation
        (KeyCode::Char('q'), KeyModifiers::CONTROL) | (KeyCode::Char('Q'), KeyModifiers::CONTROL) => {
            Action::Quit
        }
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => Action::Quit,
        (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => Action::MoveUp,
        (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => Action::MoveDown,
        (KeyCode::Tab, _) => Action::SwitchPanel,
        (KeyCode::Enter, _) => Action::EnterDirectory,
        (KeyCode::Backspace, _) => Action::GoUp,
        (KeyCode::Char('r'), KeyModifiers::NONE) => Action::Refresh,
        // T128f-i: Page navigation keys
        (KeyCode::PageDown, _) => Action::PageDown,
        (KeyCode::PageUp, _) => Action::PageUp,
        (KeyCode::Home, _) => Action::JumpToStart,
        (KeyCode::End, _) => Action::JumpToEnd,
        (KeyCode::F(2), KeyModifiers::NONE) => Action::Rename,
        (KeyCode::F(2), KeyModifiers::SHIFT) => Action::RenameWithExtension,
        (KeyCode::F(5), _) => Action::Copy,
        (KeyCode::F(6), _) => Action::Move,
        (KeyCode::F(7), _) => Action::CreateFolder,
        (KeyCode::F(8), _) => Action::Delete,
        // F3 for search, F4 for preview (T626), F9 for extract (T839), Shift+F9 for compress (T938)
        (KeyCode::F(3), _) => Action::Search,
        (KeyCode::F(4), _) => Action::OpenPreview,
        (KeyCode::F(9), KeyModifiers::NONE) => Action::ExtractArchive,
        (KeyCode::F(9), KeyModifiers::SHIFT) => Action::CompressArchive,
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
        // T128c: Alphanumeric quick navigation - jump to files starting with letter
        (KeyCode::Char(c), KeyModifiers::NONE) if c.is_alphanumeric() => {
            Action::QuickJump(c.to_ascii_lowercase())
        }
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

// T627-T630: Preview mode key handling
pub fn map_key_to_preview_action(key: KeyEvent) -> Action {
    if key.kind != KeyEventKind::Press {
        return Action::None;
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => Action::ScrollPreviewUp,
        KeyCode::Down | KeyCode::Char('j') => Action::ScrollPreviewDown,
        KeyCode::PageUp => Action::PagePreviewUp,
        KeyCode::PageDown => Action::PagePreviewDown,
        KeyCode::Home => Action::JumpPreviewStart,
        KeyCode::End => Action::JumpPreviewEnd,
        KeyCode::Esc => Action::ClosePreview,
        KeyCode::Char('q') | KeyCode::Char('Q') => Action::ClosePreview,
        _ => Action::None,
    }
}
