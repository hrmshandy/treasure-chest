# Capability: Application Settings

## ADDED Requirements

### Requirement: Settings Persistence
The system SHALL persist application settings to disk in JSON format, allowing configuration to survive app restarts.

#### Scenario: Settings saved successfully
- **WHEN** user modifies settings and clicks "Save"
- **THEN** settings are written to a config file in the Tauri app data directory
- **AND** the settings remain available after app restart

#### Scenario: Load settings on startup
- **WHEN** application launches
- **THEN** settings are loaded from the config file if it exists
- **AND** default settings are used if config file does not exist

### Requirement: Game Path Auto-Detection
The system SHALL automatically detect Stardew Valley installation path from Steam based on the user's operating system.

#### Scenario: Auto-detect on Windows
- **WHEN** application starts on Windows
- **THEN** check `C:\Program Files (x86)\Steam\steamapps\common\Stardew Valley`
- **AND** check `C:\Program Files\Steam\steamapps\common\Stardew Valley`
- **AND** use the first valid path found

#### Scenario: Auto-detect on Linux
- **WHEN** application starts on Linux
- **THEN** check `~/.local/share/Steam/steamapps/common/Stardew Valley`
- **AND** check `~/.steam/steam/steamapps/common/Stardew Valley`
- **AND** check `~/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/common/Stardew Valley` (Flatpak)
- **AND** use the first valid path found

#### Scenario: Auto-detect on macOS
- **WHEN** application starts on macOS
- **THEN** check `~/Library/Application Support/Steam/steamapps/common/Stardew Valley`
- **AND** use the path if valid

#### Scenario: Auto-detection fails
- **WHEN** no valid Steam installation path is found
- **THEN** display an alert notifying the user to set the path manually
- **AND** open the settings modal with focus on the Game Path field

### Requirement: SMAPI Path Auto-Detection
The system SHALL automatically detect SMAPI executable path based on the game installation path and platform.

#### Scenario: SMAPI detection on Windows
- **WHEN** game path is detected on Windows
- **THEN** check for `StardewModdingAPI.exe` in the game directory
- **AND** use that path if the file exists

#### Scenario: SMAPI detection on Linux
- **WHEN** game path is detected on Linux
- **THEN** check for `StardewModdingAPI` in the game directory
- **AND** use that path if the file exists

#### Scenario: SMAPI detection on macOS
- **WHEN** game path is detected on macOS
- **THEN** check for `StardewModdingAPI` in `Contents/MacOS/` within the game bundle
- **AND** use that path if the file exists

#### Scenario: SMAPI not found
- **WHEN** SMAPI executable is not found in expected location
- **THEN** leave the SMAPI path field empty
- **AND** display a warning indicator in the settings UI

### Requirement: Manual Path Override
The system SHALL allow users to manually select game and SMAPI paths using a native file picker dialog.

#### Scenario: User selects game path manually
- **WHEN** user clicks the folder icon next to Game Path field
- **THEN** open native directory picker dialog
- **AND** update the field with selected directory path
- **AND** re-validate the path

#### Scenario: User selects SMAPI path manually
- **WHEN** user clicks the folder icon next to SMAPI Path field
- **THEN** open native file picker dialog filtered to executables
- **AND** update the field with selected file path
- **AND** verify the file exists

### Requirement: Path Validation
The system SHALL validate that configured paths exist and contain expected files before saving.

#### Scenario: Valid game path
- **WHEN** user sets a game path
- **THEN** verify the directory exists
- **AND** verify a `Mods` subdirectory exists or can be created
- **AND** mark the path as valid with a visual indicator

#### Scenario: Invalid game path
- **WHEN** user sets a game path that doesn't exist
- **THEN** display an error message
- **AND** prevent saving until path is corrected

### Requirement: Nexus Mods Credentials Storage
The system SHALL store Nexus Mods authentication credentials in plain text for API access.

#### Scenario: Save Nexus credentials
- **WHEN** user enters Nexus Auth Cookie and API Key
- **THEN** store both values in the config file
- **AND** make them available to mod installation features

#### Scenario: Optional credentials
- **WHEN** user leaves Nexus credentials empty
- **THEN** allow saving settings without them
- **AND** disable Nexus-dependent features in the UI

### Requirement: Settings Configuration File Format
The system SHALL use a structured JSON format for the settings configuration file.

#### Scenario: Settings file structure
- **WHEN** settings are saved
- **THEN** the config file contains:
  - `gamePath`: string (absolute path to game directory)
  - `smapiPath`: string (absolute path to SMAPI executable)
  - `nexusAuthCookie`: string (optional, Nexus Mods cookie)
  - `nexusApiKey`: string (optional, Nexus Mods API key)
  - `theme`: string (enum: "System", "Dark", "Light")
  - `language`: string (enum: "English", "Bahasa Indonesia")
  - `modGroups`: string (enum: "None", "Folder", "Pack")
