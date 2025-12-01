# Mod Updates

## ADDED Requirements

### Requirement: Check for Mod Updates

The system MUST check installed mods against Nexus Mods to detect available updates.

#### Scenario: Check single mod for updates
- **Given** a mod installed from Nexus with `nexus_mod_id: 5098` and version `1.14.1`
- **And** the latest version on Nexus is `1.16.0`
- **When** the user clicks the update check button
- **Then** the system queries the Nexus API for latest version
- **And** compares `1.14.1` < `1.16.0`
- **And** the mod status changes to `update-available`
- **And** an update indicator appears

#### Scenario: Mod is up to date
- **Given** a mod with version `1.16.0`
- **And** the latest version on Nexus is also `1.16.0`
- **When** checking for updates
- **Then** the mod status remains `working`
- **And** no update indicator shows

#### Scenario: Batch check all mods
- **Given** multiple mods installed from Nexus
- **When** the user clicks "Check All Updates"
- **Then** the system checks each mod concurrently
- **And** updates the status for all mods
- **And** shows a summary (e.g., "3 updates available")

#### Scenario: Mod not from Nexus
- **Given** a manually installed mod with no `.nexus_meta`
- **When** checking for updates
- **Then** the check is skipped
- **And** no status change occurs

### Requirement: Install Mod Updates

The system MUST allow users to install available updates.

#### Scenario: Install single update
- **Given** a mod with status `update-available`
- **When** the user clicks the update button
- **Then** the system downloads the latest version
- **And** backs up the current version
- **And** installs the new version
- **And** the mod status changes to `working`
- **And** a success notification appears

#### Scenario: Update fails
- **Given** a mod with an available update
- **When** the download or installation fails
- **Then** an error notification appears
- **And** the current version remains installed
- **And** the status returns to `update-available`

## MODIFIED Requirements

### Requirement: Mod Scanning

The system MUST read Nexus metadata when scanning mods.

#### Scenario: Load Nexus metadata
- **Given** a mod folder with `.nexus_meta` file
- **When** scanning mods
- **Then** the `nexus_mod_id` and `nexus_file_id` are loaded
- **And** stored in the mod's metadata
