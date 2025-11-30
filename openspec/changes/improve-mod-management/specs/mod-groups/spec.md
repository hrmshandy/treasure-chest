# Specification: Mod Groups

## Overview

This specification defines how the mod manager groups related mods that were installed together from the same archive, enabling unified management operations across all members of a group.

## ADDED Requirements

### Requirement: Group ID Assignment

The system SHALL assign a unique group ID to all mods installed from the same archive when multiple mod folders are detected.

#### Scenario: Multi-folder archive installation
- **GIVEN** a user downloads an archive containing 3 mod folders
- **AND** all folders contain valid manifest.json files
- **WHEN** the installation process extracts and installs the mods
- **THEN** all 3 mods SHALL receive the same group_id (UUID)
- **AND** the group_id SHALL be set before the mods are added to the mod list

#### Scenario: Single-folder archive installation
- **GIVEN** a user downloads an archive containing 1 mod folder
- **WHEN** the installation process extracts and installs the mod
- **THEN** the mod SHALL have group_id set to None
- **AND** it SHALL appear as a standalone mod in the list

#### Scenario: Manual mod installation
- **GIVEN** a user manually copies mod folders to the Mods directory
- **WHEN** the file watcher detects the new folders
- **THEN** each mod SHALL have group_id set to None
- **AND** they SHALL appear as separate standalone mods

### Requirement: Group Metadata Storage

The system SHALL store group relationship metadata in each mod's data structure.

#### Scenario: Mod data structure
- **GIVEN** a mod is part of a group
- **WHEN** the mod data is serialized
- **THEN** it SHALL include a group_id field containing the UUID
- **AND** it SHALL include an install_source field ("nexus", "manual", etc.)
- **AND** it MAY include a download_id field linking to the download task

#### Scenario: Backwards compatibility
- **GIVEN** an existing mod installed before group tracking was implemented
- **WHEN** the mod data is loaded
- **THEN** the group_id field SHALL be None
- **AND** the mod SHALL function normally as a standalone mod
- **AND** no migration or data conversion is required

### Requirement: Group Display

The system SHALL display grouped mods as a collapsible visual unit in the mod list UI.

#### Scenario: Group rendering
- **GIVEN** 3 mods share the same group_id
- **WHEN** the mod list is rendered
- **THEN** the UI SHALL show a parent row representing the group
- **AND** the parent row SHALL display the group name (from first mod's name or archive name)
- **AND** the parent row SHALL show the count of mods in the group
- **AND** child rows SHALL be visually indented or connected to show hierarchy

#### Scenario: Group expansion and collapse
- **GIVEN** a group is displayed in the mod list
- **WHEN** the user clicks the expand/collapse icon
- **THEN** the group SHALL toggle between showing and hiding child mods
- **AND** the expansion state SHALL persist during the session
- **AND** the icon SHALL change to indicate current state (▶ collapsed, ▼ expanded)

#### Scenario: Empty mod list
- **GIVEN** no mods are installed
- **WHEN** the mod list is rendered
- **THEN** no groups SHALL be displayed
- **AND** an empty state message SHALL be shown

### Requirement: Group Operations

The system SHALL allow users to perform operations on all mods in a group simultaneously.

#### Scenario: Group toggle
- **GIVEN** a group contains 3 enabled mods
- **WHEN** the user clicks the toggle switch on the parent row
- **THEN** all 3 mods SHALL be disabled atomically
- **AND** if any mod fails to disable, all changes SHALL be rolled back
- **AND** the parent toggle switch SHALL reflect the group's state

#### Scenario: Group deletion
- **GIVEN** a group contains 3 mods
- **WHEN** the user clicks delete on the parent row
- **THEN** a confirmation dialog SHALL appear: "Delete all 3 mods from [GroupName]?"
- **AND** upon confirmation, all 3 mod folders SHALL be deleted
- **AND** if any deletion fails, the operation SHALL be aborted and previous deletions restored if possible

#### Scenario: Individual mod operation in group
- **GIVEN** a mod is part of a 3-mod group
- **WHEN** the user clicks the toggle switch on the child row
- **THEN** only that specific mod SHALL be affected
- **AND** a warning SHALL be shown: "This mod is part of [GroupName]. Disable just this mod?"
- **AND** an "Ungroup" option SHALL be offered

### Requirement: Group State Consistency

The system SHALL maintain consistent state across all mods in a group.

#### Scenario: Partial state detection
- **GIVEN** a group where 2 of 3 mods are enabled and 1 is disabled
- **WHEN** the mod list is rendered
- **THEN** the parent toggle switch SHALL display an indeterminate state
- **AND** hovering SHALL show a tooltip: "2 of 3 enabled"

#### Scenario: Group integrity validation
- **GIVEN** a group_id references multiple mods
- **WHEN** the mod list is loaded
- **THEN** all mods with that group_id SHALL be verified to exist
- **AND** if any are missing, the group SHALL still display remaining mods
- **AND** a warning SHALL be logged about incomplete group

### Requirement: Install Source Tracking

The system SHALL track the installation source for each mod.

#### Scenario: Nexus download installation
- **GIVEN** a mod is installed via NXM protocol download
- **WHEN** the installation completes
- **THEN** install_source SHALL be set to "nexus"
- **AND** download_id SHALL be set to the download task's UUID

#### Scenario: Manual file installation
- **GIVEN** a user manually copies a mod folder
- **WHEN** the file watcher detects the new folder
- **THEN** install_source SHALL be set to "manual"
- **AND** download_id SHALL be None

#### Scenario: Source display
- **GIVEN** a mod has install_source set
- **WHEN** the mod details are displayed
- **THEN** the UI SHALL show the installation source
- **AND** for "nexus" source, a "View on Nexus Mods" link SHALL be provided
