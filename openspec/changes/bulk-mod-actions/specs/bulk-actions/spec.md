# Bulk Mod Actions

## ADDED Requirements

### Requirement: Mod Selection

The system MUST allow users to select multiple mods.

#### Scenario: Select single mod
- **Given** a list of mods
- **When** the user clicks the checkbox on a mod row
- **Then** that mod is added to the selection

#### Scenario: Select all mods
- **Given** a list of mods
- **When** the user clicks the "Select All" checkbox in the header
- **Then** all visible mods are selected
- **And** if all were already selected, they are deselected

### Requirement: Bulk Enable/Disable

The system MUST allow enabling or disabling multiple mods at once.

#### Scenario: Bulk Enable
- **Given** multiple disabled mods are selected
- **When** the user clicks "Enable"
- **Then** all selected mods become enabled
- **And** the UI updates to reflect the new status

#### Scenario: Bulk Disable
- **Given** multiple enabled mods are selected
- **When** the user clicks "Disable"
- **Then** all selected mods become disabled
- **And** the UI updates to reflect the new status

### Requirement: Bulk Delete

The system MUST allow deleting multiple mods at once.

#### Scenario: Bulk Delete
- **Given** multiple mods are selected
- **When** the user clicks "Delete"
- **Then** a confirmation dialog appears summarizing the action (e.g., "Delete 3 mods?")
- **When** confirmed
- **Then** all selected mods are deleted from disk
- **And** removed from the list
