# Tasks: Quality of User Experience Improvements

**Input**: Design documents from `/specs/003-quality-of-user/`
**Prerequisites**: plan.md ✅, spec.md ✅

**Tests**: Manual testing across terminal emulators will be performed in Phase 4.

**Organization**: Tasks are organized by implementation phase as this is a single user story feature.

## Format: `[ID] [P?] [Phase] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Phase]**: Setup, Foundation, Implementation, Testing
- Include exact file paths in descriptions

---

## Phase 1: Setup (Asset Verification)

**Purpose**: Verify existing PNG logo and prepare dynamic ASCII conversion

- [ ] T001 Verify `assets/images/leekpc.png` exists and is readable
- [ ] T002 Test dynamic conversion: use existing `image_to_ascii()` from `src/preview/image_viewer.rs`
- [ ] T003 Test converted ASCII displays correctly in 80x24 terminal (minimum size)
- [ ] T004 Test converted ASCII displays correctly in larger terminals (120x40, 200x60)

**Note**: Welcome screen will use `load_image()` and `image_to_ascii()` functions that already exist in the codebase to dynamically convert `leekpc.png` to ASCII art at runtime. No static logo.txt file needed.

**Deliverable**: Confirmed that PNG → ASCII conversion works for welcome screen

---

## Phase 2: Foundation (AppState Modification)

**Purpose**: Add welcome screen state flag to AppState

**⚠️ CRITICAL**: This must be complete before UI and event handling tasks

- [ ] T004 Add `pub show_welcome: bool` field to `AppState` struct in `src/app.rs`
- [ ] T005 Initialize `show_welcome: true` in `AppState::new()` method in `src/app.rs`
- [ ] T006 Verify project compiles after AppState changes (`cargo build`)

**Checkpoint**: AppState ready with welcome screen flag - UI implementation can begin

---

## Phase 3: User Story 1 - Welcome Screen with Branding (Priority: P1) 🎯

**Goal**: Display ASCII art logo and version on launch, user presses Enter to proceed

**Independent Test**: Run `cargo run`, see welcome screen, press Enter, see file manager

### UI Implementation

- [ ] T007 Create new file `src/ui/welcome_screen.rs` with module structure
- [ ] T008 Import `load_image` and `image_to_ascii` from `crate::preview::image_viewer`
- [ ] T009 Implement `load_logo(max_width: u32, max_height: u32) -> Result<String, Error>` function
- [ ] T010 In `load_logo()`: call `load_image("assets/images/leekpc.png")`
- [ ] T011 In `load_logo()`: call `image_to_ascii(&img, max_width, max_height)` to convert dynamically
- [ ] T012 Implement fallback logic: if PNG load/convert fails, return "Leeky File Manager v{version}"
- [ ] T013 Implement `render(frame: &mut Frame, area: Rect, version: &str)` function
- [ ] T014 In `render()`: calculate max_width/max_height from terminal area dimensions
- [ ] T015 In `render()`: call `load_logo()` to get ASCII art dynamically
- [ ] T016 Use Ratatui `Paragraph` widget to display ASCII art centered vertically and horizontally
- [ ] T017 Add version display below logo using `Block` widget with centered text
- [ ] T018 Add "Press Enter to continue..." instruction at bottom of screen
- [ ] T019 Handle small terminals: simplify display if height < 24 or width < 80

### Module Export

- [ ] T020 Add `pub mod welcome_screen;` to `src/ui/mod.rs`
- [ ] T021 Add `pub fn render_welcome(frame: &mut Frame, version: &str)` function to `src/ui/mod.rs`
- [ ] T022 In `render_welcome()`, call `welcome_screen::render()` with full terminal area

### Event Loop Integration

- [ ] T023 Modify `run()` function in `src/event_loop.rs` to check `app.show_welcome` flag
- [ ] T024 If `show_welcome == true`: call `ui::render_welcome(frame, env!("CARGO_PKG_VERSION"))`
- [ ] T025 If `show_welcome == false`: proceed with existing panel rendering logic
- [ ] T026 Verify conditional rendering works (compile and basic test)

### Event Handling

- [ ] T027 Modify `handle_key()` function in `src/events/handler.rs`
- [ ] T028 Add early check: if `app.show_welcome == true` at function start
- [ ] T029 If welcome screen active and key is `KeyCode::Enter`: set `app.show_welcome = false` and return
- [ ] T030 If welcome screen active and key is NOT Enter: return early (ignore all other keys)
- [ ] T031 Verify Enter key transitions to file manager correctly

**Checkpoint**: Welcome screen fully functional - ready for testing

---

## Phase 4: Testing and Validation

**Purpose**: Verify all acceptance criteria and edge cases from spec.md

### Unit Tests

- [ ] T032 Create `tests/unit/welcome_screen_test.rs` file
- [ ] T033 [P] Write test: `test_load_logo_success()` - verify PNG converts to ASCII when file exists
- [ ] T034 [P] Write test: `test_load_logo_fallback()` - verify fallback when PNG missing/invalid
- [ ] T035 [P] Write test: `test_version_formatting()` - verify version string displays correctly
- [ ] T036 Run `cargo test` and ensure all welcome_screen tests pass

### Manual Testing - Cross-Platform

- [ ] T037 Test on Windows PowerShell 5.1 (user's default shell)
- [ ] T038 Test on Windows Terminal (modern terminal emulator)
- [ ] T039 Test on Linux terminal emulator (if available)
- [ ] T040 Test on macOS Terminal.app (if available)

### Manual Testing - Edge Cases

- [ ] T041 Test with missing PNG: temporarily rename `leekpc.png`, verify fallback text shows
- [ ] T042 Test with corrupted PNG: create invalid PNG file, verify fallback
- [ ] T043 Test minimum terminal size: resize to 80x24, verify display is readable
- [ ] T044 Test very small terminal: resize to 60x20, verify graceful degradation
- [ ] T045 Test large terminal: resize to 200x60, verify ASCII art scales appropriately
- [ ] T046 Test terminal resize during welcome screen: verify layout adjusts correctly
- [ ] T047 Test Enter key dismisses welcome screen and shows file manager
- [ ] T048 Test other keys are ignored during welcome screen (arrows, letters, Esc, etc.)

### Acceptance Criteria Verification

- [ ] T049 **SC-001**: Verify welcome screen displays on 100% of application launches
- [ ] T050 **SC-002**: Measure transition time from welcome to main UI (must be <1 second)
- [ ] T051 **SC-003**: Verify version number displays clearly and matches Cargo.toml
- [ ] T052 **SC-004**: Verify graceful fallback when PNG file is missing or corrupted
- [ ] T053 **SC-005**: Test on at least 5 different terminal emulators (aiming for 95% compatibility)

**Deliverable**: All acceptance criteria met, feature ready for production

---

## Phase 5: Documentation and Release Prep

**Purpose**: Prepare for v0.3.0 release

- [ ] T054 Update `Cargo.toml` version from "0.2.0" to "0.3.0"
- [ ] T055 Update `RELEASE.md` with v0.3.0 changelog entry
- [ ] T056 Update `README.md` to mention new welcome screen feature (optional)
- [ ] T057 Run final `cargo build --release` to verify release binary
- [ ] T058 Test release binary: run welcome screen with release build

---

## Summary

**Total Tasks**: 58
**Estimated Time**: 
- Phase 1 (Setup): 1-2 hours (verification only, no asset creation)
- Phase 2 (Foundation): 30 minutes
- Phase 3 (Implementation): 5-6 hours (dynamic conversion adds complexity)
- Phase 4 (Testing): 3-4 hours
- Phase 5 (Release): 1 hour

**Total Estimate**: 10-14 hours of development time

**Parallel Opportunities**: 
- T033-T035 can be done in parallel (unit tests)
- T037-T040 can be done in parallel if multiple platforms available

**Critical Path**: T005-T006 (Foundation) must complete before Phase 3 begins

**Minimum Viable**: T001, T005-T006, T007-T031, T047, T049 = Core functionality working

**Key Change from Original Plan**: Welcome screen now uses **dynamic PNG→ASCII conversion** at runtime using existing `image_to_ascii()` function, instead of static logo.txt file. This provides consistent branding with existing image preview functionality.
