# Refactoring Phase 2: Dialog Handlers Extraction

## Status: Ready to Execute

### Functions to Extract (in order)

All these functions should be moved from `src/events/handler.rs` to `src/events/handlers/dialogs.rs`

#### 1. handle_input_dialog
- **Lines**: 1239-1340 (102 lines)
- **Dependencies**: `map_key_to_input_action`, `create_folder`
- **Usage**: Called from `handle_dialog_action` in handler.rs

#### 2. handle_rename_dialog  
- **Lines**: 1341-1441 (101 lines)
- **Dependencies**: None (uses VFS internally)
- **Usage**: Called from main `handle_key` dispatcher

#### 3. handle_password_input_dialog
- **Lines**: 1442-1487 (46 lines)
- **Dependencies**: None
- **Usage**: Called from `handle_dialog_action`

#### 4. create_folder (helper)
- **Lines**: 1488-1545 (58 lines)
- **Dependencies**: `refresh_and_store`
- **Note**: This is called by handle_input_dialog, should move together

#### 5. handle_search_dialog
- **Lines**: 1846-1866 (21 lines)
- **Dependencies**: `crate::ui::search_dialog::DialogAction`
- **Usage**: Called from main dispatcher

#### 6. handle_compress_options_dialog
- **Lines**: 1867-2018 (152 lines)
- **Dependencies**: Archive formats and compression levels
- **Usage**: Called from `handle_dialog_action`

#### 7. handle_drive_selector_dialog
- **Lines**: 2019-2067 (49 lines)
- **Dependencies**: None
- **Usage**: Called from `handle_dialog_action`

#### 8. handle_theme_selector_dialog
- **Lines**: 2068-2110 (43 lines)
- **Dependencies**: None
- **Usage**: Called from `handle_dialog_action`

#### 9. handle_bookmark_manager_dialog
- **Lines**: 2111-2271 (161 lines)
- **Dependencies**: `crate::ui::bookmark_manager::BookmarkManagerState`
- **Usage**: Called from `handle_dialog_action`

#### 10. handle_history_viewer_dialog
- **Lines**: 2272-2369 (98 lines)
- **Dependencies**: None
- **Usage**: Called from `handle_dialog_action`

#### 11. handle_goto_dialog
- **Lines**: 2370-2492 (123 lines)
- **Dependencies**: `get_suggestions_for_input`, `autocomplete_path`, `expand_and_validate_path`
- **Note**: These helper functions need to move too (lines 2807-3095)
- **Usage**: Called from `handle_dialog_action`

#### 12. handle_connection_dialog
- **Lines**: 2493-2804 (312 lines)
- **Dependencies**: `crate::ui::connection_dialog::ConnectionDialogState`, SFTP
- **Usage**: Called from `handle_dialog_action`

### Helper Functions to Also Move

#### For goto dialog:
- `expand_and_validate_path` (lines 2807-2862, 56 lines)
- `get_suggestions_for_input` (lines 2864-2985, 122 lines)
- `autocomplete_path` (lines 2987-3095, 109 lines)

### Total Lines to Move
- Dialog handlers: ~1,267 lines
- Helper functions: ~287 lines
- **Total**: ~1,554 lines

### After Moving
- **handler.rs**: ~1,627 lines (from 3,181)
- **dialogs.rs**: ~1,704 lines (includes collision handler already there)

### Update Call Sites

After moving, update these locations in `handler.rs`:

1. Line ~48: `handle_dialog_action` - update calls to use `handlers::dialogs::`
2. Line ~106: Main dispatcher - update call to `handle_rename_dialog`  
3. Remove the moved function definitions

### Import Updates Needed

**In dialogs.rs**, add these imports:
```rust
use crate::events::keybindings::map_key_to_input_action;
use crate::ui::{
    bookmark_manager::BookmarkManagerState,
    connection_dialog::ConnectionDialogState,
    search_dialog::DialogAction,
};
use crate::archive::{
    formats::ArchiveFormat,
    compressor::CompressionLevel,
};
use crate::remote::{ConnectionManager, sftp::SftpFileSystem, VirtualFileSystem};
```

**In handler.rs**, update to:
```rust
use crate::events::handlers::dialogs;
```

And call functions as:
```rust
handlers::dialogs::handle_input(app, key)
handlers::dialogs::handle_rename(app, key)
// etc.
```

### Execution Steps

1. Copy all functions listed above to dialogs.rs
2. Make functions `pub` in dialogs.rs
3. Add all necessary imports to dialogs.rs
4. Update handler.rs to import and call `handlers::dialogs::`
5. Remove old function definitions from handler.rs
6. Compile and fix any missing imports
7. Test that all dialogs still work

### Quick Script (PowerShell)

```powershell
# TODO: Create extraction script if needed
# For now, manual copy-paste is safest given function interdependencies
```

## Notes

- `create_folder` and `refresh_and_store` are tightly coupled with file operations
- Might want to move them to `file_operations.rs` instead in Phase 3
- The goto dialog helpers are substantial (287 lines) - consider separate module if grows

## Risk Assessment

- **Low Risk**: Most functions are self-contained
- **Medium Risk**: `create_folder` used by multiple handlers
- **High Risk**: Import dependencies need careful management

## Testing Checklist

After refactoring:
- [ ] Input dialog works (create folder)
- [ ] Bookmark add/rename works
- [ ] Rename dialog works (local and remote)
- [ ] Password dialog works  
- [ ] Search dialog works
- [ ] Compress options work
- [ ] Drive selector works
- [ ] Theme selector works
- [ ] Bookmark manager works
- [ ] History viewer works
- [ ] Goto dialog works (with autocomplete)
- [ ] Connection dialog works (SFTP)
