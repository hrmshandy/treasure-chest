# Change: Add Settings Management with Auto-Detection

## Why
The application currently hardcodes game paths and lacks configuration for Nexus Mods integration. Users need a persistent settings system that auto-detects Stardew Valley installation paths and provides manual configuration options when auto-detection fails. Additionally, Nexus Mods API credentials are required for future mod browsing and downloading features.

## What Changes
- Add settings persistence using Tauri's file system
- Implement auto-detection for Stardew Valley game path and SMAPI path from Steam installations, adapting to the user's operating system (Windows, Linux, macOS)
- Create settings UI modal with:
  - Game Path field (auto-populated, with manual override)
  - SMAPI Path field (auto-populated, with manual override)
  - Nexus Auth Cookie field (new)
  - Nexus API Key field (new)
  - Theme selector (existing functionality)
  - Language selector (limited to English and Bahasa Indonesia)
  - Mod Groups preference (existing functionality)
- Add Tauri backend commands for:
  - Auto-detecting game installation paths
  - Reading/writing settings to config file
  - Validating paths exist
- Display alert/notification when auto-detection fails
- Settings stored in plain text in a JSON config file

## Impact
- **Affected specs**:
  - `app-settings` (new capability)
  - `user-preferences` (new capability)
- **Affected code**:
  - `src/App.tsx` - Remove hardcoded game path, load from settings
  - `src/components/layout/Header.tsx` - Wire up settings modal
  - `src-tauri/src/lib.rs` - Add settings commands
  - `src-tauri/src/models.rs` - Add Settings struct
- **New files**:
  - `src/components/features/settings/SettingsModal.tsx` - Settings UI component
  - `src-tauri/src/settings.rs` - Settings management module
- **Dependencies**: Uses existing `@tauri-apps/plugin-dialog` for file picker
