# Specification: Mod Deletion

## Overview

This specification defines how the mod manager permanently removes mod folders from disk when the user requests deletion, replacing the current behavior which only removes mods from the UI state.

## ADDED Requirements

### Requirement: Delete Individual Mod

The system SHALL permanently delete mod folders from the filesystem when the user clicks the delete button.

#### Scenario: Successful deletion
- **GIVEN** a mod "ExampleMod" exists in the Mods directory
- **AND** the user has write permissions
- **WHEN** the user clicks delete and confirms
- **THEN** the mod folder SHALL be moved to the system recycle bin/trash
- **AND** the mod SHALL be removed from the mod list
- **AND** a success message SHALL be shown: "[ModName] deleted"

#### Scenario: Deletion with confirmation
- **GIVEN** a mod exists in the list
- **WHEN** the user clicks the delete button
- **THEN** a confirmation dialog SHALL appear
- **AND** the dialog SHALL say: "Delete [ModName]? This will remove the mod from your game."
- **AND** the dialog SHALL have "Delete" and "Cancel" buttons
- **AND** the delete action SHALL only proceed if user clicks "Delete"

#### Scenario: Skip confirmation if setting enabled
- **GIVEN** the user has enabled "Don't ask before deleting" in settings
- **WHEN** the user clicks delete on a mod
- **THEN** no confirmation dialog SHALL appear
- **AND** the mod SHALL be deleted immediately

#### Scenario: Disabled mod deletion
- **GIVEN** a mod folder "ExampleMod.disabled" exists
- **WHEN** the user deletes the mod
- **THEN** the folder with ".disabled" suffix SHALL be deleted
- **AND** the operation SHALL succeed the same as for enabled mods

### Requirement: Delete Mod Group

The system SHALL allow deletion of all mods in a group with a single action.

#### Scenario: Group deletion confirmation
- **GIVEN** a group contains 3 mods
- **WHEN** the user clicks delete on the group
- **THEN** a confirmation dialog SHALL appear
- **AND** the dialog SHALL say: "Delete all 3 mods from [GroupName]?"
- **AND** the dialog SHALL list the mod names that will be deleted

#### Scenario: Successful group deletion
- **GIVEN** a group contains 3 mods with write permissions
- **WHEN** the user confirms group deletion
- **THEN** all 3 mod folders SHALL be moved to recycle bin/trash
- **AND** all 3 mods SHALL be removed from the list atomically
- **AND** a success message SHALL show: "Deleted 3 mods from [GroupName]"

#### Scenario: Partial group deletion failure
- **GIVEN** a group contains 3 mods: ModA, ModB, ModC
- **WHEN** deletion is confirmed
- **AND** ModA and ModB delete successfully
- **AND** ModC deletion fails (locked file)
- **THEN** ModA and ModB SHALL be restored from recycle bin if possible
- **AND** an error SHALL be shown: "Failed to delete [GroupName]: [ModC] is locked or in use"
- **AND** all 3 mods SHALL remain in the list

### Requirement: Recycle Bin Integration

The system SHALL move deleted mods to the system recycle bin instead of permanent deletion.

#### Scenario: Windows recycle bin
- **GIVEN** the app is running on Windows
- **WHEN** a mod is deleted
- **THEN** the folder SHALL be moved to the Windows Recycle Bin using the trash crate
- **AND** the user SHALL be able to restore it from Recycle Bin

#### Scenario: macOS trash
- **GIVEN** the app is running on macOS
- **WHEN** a mod is deleted
- **THEN** the folder SHALL be moved to macOS Trash using the trash crate
- **AND** the user SHALL be able to restore it from Trash

#### Scenario: Linux trash
- **GIVEN** the app is running on Linux with trash support
- **WHEN** a mod is deleted
- **THEN** the folder SHALL be moved to the FreeDesktop.org trash directory
- **AND** the user SHALL be able to restore it from trash

#### Scenario: Trash unavailable fallback
- **GIVEN** the system does not support trash (some Linux distros, network drives)
- **WHEN** a mod deletion is attempted
- **THEN** a warning dialog SHALL appear: "Recycle bin unavailable. Permanently delete [ModName]?"
- **AND** upon confirmation, the folder SHALL be permanently deleted using fs::remove_dir_all
- **AND** a warning SHALL be logged about permanent deletion

### Requirement: Atomic Operations

The system SHALL ensure deletion operations complete fully or not at all.

#### Scenario: Pre-deletion validation
- **GIVEN** a mod is about to be deleted
- **WHEN** the deletion operation starts
- **THEN** the system SHALL first verify the folder exists
- **AND** the system SHALL verify write permissions
- **AND** if validation fails, the operation SHALL be aborted with a clear error

#### Scenario: Rollback on group deletion failure
- **GIVEN** a group deletion is in progress
- **WHEN** any mod in the group fails to delete
- **THEN** all previously deleted mods SHALL be restored from trash
- **AND** if restoration fails, the error SHALL clearly state which mods are in trash

### Requirement: Error Handling

The system SHALL provide clear, actionable error messages for deletion failures.

#### Scenario: Permission denied
- **GIVEN** a mod folder is read-only or locked
- **WHEN** deletion is attempted
- **THEN** the error SHALL be: "Cannot delete [ModName]: permission denied. Check folder permissions."
- **AND** the mod SHALL remain in the list

#### Scenario: Folder in use
- **GIVEN** a mod folder is locked by another process (e.g., SMAPI is running)
- **WHEN** deletion is attempted
- **THEN** the error SHALL be: "Cannot delete [ModName]: folder is in use. Close Stardew Valley and try again."
- **AND** the mod SHALL remain in the list

#### Scenario: Folder not found
- **GIVEN** a mod is in the list but the folder no longer exists
- **WHEN** deletion is attempted
- **THEN** the mod SHALL be removed from the list without error
- **AND** a debug log SHALL record "Mod folder already deleted"

#### Scenario: Network error
- **GIVEN** the Mods folder is on a network drive
- **AND** the network connection is lost
- **WHEN** deletion is attempted
- **THEN** the operation SHALL timeout after 10 seconds
- **AND** the error SHALL be: "Deletion failed: network timeout. Check network connection."

### Requirement: State Synchronization

The system SHALL keep the UI synchronized with deletion operations.

#### Scenario: Optimistic UI update
- **GIVEN** a deletion operation is initiated
- **WHEN** the operation starts
- **THEN** the mod SHALL be marked as "deleting" in the UI
- **AND** a loading spinner SHALL appear
- **AND** the delete button SHALL be disabled

#### Scenario: Successful deletion UI update
- **GIVEN** a deletion completes successfully
- **WHEN** the folder is removed from disk
- **THEN** the mod SHALL be removed from the React state
- **AND** the UI SHALL update within 100ms
- **AND** the loading spinner SHALL disappear

#### Scenario: Failed deletion UI update
- **GIVEN** a deletion fails
- **WHEN** the error is returned
- **THEN** the "deleting" state SHALL be cleared
- **AND** the mod SHALL reappear in its original state
- **AND** the delete button SHALL be re-enabled

### Requirement: Confirmation Dialogs

The system SHALL show appropriate confirmation dialogs before destructive actions.

#### Scenario: Standard confirmation
- **GIVEN** a mod is about to be deleted
- **WHEN** the user has not disabled confirmations
- **THEN** a modal dialog SHALL appear
- **AND** the dialog SHALL have a clear title: "Delete Mod"
- **AND** the dialog SHALL show the mod name prominently
- **AND** the dialog SHALL explain the action: "This will remove [ModName] from your Mods folder."

#### Scenario: Group confirmation details
- **GIVEN** a group of 3 mods is about to be deleted
- **WHEN** the confirmation dialog appears
- **THEN** it SHALL list all 3 mod names
- **AND** it SHALL show the total count: "Delete all 3 mods?"
- **AND** it SHALL warn: "This will remove these mods from your game"

#### Scenario: Don't ask again option
- **GIVEN** a confirmation dialog is shown
- **WHEN** the user checks "Don't ask again"
- **AND** clicks "Delete"
- **THEN** the preference SHALL be saved to settings
- **AND** future deletions SHALL not show confirmations
- **AND** the setting SHALL be toggleable in the Settings modal

### Requirement: File System Watcher Integration

The system SHALL correctly handle file watcher events during deletion.

#### Scenario: Ignore self-triggered events
- **GIVEN** the file watcher is active
- **WHEN** the app deletes a mod folder
- **THEN** the watcher SHALL detect the deletion event
- **AND** the event SHALL be recognized as app-triggered (not manual)
- **AND** the event SHALL be ignored to prevent duplicate state updates

#### Scenario: External deletion during app deletion
- **GIVEN** the app is deleting a mod
- **WHEN** a user simultaneously deletes the same folder manually
- **THEN** the app deletion SHALL detect the folder is missing
- **AND** the operation SHALL be considered successful (folder is gone)
- **AND** no error SHALL be shown

### Requirement: Performance

The system SHALL delete mods efficiently without blocking the UI.

#### Scenario: Non-blocking deletion
- **GIVEN** a mod folder contains 1000+ files
- **WHEN** deletion is initiated
- **THEN** the operation SHALL run on a background thread
- **AND** the UI SHALL remain responsive
- **AND** the user SHALL be able to interact with other mods

#### Scenario: Large group deletion
- **GIVEN** a group contains 10 mods
- **WHEN** group deletion is confirmed
- **THEN** all deletions SHALL be parallelized (up to 3 concurrent)
- **AND** the operation SHALL complete within 10 seconds on typical hardware
- **AND** progress SHALL be shown: "Deleting 7 of 10 mods..."

### Requirement: Migration from Current Behavior

The system SHALL replace the existing delete implementation which only removes from UI.

#### Scenario: Current behavior removal
- **GIVEN** the old delete implementation exists in App.tsx
- **WHEN** the new backend delete command is implemented
- **THEN** the old code SHALL be removed: `setMods(mods.filter(m => m.id !== id))`
- **AND** the new code SHALL call: `await invoke('delete_mod', { path: mod.path })`
- **AND** state update SHALL only occur after backend confirms deletion

#### Scenario: Backwards compatibility
- **GIVEN** users have mods installed before this change
- **WHEN** they delete a mod using the new implementation
- **THEN** the deletion SHALL work identically regardless of install date
- **AND** no data migration is required
