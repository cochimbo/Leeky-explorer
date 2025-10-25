# Implementation Plan: Quality of User Experience Improvements

**Branch**: `003-quality-of-user` | **Date**: 2025-01-20 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/003-quality-of-user/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Implement a welcome screen that displays on application launch, featuring ASCII art branding and the current version number. User presses Enter to proceed to the main file manager interface. Technical approach uses existing Ratatui rendering capabilities, crossterm event handling, and integrates with current application state management.

## Technical Context

**Language/Version**: Rust (edition = "2024")  
**Primary Dependencies**: 
- ratatui 0.29.0 (TUI framework)
- crossterm 0.29.0 (terminal handling)
- tokio 1.35 (async runtime with "full" features)
- anyhow 1.0 (error handling)

**Storage**: Static ASCII art file in assets/images/, version from Cargo.toml  
**Testing**: cargo test with tempfile 3.8 (dev dependency), manual terminal testing  
**Target Platform**: Cross-platform terminal (Linux, Windows, macOS)
**Project Type**: Single project - dual-pane TUI file manager  
**Performance Goals**: <1 second welcome screen display time, instant Enter key response  
**Constraints**: Must work in terminals as small as 80x24, graceful fallback for missing assets  
**Scale/Scope**: Single welcome screen, ~100 lines of code, 1 ASCII art asset file

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

[Gates determined based on constitution file]

## Project Structure

### Documentation (this feature)

```
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)
### Source Code (repository root)

```
src/
├── main.rs              # Modified: Show welcome screen before initializing panels
├── ui/
│   ├── mod.rs           # Modified: Export welcome_screen module, add render_welcome function
│   ├── welcome_screen.rs # New: Welcome screen rendering and logic
│   ├── layout.rs        # Existing: May need welcome screen layout
│   └── [other ui files] # Existing: dialog.rs, panel_widget.rs, etc.
├── app.rs               # Modified: Add show_welcome: bool field to AppState
├── event_loop.rs        # Modified: Handle welcome screen state in main loop
└── events/
    └── handler.rs       # Modified: Handle Enter key when show_welcome is true

assets/
└── images/
    ├── logo.txt         # New: ASCII art logo file
    └── leekpc.png       # Existing: Project image asset

tests/
├── unit/
│   └── welcome_screen_test.rs # New: Unit tests for welcome screen
└── [other test files]   # Existing: file_operations_test.rs, config_test.rs, etc.

Cargo.toml               # Modified later: Update to version 0.3.0 when ready
```

**Structure Decision**: Single project structure with welcome screen as a new UI module in `src/ui/`. The project does NOT use an AppScreen enum - instead it uses `AppState` with various state flags and dialog states. The welcome screen will use a simple `show_welcome: bool` flag in AppState. When true, render welcome screen instead of panels. When user presses Enter, set to false and proceed to normal file manager UI.

**Structure Decision**: [Document the selected structure and reference the real
directories captured above]

## Complexity Tracking

*No constitution violations - this is a simple, focused feature addition*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |

## Implementation Phases

### Phase 0: Setup and Asset Creation
**Goal**: Prepare welcome screen infrastructure

1. **Create ASCII Art Logo**
   - Create `assets/images/logo.txt` with ASCII art
   - Test that art displays correctly in different terminal sizes
   - Create fallback message if file is missing: "Leeky File Manager"

2. **Add show_welcome Flag to AppState**
   - Modify `src/app.rs` to add `pub show_welcome: bool` field to AppState struct
   - Initialize to `true` in `AppState::new()` method
   - This flag controls whether to show welcome screen or normal UI

### Phase 1: Welcome Screen Module
**Goal**: Implement welcome screen rendering logic

1. **Create `src/ui/welcome_screen.rs`**
   ```rust
   // Core structure:
   - pub fn render(frame: &mut Frame, area: Rect, version: &str)
   - fn load_logo() -> Result<String, Error> // Load from assets/images/logo.txt
   - fn center_text(text: &str, area: Rect) -> Layout
   ```

2. **Rendering Logic**
   - Use Ratatui's `Paragraph` widget for ASCII art
   - Use `Block` with centered text for version display
   - Calculate vertical centering based on terminal height
   - Handle small terminals: show simplified version if height < 24

3. **Export Module**
   - Add `pub mod welcome_screen;` to `src/ui/mod.rs`

### Phase 2: Event Handling
**Goal**: Detect Enter key and transition to main interface

1. **Modify `src/events/handler.rs`**
   - Add early check at beginning of `handle_key` function
   - If `app.show_welcome == true` and key is Enter: set `app.show_welcome = false`
   - If `app.show_welcome == true` and key is NOT Enter: return early (ignore all other keys)
   - This prevents any file manager actions during welcome screen

2. **State Transition**
   - Setting `app.show_welcome = false` causes next render to show normal panels
   - Panels are already initialized in main.rs before event loop starts
   - No additional initialization needed

### Phase 3: Rendering Integration
**Goal**: Show welcome screen on application startup

1. **Modify `src/event_loop.rs`**
   - In the `run()` function's render loop, check `app.show_welcome`
   - If true: call `ui::render_welcome()` instead of normal UI rendering
   - If false: proceed with existing render logic (panels, dialogs, etc.)
   - Pass version string from `env!("CARGO_PKG_VERSION")` to render_welcome

2. **Modify `src/ui/mod.rs`**
   - Add public function `pub fn render_welcome(frame: &mut Frame, version: &str)`
   - This function calls `welcome_screen::render()` with full terminal area

### Phase 4: Testing and Edge Cases
**Goal**: Verify all scenarios from spec

1. **Unit Tests** (`tests/unit/welcome_screen_test.rs`)
   - Test logo loading (file exists)
   - Test fallback (file missing)
   - Test small terminal handling (width/height < threshold)
   - Test version string formatting

2. **Manual Testing**
   - Test on Windows PowerShell, Linux terminals, macOS Terminal.app
   - Test terminal resize during welcome screen
   - Test with missing logo file (delete assets/images/logo.txt temporarily)
   - Test in very small terminal (80x24 minimum)
   - Verify Enter key transitions correctly
   - Verify all other keys are ignored during welcome screen

## Technical Decisions

### Architecture

**State Flag Approach**: Welcome screen uses a `show_welcome: bool` field in AppState instead of an enum-based screen system (which doesn't exist in this codebase). When `show_welcome` is true, the event loop renders the welcome screen instead of the normal dual-pane interface. This is the simplest integration pattern for this architecture.

**Rendering Strategy**: Use standard Ratatui widgets (Paragraph, Block). Center content using Layout calculations based on terminal dimensions. The welcome screen gets the full terminal area and renders independently.

**Asset Loading**: Attempt to load logo from `assets/images/logo.txt` at render time. If file missing or unreadable, use hardcoded fallback text "Leeky File Manager v{version}".

**Version Source**: Use Rust's `env!("CARGO_PKG_VERSION")` macro at compile time. This reads version "0.2.0" (soon 0.3.0) from Cargo.toml. No runtime file reading needed.

**Event Flow**: The existing event_loop.rs module handles the main loop. We add a check early in the render cycle and in event handling to intercept when welcome screen is active.

### File Changes

| File | Change Type | Purpose |
|------|-------------|---------|
| `src/app.rs` | Modified | Add `pub show_welcome: bool` field to AppState, init to true |
| `src/ui/mod.rs` | Modified | Export welcome_screen module, add render_welcome() function |
| `src/ui/welcome_screen.rs` | New | Welcome screen rendering logic |
| `src/events/handler.rs` | Modified | Check show_welcome flag, handle Enter key to dismiss |
| `src/event_loop.rs` | Modified | Conditional rendering: welcome screen vs normal UI |
| `assets/images/logo.txt` | New | ASCII art logo asset |
| `tests/unit/welcome_screen_test.rs` | New | Unit tests |

### Dependencies

**No new dependencies required**. All functionality uses existing crates:
- Ratatui: Already used for all UI rendering
- Crossterm: Already used for key event handling
- std::fs: For reading logo file
- env!() macro: Built into Rust

## Success Criteria Mapping

| Success Criterion | Implementation | Verification |
|-------------------|----------------|--------------|
| SC-001: 100% launch display | Welcome screen is initial AppScreen state | Manual test: launches every time |
| SC-002: <1s transition | Direct state change, no async | Manual test: measure with stopwatch |
| SC-003: Clear version display | env!("CARGO_PKG_VERSION") in centered text | Visual inspection |
| SC-004: Graceful fallbacks | if file missing → hardcoded text | Test with deleted logo.txt |
| SC-005: 95% terminal compatibility | Test on 20+ terminals | Test matrix (see Phase 4) |

## Open Questions

None - all technical decisions confirmed with user.
