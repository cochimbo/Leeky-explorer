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
