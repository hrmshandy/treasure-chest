# Capability: User Preferences

## ADDED Requirements

### Requirement: Settings Modal UI
The system SHALL provide a settings modal accessible from the application header that displays all configurable options.

#### Scenario: Open settings modal
- **WHEN** user clicks the settings icon in the header
- **THEN** open a modal overlay displaying settings form
- **AND** populate all fields with current settings values
- **AND** focus remains trapped within the modal

#### Scenario: Close settings modal
- **WHEN** user clicks the X button or clicks outside the modal
- **THEN** close the modal without saving changes
- **AND** restore original values

### Requirement: Game Path Configuration UI
The system SHALL display game and SMAPI path fields with auto-populated values and manual override capability.

#### Scenario: Display auto-detected paths
- **WHEN** settings modal opens
- **THEN** display auto-detected game path in read-only field
- **AND** display auto-detected SMAPI path in read-only field
- **AND** show a folder icon button for manual selection

#### Scenario: Manual path selection
- **WHEN** user clicks folder icon next to a path field
- **THEN** open Tauri dialog plugin directory/file picker
- **AND** update the field with selected path
- **AND** enable the field for editing

### Requirement: Nexus Mods Credentials UI
The system SHALL provide input fields for Nexus Auth Cookie and API Key.

#### Scenario: Display credential fields
- **WHEN** settings modal opens
- **THEN** display a "Nexus Auth Cookie" text input field
- **AND** display a "Nexus API Key" text input field
- **AND** mask the API key input for security

#### Scenario: Save credentials
- **WHEN** user enters credentials and clicks "Save"
- **THEN** store credentials in settings
- **AND** validate that credentials are non-empty if provided

### Requirement: Language Selection
The system SHALL support English and Bahasa Indonesia language options only.

#### Scenario: Display language selector
- **WHEN** settings modal opens
- **THEN** display a dropdown with two options: "English" and "Bahasa Indonesia"
- **AND** pre-select the current language setting

#### Scenario: Change language
- **WHEN** user selects a different language and saves
- **THEN** update the language setting
- **AND** apply language to UI immediately (if i18n is implemented)

### Requirement: Theme Selection
The system SHALL allow users to choose between System, Dark, and Light themes.

#### Scenario: Display theme selector
- **WHEN** settings modal opens
- **THEN** display a dropdown with three options: "System", "Dark", "Light"
- **AND** pre-select the current theme setting

#### Scenario: Change theme
- **WHEN** user selects a different theme and saves
- **THEN** update the theme setting
- **AND** apply theme to UI immediately

### Requirement: Mod Groups Preference
The system SHALL allow users to configure mod grouping behavior (None, Folder, Pack).

#### Scenario: Display mod groups selector
- **WHEN** settings modal opens
- **THEN** display three radio buttons: "None", "Folder", "Pack"
- **AND** pre-select the current mod groups setting

#### Scenario: Change mod groups
- **WHEN** user selects a different grouping option and saves
- **THEN** update the mod groups setting
- **AND** reorganize mod list display accordingly

### Requirement: Settings Validation Feedback
The system SHALL provide visual feedback for invalid or missing required settings.

#### Scenario: Show validation errors
- **WHEN** user attempts to save with invalid paths
- **THEN** display error messages next to invalid fields
- **AND** prevent saving until all errors are resolved

#### Scenario: Show success confirmation
- **WHEN** settings are saved successfully
- **THEN** display a brief success message or indicator
- **AND** close the modal automatically

### Requirement: Auto-Detection Alert
The system SHALL notify users when game path auto-detection fails on first launch.

#### Scenario: Show alert on detection failure
- **WHEN** application launches and auto-detection finds no valid paths
- **THEN** display an alert notification
- **AND** automatically open the settings modal
- **AND** highlight the Game Path field for user attention
