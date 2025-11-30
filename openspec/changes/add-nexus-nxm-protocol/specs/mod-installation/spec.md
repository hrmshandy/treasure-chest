# Spec: Mod Installation

## Overview

Automatically extract downloaded mod archives, detect mod structure and metadata, copy files to the game mods folder, and add mods to the application's mod list. Handle edge cases like missing manifests, nested folders, and existing mods.

## ADDED Requirements

### Requirement: Archive Extraction

The application SHALL extract mod archives to a temporary directory for processing.

#### Scenario: Extract ZIP archive

- **WHEN** the download completes successfully
- **THEN** the archive SHALL be extracted to `{app_data}/temp/ModName-1.0/`
- **AND** all files and folders SHALL be extracted preserving directory structure
- **AND** the extraction SHALL complete within 30 seconds for archives < 100 MB

#### Scenario: Handle nested ZIP structure

- **WHEN** the archive is extracted with a single root folder "ModName/" containing all mod files
- **THEN** the root folder SHALL be detected
- **AND** the mod installation SHALL use "ModName/" as the mod root
- **AND** no extra nesting SHALL occur in the final installation

#### Scenario: Handle flat ZIP structure

- **WHEN** the archive is extracted with files directly at the root (manifest.json, ModName.dll, etc.)
- **THEN** the extracted directory itself SHALL be treated as the mod root
- **AND** all files SHALL be copied as-is to the mods folder

#### Scenario: Extraction fails - corrupted archive

- **WHEN** extraction is attempted on a corrupted ZIP file
- **THEN** the extraction SHALL fail with error "Archive is corrupted"
- **AND** the download status SHALL change to "failed"
- **AND** the user SHALL be prompted to retry the download
- **AND** the temp directory SHALL be cleaned up

#### Scenario: Extraction fails - unsupported format

- **WHEN** extraction is attempted on a 7z or RAR format file
- **THEN** the extraction SHALL fail with error "Unsupported archive format: .7z (only ZIP supported)"
- **AND** the downloaded file SHALL be moved to the downloads folder
- **AND** the user SHALL be instructed to manually extract and install

### Requirement: Mod Structure Detection

The application SHALL detect the mod's root directory by locating the manifest.json file.

#### Scenario: Manifest in archive root

- **WHEN** mod structure is detected in archive containing manifest.json at root
- **THEN** "ModName-1.0/" SHALL be identified as the mod root
- **AND** the UniqueID SHALL be read from manifest.json

#### Scenario: Manifest in subfolder

- **WHEN** mod structure is detected in archive with manifest.json in a subfolder
- **THEN** "ModName-1.0/ModName/" SHALL be identified as the mod root
- **AND** only the contents of "ModName/" SHALL be installed

#### Scenario: Multiple manifests found

- **WHEN** mod structure is detected in archive with manifests in multiple locations
- **THEN** the first manifest.json found (depth-first search) SHALL be used
- **AND** a warning SHALL be logged about multiple manifests

#### Scenario: No manifest found

- **WHEN** mod structure is detected in archive with no manifest.json
- **THEN** the installation SHALL be marked as "requires manual setup"
- **AND** the user SHALL be shown a warning: "No manifest.json found - install manually"
- **AND** the downloaded file SHALL be moved to the downloads folder
- **AND** an "Open folder" button SHALL be provided

### Requirement: Metadata Extraction

The application SHALL parse the manifest.json file to extract mod metadata.

#### Scenario: Parse valid manifest

- **WHEN** the manifest is parsed with all standard fields
- **THEN** all fields SHALL be extracted correctly
- **AND** the UniqueID SHALL be "ModAuthor.ExampleMod"
- **AND** the metadata SHALL be stored in the mod list

#### Scenario: Parse minimal manifest

- **WHEN** the manifest is parsed with only required fields (Name, Version, UniqueID)
- **THEN** the required fields SHALL be extracted
- **AND** optional fields SHALL be set to defaults (Author = "Unknown", etc.)

#### Scenario: Invalid manifest JSON

- **WHEN** parsing is attempted on a manifest.json with invalid JSON syntax
- **THEN** the installation SHALL fail with error "Invalid manifest.json"
- **AND** the user SHALL be prompted to manually fix the manifest or reinstall

#### Scenario: Missing required fields

- **WHEN** parsing is attempted on a manifest.json missing the "UniqueID" field
- **THEN** the installation SHALL fail with error "Manifest missing required field: UniqueID"
- **AND** the mod SHALL not be added to the list

### Requirement: Mod Installation

The application SHALL copy mod files to the game mods folder and add the mod to the mod list.

#### Scenario: Install new mod

- **WHEN** installation is triggered for mod with UniqueID "Author.NewMod"
- **THEN** the mod SHALL be copied to "/path/to/StardewValley/Mods/Author.NewMod/"
- **AND** all files SHALL be copied preserving directory structure
- **AND** the mod SHALL be added to the mod list with "enabled" status
- **AND** a "mod-installed" event SHALL be emitted to the frontend

#### Scenario: Install mod - game path not set

- **WHEN** installation is attempted without game path configured
- **THEN** the installation SHALL fail with error "Game path not configured"
- **AND** the settings modal SHALL be opened automatically
- **AND** the download SHALL remain in "completed" status for retry after settings are configured

#### Scenario: Install mod - mods folder doesn't exist

- **WHEN** installation is triggered and the "Mods" folder does not exist
- **THEN** the application SHALL create the "Mods" folder
- **AND** the mod SHALL be installed normally

#### Scenario: Install mod with auto-install disabled

- **WHEN** a download completes with "Auto-install" disabled in settings
- **THEN** the mod SHALL NOT be automatically installed
- **AND** an "Install" button SHALL be shown in the UI
- **AND** the user can click "Install" to manually trigger installation

### Requirement: Duplicate Mod Handling

The application SHALL detect and handle cases where a mod with the same UniqueID already exists.

#### Scenario: Update existing mod

- **WHEN** installation is triggered for "Author.ExampleMod" version 1.5 with version 1.0 already installed
- **THEN** a confirmation dialog SHALL be shown: "Update Author.ExampleMod from 1.0 to 1.5?" with buttons: "Update", "Keep Both", "Cancel"
- **AND** if "Update" is clicked, the old version SHALL be replaced
- **AND** if "Keep Both" is clicked, the new version SHALL be renamed to "Author.ExampleMod-1.5"

#### Scenario: Auto-update without confirmation

- **WHEN** installation is triggered with "Auto-update mods" enabled and newer version downloaded
- **THEN** the old version SHALL be backed up to `{app_data}/backups/`
- **AND** the new version SHALL replace the old version automatically
- **AND** a notification SHALL inform the user: "Updated [ModName] to [version]"

#### Scenario: Same version reinstall

- **WHEN** installation is triggered for same version 1.0 already installed
- **THEN** a confirmation dialog SHALL be shown: "Author.ExampleMod 1.0 is already installed. Reinstall?" with buttons: "Reinstall", "Cancel"
- **AND** if "Reinstall" is clicked, the existing installation SHALL be replaced

### Requirement: Post-Installation Actions

The application SHALL perform cleanup and notification actions after successful installation.

#### Scenario: Cleanup temp files

- **WHEN** the installation completes successfully
- **THEN** the temporary extraction directory SHALL be deleted
- **AND** disk space SHALL be freed

#### Scenario: Delete archive after install

- **WHEN** the installation completes with "Delete archive after install" enabled
- **THEN** the downloaded ZIP file SHALL be deleted from the downloads folder
- **AND** the download SHALL be removed from the download manager UI

#### Scenario: Keep archive after install

- **WHEN** the installation completes with "Delete archive after install" disabled
- **THEN** the downloaded ZIP file SHALL remain in the downloads folder
- **AND** the download SHALL be marked "installed" in the UI

#### Scenario: Show success notification

- **WHEN** the installation completes for mod "Example Mod"
- **THEN** a toast notification SHALL be displayed with title "Mod Installed", message "Example Mod v1.5 is ready to use", and action "View Mod" that opens mod list and highlights the mod

### Requirement: Installation Error Handling

The application SHALL handle installation errors gracefully and provide recovery options.

#### Scenario: Insufficient permissions

- **WHEN** installation is attempted on read-only or admin-protected game mods folder
- **THEN** the installation SHALL fail with error "Permission denied"
- **AND** the user SHALL be prompted to run as administrator or fix permissions
- **AND** the download SHALL remain in "completed" status for retry

#### Scenario: Disk full during installation

- **WHEN** copying mod files to the game folder with disk full
- **THEN** the installation SHALL stop immediately
- **AND** partial files SHALL be deleted (rollback)
- **AND** the error message SHALL be "Disk full - free up space and retry"

#### Scenario: Mod folder locked by another process

- **WHEN** installation is attempted with target mod folder open in another application
- **THEN** the installation SHALL retry 3 times with 1-second delays
- **AND** if all retries fail, an error SHALL be shown: "Cannot install - folder is in use. Close other programs and retry."

## Non-Functional Requirements

### Performance
- Extraction of a 50 MB ZIP archive SHALL complete in < 10 seconds
- Copying a 100 MB mod to the game folder SHALL complete in < 30 seconds
- Manifest parsing SHALL complete in < 100ms

### Reliability
- Failed installations SHALL never leave partial/corrupted mods in the game folder
- Temp files SHALL always be cleaned up, even after crashes
- Rollback SHALL restore previous state if update fails

### Usability
- Installation progress SHALL be visible to the user
- Error messages SHALL be actionable (explain how to fix)
- Success notifications SHALL be non-intrusive but noticeable

## Dependencies

- `zip` crate for ZIP extraction
- `serde_json` for manifest parsing
- `fs_extra` for recursive file copying
- `walkdir` for finding manifest.json

## Testing Criteria

### Unit Tests
- Extract ZIP archives with various structures (nested, flat, multi-folder)
- Parse valid and invalid manifest.json files
- Detect mod root directory correctly
- Handle duplicate UniqueIDs (update vs keep both)

### Integration Tests
- Full installation flow (download → extract → install)
- Installation with game path not set
- Installation with existing mod (update scenario)
- Cleanup of temp files after success/failure

### Manual Tests
- Install real mods from Nexus Mods
- Test with mods that have unusual folder structures
- Test update flow with actual mod updates
- Verify mods appear correctly in-game after installation
- Test with very large mods (> 100 MB)
