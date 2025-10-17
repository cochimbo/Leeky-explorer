# Tasks: Explorador de Archivos TUI

**Input**: Design documents from `/specs/001-explorador-de-archivos/`
**Prerequisites**: plan.md ✅, spec.md ✅

**Organization**: Tasks are grouped by user story (US1-US4) to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4, SETUP)

---

## Bugs & Issues

### Active Bugs

<!--
  Format: **BUG-###** [Priority] [Component] Description
  Priority: CRITICAL, HIGH, MEDIUM, LOW
  Status: OPEN, IN_PROGRESS, FIXED, VERIFIED
-->

<!-- Example:
- [ ] **BUG-001** [HIGH] [UI] Panel scrolling breaks when directory has >100 files
  - **Status**: OPEN
  - **Reported**: 2024-10-14
  - **Related Tasks**: T116
  - **Steps to Reproduce**: 
    1. Navigate to /usr/bin (has ~1000+ files)
    2. Try to scroll down
    3. UI freezes
  - **Expected**: Should scroll smoothly
  - **Actual**: Application hangs
-->

### Fixed Bugs

- [X] **BUG-001** [HIGH] [EVENT] Double key events causing duplicate navigation
  - **Status**: FIXED ✅
  - **Reported**: 2025-10-14
  - **Fixed**: 2025-10-14
  - **Related Tasks**: T122, T124
  - **Root Cause**: Crossterm was sending both Press and Release events for each keystroke
  - **Solution**: Added filter in `map_key_to_action()` to only process `KeyEventKind::Press` events
  - **Files Modified**: `src/events/keybindings.rs`
  - **Verified**: User confirmed single movement per keystroke ✓

<!--
  Move bugs here after they are verified as fixed
-->

---

## Phase 0: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure needed before implementing any user story

- [X] **T001** [SETUP] Create Rust project with `cargo new leeky-explorer` in repository root
- [X] **T002** [SETUP] Configure `Cargo.toml` with dependencies: ratatui 0.25+, crossterm 0.27+, tokio 1.35+, walkdir 2.5+, humansize 2.1+, serde 1.0+, serde_json 1.0+, anyhow 1.0+
- [X] **T003** [P] [SETUP] Create directory structure: `src/models/`, `src/ui/`, `src/fs/`, `src/events/`, `src/config/`
- [X] **T004** [P] [SETUP] Create module files: `src/models/mod.rs`, `src/ui/mod.rs`, `src/fs/mod.rs`, `src/events/mod.rs`, `src/config/mod.rs`
- [X] **T005** [P] [SETUP] Create `tests/unit/` and `tests/integration/` directories
- [X] **T006** [SETUP] Create basic `src/main.rs` with hello world to verify project compiles

---

## Phase 1: User Story 1 - Navegación Dual Panel (P1 - MVP)

**Goal**: Implement dual-pane navigation with keyboard control

### Models & Data Structures

- [X] **T101** [P] [US1] Create `src/models/file_entry.rs` with `FileEntry` struct (name, entry_type, size, modified, permissions)
- [X] **T102** [P] [US1] Implement `EntryType` enum (File, Dir, Symlink) in `src/models/file_entry.rs`
- [X] **T103** [US1] Add `Display` trait for `FileEntry` to format name, size (humansize), date
- [X] **T104** [P] [US1] Create `src/models/panel.rs` with `Panel` struct (current_path, entries, cursor, scroll_offset)
- [X] **T105** [US1] Implement `Panel::new(path: PathBuf)` constructor
- [X] **T106** [US1] Implement `Panel::move_cursor_up()` and `Panel::move_cursor_down()` with bounds checking
- [X] **T107** [P] [US1] Create `src/app.rs` with `AppState` struct (left_panel, right_panel, active_panel: PanelSide)
- [X] **T108** [US1] Implement `AppState::new()` to initialize both panels at HOME directory

### Filesystem Navigation

- [X] **T109** [P] [US1] Create `src/fs/navigator.rs` module
- [X] **T110** [US1] Implement `read_dir(path: &Path) -> Result<Vec<FileEntry>>` to read directory contents
- [X] **T111** [US1] Implement `Panel::enter_dir()` to navigate into selected directory
- [X] **T112** [US1] Implement `Panel::go_up()` to navigate to parent directory
- [X] **T112b** [US1] Implement cursor positioning on previous directory when navigating up with Backspace
- [X] **T113** [US1] Add error handling for permission denied / invalid paths in navigator

### UI Components

- [X] **T114** [P] [US1] Create `src/ui/layout.rs` with `create_layout(area: Rect)` function for 2-column split
- [X] **T115** [P] [US1] Create `src/ui/panel_widget.rs` with `render_panel()` function
- [X] **T116** [US1] Implement file list rendering with scroll support in `panel_widget.rs`
- [X] **T117** [US1] Add cursor highlight (different color for selected item) in `panel_widget.rs`
- [X] **T118** [P] [US1] Create `src/ui/theme.rs` with color definitions (dirs=blue, files=white, symlinks=cyan, executables=green)
- [X] **T119** [US1] Apply colors to `FileEntry` rendering based on type
- [X] **T120** [US1] Add header rendering showing current path for each panel in `src/ui/layout.rs`
- [X] **T121** [US1] Add footer with key bindings display: "↑↓:Navigate  Tab:Switch  Enter:Open  Backspace:Up  Q:Quit"

### Event Handling

- [X] **T122** [P] [US1] Create `src/events/keybindings.rs` with key code constants
- [X] **T123** [P] [US1] Create `src/events/handler.rs` with `handle_key(app: &mut AppState, key: KeyEvent)` function
- [X] **T124** [US1] Implement arrow up/down handling to move cursor in active panel
- [X] **T125** [US1] Implement Tab key handling to switch active panel (left ↔ right)
- [X] **T126** [US1] Implement Enter key handling to enter selected directory
- [X] **T127** [US1] Implement Backspace key handling to go up one directory level
- [X] **T128** [US1] Implement 'q' or 'Q' key handling to quit application
- [X] **T128b** [US1] Change quit keybinding from 'Q' to Ctrl+Q to free up alphanumeric keys
- [X] **T128c** [US1] Implement alphanumeric quick navigation: jump to first file starting with pressed letter
- [X] **T128d** [US1] Implement cyclic navigation: pressing same letter multiple times cycles through matches
- [X] **T128e** [US1] Update footer to show "Ctrl+Q:Quit" and indicate alphanumeric navigation is available
- [X] **T128f** [US1] Implement Page Down key handling: move cursor 5 positions down
- [X] **T128g** [US1] Implement Page Up key handling: move cursor 5 positions up
- [X] **T128h** [US1] Implement Home key handling: move cursor to first entry
- [X] **T128i** [US1] Implement End key handling: move cursor to last entry
- [X] **T128j** [US1] Update footer to show PgUp/PgDn and Home/End navigation hints

### Main Loop & Terminal Setup

- [X] **T129** [US1] Update `src/main.rs` to initialize crossterm terminal (enable raw mode)
- [X] **T130** [US1] Implement terminal cleanup on exit (disable raw mode, show cursor)
- [X] **T131** [US1] Create event loop: poll for keyboard events with 100ms timeout
- [X] **T132** [US1] Integrate `ui::render()` call in event loop to draw frame
- [X] **T133** [US1] Handle terminal resize events to redraw layout
- [X] **T134** [US1] Add graceful error handling in main loop (show error, don't crash)

### Testing

- [X] **T135** [P] [US1] Write unit test for `Panel::move_cursor_up/down` with boundary conditions
- [X] **T136** [P] [US1] Write unit test for `read_dir()` using temp directory fixtures
- [X] **T137** [P] [US1] Write unit test for `Panel::enter_dir()` and `Panel::go_up()`
- [X] **T138** [US1] Write integration test simulating arrow navigation workflow

---

## Phase 2: User Story 2 - Copiar y Mover Archivos (P2)

**Goal**: Implement F5 (copy) and F6 (move) operations with progress bars

### Models & Operations

- [X] **T201** [P] [US2] Create `src/models/operation.rs` with `Operation` enum (Copy, Move, Delete)
- [X] **T202** [P] [US2] Create `Progress` struct (bytes_done, bytes_total, files_done, files_total) in `operation.rs`
- [X] **T203** [US2] Add `current_operation: Option<Operation>` field to `AppState`
- [X] **T204** [P] [US2] Create `DialogState` enum (Confirm, Input, Progress, Error) in `src/ui/dialog.rs`
- [X] **T205** [US2] Add `dialog_state: Option<DialogState>` field to `AppState`

### File Operations

- [X] **T206** [P] [US2] Create `src/fs/operations.rs` module
- [X] **T207** [US2] Implement `copy_file_with_progress(src, dst, tx: Sender<Progress>) -> Result<()>` using tokio
- [X] **T208** [US2] Implement `copy_dir_recursive(src, dst, tx: Sender<Progress>) -> Result<()>` using walkdir + tokio
- [X] **T209** [US2] Implement `move_item(src, dst) -> Result<()>` (rename or copy+delete)
- [X] **T210** [US2] Add `get_total_size(path: &Path) -> Result<u64>` helper for progress calculation
- [X] **T211** [US2] Handle edge case: source and destination are the same (error)
- [X] **T212** [US2] Handle edge case: destination file already exists (prompt user)

### UI Dialogs

- [X] **T213** [P] [US2] Implement `render_confirm_dialog(msg: &str)` in `src/ui/dialog.rs`
- [X] **T214** [P] [US2] Implement `render_progress_dialog(operation: &Operation, progress: &Progress)` with bar
- [X] **T215** [US2] Add progress bar calculation (percentage, MB copied/total)
- [X] **T216** [P] [US2] Implement `render_error_dialog(error: &str)` 
- [X] **T217** [US2] Add modal overlay styling (centered box with border)

### Event Handling

- [X] **T218** [US2] Implement F5 key handler: show confirm dialog for copy
- [X] **T219** [US2] Implement F6 key handler: show confirm dialog for move
- [X] **T220** [US2] Implement Y/N handling in confirm dialog (start operation or cancel)
- [X] **T221** [US2] Spawn async tokio task for copy/move operation with progress channel
- [X] **T222** [US2] Poll progress channel in event loop and update `AppState`
- [X] **T223** [US2] Handle operation completion: show success message, refresh panels
- [X] **T224** [US2] Handle operation error: show error dialog, log error details

### Testing

- [X] **T225** [P] [US2] Write unit test for `copy_file_with_progress` with temp files
- [X] **T226** [P] [US2] Write unit test for `copy_dir_recursive` with nested directories
- [X] **T227** [P] [US2] Write unit test for `move_item` verifying source is deleted
- [X] **T228** [US2] Write integration test for F5 copy workflow (navigate, F5, confirm, verify)
- [X] **T229** [US2] Write integration test for F6 move workflow

---

## Phase 3: User Story 3 - Eliminar y Crear Carpetas (P3)

**Goal**: Implement F8 (delete) and F7 (create folder) operations

### File Operations

- [X] **T301** [P] [US3] Implement `delete_file(path: &Path) -> Result<()>` in `src/fs/operations.rs`
- [X] **T302** [P] [US3] Implement `delete_dir_recursive(path: &Path, tx: Sender<Progress>) -> Result<()>` with progress
- [X] **T303** [P] [US3] Implement `create_dir(path: &Path) -> Result<()>` wrapper
- [X] **T304** [US3] Handle edge case: delete non-empty directory (require double confirmation)
- [X] **T305** [US3] Handle edge case: insufficient permissions for delete/create

### UI Dialogs

- [X] **T306** [P] [US3] Implement `render_input_dialog(prompt: &str, current: &str)` in `src/ui/dialog.rs`
- [X] **T307** [US3] Add text input handling in dialog (type chars, backspace, enter to confirm)
- [X] **T308** [US3] Add double confirmation dialog for recursive delete (separate function)

### Event Handling

- [X] **T309** [US3] Implement F7 key handler: show input dialog for new folder name
- [X] **T310** [US3] Handle input dialog text entry and create folder on Enter
- [X] **T311** [US3] Implement F8 key handler: show confirm dialog for delete
- [X] **T312** [US3] Check if target is non-empty directory, show second confirmation if true
- [X] **T313** [US3] Spawn async delete operation with progress for directories
- [X] **T314** [US3] Refresh panel after successful create/delete operation

### Testing

- [X] **T315** [P] [US3] Write unit test for `delete_file` with temp file
- [X] **T316** [P] [US3] Write unit test for `delete_dir_recursive` with nested structure
- [X] **T317** [P] [US3] Write unit test for `create_dir` verifying directory exists after
- [X] **T318** [US3] Write integration test for error handling (nonexistent, existing dir)
- [X] **T319** [US3] Write integration test for `is_dir_empty` utility function
- [X] **T320** [US3] Write integration test for delete progress tracking validation

---

## Phase 4: User Story 4 - Búsqueda y Filtrado (P4)

**Goal**: Implement '/' search with glob pattern filtering

### Filtering Logic

- [X] **T401** [P] [US4] Add `filter: Option<String>` field to `Panel` struct
- [X] **T402** [US4] Implement `Panel::apply_filter(pattern: &str)` to filter entries list
- [X] **T403** [US4] Support simple text matching (case-insensitive contains)
- [X] **T404** [US4] Support glob patterns using `glob` crate (e.g., "*.rs", "test_*")
- [X] **T405** [US4] Implement `Panel::clear_filter()` to restore full list
- [X] **T406** [US4] Update `Panel::enter_dir()` to preserve filter when navigating

### UI Components

- [X] **T407** [P] [US4] Add search bar rendering at bottom of active panel in `src/ui/panel_widget.rs`
- [X] **T408** [US4] Show "Buscar: {pattern}_" when search is active
- [X] **T409** [US4] Show "Sin resultados para: {pattern}" when filter returns empty list
- [X] **T410** [US4] Update footer to show "/" key hint: "/:Search  Esc:Clear"

### Event Handling

- [X] **T411** [US4] Implement '/' key handler: activate search mode
- [X] **T412** [US4] Handle text input in search mode: append to filter pattern
- [X] **T413** [US4] Apply filter in real-time as user types
- [X] **T414** [US4] Implement Esc key handler: deactivate search mode and clear filter
- [X] **T415** [US4] Implement Enter key in search mode: finalize filter, return to navigation

### Testing

- [X] **T416** [P] [US4] Write unit test for `apply_filter` with simple text pattern
- [X] **T417** [P] [US4] Write unit test for `apply_filter` with glob pattern "*.txt"
- [X] **T418** [P] [US4] Write unit test for `apply_filter` with no matches (empty result)
- [X] **T419** [US4] Write integration test for search workflow (/, type pattern, see filtered results, Esc)

---

## Phase 5: Configuration & Persistence

**Goal**: Save/restore application state between sessions

### State Persistence

- [X] **T501** [P] [US-ALL] Create `src/config/state.rs` with `PersistedState` struct
- [X] **T502** [P] [US-ALL] Add serde Serialize/Deserialize derives to `PersistedState`
- [X] **T503** [P] [US-ALL] Create `src/config/paths.rs` with `get_config_dir()` helper (~/.config/leeky-explorer)
- [X] **T504** [US-ALL] Implement `PersistedState::load() -> Result<Self>` reading from JSON file
- [X] **T505** [US-ALL] Implement `PersistedState::save(&self) -> Result<()>` writing to JSON file
- [X] **T506** [US-ALL] Create config directory if it doesn't exist on first save
- [X] **T507** [US-ALL] Update `AppState::new()` to load persisted state (left_path, right_path, active_panel)
- [X] **T508** [US-ALL] Save state on application exit in `main.rs` cleanup
- [X] **T509** [US-ALL] Handle missing config file gracefully (use defaults: HOME directory)

### Testing

- [X] **T510** [P] [US-ALL] Write unit test for `load()` with valid JSON file
- [X] **T511** [P] [US-ALL] Write unit test for `load()` with missing file (returns default)
- [X] **T512** [P] [US-ALL] Write unit test for `save()` creating config directory
- [X] **T513** [US-ALL] Write integration test for full cycle: save state, restart app, verify state restored

---

## Phase 6: Polish & Documentation

**Goal**: Final touches, README, error handling improvements

### Error Handling

- [ ] **T601** [US-ALL] Audit all `unwrap()` calls and replace with proper error handling
- [ ] **T602** [US-ALL] Add context messages to all errors using anyhow `.context()`
- [ ] **T603** [US-ALL] Test permission denied scenarios (read, write, delete)
- [ ] **T604** [US-ALL] Test disk full scenario during copy operation
- [ ] **T605** [US-ALL] Test symlink handling (broken symlinks, circular references)

### Documentation

- [ ] **T606** [P] [US-ALL] Write `README.md` with installation instructions (`cargo install --path .`)
- [ ] **T607** [P] [US-ALL] Document all key bindings in README
- [ ] **T608** [P] [US-ALL] Add usage examples and screenshots (ASCII art)
- [ ] **T609** [P] [US-ALL] Write inline code documentation (/// comments) for public functions
- [ ] **T610** [P] [US-ALL] Generate cargo docs: `cargo doc --no-deps --open`

### Performance & Optimization

- [ ] **T611** [US-ALL] Profile navigation performance with large directories (10k+ files)
- [ ] **T612** [US-ALL] Optimize rendering to only redraw changed panels
- [ ] **T613** [US-ALL] Add benchmarks for file operations (copy, delete) in `benches/`
- [ ] **T614** [US-ALL] Test terminal responsiveness on Windows/Linux/macOS

### Final Testing

- [ ] **T615** [US-ALL] Run full test suite: `cargo test`
- [ ] **T616** [US-ALL] Manual QA: test all user stories end-to-end
- [ ] **T617** [US-ALL] Test edge cases from spec.md (same src/dst, permission errors, etc.)
- [ ] **T618** [US-ALL] Test terminal compatibility (minimum 80x24, resize handling)

---

## Phase 5.5: User Story 5 - Selección Múltiple (P5)

**Goal**: Implement multi-select functionality for batch operations

### Selection State Management

- [X] **T551** [P] [US5] Create `src/models/selection.rs` with `SelectionState` struct (left_marked: HashSet<PathBuf>, right_marked: HashSet<PathBuf>)
- [X] **T552** [US5] Implement `SelectionState::toggle_mark(panel: PanelSide, path: PathBuf)` to mark/unmark items
- [X] **T553** [US5] Implement `SelectionState::mark_all(panel: PanelSide, paths: Vec<PathBuf>)` for Ctrl+A
- [X] **T554** [US5] Implement `SelectionState::clear(panel: PanelSide)` to remove all marks
- [X] **T555** [US5] Implement `SelectionState::get_marked(panel: PanelSide) -> Vec<PathBuf>` to retrieve selection
- [X] **T556** [US5] Implement `SelectionState::is_marked(panel: PanelSide, path: &Path) -> bool` for visual check
- [X] **T557** [US5] Add `selection_state: SelectionState` field to `AppState` in `src/app.rs`

### UI Indicators

- [X] **T558** [US5] Update `render_panel()` in `src/ui/panel_widget.rs` to show "*" prefix for marked items
- [X] **T559** [US5] Add alternativebackground color for marked items in `src/ui/theme.rs` (e.g., DarkGray)
- [X] **T560** [US5] Add selection counter in panel header: "3 items seleccionados" when marks exist
- [X] **T561** [US5] Update footer to show: "Space: Select | Ctrl+A: All | Esc: Clear"

### Keyboard Handling

- [X] **T562** [P] [US5] Add `Action::ToggleSelection` to `src/events/handler.rs`
- [X] **T563** [P] [US5] Add `Action::SelectAll` to `src/events/handler.rs`
- [X] **T564** [P] [US5] Add `Action::ClearSelection` to `src/events/handler.rs`
- [X] **T565** [US5] Map Space key to `Action::ToggleSelection` in `src/events/keybindings.rs`
- [X] **T566** [US5] Map Ctrl+A to `Action::SelectAll` in `src/events/keybindings.rs`
- [X] **T567** [US5] Map Esc to `Action::ClearSelection` when marks exist (without closing app)
- [X] **T568** [US5] Implement toggle logic: mark item, advance cursor to next item
- [X] **T569** [US5] Implement select all: toggle all visible items in active panel

### Batch Operations Integration

- [X] **T570** [US5] Update `start_copy()` in `src/app.rs` to check for marked items first
- [X] **T571** [US5] Update `start_move()` in `src/app.rs` to check for marked items first
- [X] **T572** [US5] Update `start_delete()` in `src/app.rs` to check for marked items first
- [X] **T573** [US5] Modify confirmation dialogs to show "Copiar 3 items..." when multiple selected
- [X] **T574** [US5] Implement batch progress tracking: "Copiando 2/3: archivo.txt (45%)"
- [X] **T575** [US5] Clear marks automatically after successful batch operation
- [X] **T576** [US5] Handle errors during batch: show "(C)ontinuar / (R)eintentar / (A)bortar" dialog
- [X] **T577** [US5] Keep track of failed items to show summary: "3 copiados, 1 fallido"

### Navigation & Filtering Integration

- [X] **T578** [US5] Clear marks when navigating to different directory (Enter/Backspace)
- [X] **T579** [US5] Preserve marks when switching panels (Tab)
- [X] **T580** [US5] Update filter logic: marks on filtered-out items should be removed

### Testing

- [X] **T581** [P] [US5] Create `tests/unit/selection_tests.rs`
- [X] **T582** [US5] Test `toggle_mark()`: mark, unmark, mark again same item
- [X] **T583** [US5] Test `mark_all()`: select all, then toggle all to deselect
- [X] **T584** [US5] Test batch operations: mark 3 files, copy, verify all copied
- [X] **T585** [US5] Test marks cleared on directory change
- [X] **T586** [US5] Test marks preserved when switching panels
- [X] **T587** [US5] Test error handling: batch operation with one permission error

---

## Phase 5.6: User Story 6 - Preview de Texto (P6)

**Goal**: Implement text file preview in modal dialog

### Preview Module Setup

- [X] **T601** [P] [US6] Create `src/preview/` directory
- [X] **T602** [P] [US6] Create `src/preview/mod.rs` and declare text_viewer, encoding submodules
- [X] **T603** [P] [US6] Add `encoding_rs` dependency to `Cargo.toml` for charset detection

### Text Loading & Encoding

- [X] **T604** [P] [US6] Create `src/preview/encoding.rs`
- [X] **T605** [US6] Implement `detect_encoding(bytes: &[u8]) -> &'static Encoding` using encoding_rs
- [X] **T606** [US6] Implement `load_text_file(path: &Path) -> Result<String>` with encoding detection
- [X] **T607** [US6] Add UTF-8 validation and fallback to Latin-1 if detection fails
- [X] **T608** [US6] Handle binary file detection: return error if >10% non-printable chars
- [X] **T609** [US6] Add file size limit: show warning for files >5MB, error for >10MB

### Preview State Management

- [X] **T610** [P] [US6] Create `PreviewState` enum in `src/app.rs`: Text{content, scroll_offset, total_lines}
- [X] **T611** [US6] Add `preview_state: Option<PreviewState>` field to `AppState`
- [X] **T612** [US6] Implement `AppState::open_text_preview(path: PathBuf)` async
- [X] **T613** [US6] Implement `AppState::close_preview()` to clear preview_state
- [X] **T614** [US6] Implement `AppState::scroll_preview(direction: i32)` for up/down
- [X] **T615** [US6] Implement `AppState::jump_preview(target: JumpTarget)` for Home/End

### UI Modal Rendering

- [X] **T616** [P] [US6] Create `src/ui/preview_modal.rs`
- [X] **T617** [US6] Implement `render_text_preview(f: &mut Frame, state: &PreviewState, area: Rect)`
- [X] **T618** [US6] Calculate modal size: 80% width, 80% height, centered
- [X] **T619** [US6] Draw modal border with Clear background (to cover panels)
- [X] **T620** [US6] Render title bar: "filename.txt (2.5 KB)"
- [X] **T621** [US6] Render text content with line numbers in left margin (4 chars wide)
- [X] **T622** [US6] Implement viewport scrolling: show only visible lines within modal height
- [X] **T623** [US6] Add footer hints: "↑↓: Scroll | Home/End: Inicio/Fin | Esc/Q: Cerrar"
- [X] **T624** [US6] Show position indicator: "Línea 150/523 (28%)" in bottom right

### Keyboard Handling

- [X] **T625** [P] [US6] Add `Action::OpenPreview` to `src/events/handler.rs`
- [X] **T626** [US6] Map F4 key to `Action::OpenPreview` in `src/events/keybindings.rs`
- [X] **T627** [US6] Add preview mode check in `handle_key()`: route arrows to scroll_preview() when active
- [X] **T628** [US6] Map Esc/Q to close_preview() when preview is active
- [X] **T629** [US6] Map Home/End to jump to start/end of file
- [X] **T630** [US6] Map PageUp/PageDown to scroll by viewport height

### File Type Detection

- [X] **T631** [P] [US6] Create helper `is_text_file(path: &Path) -> bool` checking extensions
- [X] **T632** [US6] Add text extensions: .txt, .md, .rs, .py, .js, .json, .xml, .log, .conf, .ini, .toml, .yaml
- [X] **T633** [US6] Show error dialog "No se puede previsualizar: archivo binario" for non-text

### Testing

- [X] **T634** [P] [US6] Create `tests/unit/preview_tests.rs`
- [X] **T635** [US6] Test `load_text_file()` with UTF-8 file
- [X] **T636** [US6] Test encoding detection with Latin-1 file
- [X] **T637** [US6] Test binary file rejection (e.g., .png)
- [X] **T638** [US6] Test scroll: load 100-line file, scroll down, verify offset
- [X] **T639** [US6] Test Home/End jumps
- [X] **T640** [US6] Test Esc closes preview and returns to navigation

---

## Phase 5.7: User Story 7 - Preview de Imágenes (P7)

**Goal**: Implement image preview as ASCII/Unicode art in modal

### Image Processing Setup

- [X] **T701** [P] [US7] Add `image` dependency to `Cargo.toml` (version 0.24+)
- [X] **T702** [P] [US7] Add `artem` or `viuer` dependency for ASCII/Unicode conversion
- [X] **T703** [P] [US7] Create `src/preview/image_viewer.rs`

### Image Loading & Conversion

- [X] **T704** [US7] Implement `load_image(path: &Path) -> Result<DynamicImage>` using image crate
- [X] **T705** [US7] Implement `get_image_metadata(path: &Path) -> Result<ImageMeta>` (width, height, format)
- [X] **T706** [US7] Implement `image_to_ascii(img: DynamicImage, max_width: u16, max_height: u16) -> Result<String>`
- [X] **T707** [US7] Add automatic scaling: calculate aspect ratio, resize to fit modal
- [X] **T708** [US7] Detect terminal color capability using crossterm: truecolor, 256, 16, mono
- [X] **T709** [US7] Use Unicode blocks (▀▄█) for better vertical resolution when supported
- [X] **T710** [US7] Fallback to ASCII characters (.:;+=*%@#) for limited terminals
- [X] **T711** [US7] Handle image decoding errors: corrupt file, unsupported format

### Preview State Extension

- [X] **T712** [US7] Extend `PreviewState` enum: add Image{ascii_art, metadata, original_size}
- [X] **T713** [US7] Implement `AppState::open_image_preview(path: PathBuf)` async
- [X] **T714** [US7] Add file size check: confirm dialog for images >10MB before loading
- [X] **T715** [US7] Show "Cargando imagen..." dialog during processing

### UI Modal Rendering

- [X] **T716** [US7] Extend `render_preview_modal()` to handle Image variant
- [X] **T717** [US7] Calculate modal size: 90% width, 90% height (larger for images)
- [X] **T718** [US7] Render title: "imagen.png (1920x1080, 2.5 MB, PNG)"
- [X] **T719** [US7] Center ASCII art within modal area
- [X] **T720** [US7] Add footer hint: "Esc/Q: Cerrar"
- [X] **T721** [US7] Handle animated GIFs: show only first frame with note "(GIF animado - frame 1)"

### File Type Detection

- [X] **T722** [P] [US7] Create helper `is_image_file(path: &Path) -> bool`
- [X] **T723** [US7] Add image extensions: .png, .jpg, .jpeg, .gif, .bmp, .webp
- [X] **T724** [US7] Detect format by extension first, then by magic bytes if extension ambiguous

### Keyboard Handling

- [X] **T725** [US7] Reuse F4 key for both text and image preview (auto-detect file type)
- [X] **T726** [US7] Map Esc/Q to close image preview
- [X] **T727** [US7] No scroll needed for images (always fit to screen)

### Testing

- [X] **T728** [P] [US7] Create test fixtures: small PNG (100x100), large JPEG (4K), GIF
- [X] **T729** [US7] Test `load_image()` with valid PNG
- [X] **T730** [US7] Test corrupt image handling
- [X] **T731** [US7] Test aspect ratio preservation on resize
- [X] **T732** [US7] Test ASCII conversion produces non-empty output
- [X] **T733** [US7] Test GIF shows first frame
- [ ] **T734** [US7] Test file size confirmation for large images

---

## Phase 5.8: User Story 8 - Descompresión (P8)

**Goal**: Implement archive extraction with password support

### Archive Processing Setup

- [x] **T801** [P] [US8] Add dependencies to `Cargo.toml`: zip (0.6+), tar (0.4+), flate2 (1.0+), xz2 (0.1+)
- [x] **T802** [P] [US8] Add `sevenz-rust` (0.5+) for 7Z support
- [x] **T803** [P] [US8] Add `unrar` (0.5+) for RAR support (note: requires libunrar)
- [x] **T804** [P] [US8] Create `src/archive/` directory
- [x] **T805** [P] [US8] Create `src/archive/mod.rs` and declare formats, extractor, password submodules

### Format Detection

- [x] **T806** [P] [US8] Create `src/archive/formats.rs`
- [x] **T807** [US8] Implement `ArchiveFormat` enum: ZIP, TAR, TAR_GZ, TAR_BZ2, TAR_XZ, SEVENZ, RAR, UNKNOWN
- [x] **T808** [US8] Implement `detect_format(path: &Path) -> Result<ArchiveFormat>` using magic bytes
- [x] **T809** [US8] Add magic byte signatures: ZIP (PK\x03\x04), TAR (ustar), 7Z (7z\xBC\xAF\x27\x1C), RAR (Rar!)
- [x] **T810** [US8] Fallback to extension detection if magic bytes unrecognized

### Archive Listing

- [x] **T811** [P] [US8] Create `ArchiveEntry` struct: name, size_compressed, size_uncompressed, is_dir
- [x] **T812** [US8] Implement `list_archive_contents(path: &Path, format: ArchiveFormat) -> Result<Vec<ArchiveEntry>>`
- [x] **T813** [US8] Implement ZIP listing using zip crate
- [x] **T814** [US8] Implement TAR listing using tar crate
- [x] **T815** [US8] Implement 7Z listing using sevenz-rust
- [ ] **T816** [US8] Implement RAR listing using unrar
- [x] **T817** [US8] Calculate compression ratio: (1 - compressed/uncompressed) * 100

### Password Handling

- [x] **T818** [P] [US8] Create `src/archive/password.rs`
- [x] **T819** [US8] Implement `PasswordDialog` struct with input field (hidden chars)
- [x] **T820** [US8] Implement `prompt_password() -> Option<String>` returning user input or None if cancelled
- [x] **T821** [US8] Detect password-protected archives: check ZIP encryption flag, 7Z header
- [x] **T822** [US8] Handle incorrect password: show error, allow retry or cancel

### Extraction Logic

- [x] **T823** [P] [US8] Create `src/archive/extractor.rs`
- [x] **T824** [US8] Implement `extract_archive(path: &Path, dest: &Path, password: Option<String>, tx: Sender<Progress>) -> Result<()>`
- [x] **T825** [US8] Implement ZIP extraction with password support using zip crate
- [x] **T826** [US8] Implement TAR extraction (plain, GZ, BZ2, XZ) using tar + flate2/xz2
- [x] **T826b** [BUG] [US8] **FIX TAR EXTRACTION**: Currently using stub implementation (returns Ok() without extracting). Need to implement full TAR extraction logic in `extract_tar_sync()` and `extract_tar_unbounded()` functions with compression support (GZ, BZ2, XZ)
- [x] **T827** [US8] Implement 7Z extraction with password using sevenz-rust
- [x] **T827b** [BUG] [US8] **FIX 7Z EXTRACTION**: Currently using stub implementation (returns Ok() without extracting). Need to implement full 7Z extraction logic in `extract_7z_sync()` and `extract_7z_unbounded()` functions
- [ ] **T828** [US8] Implement RAR extraction with password using unrar
- [x] **T829** [US8] Preserve directory structure: create parent dirs as needed
- [x] **T830** [US8] Preserve file permissions and timestamps where supported
- [x] **T831** [US8] Handle symlinks in TAR: preserve on Unix, skip on Windows
- [x] **T832** [US8] Sanitize paths: convert absolute paths to relative for security
- [ ] **T833** [US8] Detect multi-part RAR archives: find .part1.rar, .part2.rar automatically

### Progress Tracking

- [x] **T834** [US8] Define `ExtractionProgress` struct: current_file, file_index, total_files, bytes_extracted
- [ ] **T835** [US8] Send progress updates via channel every 100ms or per-file
- [ ] **T836** [US8] Show progress modal: "Extrayendo 5/23: documento.pdf (22%)"
- [ ] **T837** [US8] Update UI with current filename and percentage
- [ ] **T837b** [BUG] [US8] **FIX PROGRESS BAR**: Progress updates not visible in UI during extraction. Architecture implemented (unbounded_channel) but messages not reaching UI. Need to debug: 1) Add logging to verify message flow, 2) Check forwarding task timing, 3) Verify main loop polling frequency, 4) Test with explicit channel flush/sync

### UI Integration

- [x] **T838** [P] [US8] Add `Action::ExtractArchive` to `src/events/handler.rs`
- [x] **T839** [US8] Map F9 key to `Action::ExtractArchive` in `src/events/keybindings.rs`
- [x] **T839b** [US8] Update footer to show F9 key hint: "F9:Extract" in main navigation view
- [x] **T840** [US8] Implement archive preview modal: show list of files with scroll
- [x] **T841** [US8] Show archive metadata in title: "archivo.zip (23 files, 15 MB → 42 MB, ratio 64%)"
- [x] **T842** [US8] Add extraction destination dialog: pre-fill with opposite panel path
- [x] **T843** [US8] Show password input dialog for encrypted archives
- [x] **T844** [US8] Handle collisions: prompt "(S)obreescribir / (T)odos / (R)enombrar / (O)mitir / (C)ancelar"

### Error Handling & Safety

- [x] **T845** [US8] Check disk space before extraction: compare available vs uncompressed size
- [x] **T846** [US8] Handle corrupt archives: catch decompression errors, show clear message
- [ ] **T847** [US8] Implement ZIP bomb protection: limit total extracted size to 10GB
- [ ] **T848** [US8] Handle permission errors during extraction: offer to skip or abort
- [ ] **T849** [US8] Support cancellation: pressing Esc shows confirmation, cleans up partial files
- [ ] **T850** [US8] Handle duplicate filenames within archive: keep last, log warning

### Testing

- [ ] **T851** [P] [US8] Create `tests/unit/archive_tests.rs`
- [ ] **T852** [US8] Create test fixtures: small.zip, small.tar.gz, small.7z with known contents
- [ ] **T853** [US8] Create password-protected test archives: encrypted.zip, encrypted.7z
- [ ] **T854** [US8] Test `detect_format()` with various archive types
- [ ] **T855** [US8] Test `list_archive_contents()` for ZIP
- [ ] **T856** [US8] Test extraction of ZIP to temp dir, verify files exist
- [ ] **T857** [US8] Test password-protected ZIP: correct password succeeds
- [ ] **T858** [US8] Test password-protected ZIP: wrong password fails with retry option
- [ ] **T859** [US8] Test TAR.GZ extraction preserves directory structure
- [ ] **T860** [US8] Test 7Z extraction
- [ ] **T861** [US8] Test cancellation: Esc during extraction cleans up partial files
- [ ] **T862** [US8] Test corrupt archive: should error gracefully

---

## Phase 5.8b: File Type Icons with Emojis (FR-003b)

**Goal**: Add emoji icons to file/folder entries based on file type for better visual identification

### Icon Mapping Module

- [x] **T863** [P] [FR-003b] Create `src/ui/file_icons.rs` module
- [x] **T864** [FR-003b] Implement `get_icon_for_entry(entry: &FileEntry) -> &'static str` function
- [x] **T865** [FR-003b] Add folder icons: 📁 for regular dirs, 📂 for open/selected dir, 🔗 for symlink dirs
- [x] **T866** [FR-003b] Add document icons: 📄 .txt/.md, 📝 .doc/.docx/.odt, 📊 .xls/.xlsx/.csv, 📈 .ppt/.pptx
- [x] **T867** [FR-003b] Add code file icons: 💻 .rs/.py/.js/.ts, ⚙️ .json/.yaml/.toml/.xml/.ini, 🔧 .sh/.bash/.zsh
- [x] **T868** [FR-003b] Add media icons: 🖼️ .png/.jpg/.jpeg/.gif/.bmp/.webp, 🎵 .mp3/.wav/.flac/.ogg, 🎬 .mp4/.avi/.mkv/.mov
- [x] **T869** [FR-003b] Add archive icons: 📦 .zip/.tar/.gz/.7z/.rar
- [x] **T870** [FR-003b] Add executable icons: ⚡ .exe/.app/.bin, 🔒 files with execute permission (Unix)
- [x] **T871** [FR-003b] Add default icon: 📄 for unknown file types

### UI Integration

- [x] **T872** [P] [FR-003b] Modify `render_file_list()` in `src/ui/panel.rs` to include icon before filename
- [x] **T873** [FR-003b] Add proper spacing: "📁 folder_name" (icon + space + name)
- [x] **T874** [FR-003b] Ensure icons don't break alignment of file size/date columns
- [ ] **T875** [FR-003b] Test with various file types to ensure correct icon mapping

### Testing

- [ ] **T876** [FR-003b] Create test directory with mixed file types
- [ ] **T877** [FR-003b] Verify all icon categories display correctly
- [ ] **T878** [FR-003b] Test that terminal handles emoji rendering (fallback if needed)

---

## Phase 5.9: User Story 9 - Compresión (P9)

**Goal**: Implement archive compression with format selection and password support

### Compression Module Setup

- [ ] **T901** [P] [US9] Create `src/archive/compressor.rs`
- [ ] **T902** [P] [US9] Export compressor module in `src/archive/mod.rs`
- [ ] **T903** [US9] Add `CompressionLevel` enum: Fast (1), Normal (6), Maximum (9)
- [ ] **T904** [US9] Add `CompressionOptions` struct: format, level, password, output_path

### Format Writers

- [ ] **T905** [P] [US9] Implement `compress_zip(sources: &[PathBuf], dest: &Path, opts: CompressionOptions, tx: Sender<Progress>) -> Result<()>`
- [ ] **T906** [US9] Use `zip::ZipWriter` with configurable compression level (0-9)
- [ ] **T907** [US9] Support ZIP64 extension for files >4GB automatically
- [ ] **T908** [US9] Apply AES-256 encryption when password provided using `zip::write::FileOptions::with_aes_encryption()`
- [ ] **T909** [US9] Preserve file timestamps (mtime) in ZIP entries
- [ ] **T910** [US9] Add files recursively: iterate directories, add each file with relative path

- [ ] **T911** [P] [US9] Implement `compress_tar(sources: &[PathBuf], dest: &Path, compression: CompressionType, tx: Sender<Progress>) -> Result<()>`
- [ ] **T912** [US9] Create TAR builder with `tar::Builder<Box<dyn Write>>`
- [ ] **T913** [US9] Wrap writer with compression: `GzEncoder` for TAR.GZ, `BzEncoder` for TAR.BZ2, `XzEncoder` for TAR.XZ
- [ ] **T914** [US9] Preserve Unix permissions (chmod) in TAR entries
- [ ] **T915** [US9] Preserve symlinks in TAR (append_link)
- [ ] **T916** [US9] Use PAX headers for UTF-8 filenames and long paths

- [ ] **T917** [P] [US9] Implement `compress_7z(sources: &[PathBuf], dest: &Path, opts: CompressionOptions, tx: Sender<Progress>) -> Result<()>`
- [ ] **T918** [US9] Use `sevenz_rust::SevenZWriter` with password support
- [ ] **T919** [US9] Configure compression level (0-9)
- [ ] **T920** [US9] Apply encryption with `Password::from()` when password provided

### Progress & Estimation

- [ ] **T921** [US9] Implement `estimate_compressed_size(sources: &[PathBuf]) -> u64`
- [ ] **T922** [US9] Use heuristics: text files ~60%, images ~95%, already compressed ~100%
- [ ] **T923** [US9] Send progress updates: current file, files done/total, bytes processed
- [ ] **T924** [US9] Calculate compression ratio: original_size / compressed_size

### UI Dialog

- [ ] **T925** [P] [US9] Add `DialogState::CompressOptions` variant to `src/app.rs`
- [ ] **T926** [US9] Fields: sources (Vec<PathBuf>), output_name (String), format (ArchiveFormat), level (CompressionLevel), password (Option<String>), confirm_password (Option<String>), selected_field (usize)
- [ ] **T927** [US9] Implement `start_compress_archive()` in `src/app.rs`
- [ ] **T928** [US9] Pre-fill output name: single file → "[name].zip", multiple → "archive_[YYYY-MM-DD].zip"
- [ ] **T929** [US9] Show count: "Comprimir X elementos" if multiple selected
- [ ] **T930** [US9] Render `render_compress_options_dialog()` in `src/ui/dialog.rs`
- [ ] **T931** [US9] Show format selector with descriptions: "ZIP (rápido, compatible)", "TAR.GZ (Linux, bueno)", "TAR.BZ2 (mejor)", "7Z (máxima)"
- [ ] **T932** [US9] Show compression level selector: "Rápido / Normal / Máximo"
- [ ] **T933** [US9] Show password checkbox (disabled for TAR formats)
- [ ] **T934** [US9] Show password input fields (two for confirmation) when checkbox active
- [ ] **T935** [US9] Show estimated size: "~1.2 MB estimado (ratio 45%)"
- [ ] **T936** [US9] Validate: passwords match, output name not empty, no file exists (or confirm overwrite)

### Key Bindings

- [ ] **T937** [P] [US9] Add `Action::CompressArchive` to `src/events/keybindings.rs`
- [ ] **T938** [US9] Map Shift+F9 to `Action::CompressArchive`
- [ ] **T939** [US9] Update footer hint: add "Shift+F9:Compress" to line2_bindings
- [ ] **T940** [US9] Handle dialog navigation: Tab/Shift+Tab between fields, arrows for selectors
- [ ] **T941** [US9] Handle Space to toggle checkboxes (password, level)
- [ ] **T942** [US9] Handle Enter to confirm and start compression

### Compression Execution

- [ ] **T943** [P] [US9] Handle `ConfirmYes` for CompressOptions dialog in `src/main.rs`
- [ ] **T944** [US9] Extract options from dialog, validate passwords match
- [ ] **T945** [US9] Check disk space: estimated size < available space
- [ ] **T946** [US9] Check output file doesn't exist, or show overwrite confirmation
- [ ] **T947** [US9] Create progress channel and show progress dialog
- [ ] **T948** [US9] Call appropriate compress function based on format
- [ ] **T949** [US9] Refresh active panel after compression completes
- [ ] **T950** [US9] Select newly created archive file automatically

### Error Handling

- [ ] **T951** [US9] Handle file not found during compression: skip with warning or abort
- [ ] **T952** [US9] Handle permission denied: show error, offer skip/retry/cancel
- [ ] **T953** [US9] Handle insufficient disk space: detect before starting, error gracefully
- [ ] **T954** [US9] Handle output file already exists: confirm overwrite before starting
- [ ] **T955** [US9] Handle cancellation: Esc shows confirmation, deletes partial archive
- [ ] **T956** [US9] Limit to 100K files or 50GB: show warning "Operación muy larga, ¿continuar?"

### Testing

- [ ] **T957** [P] [US9] Create `tests/unit/compressor_tests.rs`
- [ ] **T958** [US9] Test compress single file to ZIP
- [ ] **T959** [US9] Test compress multiple files to ZIP
- [ ] **T960** [US9] Test compress directory recursively to TAR.GZ
- [ ] **T961** [US9] Test compress with password (ZIP and 7Z)
- [ ] **T962** [US9] Test compression level affects output size
- [ ] **T963** [US9] Test ZIP64 for files >4GB
- [ ] **T964** [US9] Test TAR preserves Unix permissions
- [ ] **T965** [US9] Test estimate_compressed_size() approximation
- [ ] **T966** [US9] Test cancellation deletes partial archive

---

## Summary

**Total Tasks**: 262
- **Setup (Phase 0)**: 6 tasks
- **US1 - Navegación (P1 - MVP)**: 38 tasks
- **US2 - Copiar/Mover (P2)**: 29 tasks  
- **US3 - Eliminar/Crear (P3)**: 20 tasks
- **US4 - Búsqueda/Filtro (P4)**: 15 tasks
- **US5 - Selección Múltiple (P5)**: 37 tasks ⭐ NEW
- **US6 - Preview Texto (P6)**: 37 tasks ⭐ NEW
- **US7 - Preview Imágenes (P7)**: 34 tasks ⭐ NEW
- **US8 - Descompresión (P8)**: 62 tasks ⭐ NEW
- **Config/Persistence (Phase 5)**: 13 tasks
- **Polish/Docs (Phase 6)**: 17 tasks

**Estimated Effort**: 
- US1 (MVP): ~2-3 days (core navigation working) ✅ DONE
- US2: ~1-2 days (copy/move operations) ✅ DONE
- US3: ~1 day (delete/create) ✅ DONE
- US4: ~1 day (search/filter) ✅ DONE
- **Phase 5 (Persistence)**: ~1 day ✅ DONE
- **US5 (Multi-select)**: ~1-2 days ⏳ NEW
- **US6 (Text preview)**: ~1-2 days ⏳ NEW
- **US7 (Image preview)**: ~2-3 days ⏳ NEW (includes ASCII art conversion)
- **US8 (Archive extraction)**: ~3-4 days ⏳ NEW (complex: multiple formats, passwords)
- **Phase 6 (Polish)**: ~1 day ⏳ PENDING

**Incremental Delivery**: Each User Story (US1-US8) can be implemented and tested independently, allowing for MVP delivery after US1 completion.

**Current Status**: 
- ✅ **Phases 0-5 Complete**: Setup, Navigation, File Operations, Search, Config Persistence (121/121 tasks)
- ⏳ **Phase 5.5-5.8 Pending**: Multi-select, Preview (Text/Image), Archive Extraction (170 new tasks)
- ⏳ **Phase 6 Pending**: Polish, Documentation, Performance (17 tasks)

**New Features Added**:
1. **Multi-select**: Space to mark, Ctrl+A for all, batch operations on marked items
2. **Text Preview**: F4 on text files, modal with scroll, line numbers, encoding detection
3. **Image Preview**: F4 on images, ASCII/Unicode art representation, color adaptation
4. **Archive Extraction**: F9 to extract ZIP/TAR/7Z/RAR, password support, progress tracking
