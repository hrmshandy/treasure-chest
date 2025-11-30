# Implementation Tasks

## 1. Backend - Settings Management Module
- [x] 1.1 Create `src-tauri/src/settings.rs` module
- [x] 1.2 Define `Settings` struct with all configuration fields
- [x] 1.3 Implement settings file read/write functions using Tauri app data directory
- [x] 1.4 Add default settings constructor
- [x] 1.5 Implement Steam game path detection for Windows, Linux, and macOS
- [x] 1.6 Implement SMAPI path detection based on game path
- [x] 1.7 Add path validation functions

## 2. Backend - Tauri Commands
- [x] 2.1 Add `load_settings` command to read settings from disk
- [x] 2.2 Add `save_settings` command to persist settings to disk
- [x] 2.3 Add `auto_detect_paths` command for platform-specific detection
- [x] 2.4 Add `validate_game_path_cmd` command to verify path validity
- [x] 2.5 Add `validate_smapi_path_cmd` command to verify SMAPI executable exists
- [x] 2.6 Register all new commands in `lib.rs`

## 3. Backend - Data Models
- [x] 3.1 Settings struct defined in `settings.rs` module
- [x] 3.2 Add Serde serialization/deserialization for Settings
- [x] 3.3 Define enums for Theme, Language, and ModGroups

## 4. Frontend - Settings Modal Component
- [x] 4.1 Create `src/components/features/settings/SettingsModal.tsx`
- [x] 4.2 Port UI structure from reference HTML (lines 594-756)
- [x] 4.3 Add Game Path field with folder picker button
- [x] 4.4 Add SMAPI Path field with file picker button
- [x] 4.5 Add Nexus Auth Cookie input field
- [x] 4.6 Add Nexus API Key input field (with password masking)
- [x] 4.7 Update Language selector to only show English and Bahasa Indonesia
- [x] 4.8 Implement form state management with React hooks
- [x] 4.9 Add validation UI feedback (error messages, success indicators)

## 5. Frontend - Settings Integration
- [x] 5.1 Create `src/types/settings.ts` TypeScript interface
- [x] 5.2 Update `src/App.tsx` to load settings on mount
- [x] 5.3 Replace hardcoded `gamePath` with settings value
- [x] 5.4 Add settings state management to App component
- [x] 5.5 Wire up Header component to open SettingsModal
- [x] 5.6 Add alert notification for auto-detection failure

## 6. Frontend - File Picker Integration
- [x] 6.1 Import Tauri dialog plugin functions (`open` for directories and files)
- [x] 6.2 Implement directory picker for Game Path field
- [x] 6.3 Implement file picker for SMAPI Path field
- [x] 6.4 Handle picker cancellation gracefully

## 7. Settings Initialization & Auto-Detection
- [x] 7.1 Call `auto_detect_paths` on first app launch
- [x] 7.2 Display alert if detection fails and no settings exist
- [x] 7.3 Auto-open settings modal if paths are not configured
- [x] 7.4 Ensure settings persist and load correctly on subsequent launches

## 8. Testing & Validation
- [X] 8.1 Test auto-detection on Windows (Steam)
- [X] 8.2 Test auto-detection on Linux (Steam and Flatpak)
- [X] 8.3 Test auto-detection on macOS (Steam)
- [X] 8.4 Test manual path selection with dialog picker
- [X] 8.5 Test settings persistence across app restarts
- [X] 8.6 Test invalid path validation and error messages
- [X] 8.7 Test Nexus credentials storage and retrieval
- [X] 8.8 Test theme, language, and mod groups preferences

## 9. Documentation
- [X] 9.1 Document settings file location for each platform
- [X] 9.2 Document Steam installation paths checked per OS
- [X] 9.3 Add comments explaining auto-detection logic
- [X] 9.4 Note that non-Steam installations require manual configuration
