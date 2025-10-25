# Feature Specification: Quality of Life Improvements v0.3.0

**Feature Branch**: `003-quality-of-user`  
**Created**: 2025-10-25  
**Status**: Draft  
**Input**: User description: "Quality of user experience improvements, welcome screen included, bug fixes"

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.
  
  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - Welcome Screen with Branding (Priority: P1)

When users launch Leeky Explorer, they see a welcome screen displaying an ASCII art image and the current application version number. The user presses Enter to proceed to the main application interface.

**Why this priority**: First impression matters - it establishes brand identity and provides version information critical for support and bug reporting. It's a simple, high-impact feature that enhances professionalism.

**Independent Test**: Can be fully tested by launching the application, observing the welcome screen with ASCII art and version, pressing Enter, and verifying transition to main interface. Delivers immediate value through branding and version visibility.

**Acceptance Scenarios**:

1. **Given** application is not running, **When** user launches Leeky Explorer, **Then** welcome screen appears displaying ASCII art image
2. **Given** welcome screen is displayed, **When** screen is shown, **Then** current version number is clearly visible
3. **Given** welcome screen is displayed, **When** user presses Enter key, **Then** application transitions to main dual-panel interface
4. **Given** welcome screen is displayed, **When** user waits without pressing Enter, **Then** screen remains visible until user takes action

---

### User Story 2 - Disk Space Information in Header (Priority: P2)

As a user navigating through directories, I want to see disk space information (used, total, and percentage free) for each panel's current location in the header area, so I can make informed decisions about file operations without manually checking disk space.

**Why this priority**: Essential operational information that prevents failed copy/move operations due to insufficient space. Replaces redundant path display (already shown in panel borders) with actionable data.

**Independent Test**: Can be tested by navigating to different drives/partitions in each panel and verifying that the header shows accurate disk space information for each location. Works independently of other features.

**Acceptance Scenarios**:

1. **Given** left panel is viewing a directory on drive C:, **When** header renders, **Then** left section shows "C: 45.2GB / 120GB (62% free)"
2. **Given** right panel is viewing a directory on drive D:, **When** header renders, **Then** right section shows "D: 210GB / 500GB (58% free)"
3. **Given** both panels are on the same drive, **When** header renders, **Then** both sections show the same disk space information for that drive
4. **Given** panel is on a Linux partition, **When** header renders, **Then** section shows "/dev/sda1: 85GB / 200GB (58% free)"
5. **Given** disk space cannot be determined, **When** header renders, **Then** section shows "Space: N/A" or similar fallback message
6. **Given** user performs copy operation that fills disk, **When** header refreshes, **Then** disk space information updates to reflect new usage

---

### User Story 3 - Detailed Column View for File Listings (Priority: P3)

As a user browsing files, I want to see detailed information in a columnar format for each file and directory, so I can quickly understand file properties without needing additional commands or tools.

**Why this priority**: Essential for power users who need quick access to file metadata. Current simple list view lacks critical information (dates, permissions, extensions) that users frequently need for decision-making.

**Independent Test**: Can be tested by navigating to any directory and verifying that columns display: icon, name, extension, size, modification date, creation date, and permissions. Works independently of other features.

**Acceptance Scenarios**:

1. **Given** panel displays files, **When** viewing list, **Then** each entry shows: icon, name, extension (if file), size, modification date, creation date, and permissions in aligned columns
2. **Given** file has extension, **When** displayed in panel, **Then** name and extension are shown in separate columns (e.g., "document" | ".txt")
3. **Given** entry is a directory, **When** displayed, **Then** extension column is empty and size shows "<DIR>" or folder count
4. **Given** file size is displayed, **When** rendering, **Then** size uses appropriate units (B, KB, MB, GB) with right-alignment
5. **Given** dates are displayed, **When** rendering, **Then** format is compact and locale-aware (e.g., "2025-01-15 14:30" or "Jan 15 14:30")
6. **Given** system is Windows, **When** showing permissions, **Then** display as "R" (readonly), "H" (hidden), "S" (system), "A" (archive)
7. **Given** system is Unix-like, **When** showing permissions, **Then** display as rwxr-xr-x format (user-group-other)
8. **Given** terminal width is limited, **When** rendering, **Then** columns adapt or truncate gracefully to fit available space
9. **Given** entry is marked for operation, **When** displayed, **Then** "*" prefix is visible before icon
10. **Given** user resizes terminal, **When** panels redraw, **Then** column widths recalculate to maintain readability

---

### User Story 4 - Drive Selector for Cross-Platform Navigation (Priority: P4)

As a Windows user navigating between different drives (C:, D:, E:), or as a Unix user switching between mount points, I want a quick way to select and navigate to available drives/volumes without manually typing paths, so I can efficiently work across different storage locations.

**Why this priority**: Addresses spontaneous user request for drive switching capability. Windows users especially need this for navigating between C:, D:, and other drives. While not critical for MVP, it significantly improves navigation efficiency for multi-drive systems.

**Independent Test**: Can be tested by pressing F10 in either panel, selecting a different drive from the dialog using arrow keys, pressing Enter, and verifying the panel navigates to the selected drive. Works independently of other features.

**Acceptance Scenarios**:

1. **Given** application is running, **When** user presses F10 key, **Then** drive selector dialog appears centered on screen
2. **Given** drive selector is open on Windows, **When** dialog renders, **Then** shows list of available drives (A: through Z:) with free space information
3. **Given** drive selector is open on Unix, **When** dialog renders, **Then** shows list of common mount points (/, /home, /media, /mnt, /Volumes)
4. **Given** drive selector displays drives, **When** shown, **Then** each drive shows format "C: (123.4 GB free)" or similar
5. **Given** drive selector is open, **When** user presses Up/Down or j/k keys, **Then** selection indicator (►) moves between available drives
6. **Given** drive selector has a drive selected, **When** user presses Enter, **Then** active panel navigates to selected drive root
7. **Given** drive selector has a drive selected, **When** panel changes drive, **Then** cursor resets to first entry and entries refresh
8. **Given** drive selector is open, **When** user presses Escape key, **Then** dialog closes without changing current location
9. **Given** drive selector is open, **When** no drives available (edge case), **Then** shows message "No drives detected"
10. **Given** application footer, **When** rendered, **Then** shows "F10 :Drive" keybinding hint in blue color

---

### User Story 5 - Customizable Color Themes (Priority: P5)

As a user working in different terminal emulators with varying background colors, I want to customize the application's color scheme (panel backgrounds, text colors, borders, highlights), so that I can ensure optimal readability and visual comfort regardless of my terminal's appearance.

**Why this priority**: Visual accessibility and user preference. Current hardcoded colors (especially black backgrounds) blend with some terminal themes, reducing readability. Allowing theme customization improves user experience across diverse environments and personal preferences.

**Independent Test**: Can be tested by opening theme selector (F11), choosing different themes, and verifying all UI elements update colors appropriately. Works independently of other features.

**Acceptance Scenarios**:

1. **Given** application is running, **When** user presses F11 key, **Then** theme selector dialog appears with list of available themes
2. **Given** theme selector is open, **When** dialog renders, **Then** shows preview of each theme with sample colors
3. **Given** theme selector displays themes, **When** user navigates with Up/Down keys, **Then** preview updates to show selected theme
4. **Given** theme selector has a theme selected, **When** user presses Enter, **Then** application applies theme immediately to all UI elements
5. **Given** custom theme is applied, **When** viewing panels, **Then** panel background color uses theme's panel_bg color
6. **Given** custom theme is applied, **When** viewing file list, **Then** directories use theme's dir_color, files use theme's file_color
7. **Given** custom theme is applied, **When** active panel shown, **Then** border uses theme's active_border color
8. **Given** custom theme is applied, **When** inactive panel shown, **Then** border uses theme's inactive_border color
9. **Given** custom theme is applied, **When** item is selected, **Then** highlight uses theme's highlight_bg and highlight_fg colors
10. **Given** custom theme is applied, **When** marquee text scrolls, **Then** marquee uses theme's marquee_color (if different from normal text)
11. **Given** custom theme is applied, **When** footer renders, **Then** footer uses theme's footer_bg and footer_fg colors
12. **Given** custom theme is applied, **When** user exits and relaunches, **Then** previously selected theme persists from config file
13. **Given** theme selector is open, **When** user presses Escape, **Then** dialog closes without changing theme
14. **Given** application footer, **When** rendered, **Then** shows "F11 :Theme" keybinding hint

**Built-in Themes**:
- **Classic**: Current color scheme (black background, cyan borders, blue highlight)
- **Light**: Light backgrounds for light terminals (white/light gray panels, dark text)
- **Dark**: Enhanced dark theme (dark gray panels, bright accents)
- **High Contrast**: Maximum contrast (black/white only, bold borders)
- **Nord**: Popular Nord color palette (blue-gray aesthetic)
- **Dracula**: Popular Dracula theme (purple/pink accents)
- **Solarized Dark**: Solarized dark palette
- **Solarized Light**: Solarized light palette

---

### Edge Cases

**User Story 1 (Welcome Screen)**:
- What happens when the ASCII image file is missing or corrupted? (Show fallback text banner)
- How does the welcome screen behave on very small terminal windows? (Show simplified version or text-only)
- What if user presses keys other than Enter? (Ignore or also transition to main interface)
- How does the system handle terminal resize during welcome screen? (Redraw or proceed to main interface)

**User Story 2 (Disk Space)**:
- What happens when disk space cannot be determined (network drives, special filesystems)? (Show "N/A" or "Unknown")
- How frequently should disk space information update? (On panel navigation, not every frame - performance consideration)
- What if the path is invalid or inaccessible? (Show fallback message)
- How to display very large disk sizes (TB/PB)? (Use appropriate units: KB, MB, GB, TB)
- What about drives with multiple mount points? (Show space for the filesystem containing the current path)
- How to handle extremely long drive names/labels? (Truncate with ellipsis)

**User Story 3 (Detailed Column View)**:
- How to handle very long filenames? (Truncate with ellipsis in name column)
- What if extension is very long (.tar.gz.bak)? (Show full extension or truncate)
- How to handle files with no extension? (Leave extension column empty)
- What about hidden files on Windows vs Unix? (Windows: check Hidden attribute, Unix: starts with '.')
- How to display symlink permissions? (Show target permissions or link permissions?)
- What if creation date is unavailable (Linux ext4)? (Show "N/A" or fallback to modification date)
- How to handle very large file sizes (>1TB)? (Use TB unit with decimal places)
- What about special files (devices, sockets, pipes) on Unix? (Show type indicator in permissions: c, b, s, p)
- How wide should each column be on different terminal sizes? (Dynamic calculation based on available width)

**User Story 4 (Drive Selector)**:
- What if no drives are available (system limitation)? (Show "No drives detected" message in dialog)
- How to handle drives that become unavailable after dialog opens? (Show error message and revert to previous location)
- What about network drives on Windows (\\server\share)? (Include if they have drive letter mapping, exclude UNC paths)
- How to handle very long volume labels? (Truncate label with ellipsis, keep drive letter visible)
- What if drive space calculation fails for a drive? (Show drive letter without space info: "D: (space unavailable)")
- How to handle CD/DVD drives with no media? (Include in list but show "E: (no media)")
- What about Unix symbolic links in /media or /mnt? (Follow links to actual mount points)
- How to handle permission errors when reading drive info? (Show drive but indicate access denied)
- What if terminal is too small for dialog? (Show scrollable list or minimum viable dialog)

**User Story 5 (Color Themes)**:
- What if config file contains invalid color values? (Fall back to default Classic theme)
- How to handle custom RGB colors not supported by terminal? (Map to closest available color)
- What about terminal emulators with limited color support (8 colors vs 256)? (Gracefully degrade to basic colors)
- How to preview theme without fully applying it? (Show mini preview in theme selector dialog)
- What if user creates theme with very similar foreground/background? (Warn about low contrast or prevent)
- How to handle theme switching while dialogs are open? (Close dialogs and reopen with new theme, or update in place)
- What about text that becomes invisible with certain theme combinations? (Validate minimum contrast ratios)
- How to export/import custom themes? (Support JSON theme files in config directory)

## Requirements *(mandatory)*

### Functional Requirements

**User Story 1 (Welcome Screen)**:
- **FR-001**: System MUST display welcome screen immediately upon application launch
- **FR-002**: Welcome screen MUST show an ASCII art image
- **FR-003**: Welcome screen MUST display the current application version number in readable format
- **FR-004**: System MUST transition to main interface when user presses Enter key
- **FR-005**: System MUST handle missing or corrupted ASCII image file by showing fallback text banner
- **FR-006**: Welcome screen MUST adapt to small terminal sizes by showing simplified version
- **FR-007**: Welcome screen MUST remain visible until user presses Enter (no auto-timeout)

**User Story 2 (Disk Space)**:
- **FR-008**: Header MUST display disk space information for each panel's current location
- **FR-009**: Disk space MUST show: used space, total space, and percentage free
- **FR-010**: System MUST use appropriate units (KB, MB, GB, TB) based on size magnitude
- **FR-011**: Disk space information MUST update when panel navigates to different drive/partition
- **FR-012**: System MUST handle inaccessible or unmeasurable disk space gracefully (show "N/A")
- **FR-013**: Disk space display MUST replace current redundant path information in header
- **FR-014**: Format MUST be compact: "Drive: UsedGB / TotalGB (XX% free)"
- **FR-015**: System MUST detect correct filesystem/partition for current panel path (Windows drives, Linux partitions, macOS volumes)

**User Story 3 (Detailed Column View)**:
- **FR-016**: Panel view MUST display files in columnar format with aligned columns
- **FR-017**: Columns MUST include: icon, name, extension (files only), size, modification date, creation date, permissions
- **FR-018**: File extensions MUST be separated from name and shown in dedicated column
- **FR-019**: Directories MUST show "<DIR>" or folder count in size column, empty extension column
- **FR-020**: File sizes MUST use appropriate units (B, KB, MB, GB, TB) with right-alignment
- **FR-021**: Dates MUST be formatted compactly and consistently (YYYY-MM-DD HH:MM or locale-aware)
- **FR-022**: Windows permissions MUST display as combination of: R (readonly), H (hidden), S (system), A (archive)
- **FR-023**: Unix permissions MUST display as rwxr-xr-x format (user-group-other octal representation)
- **FR-024**: System MUST handle terminal width constraints by adapting or truncating columns gracefully
- **FR-025**: Marked entries MUST show "*" prefix before icon
- **FR-026**: Column widths MUST recalculate on terminal resize to maintain readability

**User Story 4 (Drive Selector)**:
- **FR-027**: System MUST provide F10 hotkey to open drive selector dialog
- **FR-028**: Drive selector MUST display as centered modal dialog (60% width, 70% height)
- **FR-029**: On Windows, system MUST scan drive letters A-Z and detect available drives
- **FR-030**: On Unix/Linux/macOS, system MUST list common mount points (/, /home, /media, /mnt, /Volumes)
- **FR-031**: Each drive MUST display with format "DriveLetter: (FreeSpace GB free)" or "MountPoint (FreeSpace GB free)"
- **FR-032**: Dialog MUST support Up/Down and j/k keys for navigation between drives
- **FR-033**: Dialog MUST show selection indicator (►) next to currently selected drive
- **FR-034**: Pressing Enter MUST navigate active panel to root of selected drive
- **FR-035**: System MUST reset panel cursor to 0 and refresh entries after drive change
- **FR-036**: Pressing Escape MUST close dialog without changing current location
- **FR-037**: Dialog MUST show header with instructions "Use ↑↓ to navigate, Enter to select, Esc to cancel"
- **FR-038**: Dialog MUST show footer with count of available drives
- **FR-039**: Footer MUST display "F10 :Drive" keybinding hint in blue color
- **FR-040**: System MUST handle drives with unavailable space info by showing drive without space details

**User Story 5 (Color Themes)**:
- **FR-041**: System MUST provide F11 hotkey to open theme selector dialog
- **FR-042**: Theme selector MUST display as centered modal dialog showing available themes
- **FR-043**: System MUST include at least 8 built-in themes: Classic, Light, Dark, High Contrast, Nord, Dracula, Solarized Dark, Solarized Light
- **FR-044**: Each theme MUST define colors for: panel_bg, panel_fg, active_border, inactive_border, highlight_bg, highlight_fg, dir_color, file_color, symlink_color, executable_color, marked_bg, footer_bg, footer_fg, dialog_bg, dialog_fg, error_color, warning_color
- **FR-045**: Theme selector MUST show preview of each theme with sample UI elements
- **FR-046**: Dialog MUST support Up/Down and j/k keys for navigation between themes
- **FR-047**: Pressing Enter MUST apply selected theme immediately to all UI elements
- **FR-048**: System MUST update all panels, borders, dialogs, and footer when theme changes
- **FR-049**: System MUST save selected theme to config file (~/.config/leeky/config.json or Windows equivalent)
- **FR-050**: System MUST load and apply saved theme on application startup
- **FR-051**: Pressing Escape MUST close theme selector without changing theme
- **FR-052**: Footer MUST display "F11 :Theme" keybinding hint
- **FR-053**: Theme data structure MUST support RGB colors (r, g, b values 0-255) for 256-color terminals
- **FR-054**: System MUST gracefully degrade theme colors for terminals with limited color support
- **FR-055**: Theme MUST apply to marquee scrolling text without color conflicts
- **FR-056**: System MUST validate theme definitions and fallback to Classic theme on invalid data

### Key Entities

**User Story 1**:
- **Welcome Screen**: Initial view shown at application startup
  - Contains: ASCII art image, version number
  - Dismissal: User presses Enter key
  - Transition: Moves to main dual-panel interface

- **Version Information**: Current release version of the application
  - Format: Semantic versioning (e.g., v0.3.0)
  - Display: Visible on welcome screen

**User Story 2**:
- **Disk Space Info**: Real-time filesystem statistics per panel
  - Components: Drive identifier, used space, total space, free percentage
  - Location: Header area (replaces redundant path display)
  - Update trigger: Panel navigation to different drive/partition
  - Format: Compact display suitable for terminal width constraints

- **Filesystem Detection**: Logic to identify correct drive/partition
  - Windows: Drive letters (C:, D:, etc.)
  - Linux: Mount points (/dev/sda1, /dev/nvme0n1p2, etc.)
  - macOS: Volumes (/Volumes/Macintosh HD, etc.)

**User Story 3**:
- **Column Layout**: Structured display of file metadata
  - Columns: Icon (emoji), Mark (*), Name, Extension, Size, Modified Date, Created Date, Permissions
  - Alignment: Left for text, right for numbers
  - Width: Dynamic based on terminal size and content
  
- **File Extension**: Separated component of filename
  - Extraction: Everything after last '.' in filename
  - Display: Dedicated column after name
  - Special cases: No extension (empty), multiple dots (.tar.gz)
  
- **Date Formatting**: Consistent timestamp display
  - Format options: ISO (YYYY-MM-DD HH:MM) or locale-aware
  - Modification date: Last write time (always available)
  - Creation date: Birth time (Windows/macOS) or fallback to modified (Linux ext4)
  
- **Permissions Display**: Platform-specific access control info
  - Windows: RHSA flags (Readonly, Hidden, System, Archive)
  - Unix: rwxr-xr-x format (user/group/other with read/write/execute bits)
  - Special files: Type indicators (d=dir, l=link, c=char device, b=block device, s=socket, p=pipe)

**User Story 4**:
- **Drive Selector Dialog**: Modal interface for drive/volume selection
  - Display: Centered dialog (60% width x 70% height)
  - Components: Header (instructions), Drive list, Footer (count)
  - Trigger: F10 key press
  - Dismissal: Enter (select) or Escape (cancel)

- **Available Drives**: List of accessible storage locations
  - Windows: Drive letters A-Z that exist and are accessible
  - Unix: Common mount points (/, /home, /media/*, /mnt/*, /Volumes/*)
  - Information: Drive path + free space (e.g., "C: (573.8 GB free)")
  - Detection: Filesystem metadata check for existence and space

- **Drive Selection State**: Current user selection within dialog
  - Selection indicator: "►" character
  - Navigation: Up/Down or j/k keys
  - Bounds checking: Wrap or stop at list edges
  - Action: Enter to apply, Escape to cancel

**User Story 5**:
- **Color Theme**: Complete color scheme definition for UI
  - Components: 17+ color properties covering all UI elements
  - Format: JSON structure with RGB values (0-255) or named colors
  - Storage: Saved in config file (~/.config/leeky/config.json)
  - Built-in themes: 8 predefined themes (Classic, Light, Dark, etc.)

- **Theme Selector Dialog**: Modal interface for theme selection and preview
  - Display: Centered dialog showing theme list
  - Components: Theme name, color preview squares, instructions
  - Preview: Live sample showing panel bg, borders, highlight, dir/file colors
  - Trigger: F11 key press
  - Dismissal: Enter (apply) or Escape (cancel)

- **Theme Definition**: Structure containing all color properties
  - Core colors: panel_bg, panel_fg, active_border, inactive_border
  - Highlight: highlight_bg, highlight_fg
  - Entry types: dir_color, file_color, symlink_color, executable_color
  - UI elements: marked_bg, footer_bg, footer_fg, dialog_bg, dialog_fg
  - Status: error_color, warning_color, info_color
  - Optional: marquee_color (defaults to panel_fg if not specified)

- **Theme Manager**: System component managing theme loading and application
  - Responsibilities: Load themes, validate definitions, apply to UI, persist selection
  - Validation: Check all required color properties exist, ensure valid RGB values
  - Fallback: Default to Classic theme on invalid/missing theme data
  - Hot-swap: Apply theme changes immediately without restart

## Success Criteria *(mandatory)*

### Measurable Outcomes

**User Story 1 (Welcome Screen)**:
- **SC-001**: 100% of application launches display the welcome screen with ASCII art and version
- **SC-002**: Users can proceed to main interface in under 1 second after pressing Enter
- **SC-003**: Welcome screen displays correctly on 95% of terminal emulators and sizes
- **SC-004**: Application handles missing ASCII art file gracefully in 100% of cases
- **SC-005**: Version number displayed matches actual application version in 100% of launches

**User Story 2 (Disk Space)**:
- **SC-006**: Disk space information displays accurately (within 1% margin) for 95% of filesystem types
- **SC-007**: Header renders disk space in under 50ms to avoid UI lag
- **SC-008**: System correctly identifies drive/partition for 100% of Windows drives (C:, D:, etc.)
- **SC-009**: System correctly identifies mount points for 95% of Linux/macOS filesystems
- **SC-010**: Disk space updates within 100ms when user navigates to different drive/partition
- **SC-011**: Graceful fallback ("N/A") shown for 100% of inaccessible or special filesystems (network, virtual, etc.)

**User Story 3 (Detailed Column View)**:
- **SC-012**: All 7 columns (icon, name, ext, size, mod date, create date, perms) display correctly in 95% of terminal sizes (>80 cols)
- **SC-013**: File extensions extracted correctly for 100% of common file types (.txt, .rs, .tar.gz, etc.)
- **SC-014**: Dates format consistently across all entries (within 2ms rendering time per entry)
- **SC-015**: Windows permissions (RHSA) display correctly for 100% of Windows files
- **SC-016**: Unix permissions (rwx) display correctly for 100% of Unix files and special files
- **SC-017**: Column alignment (left/right) correct for 100% of entries
- **SC-018**: Column widths recalculate within 50ms on terminal resize
- **SC-019**: Truncation with ellipsis works correctly for long names (>50 chars) in 100% of cases
- **SC-020**: View remains readable in minimum terminal width (80 columns)

**User Story 4 (Drive Selector)**:
- **SC-021**: F10 key opens drive selector dialog in 100% of cases when not in other modal state
- **SC-022**: Windows: System detects 100% of available drive letters (C:, D:, etc.) correctly
- **SC-023**: Unix: System lists common mount points with 95% coverage of typical user setups
- **SC-024**: Dialog displays free space information accurately (within 1% margin) for 95% of drives
- **SC-025**: Drive selection navigation responds to key presses within 50ms
- **SC-026**: Panel navigates to selected drive within 200ms after pressing Enter
- **SC-027**: Dialog centers correctly in 95% of terminal sizes (>80x24)
- **SC-028**: System handles 100% of unavailable drives gracefully (shows without crashing)
- **SC-029**: Footer displays "F10 :Drive" hint in 100% of application states
- **SC-030**: Drive enumeration completes within 500ms even with 10+ drives

**User Story 5 (Color Themes)**:
- **SC-031**: F11 key opens theme selector dialog in 100% of cases when not in other modal state
- **SC-032**: All 8 built-in themes load successfully in 100% of application launches
- **SC-033**: Theme applies to all UI elements (panels, borders, dialogs, footer) within 100ms
- **SC-034**: Theme preview in selector accurately represents actual theme appearance for 100% of themes
- **SC-035**: Selected theme persists across application restarts in 100% of cases
- **SC-036**: Theme selector displays correctly in 95% of terminal sizes (>80x24)
- **SC-037**: Theme validation detects 100% of invalid color definitions
- **SC-038**: System falls back to Classic theme in 100% of invalid theme configurations
- **SC-039**: Color contrast between text and background maintains minimum 3:1 ratio for 100% of built-in themes
- **SC-040**: Theme switching completes without visual artifacts in 95% of cases
- **SC-041**: Marquee text remains visible with 100% of built-in themes
- **SC-042**: Footer displays "F11 :Theme" hint in 100% of application states
- **SC-043**: Theme changes reflect in all open dialogs immediately in 100% of cases
- **SC-044**: Terminal color degradation (256→16→8 colors) maintains readability for 90% of themes
- **SC-045**: Theme loading from config file completes within 50ms

