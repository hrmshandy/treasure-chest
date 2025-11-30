# Specification: File System Synchronization

## Overview

This specification defines how the mod manager detects and synchronizes with manual changes made to mod folders outside the application, ensuring the UI always reflects the actual filesystem state.

## ADDED Requirements

### Requirement: File System Watcher

The system SHALL monitor the Mods directory for file system changes in real-time.

#### Scenario: Watcher initialization
- **GIVEN** the application starts
- **AND** a valid game path is configured
- **WHEN** the initial mod scan completes
- **THEN** a file system watcher SHALL be initialized for the Mods directory
- **AND** the watcher SHALL use non-recursive mode (only watch direct children)
- **AND** the watcher SHALL use a 1-second debounce to batch rapid changes

#### Scenario: Watcher initialization failure
- **GIVEN** the file system watcher fails to initialize (unsupported filesystem, permissions)
- **WHEN** the initialization error is caught
- **THEN** a warning SHALL be logged
- **AND** the application SHALL continue functioning with manual refresh only
- **AND** the user SHALL NOT see an error dialog

#### Scenario: Game path change
- **GIVEN** the file system watcher is active
- **WHEN** the user changes the game path in settings
- **THEN** the old watcher SHALL be stopped and disposed
- **AND** a new watcher SHALL be initialized for the new Mods directory

### Requirement: Detect Folder Creation

The system SHALL detect when new mod folders are manually added to the Mods directory.

#### Scenario: Single folder added
- **GIVEN** the file watcher is active
- **WHEN** a user copies a folder "NewMod" into the Mods directory
- **AND** the folder contains a valid manifest.json
- **THEN** within 2 seconds, the folder SHALL be scanned
- **AND** the mod SHALL be added to the mod list
- **AND** a "mod-added" event SHALL be emitted to the frontend
- **AND** the UI SHALL update to show the new mod

#### Scenario: Multiple folders added rapidly
- **GIVEN** the file watcher is active
- **WHEN** a user extracts an archive containing 3 mod folders
- **AND** all folders are created within 500ms of each other
- **THEN** the watcher SHALL debounce the events
- **AND** all 3 folders SHALL be scanned together after the 1-second debounce
- **AND** all 3 mods SHALL be added to the list in one batch update

#### Scenario: Invalid folder added
- **GIVEN** the file watcher is active
- **WHEN** a user copies a folder that does not contain manifest.json
- **THEN** the folder SHALL be scanned
- **AND** no mod SHALL be added to the list
- **AND** a debug log SHALL record the invalid folder

### Requirement: Detect Folder Deletion

The system SHALL detect when mod folders are manually deleted from the Mods directory.

#### Scenario: Single folder deleted
- **GIVEN** a mod "ExampleMod" is in the mod list
- **AND** the file watcher is active
- **WHEN** a user deletes the "ExampleMod" folder via file manager
- **THEN** within 2 seconds, a "Remove" event SHALL be detected
- **AND** the mod SHALL be removed from the mod list
- **AND** a "mod-removed" event SHALL be emitted to the frontend
- **AND** the UI SHALL update to remove the mod

#### Scenario: Multiple folders deleted
- **GIVEN** a group of 3 mods exists
- **WHEN** a user deletes all 3 folders
- **THEN** all 3 mods SHALL be removed from the list
- **AND** the group SHALL no longer be displayed

#### Scenario: Folder delete-recreate cycle
- **GIVEN** a mod folder exists
- **WHEN** a user deletes and immediately recreates it (e.g., replace with new version)
- **THEN** the watcher SHALL detect both the Remove and Create events
- **AND** the mod SHALL be removed and re-added
- **AND** the new version's metadata SHALL be loaded

### Requirement: Detect Folder Rename

The system SHALL detect when mod folders are manually renamed in the filesystem.

#### Scenario: Enable via manual rename
- **GIVEN** a mod folder "ExampleMod.disabled" exists
- **AND** the mod is shown as disabled in the UI
- **WHEN** a user renames it to "ExampleMod" via file manager
- **THEN** within 2 seconds, a "Rename" event SHALL be detected
- **AND** the mod's is_enabled field SHALL be updated to true
- **AND** the UI SHALL update to show the enabled state

#### Scenario: Disable via manual rename
- **GIVEN** a mod folder "ExampleMod" exists
- **AND** the mod is shown as enabled in the UI
- **WHEN** a user renames it to "ExampleMod.disabled" via file manager
- **THEN** within 2 seconds, a "Rename" event SHALL be detected
- **AND** the mod's is_enabled field SHALL be updated to false
- **AND** the UI SHALL update to show the disabled state

#### Scenario: Non-toggle rename
- **GIVEN** a mod folder "OldName" exists
- **WHEN** a user renames it to "NewName" (not a toggle)
- **THEN** the watcher SHALL detect the rename
- **AND** the mod's path field SHALL be updated
- **AND** the mod SHALL be re-scanned to update metadata
- **AND** if manifest.json has a different name, the mod name SHALL be updated

### Requirement: Debouncing and Batching

The system SHALL batch rapid filesystem events to avoid performance issues.

#### Scenario: Rapid events during extraction
- **GIVEN** a user extracts a large mod archive with 100+ files
- **WHEN** file creation events fire rapidly
- **THEN** the watcher SHALL debounce events with a 1-second delay
- **AND** only one scan SHALL be triggered per folder after events settle
- **AND** the UI SHALL update once with all changes batched

#### Scenario: Continuous file writes
- **GIVEN** a mod folder is being written to continuously (e.g., log file updates)
- **WHEN** write events fire every 100ms
- **THEN** the watcher SHALL ignore non-folder events (only watch folders, not files)
- **AND** no scans SHALL be triggered for file modifications

### Requirement: Event Filtering

The system SHALL filter filesystem events to only process relevant changes.

#### Scenario: Ignore non-mod folders
- **GIVEN** the Mods directory contains a non-mod folder (e.g., ".git" or "README.txt")
- **WHEN** the watcher detects changes to that folder
- **THEN** the changes SHALL be ignored
- **AND** no mod scan SHALL be triggered

#### Scenario: Ignore temporary files
- **GIVEN** a mod extraction creates temporary ".part" files
- **WHEN** the watcher detects these file events
- **THEN** the events SHALL be ignored (only folder events processed)
- **AND** the final folder creation SHALL trigger a scan

### Requirement: State Consistency

The system SHALL ensure the mod list always reflects the actual filesystem state.

#### Scenario: List-filesystem mismatch detection
- **GIVEN** the application resumes from sleep/suspension
- **WHEN** the window regains focus
- **THEN** a full rescan SHALL be triggered
- **AND** any discrepancies between list and filesystem SHALL be resolved
- **AND** missing mods SHALL be removed
- **AND** new mods SHALL be added

#### Scenario: Manual refresh
- **GIVEN** the user suspects the list is out of sync
- **WHEN** the user clicks a "Refresh" button or presses F5
- **THEN** a full rescan SHALL be triggered
- **AND** the mod list SHALL be rebuilt from filesystem
- **AND** any file watcher errors SHALL be cleared

### Requirement: Cross-Platform Support

The system SHALL handle filesystem differences across operating systems.

#### Scenario: Windows filesystem events
- **GIVEN** the app is running on Windows
- **WHEN** the file watcher is initialized
- **THEN** it SHALL use Windows ReadDirectoryChangesW API via notify crate
- **AND** all event types SHALL be supported

#### Scenario: macOS filesystem events
- **GIVEN** the app is running on macOS
- **WHEN** the file watcher is initialized
- **THEN** it SHALL use macOS FSEvents API via notify crate
- **AND** all event types SHALL be supported

#### Scenario: Linux filesystem events
- **GIVEN** the app is running on Linux
- **WHEN** the file watcher is initialized
- **THEN** it SHALL use inotify on supported systems
- **AND** if inotify is unavailable, SHALL fall back to polling
- **AND** a warning SHALL be logged about reduced performance

### Requirement: Performance

The system SHALL maintain acceptable performance even with many mods.

#### Scenario: Large mod directory
- **GIVEN** the Mods directory contains 200+ mod folders
- **WHEN** a single folder is added or removed
- **THEN** only that specific folder SHALL be scanned (not full rescan)
- **AND** the UI SHALL update within 500ms
- **AND** CPU usage SHALL remain under 5% during scan

#### Scenario: Network drive
- **GIVEN** the Mods directory is on a network drive
- **WHEN** filesystem events are detected
- **THEN** the watcher SHALL still function
- **AND** timeouts SHALL be extended to 10 seconds for slow networks
- **AND** a warning SHALL be shown if operations are slow

### Requirement: Error Recovery

The system SHALL recover gracefully from file watcher errors.

#### Scenario: Watcher stops unexpectedly
- **GIVEN** the file watcher is active
- **WHEN** the watcher encounters an error and stops
- **THEN** the error SHALL be logged
- **AND** a background task SHALL attempt to reinitialize the watcher every 30 seconds
- **AND** if reinitialization succeeds, normal operation SHALL resume

#### Scenario: Permission denied on folder
- **GIVEN** a mod folder becomes read-only or locked
- **WHEN** the watcher tries to process it
- **THEN** the error SHALL be logged
- **AND** the mod SHALL remain in the list with a warning state
- **AND** the watcher SHALL continue monitoring other folders

### Requirement: User Notifications

The system SHALL provide feedback for automatic synchronization events.

#### Scenario: Silent sync for removals
- **GIVEN** a user manually deletes a mod folder
- **WHEN** the file watcher detects the deletion
- **THEN** the mod SHALL be removed from the list silently
- **AND** no notification SHALL be shown (user's intentional action)

#### Scenario: Notification for additions
- **GIVEN** a user manually adds a new mod folder
- **WHEN** the file watcher detects and processes it
- **THEN** a subtle notification SHALL be shown: "[ModName] detected and added"
- **AND** the notification SHALL auto-dismiss after 3 seconds

#### Scenario: No notification for renames (toggles)
- **GIVEN** a user manually toggles a mod via folder rename
- **WHEN** the file watcher detects the rename
- **THEN** the mod state SHALL update silently
- **AND** no notification SHALL be shown
