# Tasks: Quality of User Experience Improvements

**Input**: Design documents from `/specs/003-quality-of-user/`
**Prerequisites**: plan.md ✅, spec.md ✅

**Tests**: Manual testing across terminal emulators will be performed in Phase 4.

**Organization**: Tasks are organized by implementation phase. Now includes two user stories:
- **User Story 1**: Welcome Screen (T001-T060) - COMPLETED ✅
- **User Story 2**: Disk Space Information (T061-T100) - NEW

## Format: `[ID] [P?] [Phase] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Phase]**: Setup, Foundation, Implementation, Testing
- Include exact file paths in descriptions

---

## Phase 1: Setup (Asset Verification)

**Purpose**: Verify existing PNG logo and prepare dynamic ASCII conversion

- [x] T001 Verify `assets/images/leekpc.png` exists and is readable ✅
- [x] T002 Test dynamic conversion: use existing `image_to_ascii()` from `src/preview/image_viewer.rs` ✅
- [x] T003 Test converted ASCII displays correctly in 80x24 terminal (minimum size) ✅
- [x] T004 Test converted ASCII displays correctly in larger terminals (120x40, 200x60) ✅

**Note**: Welcome screen will use `image::open()` and `image_to_ascii()` functions that already exist in the codebase to dynamically convert `leekpc.png` to ASCII art at runtime. No static logo.txt file needed.

**Deliverable**: Confirmed that PNG → ASCII conversion works for welcome screen ✅

---

## Phase 2: Foundation (AppState Modification)

**Purpose**: Add welcome screen state flag to AppState

**⚠️ CRITICAL**: This must be complete before UI and event handling tasks

- [x] T005 Add `pub show_welcome: bool` field to `AppState` struct in `src/app.rs` ✅
- [x] T006 Initialize `show_welcome: true` in `AppState::new()` method in `src/app.rs` ✅
- [x] T007 Verify project compiles after AppState changes (`cargo build`) ✅

**Checkpoint**: AppState ready with welcome screen flag - UI implementation can begin ✅

---

## Phase 3: User Story 1 - Welcome Screen with Branding (Priority: P1) 🎯

**Goal**: Display ASCII art logo and version on launch, user presses Enter to proceed

**Independent Test**: Run `cargo run`, see welcome screen, press Enter, see file manager

### UI Implementation

- [x] T008 Create new file `src/ui/welcome_screen.rs` with module structure ✅
- [x] T009 Import `image_to_ascii` from `crate::preview::image_viewer` ✅
- [x] T010 Implement `load_logo(terminal_area: Rect) -> String` function ✅
- [x] T011 In `load_logo()`: use `image::open("assets/images/leekpc.png")` for sync loading ✅
- [x] T012 In `load_logo()`: call `image_to_ascii(&img, max_width, max_height)` to convert dynamically ✅
- [x] T013 Implement fallback logic: if PNG load/convert fails, return fallback ASCII art ✅
- [x] T014 Implement `render(frame: &mut Frame, area: Rect, version: &str)` function ✅
- [x] T015 In `render()`: calculate max_width/max_height from terminal area dimensions ✅
- [x] T016 In `render()`: call `load_logo()` to get ASCII art dynamically ✅
- [x] T017 Use Ratatui `Paragraph` widget with `parse_ansi_line()` to display colored ASCII art ✅
- [x] T018 Add version display below logo using centered text with cyan color ✅
- [x] T019 Add "Press Enter to continue..." instruction at bottom with green highlight ✅
- [x] T020 Handle small terminals: implement `render_minimal()` if height < 10 or width < 40 ✅

### Module Export

- [x] T021 Add `pub mod welcome_screen;` to `src/ui/mod.rs` ✅
- [x] T022 Add `pub fn render_welcome(frame: &mut Frame, version: &str)` function to `src/ui/mod.rs` ✅
- [x] T023 In `render_welcome()`, call `welcome_screen::render()` with full terminal area ✅
- [x] T024 **BONUS**: Refactor `parse_ansi_line()` to `ui/mod.rs` to share between welcome_screen and preview_modal (DRY) ✅

### Event Loop Integration

- [x] T025 Modify `run()` function in `src/event_loop.rs` to check `app.show_welcome` flag ✅
- [x] T026 If `show_welcome == true`: call `ui::render_welcome(frame, env!("CARGO_PKG_VERSION"))` ✅
- [x] T027 If `show_welcome == false`: proceed with existing panel rendering logic ✅
- [x] T028 Verify conditional rendering works (compile and basic test) ✅
- [x] T029 **BUGFIX**: Add KeyEventKind::Press filter to prevent double key events ✅

### Event Handling

- [x] T030 Modify `handle_key()` function in `src/events/handler.rs` ✅
- [x] T031 Add early check: if `app.show_welcome == true` at function start ✅
- [x] T032 If welcome screen active and key is `KeyCode::Enter`: set `app.show_welcome = false` and return ✅
- [x] T033 If welcome screen active and key is NOT Enter: return early (ignore all other keys) ✅
- [x] T034 Verify Enter key transitions to file manager correctly ✅

### Image Conversion Optimization

- [x] T035 **IMPROVEMENT**: Adjust brightness thresholds from [64,128,192,255] to [32,96,160,220] ✅
- [x] T036 **IMPROVEMENT**: Remove light shade (░) character to reduce background noise ✅
- [x] T037 **IMPROVEMENT**: Remove dark shade (▓) character, use full block (█) instead ✅
- [x] T038 **IMPROVEMENT**: Keep only medium shade (▒) for mid-tone transitions ✅

**Checkpoint**: Welcome screen fully functional - ready for testing ✅

---

## Phase 4: Testing and Validation

**Purpose**: Verify all acceptance criteria and edge cases from spec.md

### Manual Testing - Cross-Platform

- [x] T039 Test on Windows PowerShell 5.1 (user's default shell) ✅
- [ ] T040 Test on Windows Terminal (modern terminal emulator)
- [ ] T041 Test on Linux terminal emulator (if available)
- [ ] T042 Test on macOS Terminal.app (if available)

### Manual Testing - Edge Cases

- [ ] T043 Test with missing PNG: temporarily rename `leekpc.png`, verify fallback text shows
- [ ] T044 Test with corrupted PNG: create invalid PNG file, verify fallback
- [x] T045 Test minimum terminal size: resize to 80x24, verify display is readable ✅
- [x] T046 Test very small terminal: resize to 60x20, verify graceful degradation ✅
- [x] T047 Test large terminal: resize to 200x60, verify ASCII art scales appropriately ✅
- [ ] T048 Test terminal resize during welcome screen: verify layout adjusts correctly
- [x] T049 Test Enter key dismisses welcome screen and shows file manager ✅
- [x] T050 Test other keys are ignored during welcome screen (arrows, letters, Esc, etc.) ✅

### Acceptance Criteria Verification

- [x] T051 **SC-001**: Verify welcome screen displays on 100% of application launches ✅
- [x] T052 **SC-002**: Measure transition time from welcome to main UI (must be <1 second) ✅
- [x] T053 **SC-003**: Verify version number displays clearly and matches Cargo.toml ✅
- [ ] T054 **SC-004**: Verify graceful fallback when PNG file is missing or corrupted
- [ ] T055 **SC-005**: Test on at least 5 different terminal emulators (aiming for 95% compatibility)

**Deliverable**: Core functionality verified, ready for additional user stories

---

## Phase 5: Documentation and Release Prep

**Purpose**: Prepare for v0.3.0 release (ON HOLD - more user stories pending)

- [ ] T056 Update `Cargo.toml` version from "0.2.0" to "0.3.0"
- [ ] T057 Update `RELEASE.md` with v0.3.0 changelog entry
- [x] T058 Update `README.md` to add logo image (line 3) ✅
- [ ] T059 Run final `cargo build --release` to verify release binary
- [ ] T060 Test release binary: run welcome screen with release build

**Note**: Release preparation on hold pending additional user stories for v0.3.0

---

## User Story 2: Disk Space Information in Header

**Goal**: Replace redundant path display in header with actionable disk space information

### Phase 6: Disk Space Module (Foundation)

- [x] T061 [P] Create `src/fs/disk_info.rs` module file ✅
- [x] T062 [P] Define `DiskSpaceInfo` struct with fields: used_bytes, total_bytes, free_bytes, drive_label ✅
- [x] T063 Implement `get_disk_space(path: &Path) -> Result<DiskSpaceInfo>` using fs2 crate ✅
- [x] T064 Add Windows-specific drive letter extraction logic (C:, D:, etc.) ✅
- [x] T065 Add Linux/macOS mount point detection (/, /home, /mnt, /Volumes) ✅
- [x] T066 Implement `format_size(bytes: u64) -> String` to convert to KB/MB/GB/TB ✅
- [x] T067 Implement `format_disk_space(info: &DiskSpaceInfo) -> String` as "Drive: XXgb / YYgb (ZZ% free)" ✅
- [x] T068 Add error handling for inaccessible paths, network drives, virtual filesystems ✅
- [x] T069 Export `disk_info` module in `src/fs/mod.rs` ✅

**Checkpoint**: Disk space query utilities ready for integration ✅

### Phase 7: Header Modification (Implementation)

- [x] T070 Modify `src/ui/mod.rs` - Remove old path display logic from `render_header()` (lines 141-142) ✅
- [x] T071 Add disk space query for left panel: `get_disk_space(&app.left_panel.current_path)` ✅
- [x] T072 Add disk space query for right panel: `get_disk_space(&app.right_panel.current_path)` ✅
- [x] T073 Format disk space strings and render in header with cyan color ✅
- [x] T074 Add error handling: show "Space: N/A" if query fails ✅
- [ ] T075 Test header rendering with disk space info in dual panels
- [ ] T076 Verify text truncation for long drive labels (ellipsis if needed)

**Checkpoint**: Header displays disk space information correctly ⏳

### Phase 8: Performance Optimization (Caching)

- [x] T077 Add `pub disk_space_cache: HashMap<PathBuf, (DiskSpaceInfo, Instant)>` to AppState struct ✅
- [x] T078 Implement `get_cached_disk_space(app: &mut AppState, path: &Path) -> DiskSpaceInfo` helper ✅
- [x] T079 Set cache lifetime to 5 seconds (const CACHE_TTL: Duration = Duration::from_secs(5)) ✅
- [x] T080 Update `render_header()` to check cache before querying filesystem ✅
- [ ] T081 Invalidate cache entry when panel navigates to new path
- [ ] T082 Test caching: verify no lag when navigating within same drive
- [ ] T083 Test cache expiration: verify updates after 5 seconds (wait and check)

**Checkpoint**: Disk space queries cached efficiently, no UI lag ⏳

### Phase 9: Testing and Validation

**Manual Testing - Windows**:
- [ ] T084 Test on C: drive - verify correct space display
- [ ] T085 Test on D: drive (if available) - verify different space display
- [ ] T086 Navigate between C: and D: - verify header updates correctly
- [ ] T087 Test with network drive (\\server\share) - verify "N/A" fallback
- [ ] T088 Test very large drive (>1TB) - verify TB unit display

**Manual Testing - Cross-Platform** (if available):
- [ ] T089 Test on Linux root filesystem (/) - verify space display
- [ ] T090 Test on Linux /home mount point - verify separate space display
- [ ] T091 Test on macOS /Volumes - verify external drive detection
- [ ] T092 Test with unmounted/inaccessible path - verify "N/A" fallback

**Edge Cases**:
- [ ] T093 Test terminal resize with disk space displayed - verify layout adjusts
- [ ] T094 Test very long drive labels (>30 chars) - verify truncation
- [ ] T095 Test rapid navigation between drives - verify cache prevents lag
- [ ] T096 Wait 5+ seconds on same path - verify disk space refreshes
- [ ] T097 Copy large file to drive - verify disk space updates

**Unit Tests** (`tests/unit/disk_info_test.rs`):
- [ ] T098 Write test: `test_format_size()` with KB/MB/GB/TB ranges
- [ ] T099 Write test: `test_get_drive_label()` on Windows paths (C:\, D:\Users\)
- [ ] T100 Write test: `test_get_drive_label()` on Linux paths (/, /home, /mnt/data)

**Deliverable**: Disk space information displays accurately in header, replacing redundant paths

---

## Summary

**Total Tasks**: 100 (60 from US1 + 40 from US2)
**Completed**: 38/100 (38%)
**Remaining**: 62 (22 from US1 + 40 from US2)

**User Story 1 (Welcome Screen)**:
- Total: 60 tasks
- Completed: 38 (63.3%)
- Status: ✅ Implementation complete, manual testing done, release prep pending

**User Story 2 (Disk Space Info)**:
- Total: 40 tasks
- Completed: 0 (0%)
- Status: 🆕 Ready to start implementation

**Time Spent**:
- Phase 1-3 (US1 Implementation): ✅ ~7.5 hours (completed)
- Phase 4 (US1 Testing): ⏳ Partial
- Phase 5 (Release): ⏳ ON HOLD
- Phase 6-9 (US2): 📝 Not started

**Critical Path**: 
- US1: ✅ All blocking tasks done
- US2: T061-T069 (disk space module) must complete before T070-T076 (header modification)

**Minimum Viable**: 
- US1: ✅ ACHIEVED
- US2: ❌ Not started

**Current Status**: 
- User Story 1 (Welcome Screen) implementation complete and functional ✅
- User Story 2 (Disk Space Info) spec and plan complete, ready for implementation 📝
- Both stories will be included in v0.3.0 release

**Next Steps**: 
1. Implement User Story 2 (T061-T100)
2. Complete cross-platform testing for both stories
3. Prepare final release (version bump, changelog, build testing)
4. Prepare final release (T056-T060)
