// Keybindings
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, KeyEventKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    ShowHelp,         // F1 to show help dialog with all keybindings
    MoveUp,
    MoveDown,
    SwitchPanel,
    EnterDirectory,   // Enter: if directory -> enter, if file -> preview
    GoUp,
    Refresh,
    Copy,             // Ctrl+C to copy files
    Move,             // Ctrl+X to move files
    Delete,           // Delete key
    CreateFolder,     // Ctrl+Shift+N to create new folder
    Rename,           // Ctrl+R to rename file/directory
    Search,           // Ctrl+F for local search
    ClearSearch,      // Clear search pattern and filter
    ToggleSelection,  // Space to mark/unmark
    SelectAll,        // Ctrl+A to select all
    ClearSelection,   // Esc to clear selection (when marks exist)
    OpenPreview,      // Enter on file to preview (or explicit action)
    ClosePreview,     // Esc/Q to close preview
    OpenEditor,       // Ctrl+E to open text editor
    CloseEditor,      // Esc to close editor
    SaveEditor,       // Ctrl+S to save editor
    ExtractArchive,   // Ctrl+Shift+E to extract archive
    CompressArchive,  // Ctrl+Shift+A to compress archive
    OpenDriveSelector, // Ctrl+D to open drive selector (Windows)
    OpenThemeSelector, // Ctrl+, to open theme selector
    AddBookmark,      // Ctrl+Shift+D to add current directory to bookmarks
    ToggleBookmarkManager, // Ctrl+B to toggle bookmark manager
    ToggleHistoryViewer, // Ctrl+H to toggle navigation history
    ToggleGoToPath,   // Ctrl+G to toggle Go To Path dialog
    OpenRecursiveSearch, // Ctrl+Shift+F to open recursive search dialog
    OpenRemoteConnection, // Ctrl+M to open remote connection dialog
    DisconnectRemote,     // Ctrl+Shift+M to disconnect from remote filesystem
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
    PageDown,         // Page Down - move 5 positions down
    PageUp,           // Page Up - move 5 positions up
    JumpToStart,      // Home key - jump to first entry
    JumpToEnd,        // End key - jump to last entry
    None,
}

pub fn map_key_to_action(key: KeyEvent) -> Action {
    // Only handle key press events, ignore release and repeat
    if key.kind != KeyEventKind::Press {
        return Action::None;
    }

    // Helper to check modifier combinations
    let has_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let has_shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let has_ctrl_shift = has_ctrl && has_shift;

    match (key.code, key.modifiers) {
        // Quit: Only Ctrl+Q now (Ctrl+C is for Copy)
        (KeyCode::Char('q'), KeyModifiers::CONTROL) | (KeyCode::Char('Q'), KeyModifiers::CONTROL) => {
            Action::Quit
        }
        // F1: Help dialog
        (KeyCode::F(1), _) => Action::ShowHelp,
        // Navigation (removed j/k to avoid collision with quick jump)
        (KeyCode::Up, _) => Action::MoveUp,
        (KeyCode::Down, _) => Action::MoveDown,
        (KeyCode::Tab, _) => Action::SwitchPanel,
        (KeyCode::Enter, _) => Action::EnterDirectory, // Will be smart: dir=enter, file=preview
        (KeyCode::Backspace, _) => Action::GoUp,
        (KeyCode::Char('r'), KeyModifiers::NONE) => Action::Refresh,
        // Page navigation keys
        (KeyCode::PageDown, _) => Action::PageDown,
        (KeyCode::PageUp, _) => Action::PageUp,
        (KeyCode::Home, _) => Action::JumpToStart,
        (KeyCode::End, _) => Action::JumpToEnd,
        // File operations - NEW BINDINGS
        (KeyCode::Char('r'), KeyModifiers::CONTROL) | (KeyCode::Char('R'), KeyModifiers::CONTROL) => Action::Rename,
        (KeyCode::Char('c'), KeyModifiers::CONTROL) | (KeyCode::Char('C'), KeyModifiers::CONTROL) if !has_shift => Action::Copy,
        (KeyCode::Char('x'), KeyModifiers::CONTROL) | (KeyCode::Char('X'), KeyModifiers::CONTROL) if !has_shift => Action::Move,
        (KeyCode::Char('n'), KeyModifiers::CONTROL | KeyModifiers::SHIFT) | 
        (KeyCode::Char('N'), KeyModifiers::CONTROL | KeyModifiers::SHIFT) => Action::CreateFolder,
        (KeyCode::Delete, _) => Action::Delete,
        // Search - use guards to distinguish Ctrl vs Ctrl+Shift
        (KeyCode::Char('f'), _) | (KeyCode::Char('F'), _) if has_ctrl_shift => Action::OpenRecursiveSearch,
        (KeyCode::Char('f'), _) | (KeyCode::Char('F'), _) if has_ctrl && !has_shift => Action::Search,
        // Archives - Ctrl+Shift+E/A
        (KeyCode::Char('e'), _) | (KeyCode::Char('E'), _) if has_ctrl_shift => Action::ExtractArchive,
        (KeyCode::Char('a'), _) | (KeyCode::Char('A'), _) if has_ctrl_shift => Action::CompressArchive,
        // Editor - Ctrl+E (without shift)
        (KeyCode::Char('e'), _) | (KeyCode::Char('E'), _) if has_ctrl && !has_shift => Action::OpenEditor,
        // Bookmarks - Ctrl+Shift+D vs Ctrl+D
        (KeyCode::Char('d'), _) | (KeyCode::Char('D'), _) if has_ctrl_shift => Action::AddBookmark,
        (KeyCode::Char('d'), _) | (KeyCode::Char('D'), _) if has_ctrl && !has_shift => Action::OpenDriveSelector,
        (KeyCode::Char('w'), KeyModifiers::CONTROL) | (KeyCode::Char('W'), KeyModifiers::CONTROL) if !has_shift => Action::OpenThemeSelector,
        (KeyCode::Char('b'), KeyModifiers::CONTROL) | (KeyCode::Char('B'), KeyModifiers::CONTROL) if !has_shift => {
            Action::ToggleBookmarkManager
        }
        // Navigation history and Go To
        (KeyCode::Char('h'), KeyModifiers::CONTROL) | (KeyCode::Char('H'), KeyModifiers::CONTROL) if !has_shift => {
            Action::ToggleHistoryViewer
        }
        (KeyCode::Char('g'), KeyModifiers::CONTROL) | (KeyCode::Char('G'), KeyModifiers::CONTROL) if !has_shift => {
            Action::ToggleGoToPath
        }
        // Remote connections - Ctrl+M
        (KeyCode::Char('m'), KeyModifiers::CONTROL) | (KeyCode::Char('M'), KeyModifiers::CONTROL) if !has_shift => {
            Action::OpenRemoteConnection
        }
        // Disconnect from remote - Ctrl+Shift+M
        (KeyCode::Char('m'), _) | (KeyCode::Char('M'), _) if has_ctrl_shift => {
            Action::DisconnectRemote
        }
        // Selection - Ctrl+A without Shift
        (KeyCode::Char('a'), _) | (KeyCode::Char('A'), _) if has_ctrl && !has_shift => Action::SelectAll,
        (KeyCode::Char(' '), KeyModifiers::NONE) => Action::ToggleSelection,
        // Confirm dialogs
        (KeyCode::Char('y'), KeyModifiers::NONE) | (KeyCode::Char('Y'), KeyModifiers::NONE) => {
            Action::ConfirmYes
        }
        (KeyCode::Char('n'), KeyModifiers::NONE) | (KeyCode::Char('N'), KeyModifiers::NONE) => {
            Action::ConfirmNo
        }
        (KeyCode::Esc, _) => Action::Cancel,
        // Alphanumeric quick navigation - jump to files starting with letter
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
        KeyCode::Up => Action::ScrollPreviewUp,
        KeyCode::Down => Action::ScrollPreviewDown,
        KeyCode::PageUp => Action::PagePreviewUp,
        KeyCode::PageDown => Action::PagePreviewDown,
        KeyCode::Home => Action::JumpPreviewStart,
        KeyCode::End => Action::JumpPreviewEnd,
        KeyCode::Esc => Action::ClosePreview,
        KeyCode::Char('q') | KeyCode::Char('Q') => Action::ClosePreview,
        _ => Action::None,
    }
}
