# Implementation Tasks: Nexus NXM Protocol Integration

## Overview

This document outlines the implementation tasks for adding Nexus Mods NXM protocol support. Tasks are organized into 4 phases: Foundation, Download Manager, Mod Installation, and Polish.

**Estimated Total:** 80-100 tasks across 4 phases

---

## Phase 1: Foundation (NXM Protocol Handler)

### 1.1 Dependencies and Setup

- [x] Add `tauri-plugin-deep-link` to Cargo.toml
- [x] Add `url` crate for URL parsing
- [x] Add `regex` crate for URL validation
- [x] Update Tauri configuration to enable deep link plugin
- [x] Configure permissions in tauri.conf.json for deep linking

### 1.2 NXM Protocol Registration

- [x] Create `src-tauri/src/nxm_protocol.rs` module
- [x] Implement protocol registration in `lib.rs` setup function
- [x] Register "nxm" scheme using deep link plugin
- [x] Add platform-specific registration (Windows registry, macOS plist, Linux desktop file)
- [ ] Test protocol registration on Windows
- [ ] Test protocol registration on macOS
- [ ] Test protocol registration on Linux
- [x] Handle case where another app already owns nxm:// protocol
- [x] Implement graceful error message if registration fails

### 1.3 NXM URL Parsing

- [x] Define `NxmUrl` struct with fields: game, mod_id, file_id, key, expires, user_id
- [x] Implement `NxmUrl::parse()` function
- [x] Add regex pattern for nxm:// URL format validation
- [x] Extract game domain from URL
- [x] Extract mod_id and file_id as integers
- [x] Extract query parameters (key, expires, user_id)
- [x] Validate game domain is "stardewvalley" (reject others)
- [x] Validate mod_id and file_id are positive integers
- [x] Validate key parameter is present and non-empty
- [x] Parse expires as Unix timestamp (if present)
- [x] Handle missing optional parameters gracefully
- [x] Write unit tests for valid URL parsing
- [x] Write unit tests for invalid URL rejection

### 1.4 Expiration Validation

- [x] Implement `NxmUrl::is_expired()` method
- [x] Get current Unix timestamp using `SystemTime`
- [x] Compare current time with expires timestamp
- [x] Return false if expires is None (no expiration)
- [x] Write unit tests for expiration logic

### 1.5 Event Handling

- [x] Create event listener for deep link events in `lib.rs`
- [x] Handle "deep-link://nxm" events
- [x] Parse incoming nxm:// URLs
- [x] Emit "nxm-url-received" event to frontend on success
- [x] Emit "nxm-error" event to frontend on parse failure
- [x] Handle app launching from nxm:// link (app not running)
- [x] Handle app already running when nxm:// link clicked (focus window)
- [x] Queue multiple nxm:// URLs if received rapidly
- [ ] Write integration test for event handling

### 1.6 Basic Download Placeholder

- [x] Create temporary download function that logs nxm:// URL
- [x] Log mod_id, file_id, and key to console
- [x] Return success for now (actual download in Phase 2)
- [ ] Test end-to-end: Click "Mod Manager Download" on Nexus → App receives URL

---

## Phase 2: Download Manager

### 2.1 Download Manager Structure

- [x] Create `src-tauri/src/download_manager.rs` module
- [x] Define `DownloadManager` struct with queue, active, semaphore
- [x] Define `DownloadTask` struct with id, nxm_url, status, metadata
- [x] Define `DownloadStatus` enum (Queued, Downloading, Paused, Completed, Failed)
- [x] Define `DownloadProgress` struct for progress tracking
- [x] Implement `DownloadManager::new()` constructor
- [x] Add `Arc<Mutex<>>` wrappers for thread-safe queue access

### 2.2 Queue Management

- [x] Implement `DownloadManager::add_to_queue()` function
- [x] Generate unique download ID (UUID)
- [x] Create DownloadTask from NxmUrl
- [x] Add task to queue VecDeque
- [x] Emit "download-queued" event to frontend
- [x] Implement `DownloadManager::remove_from_queue()` function
- [x] Implement `DownloadManager::get_queue_state()` for UI
- [x] Return list of all downloads (queued, active, completed, failed)

### 2.3 Concurrency Control

- [x] Create Tokio semaphore with configurable permits (default: 1)
- [x] Implement `DownloadManager::start_next_download()` function
- [x] Acquire semaphore permit before starting download
- [x] Release permit when download completes/fails
- [x] Enforce concurrency limit based on settings
- [x] Implement logic to start queued downloads when active ones complete
- [x] Add setting for concurrent download limit (1-5)
- [ ] Write unit tests for concurrency enforcement

### 2.4 Download Execution

- [x] Implement `DownloadManager::download_file()` async function
- [x] Use `reqwest` to send HTTP GET request with key parameter
- [x] Try direct download: `https://www.nexusmods.com/Core/Libs/Common/Widgets/DownloadPopUp?fid={file_id}&key={key}`
- [x] Handle redirect responses (302, 307)
- [x] Follow redirects to CDN download URL
- [x] Stream response body to file in chunks (8KB buffer)
- [x] Calculate bytes downloaded, total bytes, progress percentage
- [x] Calculate download speed (bytes per second)
- [x] Calculate ETA (remaining bytes / speed)
- [x] Emit progress events (throttled to 10/sec max)
- [x] Handle Content-Length header missing (indeterminate progress)
- [x] Save file to downloads directory with proper filename
- [ ] Handle filename collisions (append "(1)", "(2)", etc.)

### 2.5 Progress Tracking

- [x] Implement progress event emission with Tokio broadcast channel
- [x] Create `DownloadProgress` messages with bytes, speed, ETA
- [x] Send progress to frontend via Tauri event
- [x] Throttle events to max 10 per second
- [x] Update download status in queue when progress changes
- [x] Handle 0-byte downloads gracefully
- [x] Display indeterminate progress when total size unknown

### 2.6 Pause and Resume

- [ ] Implement `DownloadManager::pause_download()` function
- [ ] Cancel ongoing reqwest stream
- [ ] Save current byte position to disk
- [ ] Update status to "Paused"
- [ ] Preserve partial file on disk
- [ ] Implement `DownloadManager::resume_download()` function
- [ ] Use HTTP Range header to resume from byte position
- [ ] Handle servers that don't support Range requests (restart from 0)
- [ ] Test pause/resume with real Nexus downloads

### 2.7 Cancel Downloads

- [x] Implement `DownloadManager::cancel_download()` function
- [x] Stop active download stream
- [ ] Delete partial file from disk
- [x] Update status to "Cancelled"
- [x] Remove from active downloads map
- [x] Keep in queue history as "cancelled" for user reference

### 2.8 Retry Logic

- [ ] Implement `DownloadManager::retry_download()` function
- [ ] Re-queue failed download with same nxm_url
- [ ] Reset progress to 0%
- [ ] Clear previous error message
- [ ] Implement automatic retry for transient failures (5xx errors)
- [ ] Add exponential backoff (5s, 10s, 20s)
- [ ] Max 3 automatic retries, then mark as failed
- [ ] No automatic retry for auth failures (401, 403)
- [ ] Show appropriate error messages for different failure types

### 2.9 Queue Persistence

- [ ] Create `downloads/queue.json` file structure
- [ ] Implement `DownloadManager::save_state()` function
- [ ] Serialize queue to JSON using serde
- [ ] Save atomically (write to temp file, then rename)
- [ ] Implement `DownloadManager::load_state()` function
- [ ] Load queue from JSON on app startup
- [ ] Restore queued and paused downloads
- [ ] Mark previously active downloads as paused (for manual resume)
- [ ] Handle corrupted queue.json (backup and initialize empty)
- [ ] Write integration test for save/load cycle

### 2.10 Download Settings

- [ ] Add download settings to Settings struct
- [ ] Add `download_directory: PathBuf` field
- [ ] Add `concurrent_downloads: u8` field (default: 1)
- [ ] Add `delete_after_install: bool` field (default: false)
- [ ] Load settings in DownloadManager constructor
- [ ] Apply concurrency limit from settings
- [ ] Watch for settings changes and update semaphore permits

### 2.11 Tauri Commands

- [x] Create `get_downloads` command (return queue state)
- [ ] Create `pause_download` command
- [ ] Create `resume_download` command
- [x] Create `cancel_download` command
- [ ] Create `retry_download` command
- [x] Create `clear_completed` command
- [ ] Create `open_download_folder` command
- [x] Register all commands in lib.rs invoke_handler

---

## Phase 3: Mod Installation

### 3.1 Archive Extraction

- [x] Add `zip` crate to Cargo.toml
- [x] Create `src-tauri/src/mod_installer.rs` module
- [x] Implement `extract_archive()` function
- [x] Open ZIP file with zip::ZipArchive
- [x] Extract to temp directory: `{app_data}/temp/{download_id}/`
- [x] Iterate through all files in archive
- [x] Preserve directory structure during extraction
- [x] Handle nested folders (flatten if single root folder)
- [x] Set file permissions correctly (especially on Linux/macOS)
- [x] Handle corrupted ZIP files gracefully
- [x] Return error for unsupported formats (7z, RAR)
- [ ] Write unit tests for extraction

### 3.2 Mod Structure Detection

- [x] Implement `find_manifest()` function
- [x] Use `walkdir` to recursively search for manifest.json
- [x] Return path to directory containing manifest.json
- [x] Handle multiple manifests (use first found, log warning)
- [x] Handle no manifest found (return error)
- [x] Detect common mod structures:
  - Flat: manifest.json at archive root
  - Nested: manifest.json in subfolder
  - Double-nested: archive → ModName → manifest.json
- [ ] Write unit tests for different structures

### 3.3 Metadata Parsing

- [x] Implement `parse_manifest()` function
- [x] Read manifest.json file
- [x] Parse JSON using serde_json
- [x] Extract required fields: Name, Version, UniqueID
- [x] Extract optional fields: Author, Description, EntryDll, UpdateKeys
- [x] Validate required fields are present
- [x] Return error if JSON is invalid
- [x] Return error if required fields missing
- [x] Create Mod struct from manifest data
- [ ] Write unit tests for valid/invalid manifests

### 3.4 Mod Installation

- [x] Implement `install_mod()` function
- [x] Get game path from settings
- [x] Return error if game path not configured
- [x] Construct target path: `{game_path}/Mods/{UniqueID}/`
- [x] Check if target already exists (duplicate handling)
- [x] Create Mods folder if it doesn't exist
- [x] Copy all mod files from temp to game folder
- [x] Preserve directory structure
- [ ] Verify copy was successful (check file sizes)
- [x] Add mod entry to mod list JSON (via scan_mods on frontend)
- [x] Set mod status as "enabled" by default
- [x] Emit "mod-installed" event to frontend

### 3.5 Duplicate Mod Handling

- [x] Detect if mod with same UniqueID exists
- [x] Compare versions (old vs new)
- [ ] Show confirmation dialog for updates
- [x] Implement "Update" option (replace old with new)
- [ ] Implement "Keep Both" option (rename new mod folder)
- [ ] Implement "Cancel" option (leave download in completed state)
- [x] Backup old version before update (to `{app_data}/backups/`)
- [x] Add setting for "Auto-update mods without confirmation"
- [ ] Respect auto-update setting in duplicate handling

### 3.6 Post-Installation Cleanup

- [x] Delete temp extraction directory after successful install
- [x] Delete downloaded archive if "delete_after_install" enabled
- [ ] Update download status to "installed"
- [x] Show success notification with mod name and version
- [x] Refresh mod list in frontend UI
- [ ] Highlight newly installed mod in mod list

### 3.7 Installation Error Handling

- [ ] Handle permission denied errors (read-only folders)
- [ ] Show actionable error messages
- [ ] Handle disk full errors (rollback partial installation)
- [ ] Handle locked files (retry with delay)
- [ ] Implement rollback on failure (delete partial files)
- [ ] Keep download in "completed" status if install fails (for retry)
- [ ] Log detailed errors for debugging
- [ ] Show user-friendly errors in UI

### 3.8 Manual Installation Support

- [ ] Show "Install" button for completed downloads (if auto-install disabled)
- [ ] Implement manual install trigger from UI
- [ ] Show "Open Folder" button for failed/manual installs
- [ ] Open file manager to downloads folder when clicked

### 3.9 Settings Integration

- [x] Add `auto_install: bool` setting (default: true)
- [x] Add `confirm_before_install: bool` setting (default: false)
- [x] Add `delete_after_install: bool` setting (default: false)
- [x] Load settings in mod_installer
- [x] Respect auto_install setting
- [ ] Show confirmation dialog if confirm_before_install enabled
- [ ] Delete archive based on delete_after_install setting

### 3.10 Tauri Commands

- [x] Create `install_mod` command (manual install trigger)
- [x] Create `open_downloads_folder` command
- [x] Create `open_mod_folder` command (open installed mod in file manager)
- [x] Register commands in lib.rs

---

## Phase 4: Polish and UI

### 4.1 Frontend Types

- [x] Create `src/types/download.ts`
- [x] Define `Download` interface
- [x] Define `DownloadStatus` type
- [x] Define `DownloadProgress` interface
- [x] Export all types

### 4.2 Download Manager UI Component

- [x] Create `src/components/features/downloads/DownloadManager.tsx`
- [x] Create collapsible panel in main UI
- [x] Show download count badge in toolbar
- [x] Implement tabs: "Active", "Queued", "Completed", "Failed"
- [x] Display empty state when no downloads
- [x] Add "Clear All Completed" button
- [ ] Add "Pause All" / "Resume All" buttons
- [x] Make panel resizable or scrollable for many downloads

### 4.3 Download Item Component

- [x] Create `src/components/features/downloads/DownloadItem.tsx`
- [x] Display mod name and file name
- [x] Show progress bar with percentage
- [x] Display download speed (formatted: KB/s, MB/s)
- [x] Display ETA (formatted: "2m 30s", "1h 15m")
- [x] Show status icon (queued, downloading, completed, failed)
- [x] Add action buttons: Pause/Resume, Cancel, Retry
- [x] Show error message for failed downloads
- [x] Make item clickable to expand/collapse details
- [ ] Add "Install Now" button for completed downloads (if auto-install off)
- [x] Add "Open Folder" button

### 4.4 Download Notifications

- [x] Create `src/components/ui/Toast.tsx` (toast notification system)
- [x] Show toast when download starts: "Downloading [ModName]..."
- [x] Show toast when download completes: "[ModName] downloaded"
- [x] Show toast when mod installed: "[ModName] installed successfully"
- [x] Show toast on download failure with error message
- [ ] Add "Retry" action to failure notifications
- [ ] Add "View" action to success notifications (jump to mod in list)
- [x] Auto-dismiss notifications after 5 seconds (except errors)

### 4.5 Settings UI

- [x] Add "Downloads" section to SettingsModal
- [ ] Add download directory selector with "Browse" button
- [ ] Add concurrent downloads input (number, 1-5)
- [x] Add "Auto-install mods" checkbox
- [x] Add "Confirm before installing" checkbox
- [x] Add "Delete archive after install" checkbox
- [ ] Add help text explaining free vs premium limits
- [x] Add "Nexus API Key" field (optional, for future Premium support)
- [x] Save settings when changed
- [x] Show validation errors if invalid values entered

### 4.6 Event Listeners

- [x] Create `useDownloads()` hook
- [x] Listen for "nxm-url-received" event
- [x] Listen for "download-queued" event
- [x] Listen for "download-progress" event
- [x] Listen for "download-completed" event
- [x] Listen for "download-failed" event
- [x] Listen for "mod-installed" event
- [x] Update state when events received
- [x] Refresh mod list when "mod-installed" received
- [x] Clean up event listeners on unmount

### 4.7 Download Manager Integration

- [x] Add DownloadManager button to main toolbar
- [x] Position button near settings icon
- [x] Show badge with active download count
- [x] Toggle DownloadManager panel when clicked
- [x] Integrate with existing app layout
- [x] Ensure responsive design (works on small windows)

### 4.8 Mod List Integration

- [x] Add "Downloaded from Nexus" indicator to mod items
- [x] Store Nexus mod_id and file_id in mod metadata
- [x] Add "View on Nexus" button for Nexus mods
- [x] Highlight newly installed mods (with "NEW" badge and pulsing animation)
- [ ] Add download timestamp to mod metadata

### 4.9 Error Handling UI

- [x] Show user-friendly error messages (via toast notifications)
- [ ] Provide actionable recovery steps
- [x] Add "Copy Error" button for bug reports
- [ ] Link to help documentation for common errors
- [ ] Handle offline state gracefully

### 4.10 Testing and Debugging

- [ ] Test full flow on Windows
- [ ] Test full flow on macOS
- [ ] Test full flow on Linux
- [ ] Test with various mods from Nexus (different sizes, structures)
- [ ] Test pause/resume functionality
- [ ] Test with slow network (verify speed/ETA accurate)
- [ ] Test with network disconnection (verify retry works)
- [ ] Test concurrent downloads (multiple queued)
- [ ] Test duplicate mod handling (updates)
- [ ] Test with mods missing manifest.json
- [ ] Performance test: 50+ downloads in history
- [ ] Test queue persistence (close app while downloading, reopen)

---

## Phase 5: Documentation and Release

### 5.1 User Documentation

- [ ] Write README section explaining NXM protocol setup
- [ ] Document how to click "Mod Manager Download" on Nexus
- [ ] Document browser permission prompt (first time)
- [ ] Document free vs premium account differences
- [ ] Create troubleshooting guide for common issues
- [ ] Add screenshots of download manager UI

### 5.2 Developer Documentation

- [ ] Document NXM protocol architecture in code comments
- [ ] Document download manager queue system
- [ ] Document mod installation flow
- [ ] Add JSDoc comments to TypeScript interfaces
- [ ] Add rustdoc comments to public Rust functions

### 5.3 Testing Checklist

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Manual testing complete on all platforms
- [ ] No memory leaks in download manager
- [ ] No file handles leaked
- [ ] Temp directories cleaned up properly

### 5.4 Release Preparation

- [ ] Update CHANGELOG.md
- [ ] Bump version number
- [ ] Create GitHub release notes
- [ ] Tag release with version number
- [ ] Build release binaries for all platforms
- [ ] Test release binaries (not dev builds)

---

## Task Summary

**Phase 1:** ~35 tasks (Foundation)
**Phase 2:** ~50 tasks (Download Manager)
**Phase 3:** ~40 tasks (Mod Installation)
**Phase 4:** ~45 tasks (Polish and UI)
**Phase 5:** ~15 tasks (Documentation and Release)

**Total:** ~185 tasks

## Estimated Timeline

- **Phase 1:** 3-5 days (protocol registration and parsing)
- **Phase 2:** 5-7 days (download queue and manager)
- **Phase 3:** 4-6 days (mod installation and extraction)
- **Phase 4:** 5-7 days (UI components and polish)
- **Phase 5:** 2-3 days (documentation and testing)

**Total Estimate:** 3-4 weeks for complete implementation

## Notes

- Tasks can be worked on in parallel within phases (e.g., UI development while backend is being tested)
- Some tasks may be split into smaller subtasks during implementation
- Testing tasks should be performed continuously, not just at the end
- User feedback during beta testing may add additional polish tasks
