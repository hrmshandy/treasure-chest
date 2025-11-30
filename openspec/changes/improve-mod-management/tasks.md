# Implementation Tasks: Mod Management Improvements

## Overview

This document outlines the implementation tasks for improving mod management operations. Tasks are organized into 7 phases covering data models, group tracking, toggle implementation, delete fixes, file sync, UI updates, and testing.

**Estimated Total:** ~70 tasks across 7 phases
**Estimated Time:** 10-14 days

---

## Phase 1: Data Model Updates

### 1.1 Backend Model Changes

- [ ] 1.1.1 Add `group_id: Option<String>` field to Mod struct in `models.rs`
- [ ] 1.1.2 Add `install_source: Option<String>` field to Mod struct
- [ ] 1.1.3 Add `download_id: Option<String>` field to Mod struct
- [ ] 1.1.4 Add serde annotations for new fields
- [ ] 1.1.5 Write unit tests for Mod serialization/deserialization
- [ ] 1.1.6 Test backwards compatibility with existing mod data

### 1.2 Frontend Type Updates

- [ ] 1.2.1 Add `groupId?: string` field to Mod interface in `src/types/mod.ts`
- [ ] 1.2.2 Add `installSource?: string` field to Mod interface
- [ ] 1.2.3 Add `downloadId?: string` field to Mod interface
- [ ] 1.2.4 Update mock data in `src/data/mock.ts` to include new fields

---

## Phase 2: Group Tracking on Install

### 2.1 Multi-Folder Detection

- [ ] 2.1.1 Review `detect_mod_folders()` in `mod_installer.rs` (already implemented)
- [ ] 2.1.2 Verify it correctly detects multiple mod folders in archives
- [ ] 2.1.3 Test with Stardew Valley Expanded archive

### 2.2 Group ID Generation

- [ ] 2.2.1 Add `uuid` crate to Cargo.toml if not already present
- [ ] 2.2.2 In `install_from_archive()`, generate UUID when `mod_folders.len() > 1`
- [ ] 2.2.3 Assign same `group_id` to all mods installed from archive
- [ ] 2.2.4 Set `install_source` to "nexus" for NXM downloads, "manual" for file imports
- [ ] 2.2.5 Set `download_id` to the download task ID if from download manager
- [ ] 2.2.6 Write unit tests for group ID assignment logic

### 2.3 Single-Folder Handling

- [ ] 2.3.1 Ensure single-folder installs get `group_id: None`
- [ ] 2.3.2 Ensure manually-added mods get `group_id: None`
- [ ] 2.3.3 Test that existing behavior is unchanged for single mods

---

## Phase 3: Enable/Disable (Toggle) Implementation

### 3.1 Backend Module

- [ ] 3.1.1 Create `src-tauri/src/mod_toggle.rs` module
- [ ] 3.1.2 Implement `toggle_mod(path, enable)` function
- [ ] 3.1.3 Implement folder rename logic (append/remove `.disabled` suffix)
- [ ] 3.1.4 Handle edge cases (already disabled, name collisions)
- [ ] 3.1.5 Return descriptive errors (permission denied, path not found, etc.)

### 3.2 Group Toggle Logic

- [ ] 3.2.1 Implement `toggle_mod_group(group_id, mods, enable)` function
- [ ] 3.2.2 Add pre-validation phase (check all paths exist and are writable)
- [ ] 3.2.3 Implement atomic operation with rollback on failure
- [ ] 3.2.4 Test rollback logic with simulated failures
- [ ] 3.2.5 Write unit tests for group toggle edge cases

### 3.3 Tauri Commands

- [ ] 3.3.1 Create `toggle_mod` command in `commands.rs`
- [ ] 3.3.2 Create `toggle_mod_group` command in `commands.rs`
- [ ] 3.3.3 Add commands to invoke_handler in `lib.rs`
- [ ] 3.3.4 Emit "mod-toggled" event to frontend on success
- [ ] 3.3.5 Emit "mod-toggle-failed" event on error with details

### 3.4 Scanner Integration

- [ ] 3.4.1 Update `scan_mods()` to detect `.disabled` suffix
- [ ] 3.4.2 Set `is_enabled: false` for folders ending in `.disabled`
- [ ] 3.4.3 Strip `.disabled` from folder name when setting mod path
- [ ] 3.4.4 Test that scan correctly identifies disabled mods

---

## Phase 4: Delete Implementation

### 4.1 Backend Changes

- [ ] 4.1.1 Add `trash` crate to Cargo.toml
- [ ] 4.1.2 Create `delete_mod(path)` function in `commands.rs` or new `mod_deletion.rs` module
- [ ] 4.1.3 Implement trash integration using `trash::delete()`
- [ ] 4.1.4 Add fallback to `fs::remove_dir_all()` if trash fails
- [ ] 4.1.5 Log when fallback to permanent deletion occurs
- [ ] 4.1.6 Return clear error messages (permission denied, folder locked, etc.)

### 4.2 Group Deletion Logic

- [ ] 4.2.1 Implement `delete_mod_group(group_id, mods)` function
- [ ] 4.2.2 Add pre-validation phase (check all paths exist and are deletable)
- [ ] 4.2.3 Implement atomic operation (all succeed or none deleted)
- [ ] 4.2.4 On partial failure, restore already-deleted mods from trash if possible
- [ ] 4.2.5 Write unit tests for group deletion

### 4.3 Tauri Commands

- [ ] 4.3.1 Update existing `delete_mod` command to actually delete folders (currently only removes from state)
- [ ] 4.3.2 Create `delete_mod_group` command
- [ ] 4.3.3 Add commands to invoke_handler
- [ ] 4.3.4 Emit "mod-deleted" event to frontend on success
- [ ] 4.3.5 Emit "mod-delete-failed" event on error

### 4.4 Frontend Confirmation Dialogs

- [ ] 4.4.1 Create ConfirmDialog component if not exists
- [ ] 4.4.2 Show confirmation before delete: "Delete [ModName]? This cannot be undone."
- [ ] 4.4.3 For groups: "Delete all 3 mods from Stardew Valley Expanded?"
- [ ] 4.4.4 Add "Don't ask again" checkbox (save to settings)
- [ ] 4.4.5 Respect `confirmBeforeDelete` setting

---

## Phase 5: File System Sync

### 5.1 Backend Watcher Setup

- [ ] 5.1.1 Add `notify` crate to Cargo.toml
- [ ] 5.1.2 Create `src-tauri/src/file_watcher.rs` module
- [ ] 5.1.3 Implement `FileWatcher` struct with notify watcher
- [ ] 5.1.4 Initialize watcher in `lib.rs` after app setup
- [ ] 5.1.5 Watch `{gamePath}/Mods/` folder in NonRecursive mode
- [ ] 5.1.6 Use debounced watcher with 1-second delay

### 5.2 Event Handling

- [ ] 5.2.1 Handle `Create` events (new folder added)
- [ ] 5.2.2 Handle `Remove` events (folder deleted)
- [ ] 5.2.3 Handle `Rename` events (folder renamed, e.g., to `.disabled`)
- [ ] 5.2.4 Filter out non-mod folders (no manifest.json)
- [ ] 5.2.5 Debounce rapid events (e.g., when extracting many files)
- [ ] 5.2.6 Emit events to frontend for each change

### 5.3 State Synchronization

- [ ] 5.3.1 On `Create`: Scan new folder, add to mod list if valid
- [ ] 5.3.2 On `Remove`: Remove mod from state (no error, user intended it)
- [ ] 5.3.3 On `Rename`: Update `is_enabled` if toggle detected, update `path` otherwise
- [ ] 5.3.4 Handle race conditions (e.g., delete while scanning)
- [ ] 5.3.5 Write integration tests for watcher

### 5.4 Error Handling

- [ ] 5.4.1 Handle watcher initialization failures gracefully
- [ ] 5.4.2 Fallback to manual refresh if watcher unavailable
- [ ] 5.4.3 Log watcher errors for debugging
- [ ] 5.4.4 Don't crash app if watcher fails

---

## Phase 6: UI Implementation

### 6.1 ModGroup Component

- [ ] 6.1.1 Create `src/components/features/mods/ModGroup.tsx`
- [ ] 6.1.2 Implement collapsible parent row with expand/collapse icon
- [ ] 6.1.3 Show group metadata (name, mod count, install source)
- [ ] 6.1.4 Add toggle switch for entire group
- [ ] 6.1.5 Add delete button for entire group
- [ ] 6.1.6 Render child ModItems when expanded
- [ ] 6.1.7 Style parent row distinctly from child rows

### 6.2 ModList Updates

- [ ] 6.2.1 Update `ModList.tsx` to group mods by `groupId`
- [ ] 6.2.2 Render ModGroup for groups, ModItem for standalone mods
- [ ] 6.2.3 Maintain sort order (groups sorted by first mod's name)
- [ ] 6.2.4 Handle empty groups gracefully
- [ ] 6.2.5 Test with various mod list sizes and group counts

### 6.3 ModItem Updates

- [ ] 6.3.1 Add toggle switch to ModItem (replace status indicator)
- [ ] 6.3.2 Wire toggle switch to `handleToggleMod` in App.tsx
- [ ] 6.3.3 Show "Ungroup" action when child mod is toggled individually
- [ ] 6.3.4 Update delete button behavior for grouped mods
- [ ] 6.3.5 Style child rows (indentation, connecting lines)

### 6.4 App.tsx Integration

- [ ] 6.4.1 Implement `handleToggleMod(id, enabled)` to call backend
- [ ] 6.4.2 Implement `handleToggleGroup(groupId, enabled)` for group toggles
- [ ] 6.4.3 Update `handleDeleteMod(id)` to call backend delete command (not just setState)
- [ ] 6.4.4 Implement `handleDeleteGroup(groupId)` for group deletions
- [ ] 6.4.5 Show loading states during async operations
- [ ] 6.4.6 Handle errors with toast notifications

### 6.5 Event Listeners

- [ ] 6.5.1 Listen for "mod-toggled" events and update state
- [ ] 6.5.2 Listen for "mod-deleted" events and remove from state
- [ ] 6.5.3 Listen for "mod-added" events from file watcher
- [ ] 6.5.4 Listen for "mod-removed" events from file watcher
- [ ] 6.5.5 Listen for "mod-renamed" events from file watcher
- [ ] 6.5.6 Update UI immediately on events (optimistic updates)

### 6.6 Confirmation Dialogs

- [ ] 6.6.1 Show confirmation before deleting single mod
- [ ] 6.6.2 Show confirmation before deleting group: "Delete all X mods from [GroupName]?"
- [ ] 6.6.3 Show confirmation before toggling group: "Disable all X mods?"
- [ ] 6.6.4 Add "Don't ask again" option (save to settings)
- [ ] 6.6.5 Style dialogs consistently with app theme

---

## Phase 7: Testing and Polish

### 7.1 Unit Tests

- [ ] 7.1.1 Write tests for toggle_mod function (various scenarios)
- [ ] 7.1.2 Write tests for toggle_mod_group with rollback
- [ ] 7.1.3 Write tests for delete_mod function
- [ ] 7.1.4 Write tests for delete_mod_group
- [ ] 7.1.5 Write tests for group ID assignment on install
- [ ] 7.1.6 Ensure all tests pass

### 7.2 Integration Tests

- [ ] 7.2.1 Test multi-folder install creates group correctly
- [ ] 7.2.2 Test toggling group disables all members
- [ ] 7.2.3 Test deleting group removes all folders
- [ ] 7.2.4 Test file watcher detects manual additions
- [ ] 7.2.5 Test file watcher detects manual deletions
- [ ] 7.2.6 Test file watcher detects manual renames
- [ ] 7.2.7 Test backwards compatibility with existing mods

### 7.3 Platform Testing

- [ ] 7.3.1 Test on Windows (folder rename, delete, trash)
- [ ] 7.3.2 Test on macOS (folder rename, delete, trash)
- [ ] 7.3.3 Test on Linux (folder rename, delete, trash fallback)
- [ ] 7.3.4 Test with network drives (slower operations)
- [ ] 7.3.5 Test with read-only folders (error handling)

### 7.4 Edge Cases

- [ ] 7.4.1 Test with locked files (SMAPI running)
- [ ] 7.4.2 Test with permission denied scenarios
- [ ] 7.4.3 Test with very long folder names
- [ ] 7.4.4 Test with special characters in folder names
- [ ] 7.4.5 Test with 100+ mods (performance)
- [ ] 7.4.6 Test with empty Mods folder

### 7.5 UI/UX Polish

- [ ] 7.5.1 Add loading spinners for async operations
- [ ] 7.5.2 Add success toasts for completed operations
- [ ] 7.5.3 Add error toasts with actionable messages
- [ ] 7.5.4 Ensure toggle switches animate smoothly
- [ ] 7.5.5 Ensure group expand/collapse animations work
- [ ] 7.5.6 Test keyboard navigation (accessibility)

### 7.6 Documentation

- [ ] 7.6.1 Update README with group feature explanation
- [ ] 7.6.2 Document `.disabled` suffix convention
- [ ] 7.6.3 Add troubleshooting section for common errors
- [ ] 7.6.4 Update screenshots to show grouped mods
- [ ] 7.6.5 Add code comments for complex logic

---

## Task Summary

**Phase 1:** 10 tasks (Data Model)
**Phase 2:** 10 tasks (Group Tracking)
**Phase 3:** 14 tasks (Toggle Implementation)
**Phase 4:** 15 tasks (Delete Fix)
**Phase 5:** 14 tasks (File System Sync)
**Phase 6:** 23 tasks (UI Implementation)
**Phase 7:** 26 tasks (Testing and Polish)

**Total:** ~112 tasks

## Estimated Timeline

- **Phase 1:** 1-2 days (Data model updates)
- **Phase 2:** 1 day (Group tracking)
- **Phase 3:** 2 days (Toggle implementation)
- **Phase 4:** 1 day (Delete fix)
- **Phase 5:** 2-3 days (File system sync)
- **Phase 6:** 2-3 days (UI components)
- **Phase 7:** 2 days (Testing and polish)

**Total Estimate:** 10-14 days for complete implementation

## Dependencies

- `uuid` crate (for group ID generation)
- `notify` crate (for file system watching)
- `trash` crate (for recycle bin integration)

## Notes

- Phases 1-2 can be done first, then 3-4 in parallel, then 5, then 6, then 7
- UI work (Phase 6) can start after Phase 3 is complete (toggle commands exist)
- Testing (Phase 7) should be continuous, not just at the end
- File watcher (Phase 5) can be implemented independently and enabled later
