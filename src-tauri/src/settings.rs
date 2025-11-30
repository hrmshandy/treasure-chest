use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    #[serde(rename = "gamePath")]
    pub game_path: String,
    #[serde(rename = "smapiPath")]
    pub smapi_path: String,
    #[serde(rename = "nexusAuthCookie")]
    pub nexus_auth_cookie: String,
    #[serde(rename = "nexusApiKey")]
    pub nexus_api_key: String,
    pub theme: Theme,
    pub language: Language,
    #[serde(rename = "modGroups")]
    pub mod_groups: ModGroups,
    #[serde(rename = "autoInstall")]
    pub auto_install: bool,
    #[serde(rename = "confirmBeforeInstall")]
    pub confirm_before_install: bool,
    #[serde(rename = "deleteAfterInstall")]
    pub delete_after_install: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Theme {
    System,
    Dark,
    Light,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Language {
    English,
    #[serde(rename = "Bahasa Indonesia")]
    BahasaIndonesia,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ModGroups {
    None,
    Folder,
    Pack,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            game_path: String::new(),
            smapi_path: String::new(),
            nexus_auth_cookie: String::new(),
            nexus_api_key: String::new(),
            theme: Theme::System,
            language: Language::English,
            mod_groups: ModGroups::Folder,
            auto_install: true,
            confirm_before_install: false,
            delete_after_install: false,
        }
    }
}

impl Settings {
    /// Get the settings file path in the Tauri app data directory
    pub fn get_settings_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data directory: {}", e))?;

        // Ensure directory exists
        fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create app data directory: {}", e))?;

        Ok(app_data_dir.join("settings.json"))
    }

    /// Load settings from disk, returns default if file doesn't exist
    pub fn load(app_handle: &tauri::AppHandle) -> Result<Settings, String> {
        let settings_path = Self::get_settings_path(app_handle)?;

        if !settings_path.exists() {
            return Ok(Settings::default());
        }

        let contents = fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;

        serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse settings: {}", e))
    }

    /// Save settings to disk
    pub fn save(&self, app_handle: &tauri::AppHandle) -> Result<(), String> {
        let settings_path = Self::get_settings_path(app_handle)?;

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(&settings_path, json)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;

        Ok(())
    }
}

/// Auto-detect Stardew Valley game path from Steam installation
/// Returns the first valid path found, or None if not found
pub fn auto_detect_game_path() -> Option<PathBuf> {
    let steam_paths = get_steam_paths();

    for path in steam_paths {
        if validate_game_path(&path) {
            return Some(path);
        }
    }

    None
}

/// Get platform-specific Steam installation paths
fn get_steam_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "windows")]
    {
        // Windows Steam paths
        paths.push(PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps\common\Stardew Valley"));
        paths.push(PathBuf::from(r"C:\Program Files\Steam\steamapps\common\Stardew Valley"));
    }

    #[cfg(target_os = "linux")]
    {
        // Linux Steam paths
        if let Some(home) = std::env::var_os("HOME") {
            let home_path = PathBuf::from(home);

            paths.push(home_path.join(".local/share/Steam/steamapps/common/Stardew Valley"));
            paths.push(home_path.join(".steam/steam/steamapps/common/Stardew Valley"));
            // Flatpak Steam
            paths.push(home_path.join(".var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/common/Stardew Valley"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS Steam path
        if let Some(home) = std::env::var_os("HOME") {
            let home_path = PathBuf::from(home);
            paths.push(home_path.join("Library/Application Support/Steam/steamapps/common/Stardew Valley"));
        }
    }

    paths
}

/// Validate that a path is a valid Stardew Valley installation
pub fn validate_game_path(path: &Path) -> bool {
    if !path.exists() || !path.is_dir() {
        return false;
    }

    // Check for game files
    #[cfg(target_os = "windows")]
    {
        path.join("StardewValley.exe").exists() || path.join("Stardew Valley.deps.json").exists()
    }

    #[cfg(not(target_os = "windows"))]
    {
        path.join("Stardew Valley").exists() || path.join("Stardew Valley.deps.json").exists()
    }
}

/// Auto-detect SMAPI path from game path
pub fn detect_smapi_path(game_path: &Path) -> Option<PathBuf> {
    if !game_path.exists() {
        return None;
    }

    #[cfg(target_os = "windows")]
    let smapi_name = "StardewModdingAPI.exe";

    #[cfg(target_os = "macos")]
    let smapi_path = game_path.join("Contents/MacOS/StardewModdingAPI");

    #[cfg(target_os = "linux")]
    let smapi_name = "StardewModdingAPI";

    // For macOS, check special path
    #[cfg(target_os = "macos")]
    {
        if smapi_path.exists() {
            return Some(smapi_path);
        }
    }

    // For Windows and Linux, check game directory
    #[cfg(not(target_os = "macos"))]
    {
        let smapi_path = game_path.join(smapi_name);
        if smapi_path.exists() {
            return Some(smapi_path);
        }
    }

    None
}

/// Validate that SMAPI path exists and is executable
pub fn validate_smapi_path(path: &Path) -> bool {
    path.exists() && path.is_file()
}
