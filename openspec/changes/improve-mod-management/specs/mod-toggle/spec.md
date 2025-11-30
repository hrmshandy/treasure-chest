# Specification: Mod Toggle (Enable/Disable)

## Overview

This specification defines how users can temporarily enable and disable mods using SMAPI's `.disabled` folder suffix convention, without uninstalling them.

## ADDED Requirements

### Requirement: Toggle Individual Mod

The system SHALL allow users to enable or disable individual mods by renaming the mod folder.

#### Scenario: Disable enabled mod
- **GIVEN** a mod folder named "ExampleMod" exists in the Mods directory
- **AND** the mod is currently enabled
- **WHEN** the user clicks the toggle switch to disable it
- **THEN** the folder SHALL be renamed to "ExampleMod.disabled"
- **AND** the mod's is_enabled field SHALL be set to false
- **AND** the UI SHALL update to show the disabled state

#### Scenario: Enable disabled mod
- **GIVEN** a mod folder named "ExampleMod.disabled" exists in the Mods directory
- **AND** the mod is currently disabled
- **WHEN** the user clicks the toggle switch to enable it
- **THEN** the folder SHALL be renamed to "ExampleMod"
- **AND** the mod's is_enabled field SHALL be set to true
- **AND** the UI SHALL update to show the enabled state

#### Scenario: Toggle already in desired state
- **GIVEN** a mod is already disabled
- **WHEN** the user attempts to disable it again
- **THEN** the operation SHALL succeed without error
- **AND** no folder rename SHALL occur
- **AND** a warning SHALL be logged

### Requirement: Toggle Mod Group

The system SHALL allow users to enable or disable all mods in a group atomically.

#### Scenario: Disable group
- **GIVEN** a group contains 3 enabled mods
- **WHEN** the user clicks the toggle switch on the group
- **THEN** all 3 mod folders SHALL be renamed to add ".disabled" suffix
- **AND** all operations SHALL complete before updating the UI
- **AND** if any operation fails, all previous renames SHALL be rolled back

#### Scenario: Enable group
- **GIVEN** a group contains 3 disabled mods
- **WHEN** the user clicks the toggle switch on the group
- **THEN** all 3 mod folders SHALL be renamed to remove ".disabled" suffix
- **AND** all operations SHALL complete before updating the UI
- **AND** if any operation fails, all previous renames SHALL be rolled back

#### Scenario: Partial group state
- **GIVEN** a group where 2 mods are enabled and 1 is disabled
- **WHEN** the user clicks the toggle switch on the group
- **THEN** a dialog SHALL ask "Enable all mods?" or "Disable all mods?"
- **AND** the operation SHALL bring all mods to the selected state

### Requirement: Folder Rename Operation

The system SHALL rename mod folders safely and atomically.

#### Scenario: Successful rename
- **GIVEN** a mod folder "ExampleMod" with full write permissions
- **WHEN** a toggle operation is performed
- **THEN** the folder SHALL be renamed using the filesystem's atomic rename operation
- **AND** the operation SHALL complete in under 1 second
- **AND** a "mod-toggled" event SHALL be emitted to the frontend

#### Scenario: Permission denied
- **GIVEN** a mod folder that is read-only or locked
- **WHEN** a toggle operation is attempted
- **THEN** the rename SHALL fail with a permission error
- **AND** an error message SHALL be shown: "Cannot disable [ModName]: folder is locked or read-only"
- **AND** the mod state SHALL remain unchanged

#### Scenario: Folder not found
- **GIVEN** a mod is listed but its folder no longer exists
- **WHEN** a toggle operation is attempted
- **THEN** the operation SHALL fail with a not-found error
- **AND** an error message SHALL be shown: "Cannot toggle [ModName]: folder not found"
- **AND** the mod SHALL be removed from the list

#### Scenario: Name collision
- **GIVEN** a mod folder "ExampleMod" exists
- **AND** a folder "ExampleMod.disabled" also exists
- **WHEN** the user attempts to disable "ExampleMod"
- **THEN** the operation SHALL fail with a conflict error
- **AND** an error message SHALL be shown: "Cannot disable [ModName]: a disabled version already exists"

### Requirement: Atomic Group Operations

The system SHALL ensure group toggle operations complete fully or not at all.

#### Scenario: Rollback on failure
- **GIVEN** a group with 3 mods: ModA, ModB, ModC
- **WHEN** the user disables the group
- **AND** ModA and ModB are successfully renamed
- **AND** ModC rename fails due to permissions
- **THEN** ModA and ModB SHALL be renamed back to their original names
- **AND** all mods SHALL remain in their original enabled state
- **AND** an error SHALL be shown: "Failed to disable 1 of 3 mods: [ModC] is locked"

#### Scenario: Pre-validation
- **GIVEN** a group with 3 mods
- **WHEN** the user initiates a group toggle
- **THEN** the system SHALL first verify all folders exist and are writable
- **AND** if any validation fails, no renames SHALL be attempted
- **AND** a clear error SHALL be shown listing which mods cannot be toggled

### Requirement: State Synchronization

The system SHALL keep UI state synchronized with filesystem state after toggle operations.

#### Scenario: Immediate UI update
- **GIVEN** a mod is successfully toggled
- **WHEN** the rename operation completes
- **THEN** the UI SHALL update within 100ms
- **AND** the toggle switch SHALL animate to the new state
- **AND** the mod SHALL move to the appropriate section (enabled/disabled) if list is filtered

#### Scenario: Event emission
- **GIVEN** a mod is toggled
- **WHEN** the operation completes successfully
- **THEN** a "mod-toggled" event SHALL be emitted with mod ID and new state
- **AND** the frontend SHALL listen for this event and update React state

### Requirement: SMAPI Integration

The system SHALL use SMAPI's standard convention for disabled mods.

#### Scenario: SMAPI recognition
- **GIVEN** a mod folder is renamed with ".disabled" suffix
- **WHEN** SMAPI (Stardew Valley Mod Loader) starts
- **THEN** SMAPI SHALL automatically ignore the disabled mod
- **AND** the mod SHALL not be loaded into the game

#### Scenario: Manual toggle in filesystem
- **GIVEN** a user manually renames "ExampleMod" to "ExampleMod.disabled" via file manager
- **WHEN** the file watcher detects the rename
- **THEN** the mod's is_enabled field SHALL update to false
- **AND** the UI SHALL reflect the disabled state

### Requirement: Scan Integration

The system SHALL correctly identify disabled mods during folder scanning.

#### Scenario: Scan disabled mod
- **GIVEN** a folder "ExampleMod.disabled" exists in the Mods directory
- **WHEN** the mod scanner runs
- **THEN** the mod SHALL be detected and added to the list
- **AND** the is_enabled field SHALL be set to false
- **AND** the path SHALL be stored with the ".disabled" suffix
- **AND** the display name SHALL strip the ".disabled" suffix

#### Scenario: Scan enabled mod
- **GIVEN** a folder "ExampleMod" exists without ".disabled" suffix
- **WHEN** the mod scanner runs
- **THEN** the mod SHALL be detected and added to the list
- **AND** the is_enabled field SHALL be set to true

### Requirement: Error Handling

The system SHALL provide clear, actionable error messages for toggle failures.

#### Scenario: Locked file error
- **GIVEN** a mod folder is locked by SMAPI (game is running)
- **WHEN** a toggle operation is attempted
- **THEN** the error message SHALL be: "Cannot toggle [ModName]: folder is in use. Close Stardew Valley and try again."

#### Scenario: Network drive timeout
- **GIVEN** the Mods folder is on a slow network drive
- **WHEN** a toggle operation times out after 5 seconds
- **THEN** the error SHALL be: "Toggle operation timed out. Check network connection."
- **AND** the mod state SHALL remain unchanged

#### Scenario: Multiple failures in group
- **GIVEN** a group toggle where 2 of 3 mods fail
- **WHEN** the operation fails and rolls back
- **THEN** the error SHALL list all failed mods: "Failed to toggle: [ModB] is locked, [ModC] permission denied"
