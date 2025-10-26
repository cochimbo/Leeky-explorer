# Implementation Plan: Quick Wins - Enhanced Navigation & Productivity

**Branch**: `004-quick-wins-bookmarks` | **Date**: 2025-10-26 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/004-quick-wins-bookmarks/spec.md`

## Summary

Implement five high-value, low-complexity features to enhance user productivity:
1. **Bookmarks (P1)**: Persistent favorite directories accessible via Ctrl+B
2. **Disk Usage Indicators (P2)**: Visual progress bars in drive selector showing space usage
3. **Navigation History (P3)**: Modal history viewer with Ctrl+H showing last 20 directories
4. **Go To Path (P3)**: Quick navigation dialog via Ctrl+G to jump to any directory path
5. **Text Editor (P4)**: Simple in-app editor for text files via F4 (edit mode in preview) - DEFERRED

**Technical Approach**: 
- Extend existing `config::state` module for bookmarks persistence
- Enhance `ui::drive_selector` with disk usage visualization using existing `fs::disk_info`
- Add history buffer to `models::panel` for navigation tracking (20-entry circular buffer)
- Create new `ui::goto_dialog` for path input and validation
- Create new `ui::text_editor` modal component following existing modal patterns (Phase 5)

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2024)
**Primary Dependencies**: 
- `ratatui` 0.29.0 - TUI framework
- `crossterm` 0.28 - Terminal manipulation
- `serde` + `serde_json` - Configuration serialization
- `anyhow` - Error handling
- `chrono` - Timestamp handling for bookmarks

**Storage**: 
- Configuration files in `~/.config/leeky/` (Linux/macOS) or `%APPDATA%/leeky/` (Windows)
- JSON format for bookmarks: `bookmarks.json`
- History stored in memory (not persisted)

**Testing**: `cargo test` with existing test infrastructure (83 tests across 8 suites)
**Target Platform**: Cross-platform (Linux, Windows, macOS, ARM)
**Project Type**: Single binary TUI application
**Performance Goals**: 
- Bookmark operations: <100ms
- Disk usage queries: <500ms
- History navigation: <50ms per operation
- Editor file load: <200ms for files <100KB

**Constraints**:
- Binary size impact: <500KB
- Memory footprint: <10MB additional for all features
- No external editor dependencies
- Cross-platform compatibility maintained

**Scale/Scope**:
- Support 50+ bookmarks
- 20 history entries per panel (circular buffer)
- Path input with environment variable expansion
- Files up to 1MB for editing (Phase 5)
- 5 independent features with incremental delivery (4 in v0.4.0, editor deferred)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

✅ **PASSES** - All features align with existing architecture:
- Single binary, no new projects
- Extends existing modules (config, ui, models, fs)
- No new external dependencies beyond chrono (already used)
- Follows established patterns (modals, keybindings, state management)
- Cross-platform compatibility maintained
- No violations of complexity constraints

## Project Structure

### Documentation (this feature)

```
specs/004-quick-wins-bookmarks/
├── plan.md              # This file - Implementation strategy
├── spec.md              # Feature specification (complete)
├── tasks.md             # Task breakdown (to be generated)
└── checklists/          # Test checklists (to be created)
    └── requirements.md
```

### Source Code (repository root)

```
src/
├── models/
│   ├── panel.rs              # [MODIFY] Add NavigationHistory
│   └── bookmark.rs           # [NEW] Bookmark struct and collection
│
├── config/
│   ├── state.rs              # [MODIFY] Add bookmarks field
│   └── bookmarks.rs          # [NEW] BookmarkManager with persistence
│
├── ui/
│   ├── mod.rs                # [MODIFY] Export new modules
│   ├── bookmark_manager.rs   # [NEW] Ctrl+B bookmark menu widget
│   ├── history_dialog.rs     # [NEW] Ctrl+H history viewer modal
│   ├── goto_dialog.rs        # [NEW] Ctrl+G go to path dialog
│   ├── text_editor.rs        # [NEW] Simple text editor modal (Phase 5)
│   ├── drive_selector.rs     # [MODIFY] Add disk usage bars
│   └── panel_widget.rs       # [MODIFY] Show disk space in status bar
│
├── fs/
│   ├── disk_info.rs          # [MODIFY] Add usage percentage calculation
│   └── path_utils.rs         # [NEW or MODIFY] Path expansion utilities
│
├── events/
│   ├── keybindings.rs        # [MODIFY] Add Ctrl+B, Ctrl+H, Ctrl+G, F4 edit mode
│   └── handler.rs            # [MODIFY] Handle new actions
│
└── app.rs                    # [MODIFY] Integrate new features

tests/
├── bookmark_test.rs          # [NEW] Bookmark operations
├── history_test.rs           # [NEW] Navigation history
├── goto_test.rs              # [NEW] Go to path functionality
├── disk_usage_test.rs        # [NEW] Disk usage calculations
└── text_editor_test.rs       # [NEW] Editor functionality (Phase 5)
```

**Structure Decision**: Single project structure maintained. All features integrate into existing architecture through module extensions. No new external projects or services required. Follows established patterns for UI components (modals), state management (config module), and event handling (keybindings + handler).
## Implementation Phases

### Phase 0: Foundation (Shared Infrastructure)

**Duration**: 0.5 days
**Deliverable**: Core models and configuration setup

**Tasks**:
1. Add `chrono` dependency to `Cargo.toml` if not present
2. Create `src/models/bookmark.rs` with `Bookmark` struct
3. Create `src/config/bookmarks.rs` with `BookmarkManager`
4. Update `src/config/state.rs` to include bookmarks field
5. Add bookmark keybindings to `src/events/keybindings.rs` (Ctrl+B, bookmark actions)
6. Write unit tests for bookmark model and manager

**Success Criteria**:
- Bookmarks can be created, serialized, and deserialized
- BookmarkManager successfully reads/writes `bookmarks.json`
- Tests pass for bookmark persistence

---

### Phase 1: Bookmarks Feature (P1) - MVP

**Duration**: 1.5 days
**Deliverable**: Fully functional bookmarks with Ctrl+B menu

**Dependencies**: Phase 0

**Tasks**:
1. Create `src/ui/bookmark_manager.rs` modal widget
   - Menu rendering with bookmark list
   - "Add Bookmark" option
   - Delete ('d') and rename ('r') actions
   - Navigation to selected bookmark
2. Implement name input dialog for add/rename operations
3. Integrate bookmark manager into `src/app.rs`
4. Add bookmark actions to `src/events/handler.rs`
5. Handle edge cases:
   - Non-existent directory warnings
   - Duplicate name handling
   - Empty bookmark list display
6. Write integration tests for bookmark workflow
7. Update keybinding documentation

**Success Criteria**:
- Ctrl+B opens bookmark menu
- Users can add current directory as bookmark with custom name
- Bookmarks persist across restarts
- Navigation to bookmarked directories works
- Delete and rename operations function correctly
- All FR-001 through FR-008 requirements met

**Testing Checklist**:
- [ ] Create bookmark from various directories
- [ ] Navigate to bookmark changes active panel
- [ ] Bookmarks persist after app restart
- [ ] Delete bookmark removes it from list
- [ ] Rename bookmark updates display name
- [ ] Bookmark to deleted directory shows warning
- [ ] 50+ bookmarks perform without lag

---

### Phase 2: Disk Usage Indicators (P2)

**Duration**: 1 day
**Deliverable**: Visual disk usage in drive selector and status bar

**Dependencies**: None (can be parallel with Phase 1)

**Tasks**:
1. Enhance `src/fs/disk_info.rs`:
   - Add `usage_percentage()` method
   - Add `warning_level()` method (normal/warning/critical)
2. Modify `src/ui/drive_selector.rs`:
   - Render progress bar for each drive
   - Apply color based on usage level (green/yellow/red)
   - Show percentage text
3. Update `src/ui/panel_widget.rs`:
   - Display current drive free space in status bar
4. Add cross-platform testing for disk usage calculation
5. Write unit tests for usage percentage calculations
6. Handle edge cases:
   - Unmounted drives
   - Network drives with slow response
   - Drives with no stats available

**Success Criteria**:
- Drive selector shows visual bars for all drives
- High usage (>90%) displays in red
- Status bar shows current drive space
- Calculations accurate within 5% on all platforms
- All FR-009 through FR-014 requirements met

**Testing Checklist**:
- [ ] All drives show correct usage percentage
- [ ] Nearly full drives display warning colors
- [ ] Status bar updates when switching drives
- [ ] Network drives don't cause hangs
- [ ] Unmounted drives handled gracefully

---

### Phase 3: Navigation History (P3)

**Duration**: 1 day
**Deliverable**: Back/forward navigation with Alt+Left/Right

**Dependencies**: Phase 0 (keybinding infrastructure)

**Tasks**:
1. Add `NavigationHistory` struct to `src/models/panel.rs`:
   - `entries: Vec<PathBuf>` - History stack
   - `current_index: usize` - Current position
   - `max_size: usize` - Limit (100)
   - Methods: `push()`, `back()`, `forward()`, `clear_forward()`
2. Integrate history into `Panel` struct
3. Add history keybindings (Alt+Left, Alt+Right) to `src/events/keybindings.rs`
4. Implement history navigation in `src/events/handler.rs`
5. Update `Panel::enter_dir()` and `Panel::go_up()` to record history
6. Handle edge cases:
   - Navigation to non-existent directory in history
   - History limit overflow
   - Forward history cleared on new navigation
7. Write unit tests for history operations
8. Add visual indicators (optional): show history depth in status bar

**Success Criteria**:
- Alt+Left navigates backward in history
- Alt+Right navigates forward in history
- Each panel maintains independent history
- History limited to 100 entries per panel
- All FR-015 through FR-020 requirements met

**Testing Checklist**:
- [ ] Navigate through 10+ directories, back button works
- [ ] Forward button works after going back
- [ ] Forward history cleared when navigating to new directory
- [ ] Each panel has independent history
- [ ] Navigation to deleted historical directory handled
- [ ] 100+ navigation entries don't cause memory issues

---

### Phase 4: Text Editor (P4) - Optional/Deferred

**Duration**: 2 days
**Deliverable**: Simple in-app text editor

**Dependencies**: Phase 0, Phase 1 (modal patterns established)

**Tasks**:
1. Create `src/ui/text_editor.rs` modal component:
   - `EditorBuffer` struct with content, cursor, scroll
   - Render text with line numbers
   - Cursor rendering and movement
   - Insert/delete character operations
2. Add editor keybindings (F4 for preview, 'e' within preview for edit, Ctrl+S, Esc)
3. Implement file reading and writing
4. Add dirty flag and unsaved changes warning
5. Integrate editor into `src/events/handler.rs`
6. File size and type validation:
   - Reject binary files
   - Warn for large files (>1MB)
7. Handle edge cases:
   - File permissions (read-only)
   - UTF-8 encoding detection
   - Concurrent file modifications
8. Write integration tests for editor workflow
9. Optional: Basic syntax highlighting using `syntect`

**Success Criteria**:
- F4 opens preview, 'e' in preview switches to edit mode for text files
- Basic text editing works (insert, delete, navigate)
- Ctrl+S saves changes successfully
- Esc closes editor with unsaved warning
- Binary files rejected with error
- All FR-021 through FR-030 requirements met

**Testing Checklist**:
- [ ] Open and edit small text file (<10KB)
- [ ] Save changes persist to disk
- [ ] Unsaved changes warning on Esc
- [ ] Binary file detection works
- [ ] Large file warning displays
- [ ] Read-only file shows error on save
- [ ] Cursor movement works (arrows, home, end)
- [ ] Line numbers display correctly

**Note**: This phase has lowest priority and can be deferred to v0.4.0 if time constraints exist.

---

## Integration Strategy

### Phase 0 + Phase 1 + Phase 2 (MVP for v0.4.0)
- **Timeline**: 3 days total
- **Deliverables**: Bookmarks + Disk Usage (highest value features)
- **Testing**: 2 feature suites, ~20 new tests
- **Release**: Can be released as v0.4.0 with just these two

### Phase 3 Addition (Enhanced v0.4.0)
- **Timeline**: +1 day (4 days total)
- **Deliverables**: All three core features
- **Testing**: 3 feature suites, ~30 new tests
- **Release**: Full v0.4.0 with navigation history

### Phase 4 (Future v0.4.1 or v0.5.0)
- **Timeline**: +2 days (6 days total if done immediately)
- **Deliverables**: Complete feature set
- **Testing**: 4 feature suites, ~40 new tests
- **Release**: Can be separate minor version due to complexity

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|---------|------------|
| Bookmark persistence fails on some platforms | Low | Medium | Extensive cross-platform testing, fallback to defaults |
| Disk usage queries slow on network drives | Medium | Low | Add timeout, show "calculating..." indicator |
| History memory growth | Low | Medium | Strict 100-entry limit, efficient PathBuf storage |
| Editor complexity scope creep | High | High | Defer to Phase 4, keep feature minimal, consider external editor |
| Keybinding conflicts | Low | Low | Document in README, make configurable in future |

## Dependencies Graph

```
Phase 0 (Foundation)
    ├── Phase 1 (Bookmarks) ← CRITICAL PATH
    ├── Phase 3 (History)
    └── Phase 4 (Editor)

Phase 2 (Disk Usage) ← PARALLEL, no dependencies
```

**Recommended Execution Order**:
1. Phase 0 + Phase 1 (Bookmarks) - 2 days - Deploy as MVP
2. Phase 2 (Disk Usage) - 1 day - Add to release
3. Phase 3 (History) - 1 day - Complete v0.4.0
4. Phase 4 (Editor) - Defer to v0.4.1 or later

## Performance Budget

| Feature | Binary Size | Memory | Startup Time | Operation Time |
|---------|------------|---------|--------------|----------------|
| Bookmarks | <50KB | <1MB | <10ms | <100ms |
| Disk Usage | <20KB | <512KB | <5ms | <500ms |
| History | <30KB | <5MB | <5ms | <50ms |
| Editor | <200KB | <10MB | <50ms | <200ms |
| **Total** | **<300KB** | **<17MB** | **<70ms** | **N/A** |

## Rollout Plan

1. **Development**: Feature branch `004-quick-wins-bookmarks`
2. **Testing**: All tests pass, manual testing on Windows/Linux/macOS
3. **Documentation**: Update README.md with new keybindings and features
4. **Release**: Merge to master, tag v0.4.0
5. **Monitoring**: GitHub issues for bug reports, user feedback

## Success Metrics (Post-Release)

- Bookmark usage: 80% of users create at least 1 bookmark in first week
- Disk usage: Visible immediately, no performance complaints
- History: 50% of users discover and use within first session
- Editor: (if included) 30% of users try editor feature
- Overall: Zero critical bugs, <5 minor bugs in first month
