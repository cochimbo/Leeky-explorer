# Feature Specification: Quick Wins - Enhanced Navigation & Productivity

**Feature Branch**: `004-quick-wins-bookmarks`  
**Created**: 2025-10-26  
**Updated**: 2025-10-27  
**Status**: Draft  
**Input**: User description: "Quick Wins - Bookmarks, Disk Usage, History Navigation, Text Editor, SFTP and SMB/Samba support"

## Overview

This feature specification covers 8 user stories focused on productivity improvements and remote file access:
1. **Bookmarks** (P1) - Quick access to favorite directories
2. **Disk Usage** (P2) - Visual disk space indicators
3. **Navigation History** (P3) - Back/forward navigation
4. **Go To Path** (P3) - Direct path navigation with Ctrl+G
5. **Text Editor** (P4) - Simple in-app text file editing
6. **Recursive Search** (P2) - Deep search across directory trees
7. **SFTP Remote Access** (P2) - Browse and manage files over SSH
8. **SMB/Samba Network Shares** (P2) - Access Windows/Samba file shares

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Bookmark Favorite Directories (Priority: P1)

Users frequently work with the same directories (projects, downloads, documents) and need quick access to them without repetitive navigation.

**Why this priority**: Bookmarks are the most universally useful feature - every user will benefit immediately. High impact, moderate complexity.

**Independent Test**: Can be fully tested by creating bookmarks, reopening the app, and verifying persistence. Delivers instant value by reducing navigation time.

**Note**: Uses Ctrl+B keybinding (F5 is already assigned to Copy operation).

**Acceptance Scenarios**:

1. **Given** I'm in any directory, **When** I press Ctrl+B, **Then** a bookmark menu opens showing saved bookmarks and option to add current directory
2. **Given** the bookmark menu is open, **When** I select "Add Bookmark", **Then** the current directory is added to bookmarks with a custom name prompt
3. **Given** I have saved bookmarks, **When** I select a bookmark from the menu, **Then** the active panel navigates to that directory
4. **Given** I have bookmarks saved, **When** I restart leeky-explorer, **Then** all bookmarks persist and are available
5. **Given** I'm viewing the bookmark menu, **When** I press 'd' on a bookmark, **Then** that bookmark is deleted
6. **Given** I'm viewing the bookmark menu, **When** I press 'r' on a bookmark, **Then** I can rename the bookmark

---

### User Story 2 - Visual Disk Usage Indicators (Priority: P2)

Users need to quickly assess disk space availability across all drives without leaving the file explorer.

**Why this priority**: Highly visible quality-of-life improvement. Low complexity, high visual impact. Prevents "disk full" surprises.

**Independent Test**: Can be tested by viewing the drive selector and verifying correct usage percentages and visual bars for all mounted drives.

**Acceptance Scenarios**:

1. **Given** I open the drive selector (F9), **When** viewing the drive list, **Then** each drive shows a percentage bar of space used
2. **Given** a drive is nearly full (>90%), **When** viewing drive selector, **Then** the usage bar displays in warning color (red/yellow)
3. **Given** multiple drives exist, **When** viewing drive selector, **Then** all drives show accurate real-time usage information
4. **Given** I'm in any panel, **When** viewing the status bar, **Then** I see the current drive's available space

---

### User Story 3 - Navigation History (Back/Forward) (Priority: P3)

Users often need to return to previously visited directories or retrace their navigation path, similar to web browser behavior.

**Why this priority**: Nice-to-have feature that improves navigation fluidity. Familiar pattern from web browsers. Medium complexity.

**Independent Test**: Can be tested by navigating through several directories, pressing back/forward keys, and verifying correct path traversal.

**Acceptance Scenarios**:

1. **Given** I've navigated through multiple directories, **When** I press Alt+Left, **Then** I return to the previous directory in history
2. **Given** I've gone back in history, **When** I press Alt+Right, **Then** I move forward in history
3. **Given** I'm at the oldest point in history, **When** I press Alt+Left, **Then** nothing happens (or a subtle indicator shows no more history)
4. **Given** I've gone back and then navigate to a new directory, **When** I press Alt+Right, **Then** the forward history is cleared
5. **Given** I switch between panels, **When** I use history navigation, **Then** each panel maintains its own independent history

---

### User Story 4 - Go To Path (Ctrl+G) (Priority: P3)

Users need to quickly navigate to a known directory path without manually clicking through the directory tree, especially for deeply nested or frequently accessed locations.

**Why this priority**: High productivity boost for power users. Simple implementation with immediate value. Complements bookmarks by handling one-off navigation needs.

**Independent Test**: Can be tested by pressing Ctrl+G, typing a path, and verifying navigation to that directory in the active panel.

**Acceptance Scenarios**:

1. **Given** I'm in any directory, **When** I press Ctrl+G, **Then** a "Go To Path" dialog opens with an input field
2. **Given** the Go To dialog is open, **When** I type a valid absolute path and press Enter, **Then** the active panel navigates to that directory
3. **Given** the Go To dialog is open, **When** I type a valid relative path and press Enter, **Then** the active panel navigates to that path relative to current directory
4. **Given** the Go To dialog is open, **When** I type an invalid or non-existent path, **Then** an error message is shown and the dialog remains open
5. **Given** the Go To dialog is open, **When** I press Esc, **Then** the dialog closes without navigation
6. **Given** the Go To dialog is open, **When** I type a path to a file (not a directory), **Then** an error indicates only directories are valid
7. **Given** I have a path in my clipboard, **When** I open Go To dialog, **Then** I can paste the path (Ctrl+V)
8. **Given** I'm typing a path, **When** the path contains environment variables like %USERPROFILE% or ~, **Then** the system expands them correctly

---

### User Story 5 - Simple Text File Editor (Priority: P4)

Users want to quickly edit configuration files, notes, or small text files without leaving the file explorer and opening external editors.

**Why this priority**: Lower priority due to complexity and potential scope creep. Most users have preferred external editors. Can be deferred to later iteration.

**Independent Test**: Can be tested by selecting a text file, pressing F4, making edits, saving, and verifying changes persist.

**Note**: Uses F4 keybinding (already assigned to preview, will be enhanced to support editing mode).

**Acceptance Scenarios**:

1. **Given** I have a text file selected, **When** I press F4 and then 'e' for edit mode, **Then** a modal editor opens with the file content
2. **Given** the editor is open, **When** I make changes, **Then** the changes are reflected in the editor buffer
3. **Given** I've made changes, **When** I press Ctrl+S, **Then** the file is saved to disk
4. **Given** the editor is open, **When** I press Esc, **Then** the editor closes (with unsaved changes warning if applicable)
5. **Given** I try to edit a binary file, **When** I press F4 and 'e', **Then** an error message indicates the file is not editable
6. **Given** I try to edit a large file (>1MB), **When** I press F4 and 'e', **Then** a warning prompts to use external editor instead

---

### User Story 6 - Recursive Deep Search (Ctrl+F) (Priority: P2)

Users need to find files across entire directory trees, not just in the current folder, when they know part of a filename but not its exact location.

**Why this priority**: High-value feature for large codebases and nested directory structures. Distinguishes from F3 filter which only searches current directory. Similar to `fd` or `fzf` tools.

**Independent Test**: Can be tested by pressing Ctrl+F, typing a search term, and verifying results from all subdirectories appear with full paths.

**Key Difference from F3 Filter**:
- **F3 (Filter)**: Filters files in CURRENT directory only. Fast, instant results, no subdirectories.
- **Ctrl+F (Search)**: Searches RECURSIVELY through all subdirectories. Shows results with full paths. Can take time on large trees.

**Acceptance Scenarios**:

1. **Given** I'm in any directory, **When** I press Ctrl+F, **Then** a search dialog opens with an input field for search term
2. **Given** the search dialog is open, **When** I type a search term, **Then** results appear in real-time showing files matching the term from current directory and all subdirectories
3. **Given** search results are displayed, **When** I select a result and press Enter, **Then** the active panel navigates to the parent directory of that file and selects it
4. **Given** the search is running, **When** I press Esc, **Then** the search is cancelled and dialog closes
5. **Given** I have search results, **When** I press Up/Down arrows, **Then** I can navigate through the result list
6. **Given** search results are displayed, **When** there are many results, **Then** results are paginated or scrollable
7. **Given** I'm searching, **When** I type `*.rs` or `*.txt`, **Then** the search supports glob patterns
8. **Given** the search finds no matches, **When** the search completes, **Then** a "No results found" message is displayed
9. **Given** I start a search in a large directory tree, **When** the search takes time, **Then** a progress indicator shows search is ongoing
10. **Given** I'm viewing search results, **When** results show full paths, **Then** paths are displayed relative to the search root for readability

**Performance Considerations**:
- Search should be interruptible (cancel with Esc)
- Results should stream in as they're found (don't wait for full tree scan)
- Large directories (>10,000 files) should show progress indicator
- Option to limit search depth or file count

---

### User Story 7 - Remote File Access via SFTP (Priority: P2)

Users working with remote servers need to browse and manage files over SSH/SFTP connections without leaving the file explorer or using separate FTP clients.

**Why this priority**: Essential for DevOps, sysadmins, and developers managing remote servers. Enables seamless workflow between local and remote files. Medium-high complexity but high value for target users.

**Independent Test**: Can be tested by connecting to a remote SFTP server, browsing directories, and performing basic file operations (copy, move, delete).

**Acceptance Scenarios**:

1. **Given** I press Ctrl+Shift+S, **When** the SFTP connection dialog opens, **Then** I can enter hostname, port, username, and authentication method
2. **Given** the SFTP connection dialog is open, **When** I enter valid credentials and press Connect, **Then** the active panel navigates to the remote server's home directory
3. **Given** I'm connected to an SFTP server, **When** I browse directories, **Then** remote files and folders display with appropriate icons and metadata
4. **Given** I'm viewing remote files, **When** I select a file and press F3, **Then** the preview works for text files
5. **Given** I have a remote file selected, **When** I press F5 (Copy), **Then** I can copy it to the local panel (download)
6. **Given** I have a local file selected, **When** the other panel shows remote directory and I press F5, **Then** the file uploads to remote server
7. **Given** I'm connected to SFTP, **When** I press F6 (Move), **Then** files can be moved/renamed on remote server
8. **Given** I'm connected to SFTP, **When** I press F8 (Delete), **Then** remote files can be deleted with confirmation
9. **Given** I'm connected to SFTP, **When** I create a folder (F7), **Then** the folder is created on remote server
10. **Given** I'm connected to SFTP, **When** connection is lost, **Then** an error message displays and I can reconnect
11. **Given** I have multiple SFTP connections, **When** I save them, **Then** they appear in bookmarks for quick reconnection
12. **Given** I'm connected to SFTP, **When** I press Ctrl+D, **Then** the connection is closed and panel returns to local filesystem

**Authentication Methods**:
- Password authentication
- SSH key authentication (RSA, Ed25519)
- SSH agent support (pageant on Windows, ssh-agent on Linux)
- Option to save credentials securely (OS keyring)

**Performance Considerations**:
- Directory listings should cache for performance
- Large file transfers should show progress
- Support for connection pooling/keep-alive
- Timeout handling for slow connections

---

### User Story 8 - Network Share Access via SMB/CIFS (Priority: P2)

Users working in Windows/mixed environments need to access network shares (Samba/SMB) for file sharing and collaboration without leaving the file explorer.

**Why this priority**: Critical for enterprise/corporate environments where file shares are standard. Complements SFTP for different network protocols. Essential for Windows network integration.

**Independent Test**: Can be tested by connecting to a Windows network share or Samba server, browsing shared folders, and performing file operations.

**Acceptance Scenarios**:

1. **Given** I press Ctrl+Shift+N, **When** the network share dialog opens, **Then** I can enter UNC path (\\server\share) or smb://server/share
2. **Given** the share connection dialog is open, **When** I enter valid credentials (if required) and press Connect, **Then** the active panel shows the network share contents
3. **Given** I'm viewing network share, **When** authentication is required, **Then** a credential dialog prompts for domain/username/password
4. **Given** I'm connected to a network share, **When** I browse directories, **Then** files display with correct permissions indicators (read-only, etc.)
5. **Given** I have a file on network share selected, **When** I press F3, **Then** preview works for supported file types
6. **Given** I'm viewing a network share, **When** I press F5/F6/F8, **Then** file operations (copy/move/delete) work correctly
7. **Given** I'm connected to a share, **When** I create a folder (F7), **Then** the folder is created on the network share
8. **Given** I have a network share open, **When** connection is lost, **Then** an error message displays and I can reconnect
9. **Given** I have multiple network shares, **When** I save them, **Then** they appear in bookmarks for quick access
10. **Given** I'm connected to a share, **When** I press Ctrl+D, **Then** the connection is closed and panel returns to local filesystem
11. **Given** I'm on Windows, **When** I browse "Network" location, **Then** I can discover available shares on the network
12. **Given** a file is locked by another user, **When** I try to delete/move it, **Then** a clear error message indicates the file is in use

**Platform Considerations**:
- Windows: Native SMB support via UNC paths (\\server\share)
- Linux: Mount via smbclient or FUSE (requires samba-client package)
- Cross-platform: Consider using libsmb or similar library

**Authentication**:
- Windows domain authentication
- Guest access (if allowed)
- Option to save credentials per share
- Kerberos support for domain environments

**Performance Considerations**:
- Network share browsing may be slower than local
- Cache directory listings where appropriate
- Show progress for operations on slow networks
- Handle network timeouts gracefully

---

### Edge Cases

- **Bookmarks**: What happens when a bookmarked directory is deleted or moved?
- **Bookmarks**: How to handle duplicate bookmark names?
- **Bookmarks**: Maximum number of bookmarks (performance consideration)?
- **Disk Usage**: What happens when drive is unmounted while app is running?
- **Disk Usage**: How to handle network drives with slow response times?
- **History**: What happens when navigating to a directory that no longer exists?
- **History**: Maximum history size (memory consideration)?
- **Go To Path**: What happens when user enters a path with invalid characters?
- **Go To Path**: How to handle paths with insufficient permissions?
- **Go To Path**: How to handle very long paths (>260 chars on Windows)?
- **Go To Path**: Should the input support path auto-completion?
- **Editor**: What happens when file is modified externally while being edited?
- **Editor**: How to handle files without write permissions?
- **Editor**: Character encoding detection (UTF-8, UTF-16, ASCII)?
- **Recursive Search**: What happens when searching a directory tree with thousands of files?
- **Recursive Search**: How to handle permission denied errors on some subdirectories?
- **Recursive Search**: Should hidden files/folders be included in search results?
- **Recursive Search**: How to handle symbolic links that create circular references?
- **Recursive Search**: What's the maximum search depth to prevent infinite loops?
- **Recursive Search**: How to differentiate from F3 filter in the UI?
- **SFTP**: What happens when SSH host key changes (MITM warning)?
- **SFTP**: How to handle SSH key passphrases?
- **SFTP**: What happens when network connection drops mid-transfer?
- **SFTP**: Should SFTP connections timeout after inactivity?
- **SFTP**: How to handle different SSH server implementations (OpenSSH, Dropbear, etc.)?
- **SFTP**: What happens when remote server runs out of disk space during upload?
- **SFTP**: How to handle symbolic links on remote server?
- **SFTP**: Should we support SFTP protocol v3, v4, v5, or v6?
- **SMB/Samba**: What happens when domain authentication fails?
- **SMB/Samba**: How to handle guest access (anonymous login)?
- **SMB/Samba**: What happens when share is suddenly disconnected?
- **SMB/Samba**: How to handle different SMB protocol versions (SMB1, SMB2, SMB3)?
- **SMB/Samba**: What happens when credentials expire (in domain environments)?
- **SMB/Samba**: Should we show available shares when browsing network?
- **SMB/Samba**: How to handle locked files (opened by other users)?
- **SMB/Samba**: What happens when copying large files over slow network?
- **Remote (Both)**: How to distinguish remote vs local files in UI?
- **Remote (Both)**: Should bookmarks save connection credentials?
- **Remote (Both)**: How to handle timezone differences for file timestamps?
- **Remote (Both)**: What's the behavior when copying between two remote connections?

## Requirements *(mandatory)*

### Functional Requirements

**Bookmarks (P1):**
- **FR-001**: System MUST persist bookmarks to disk in configuration file (~/.config/leeky/bookmarks.json or equivalent)
- **FR-002**: System MUST allow adding current directory as bookmark with custom name
- **FR-003**: System MUST allow removing existing bookmarks
- **FR-004**: System MUST allow renaming existing bookmarks
- **FR-005**: System MUST navigate to bookmarked directory when selected from menu
- **FR-006**: System MUST handle bookmarks to non-existent directories gracefully (show warning, offer to remove)
- **FR-007**: Bookmark menu MUST be accessible via Ctrl+B keybinding
- **FR-008**: System MUST support at least 50 bookmarks without performance degradation

**Disk Usage (P2):**
- **FR-009**: System MUST display disk usage percentage for all available drives
- **FR-010**: System MUST show visual progress bar for disk usage in drive selector
- **FR-011**: System MUST use warning colors (yellow >80%, red >90%) for high disk usage
- **FR-012**: System MUST display current drive's free space in status bar
- **FR-013**: System MUST refresh disk usage information when drive selector is opened
- **FR-014**: System MUST handle unmounted/unavailable drives without crashing

**Navigation History (P3):**
- **FR-015**: System MUST maintain separate navigation history for each panel
- **FR-016**: System MUST support backward navigation via Alt+Left keybinding
- **FR-017**: System MUST support forward navigation via Alt+Right keybinding
- **FR-018**: System MUST clear forward history when navigating to new directory from middle of history
- **FR-019**: System MUST persist history for at least 100 entries per panel
- **FR-020**: System MUST handle navigation to non-existent historical directories gracefully

**Go To Path (P3):**
- **FR-021**: System MUST open Go To Path dialog via Ctrl+G keybinding
- **FR-022**: System MUST accept absolute paths (e.g., C:\Users\John\Documents, /home/user/documents)
- **FR-023**: System MUST accept relative paths from current directory (e.g., ../parent, ./subdir)
- **FR-024**: System MUST expand environment variables (%USERPROFILE%, $HOME, ~)
- **FR-025**: System MUST validate path before navigation and show error for invalid paths
- **FR-026**: System MUST reject paths that point to files (only directories allowed)
- **FR-027**: System MUST handle permission errors gracefully (show clear error message)
- **FR-028**: System MUST support clipboard paste in path input (Ctrl+V)
- **FR-029**: System MUST trim whitespace from input path
- **FR-030**: System MUST add successful navigation to history

**Text Editor (P4):**
- **FR-031**: System MUST open text editor via F4 keybinding for selected file (edit mode in preview)
- **FR-032**: System MUST support basic text editing (insert, delete, navigation)
- **FR-033**: System MUST save file via Ctrl+S keybinding
- **FR-034**: System MUST close editor via Esc keybinding
- **FR-035**: System MUST warn before closing editor with unsaved changes
- **FR-036**: System MUST prevent editing binary files (show error message)
- **FR-037**: System MUST warn when attempting to edit large files (>1MB)
- **FR-038**: System MUST handle UTF-8 encoded files
- **FR-039**: System MUST show line numbers in editor
- **FR-040**: System MUST support syntax highlighting [NICE TO HAVE - can defer]

**Recursive Search (P2):**
- **FR-041**: System MUST open search dialog via Ctrl+F keybinding
- **FR-042**: System MUST search recursively through all subdirectories from current directory
- **FR-043**: System MUST display results with relative paths from search root
- **FR-044**: System MUST support case-insensitive search by default
- **FR-045**: System MUST support glob patterns (*.rs, *.txt, file?.log)
- **FR-046**: System MUST allow cancelling search via Esc keybinding
- **FR-047**: System MUST navigate to selected result's parent directory on Enter
- **FR-048**: System MUST stream results as they are found (incremental display)
- **FR-049**: System MUST show progress indicator for searches taking >500ms
- **FR-050**: System MUST handle permission errors gracefully (skip and continue)
- **FR-051**: System MUST limit search depth to prevent stack overflow (default: 20 levels)
- **FR-052**: System MUST differentiate visually from F3 filter (different dialog title/style)
- **FR-053**: System MUST display "No results found" when search completes with no matches
- **FR-054**: System MUST support up/down navigation through results list
- **FR-055**: System MUST exclude hidden files/folders by default [CONFIGURABLE]

**SFTP Remote Access (P2):**
- **FR-056**: System MUST open SFTP connection dialog via Ctrl+Shift+S keybinding
- **FR-057**: System MUST support password authentication for SFTP
- **FR-058**: System MUST support SSH key authentication (RSA, Ed25519)
- **FR-059**: System MUST support SSH agent (pageant/ssh-agent) integration
- **FR-060**: System MUST verify SSH host keys and warn on changes
- **FR-061**: System MUST allow saving SFTP connections as bookmarks
- **FR-062**: System MUST display remote files with appropriate indicators in UI
- **FR-063**: System MUST support all file operations on remote files (copy/move/delete/rename)
- **FR-064**: System MUST support downloading files from remote to local (F5 from SFTP to local panel)
- **FR-065**: System MUST support uploading files from local to remote (F5 from local to SFTP panel)
- **FR-066**: System MUST show progress indicator for remote file transfers
- **FR-067**: System MUST handle connection timeouts gracefully (show error, allow reconnect)
- **FR-068**: System MUST close SFTP connection via Ctrl+D keybinding
- **FR-069**: System MUST cache remote directory listings for performance (configurable TTL)
- **FR-070**: System MUST support creating folders on remote server (F7)
- **FR-071**: System MUST handle connection drops mid-transfer (show error, allow retry)
- **FR-072**: System MUST support standard SFTP ports (22) and custom ports
- **FR-073**: System MUST store credentials securely using OS keyring [NICE TO HAVE]

**SMB/Samba Network Shares (P2):**
- **FR-074**: System MUST open network share dialog via Ctrl+Shift+N keybinding
- **FR-075**: System MUST support UNC paths on Windows (\\server\share)
- **FR-076**: System MUST support SMB URLs (smb://server/share)
- **FR-077**: System MUST support domain authentication (DOMAIN\username)
- **FR-078**: System MUST support guest access (anonymous login)
- **FR-079**: System MUST allow saving SMB connections as bookmarks
- **FR-080**: System MUST display share files with appropriate permission indicators
- **FR-081**: System MUST support all file operations on network shares
- **FR-082**: System MUST handle connection drops gracefully (show error, allow reconnect)
- **FR-083**: System MUST show progress for operations on slow networks
- **FR-084**: System MUST handle locked files with clear error messages
- **FR-085**: System MUST support SMB2/SMB3 protocols (avoid SMB1 security issues)
- **FR-086**: System MUST close share connection via Ctrl+D keybinding
- **FR-087**: System MUST discover available network shares [WINDOWS ONLY - NICE TO HAVE]
- **FR-088**: System MUST cache share listings for performance
- **FR-089**: System MUST handle credential expiry in domain environments
- **FR-090**: System MUST support Kerberos authentication [NICE TO HAVE]

### Key Entities

- **Bookmark**: Represents a saved directory location
  - `name`: String - User-friendly display name
  - `path`: PathBuf - Absolute path to directory
  - `created_at`: DateTime - When bookmark was created
  - `last_accessed`: DateTime - Last time bookmark was used (for sorting)

- **NavigationHistory**: Represents browsing history for a panel
  - `entries`: Vec<PathBuf> - Stack of visited directories
  - `current_index`: usize - Current position in history
  - `max_size`: usize - Maximum history size (default 100)

- **DiskUsageInfo**: Represents disk space information
  - `total_space`: u64 - Total disk capacity in bytes
  - `used_space`: u64 - Used space in bytes
  - `available_space`: u64 - Available space in bytes
  - `usage_percentage`: f64 - Calculated percentage (0-100)
  - `mount_point`: PathBuf - Drive/volume mount point

- **EditorBuffer**: Represents text file being edited
  - `content`: String - File content
  - `file_path`: PathBuf - Path to file
  - `modified`: bool - Whether buffer has unsaved changes
  - `cursor_position`: (usize, usize) - Line and column
  - `scroll_offset`: usize - Vertical scroll position

- **SearchResult**: Represents a file found by recursive search
  - `file_name`: String - Name of the file
  - `full_path`: PathBuf - Absolute path to file
  - `relative_path`: PathBuf - Path relative to search root
  - `file_size`: u64 - Size in bytes
  - `modified_time`: SystemTime - Last modification time
  - `match_score`: f32 - Relevance score (for fuzzy matching, optional)

- **SearchState**: Represents ongoing search operation
  - `query`: String - Search term or pattern
  - `root_path`: PathBuf - Starting directory for search
  - `results`: Vec<SearchResult> - Found files
  - `is_running`: bool - Whether search is in progress
  - `files_scanned`: usize - Progress counter
  - `use_glob`: bool - Whether query is a glob pattern

- **SftpConnection**: Represents an active SFTP connection
  - `connection_id`: String - Unique identifier for connection
  - `hostname`: String - Remote server hostname/IP
  - `port`: u16 - SSH port (default 22)
  - `username`: String - SSH username
  - `auth_method`: AuthMethod - Password, Key, or Agent
  - `current_path`: PathBuf - Current remote directory
  - `session`: SshSession - Underlying SSH session handle
  - `connected_at`: SystemTime - Connection timestamp
  - `last_activity`: SystemTime - For timeout management

- **AuthMethod**: Represents SSH authentication method
  - `Password(String)` - Password authentication
  - `Key(PathBuf)` - SSH key file path
  - `Agent` - Use SSH agent (pageant/ssh-agent)

- **SmbConnection**: Represents an active SMB/CIFS connection
  - `connection_id`: String - Unique identifier for connection
  - `server`: String - Server hostname/IP
  - `share_name`: String - Share name
  - `unc_path`: String - Full UNC path (\\server\share)
  - `username`: Option<String> - Username (None for guest)
  - `domain`: Option<String> - Domain name (for domain auth)
  - `current_path`: PathBuf - Current path within share
  - `session`: SmbSession - Underlying SMB session handle
  - `connected_at`: SystemTime - Connection timestamp
  - `protocol_version`: String - SMB protocol version (SMB2/SMB3)

- **RemoteFileEntry**: Represents a file/folder on remote connection
  - `name`: String - File/folder name
  - `path`: PathBuf - Full remote path
  - `size`: u64 - File size in bytes
  - `modified_time`: SystemTime - Last modification time
  - `is_directory`: bool - Whether it's a directory
  - `permissions`: String - Unix-style permissions (rwxr-xr-x) or Windows ACL indicator
  - `is_readonly`: bool - Whether file is read-only
  - `connection_type`: ConnectionType - SFTP or SMB

- **ConnectionType**: Enum for remote connection types
  - `Local` - Local filesystem
  - `Sftp(SftpConnection)` - SFTP connection
  - `Smb(SmbConnection)` - SMB/CIFS connection


  - `results`: Vec<SearchResult> - Found files
  - `is_running`: bool - Whether search is in progress
  - `files_scanned`: usize - Progress counter
  - `use_glob`: bool - Whether query is a glob pattern

## Success Criteria *(mandatory)*

### Measurable Outcomes

**Bookmarks:**
- **SC-001**: Users can create and access bookmarks in under 5 seconds
- **SC-002**: Bookmarks persist across application restarts 100% of the time
- **SC-003**: Users report 50% reduction in time spent navigating to frequently used directories
- **SC-004**: Bookmark operations (add, remove, rename, navigate) complete in <100ms

**Disk Usage:**
- **SC-005**: Disk usage information displays within 500ms of opening drive selector
- **SC-006**: Users can identify nearly-full drives at a glance (without reading percentages)
- **SC-007**: Status bar shows current drive space without performance impact
- **SC-008**: 95% accuracy in disk usage calculations across all supported platforms

**Navigation History:**
- **SC-009**: Users can navigate backward/forward through history in <50ms per operation
- **SC-010**: History maintains accurate state for 100 navigation operations without memory issues
- **SC-011**: 90% of users discover and use history navigation within first session (with keybinding hints)
- **SC-012**: Each panel maintains independent history without cross-contamination

**Go To Path:**
- **SC-013**: Dialog opens and accepts input in <50ms
- **SC-014**: Path validation completes in <100ms for local paths
- **SC-015**: Users can navigate to any valid directory with 100% accuracy
- **SC-016**: Environment variable expansion works correctly on all supported platforms
- **SC-017**: Error messages clearly indicate why a path is invalid
- **SC-018**: 80% of power users adopt Ctrl+G as preferred navigation method for known paths

**Text Editor:**
- **SC-019**: Editor opens files <100KB in under 200ms
- **SC-020**: Users can make simple edits (config files, notes) without exiting application
- **SC-021**: 100% of file saves complete successfully or show clear error message
- **SC-022**: Zero data loss incidents due to editor bugs
- **SC-023**: Editor prevents editing binary files 100% of the time

**Recursive Search:**
- **SC-024**: Search dialog opens in <50ms
- **SC-025**: First results appear within 100ms for directories with <1000 files
- **SC-026**: Search can be cancelled instantly with Esc at any point
- **SC-027**: Users can find files 3x faster than manual navigation through subdirectories
- **SC-028**: Search correctly handles permission errors without crashing
- **SC-029**: Glob patterns work accurately for all common cases (*.ext, file?, prefix*)
- **SC-030**: Users clearly understand difference between F3 (filter) and Ctrl+F (search)
- **SC-031**: Search scales to directories with 10,000+ files without freezing UI
- **SC-032**: 90% of users discover and use recursive search within first 3 sessions

**SFTP Remote Access:**
- **SC-033**: SFTP connection establishes within 3 seconds on average network
- **SC-034**: Remote directory listings appear within 500ms after connection
- **SC-035**: File transfers show progress updates at least every 100ms
- **SC-036**: Users can seamlessly copy files between local and remote with F5
- **SC-037**: Connection drops are handled gracefully with clear error message
- **SC-038**: SSH key authentication works with all common key types (RSA, Ed25519)
- **SC-039**: SFTP bookmarks reconnect successfully 95% of the time
- **SC-040**: Zero crashes due to network errors or protocol issues
- **SC-041**: Remote operations feel responsive (<100ms lag for UI updates)
- **SC-042**: Users report 70% time savings vs using separate SFTP clients

**SMB/Samba Network Shares:**
- **SC-043**: SMB connection establishes within 2 seconds on local network
- **SC-044**: Share listings appear within 500ms after connection
- **SC-045**: File operations on network shares complete within 2x local time
- **SC-046**: Locked files show clear error message indicating file is in use
- **SC-047**: Domain authentication works correctly with all common AD setups
- **SC-048**: Share bookmarks reconnect successfully 95% of the time
- **SC-049**: Zero crashes due to network errors or protocol issues
- **SC-050**: Users report seamless experience compared to native file explorers
- **SC-051**: Guest access works correctly when allowed by server
- **SC-052**: UNC path parsing works for 100% of valid Windows paths

**Overall:**
- **SC-053**: All quick wins features combined add <500KB to binary size
- **SC-054**: No measurable performance degradation in existing features
- **SC-055**: Feature discoverability: 70% of users find at least 4 of 8 features in first 10 minutes
- **SC-056**: Remote features (SFTP/SMB) add <1MB to binary size
- **SC-057**: Memory usage stays under 100MB even with multiple remote connections

---

## Technical Stories - Refactoring *(completed)*

### Tech Story 1 - Extract Collision Handlers (Priority: P0 - Technical Debt)

**Completed**: 2025-10-27  
**Commit**: bce0791  
**Impact**: Foundation for modular handler architecture

The collision handler logic (230+ lines) was embedded in the monolithic `handler.rs` file, making the codebase difficult to navigate and maintain. This story extracts collision-specific handling into its own module.

**Acceptance Criteria**:
1. ✅ Create `src/events/handlers/collision.rs` module
2. ✅ Move `handle_collision_dialog` function to collision.rs (~180 lines)
3. ✅ Update `handler.rs` to call through `handlers::collision::` namespace
4. ✅ Remove old function definition from handler.rs
5. ✅ Verify compilation with `cargo check`
6. ✅ Create `handlers/mod.rs` for module exports

**Results**:
- handler.rs reduced from 3,476 → 3,181 lines (295 lines moved)
- Created handlers module structure foundation
- Zero functionality changes, pure refactor

---

### Tech Story 2 - Extract Dialog Handlers (Priority: P0 - Technical Debt)

**Completed**: 2025-10-27  
**Commit**: 7cbd761  
**Impact**: Major reduction in handler.rs complexity

The `handler.rs` file contained 12 different dialog handler functions plus 9 helper functions (~1,579 lines total), making it extremely difficult to locate and modify specific dialog logic. This story extracts all dialog-related handling into a dedicated module.

**Acceptance Criteria**:
1. ✅ Create `src/events/handlers/dialogs.rs` module
2. ✅ Move 12 dialog handler functions:
   - handle_input_dialog (create folder, rename, add bookmark, goto path)
   - handle_drive_selector_action
   - handle_theme_selector_action
   - handle_bookmark_dialog_action
   - handle_help_viewer_action
   - handle_history_viewer_action
   - handle_archive_extract_options
   - And other dialog-specific handlers
3. ✅ Move 9 dialog helper functions:
   - create_folder, apply_rename, apply_bookmark_add
   - handle_goto_path, navigate_to_path
   - validate_path, expand_path, is_directory
   - calculate_relative_path
4. ✅ Update handler.rs to call through `handlers::dialogs::` namespace
5. ✅ Remove old function definitions from handler.rs
6. ✅ Verify compilation and all dialog types work correctly

**Results**:
- handler.rs reduced from 3,181 → 1,501 lines (1,579 lines moved, 50% reduction)
- Created comprehensive dialogs.rs module (1,579 lines)
- All dialog types now in single, focused module
- Easier to add new dialog types

---

### Tech Story 3 - Extract File Operations (Priority: P0 - Technical Debt)

**Completed**: 2025-10-27  
**Commit**: 3af2f11  
**Impact**: Logical grouping of file operation workflows

The file operation functions (copy, move, delete with variants) were scattered in handler.rs, mixing concerns with event handling. This story extracts all file operation logic into a dedicated module.

**Acceptance Criteria**:
1. ✅ Create `src/events/handlers/file_operations.rs` module
2. ✅ Move 7 file operation functions (~812 lines):
   - start_copy_operation
   - start_copy_operation_skip_check
   - start_copy_operation_with_rename
   - start_move_operation
   - start_move_operation_skip_check
   - start_move_operation_with_rename
   - start_delete_operation
3. ✅ Update handler.rs to call through `handlers::file_operations::` namespace
4. ✅ Remove old function definitions from handler.rs
5. ✅ Verify compilation and file operations work correctly

**Results**:
- handler.rs reduced from 1,501 → 764 lines (812 lines moved, 49% reduction from Phase 2)
- Created file_operations.rs module (812 lines)
- All copy/move/delete logic now in dedicated module
- Clearer separation between event handling and operation execution

---

### Tech Story 4 - Extract Special Mode Handlers (Priority: P0 - Technical Debt)

**Completed**: 2025-10-27  
**Commit**: 59bc259  
**Impact**: Separation of special interaction modes

The search, preview, and editor mode handlers were still in handler.rs, adding complexity to the main event dispatcher. This story extracts special interaction modes into their own module.

**Acceptance Criteria**:
1. ✅ Create `src/events/handlers/modes.rs` module
2. ✅ Move 3 mode handler functions (~199 lines):
   - handle_search_mode (T411-T415 - search filtering)
   - handle_preview_mode (T627-T630 - preview navigation)
   - handle_editor_mode (TASK-030 - text editor)
3. ✅ Update handler.rs to call through `handlers::modes::` namespace
4. ✅ Remove old function definitions from handler.rs
5. ✅ Fix import: `crate::events::Action` → `crate::events::keybindings::Action`
6. ✅ Verify compilation and all modes work correctly

**Results**:
- handler.rs reduced from 764 → 565 lines (199 lines moved, 26% reduction from Phase 3)
- Created modes.rs module (213 lines)
- Search, preview, and editor modes now logically grouped
- Easier to add new interaction modes

---

### Tech Story 5 - Extract Request Handlers (Priority: P0 - Technical Debt)

**Completed**: 2025-10-27  
**Commit**: 329fb24  
**Impact**: Final cleanup of helper functions

The remaining helper functions in handler.rs were request handlers that show confirmation dialogs before initiating file operations. This story moves them to the file_operations module where they logically belong.

**Acceptance Criteria**:
1. ✅ Add 5 request handler functions to file_operations.rs (~140 lines):
   - handle_copy_request (T570 - marked items support)
   - handle_move_request (T571)
   - handle_delete_request (T572)
   - handle_create_folder_request
   - handle_rename_request (F2 vs Shift+F2 handling)
2. ✅ Update handlers/mod.rs to export new functions
3. ✅ Update handler.rs to call through `handlers::file_operations::` namespace
4. ✅ Remove old function definitions from handler.rs (~127 lines)
5. ✅ Clean up unused imports in handler.rs
6. ✅ Verify compilation

**Results**:
- handler.rs reduced from 565 → 441 lines (124 lines moved, 87% total reduction from original)
- file_operations.rs expanded to 897 lines (complete file operations module)
- handler.rs now focused on main event dispatch logic
- Clean separation between request handling and operation execution

---

### Refactoring Summary - Overall Impact

**Original State**: `handler.rs` = 3,476 lines (monolithic, difficult to maintain)

**Final State**: Modular handler architecture
- `handler.rs` = 441 lines (main event dispatcher only) - **87% reduction**
- `handlers/collision.rs` = 295 lines (collision dialog handling)
- `handlers/dialogs.rs` = 1,579 lines (all dialog types)
- `handlers/file_operations.rs` = 897 lines (file operations + requests)
- `handlers/modes.rs` = 213 lines (search/preview/editor modes)
- `handlers/navigation.rs` = stub (future navigation logic)

**Benefits**:
- ✅ Dramatically improved code navigation and maintainability
- ✅ Clear separation of concerns by functionality
- ✅ Easier to locate and modify specific features
- ✅ Reduced cognitive load when working on any single feature
- ✅ Better foundation for adding new features
- ✅ Easier code review process
- ✅ Zero functionality changes - pure refactoring
- ✅ All phases compiled and tested successfully

