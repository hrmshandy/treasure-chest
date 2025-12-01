# Mod Deletion

## ADDED Requirements

### Requirement: Delete Mod from Disk

The system MUST delete mod folders from the filesystem when the user deletes a mod.

#### Scenario: Delete mod from UI
- **Given** an installed mod "Example Mod" at `Mods/Example Mod`
- **When** the user clicks the delete button for "Example Mod"
- **And** confirms the deletion
- **Then** the folder `Mods/Example Mod` is completely removed from disk
- **And** the mod is removed from the mod list
- **And** a success notification is shown

#### Scenario: Delete disabled mod
- **Given** a disabled mod at `Mods/Example Mod.disabled`
- **When** the user deletes the mod
- **Then** the folder `Mods/Example Mod.disabled` is removed from disk
- **And** the mod is removed from the list

#### Scenario: Handle deletion errors
- **Given** a mod folder that is locked by another process
- **When** the user attempts to delete it
- **Then** the deletion fails with an error message
- **And** the mod remains in the list
- **And** an error toast is shown to the user

### Requirement: Detect Manual Deletions

The system MUST reflect manual folder deletions when rescanning mods.

#### Scenario: User manually deletes mod folder
- **Given** a mod "Example Mod" is shown in the list
- **And** the user manually deletes the folder from disk
- **When** the application rescans mods (via refresh or app restart)
- **Then** "Example Mod" is no longer shown in the list
