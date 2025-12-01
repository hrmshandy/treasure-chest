mod models;
mod settings;
mod nxm_protocol;
mod download_manager;
mod mod_installer;
mod api_usage_tracker;

use models::Mod;
use settings::{Settings, auto_detect_game_path, detect_smapi_path, validate_game_path, validate_smapi_path};
use nxm_protocol::NxmUrl;
use download_manager::{DownloadManager, DownloadTask};
use mod_installer::{ModInstaller, InstallResult};
use api_usage_tracker::{ApiUsageTracker, ApiUsage};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tauri::{Emitter, Listener, Manager};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn install_mod(url: String, game_path: String) -> Result<String, String> {
    println!("Installing mod from: {}", url);
    
    let bytes = if url.starts_with("http") {
        // 1. Download the file
        let response = reqwest::get(&url)
            .await
            .map_err(|e| format!("Failed to download file: {}", e))?;
        
        response.bytes()
            .await
            .map_err(|e| format!("Failed to read bytes: {}", e))?
            .to_vec()
    } else {
        // 1. Read from local file
        fs::read(&url)
            .map_err(|e| format!("Failed to read local file: {}", e))?
    };

    // 2. Determine extraction path (Mods folder)
    let mods_path = Path::new(&game_path).join("Mods");
    if !mods_path.exists() {
        fs::create_dir_all(&mods_path).map_err(|e| format!("Failed to create Mods directory: {}", e))?;
    }

    // 3. Extract (assuming zip for now)
    let reader = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| format!("Failed to read zip archive: {}", e))?;

    archive.extract(&mods_path)
        .map_err(|e| format!("Failed to extract zip: {}", e))?;

    Ok("Mod installed successfully".to_string())
}

#[tauri::command]
fn scan_mods(game_path: String) -> Result<Vec<Mod>, String> {
    let mods_path = Path::new(&game_path).join("Mods");
    if !mods_path.exists() {
        return Err("Mods folder not found".to_string());
    }

    Ok(mod_installer::scan_mods(Path::new(&game_path)))
}

// Settings commands
#[tauri::command]
fn load_settings(app_handle: tauri::AppHandle) -> Result<Settings, String> {
    Settings::load(&app_handle)
}

#[tauri::command]
fn save_settings(app_handle: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    settings.save(&app_handle)
}

#[tauri::command]
fn auto_detect_paths() -> Result<(Option<String>, Option<String>), String> {
    let game_path = auto_detect_game_path();

    let (game_path_str, smapi_path_str) = match game_path {
        Some(ref path) => {
            let smapi = detect_smapi_path(path);
            (
                Some(path.to_string_lossy().to_string()),
                smapi.map(|p| p.to_string_lossy().to_string())
            )
        }
        None => (None, None)
    };

    Ok((game_path_str, smapi_path_str))
}

#[tauri::command]
fn validate_game_path_cmd(path: String) -> bool {
    validate_game_path(&PathBuf::from(path))
}

#[tauri::command]
fn validate_smapi_path_cmd(path: String) -> bool {
    validate_smapi_path(&PathBuf::from(path))
}

// Download manager commands
#[tauri::command]
async fn get_downloads(app_handle: tauri::AppHandle) -> Result<Vec<DownloadTask>, String> {
    let manager = app_handle.state::<DownloadManager>();
    Ok(manager.get_queue_state().await)
}

// API usage tracking command
#[tauri::command]
async fn get_api_usage(app_handle: tauri::AppHandle) -> Result<ApiUsage, String> {
    let tracker = app_handle.state::<ApiUsageTracker>();
    Ok(tracker.get_usage().await)
}

#[tauri::command]
async fn cancel_download(app_handle: tauri::AppHandle, download_id: String) -> Result<(), String> {
    let manager = app_handle.state::<DownloadManager>();
    manager.cancel_download(&download_id).await
}

#[tauri::command]
async fn clear_completed_downloads(app_handle: tauri::AppHandle) -> Result<(), String> {
    let manager = app_handle.state::<DownloadManager>();
    manager.clear_completed().await
}

// Mod installer commands
#[tauri::command]
async fn install_mod_from_file(
    app_handle: tauri::AppHandle,
    file_path: String,
) -> Result<InstallResult, String> {
    // Load settings to get game path
    let settings = Settings::load(&app_handle).map_err(|e| format!("Failed to load settings: {}", e))?;

    if settings.game_path.is_empty() {
        return Err("Game path not configured. Please set it in settings.".to_string());
    }

    let game_path = PathBuf::from(&settings.game_path);
    let app_data_dir = app_handle.path().app_data_dir().unwrap();
    let temp_dir = app_data_dir.join("temp");

    let installer = ModInstaller::new(app_handle.clone(), temp_dir);

    installer
        .install_from_archive(&PathBuf::from(file_path), &game_path, &settings, None, None)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn test_nxm_url(app_handle: tauri::AppHandle, url: String) -> Result<(), String> {
    println!("🧪 Manual NXM test triggered from frontend");
    println!("   URL: {}", url);

    // Parse the NXM URL
    let nxm_url = NxmUrl::parse(&url).map_err(|e| format!("Failed to parse NXM URL: {}", e))?;

    // Validate
    nxm_url.validate().map_err(|e| format!("NXM URL validation failed: {}", e))?;

    println!("✅ NXM URL parsed and validated successfully");
    println!("   Game: {}", nxm_url.game);
    println!("   Mod ID: {}", nxm_url.mod_id);
    println!("   File ID: {}", nxm_url.file_id);

    // Emit event
    let _ = app_handle.emit("nxm-url-received", &nxm_url);

    // Add to download queue
    let manager = app_handle.state::<DownloadManager>();
    let download_id = manager.add_to_queue(nxm_url.clone()).await
        .map_err(|e| format!("Failed to queue download: {}", e))?;

    println!("📥 Download queued: {}", download_id);

    Ok(())
}

#[tauri::command]
async fn open_downloads_folder(app_handle: tauri::AppHandle) -> Result<(), String> {
    let app_data_dir = app_handle.path().app_data_dir().unwrap();
    let download_dir = app_data_dir.join("downloads").join("nexus");
    
    if !download_dir.exists() {
        fs::create_dir_all(&download_dir).map_err(|e| e.to_string())?;
    }

    open_folder(&download_dir)
}

#[tauri::command]
async fn open_mod_folder(path: String) -> Result<(), String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err("Mod folder does not exist".to_string());
    }
    open_folder(&path)
}

#[tauri::command]
async fn open_game_mods_folder(game_path: String) -> Result<(), String> {
    let mods_path = Path::new(&game_path).join("Mods");
    if !mods_path.exists() {
        fs::create_dir_all(&mods_path).map_err(|e| e.to_string())?;
    }
    open_folder(&mods_path)
}

#[tauri::command]
async fn toggle_mod_enabled(mod_path: String, enabled: bool) -> Result<String, String> {
    let path = PathBuf::from(&mod_path);
    if !path.exists() {
        return Err("Mod path does not exist".to_string());
    }

    let parent = path.parent().ok_or("Invalid mod path")?;
    let file_name = path.file_name().ok_or("Invalid mod path")?.to_string_lossy().to_string();

    let new_name = if enabled {
        // Enable: Remove .disabled suffix if present
        if file_name.ends_with(".disabled") {
            file_name.trim_end_matches(".disabled").to_string()
        } else {
            return Ok(mod_path); // Already enabled
        }
    } else {
        // Disable: Add .disabled suffix if not present
        if !file_name.ends_with(".disabled") {
            format!("{}.disabled", file_name)
        } else {
            return Ok(mod_path); // Already disabled
        }
    };

    let new_path = parent.join(&new_name);
    fs::rename(&path, &new_path).map_err(|e| e.to_string())?;

    Ok(new_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn delete_mod(_app_handle: tauri::AppHandle, mod_path: String) -> Result<(), String> {
    let path = PathBuf::from(&mod_path);
    if !path.exists() {
        return Err("Mod path does not exist".to_string());
    }

    // Use the force_remove_dir_all method through a helper
    fn force_remove(path: &Path) -> std::io::Result<()> {
        if !path.exists() {
            return Ok(());
        }

        // Try normal remove first
        if fs::remove_dir_all(path).is_ok() {
            return Ok(());
        }

        println!("   ⚠ Normal remove failed, attempting to force permissions on: {}", path.display());

        // Make everything writable
        use walkdir::WalkDir;
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
             #[cfg(unix)]
             {
                 use std::os::unix::fs::PermissionsExt;
                 let p = entry.path();
                 if let Ok(metadata) = p.metadata() {
                     let mut perms = metadata.permissions();
                     let mode = perms.mode() | 0o700; // u+rwx
                     perms.set_mode(mode);
                     let _ = fs::set_permissions(p, perms);
                 }
             }
        }

        fs::remove_dir_all(path)
    }

    force_remove(&path).map_err(|e| format!("Failed to delete mod: {}", e))?;
    
    println!("Successfully deleted mod at: {}", path.display());
    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct UpdateInfo {
    has_update: bool,
    current_version: String,
    latest_version: Option<String>,
    latest_file_id: Option<u32>,
}

#[tauri::command]
async fn check_mod_updates(
    app_handle: tauri::AppHandle,
    _mod_path: String,
    current_version: String,
    nexus_mod_id: u32,
) -> Result<UpdateInfo, String> {
    println!("Checking updates for mod {} (version {})", nexus_mod_id, current_version);

    // Query Nexus API for mod information
    let api_tracker = app_handle.state::<ApiUsageTracker>();
    let settings = Settings::load(&app_handle).map_err(|e| e.to_string())?;
    
    let api_key = settings.nexus_api_key;
    if api_key.is_empty() {
        return Err("Nexus API key not configured".to_string());
    }

    let url = format!("https://api.nexusmods.com/v1/games/stardewvalley/mods/{}.json", nexus_mod_id);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("apikey", &api_key)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch mod info: {}", e))?;

    // Update API usage
    api_tracker.inner().update_from_headers(response.headers()).await;

    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()));
    }

    let mod_info: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Get the latest file version
    // The API returns mod info, we need to find the latest main file
    let latest_version = mod_info
        .get("version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let latest_file_id = mod_info
        .get("latest_file_id")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    // Compare versions using semver if possible
    let has_update = if let Some(ref latest) = latest_version {
        match (semver::Version::parse(&current_version), semver::Version::parse(latest)) {
            (Ok(current), Ok(latest)) => latest > current,
            _ => {
                // Fallback to string comparison if semver parsing fails
                latest != &current_version
            }
        }
    } else {
        false
    };

    println!("Update check result: has_update={}, latest_version={:?}", has_update, latest_version);

    Ok(UpdateInfo {
        has_update,
        current_version,
        latest_version,
        latest_file_id,
    })
}

fn open_folder(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            println!("\n╔══════════════════════════════════════════════════╗");
            println!("║  🔄 SECOND INSTANCE DETECTED!                   ║");
            println!("╚══════════════════════════════════════════════════╝");
            println!("📦 Received {} arguments from second instance:", args.len());

            for (i, arg) in args.iter().enumerate() {
                println!("   [{}]: {}", i, arg);

                // Check if it's an NXM URL
                if arg.starts_with("nxm://") {
                    println!("   ⚡ NXM URL detected in second instance!");

                    // Parse the URL
                    if let Ok(nxm_url) = crate::nxm_protocol::NxmUrl::parse(arg) {
                        if let Err(e) = nxm_url.validate() {
                            eprintln!("   ❌ NXM URL validation failed: {}", e);
                            let _ = app.emit("nxm-error", e.to_string());
                            continue;
                        }

                        println!("   ✅ NXM URL parsed: mod_id={}, file_id={}", nxm_url.mod_id, nxm_url.file_id);

                        // Emit event to frontend
                        let _ = app.emit("nxm-url-received", &nxm_url);
                        println!("   📡 Emitted nxm-url-received event");

                        // Queue the download
                        let handle = app.clone();
                        let url = nxm_url.clone();
                        tauri::async_runtime::spawn(async move {
                            let manager = handle.state::<crate::download_manager::DownloadManager>();
                            match manager.add_to_queue(url.clone()).await {
                                Ok(download_id) => {
                                    println!("   📥 Download queued: {} (mod_id={}, file_id={})",
                                        download_id, url.mod_id, url.file_id);
                                }
                                Err(e) => {
                                    eprintln!("   ❌ Failed to queue download: {}", e);
                                    let _ = handle.emit("nxm-error", format!("Failed to queue download: {}", e));
                                }
                            }
                        });
                    } else {
                        eprintln!("   ❌ Failed to parse NXM URL");
                    }
                }
            }

            // Focus the existing window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
                println!("   🪟 Focused existing window");
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            // Initialize API usage tracker
            let api_tracker = ApiUsageTracker::new();
            app.manage(api_tracker);

            // Initialize download manager
            let app_data_dir = app.path().app_data_dir().unwrap();
            let download_dir = app_data_dir.join("downloads").join("nexus");
            let download_manager = DownloadManager::new(app.handle().clone(), download_dir.clone(), 1);
            app.manage(download_manager);

            // Listen for download completion and trigger auto-installation
            let app_handle = app.handle().clone();
            let download_dir_clone = download_dir.clone();
            app.listen("download-completed", move |event| {
                let download_id = match event.payload().parse::<String>() {
                    Ok(id) => id.trim_matches('"').to_string(),
                    Err(_) => return,
                };

                println!("Download completed, triggering installation: {}", download_id);

                let handle = app_handle.clone();
                let dl_dir = download_dir_clone.clone();
                tauri::async_runtime::spawn(async move {
                    // Load settings
                    let settings = match Settings::load(&handle) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Failed to load settings for auto-install: {}", e);
                            return;
                        }
                    };

                    if settings.game_path.is_empty() {
                        eprintln!("Game path not configured, skipping auto-install");
                        return;
                    }

                    // Get downloads to find the file path
                    let manager = handle.state::<DownloadManager>();
                    let downloads = manager.get_queue_state().await;

                    let download = match downloads.iter().find(|d| d.id == download_id) {
                        Some(d) => d,
                        None => {
                            eprintln!("Download not found: {}", download_id);
                            return;
                        }
                    };

                    // Get file path
                    let file_path = match &download.file_path {
                        Some(p) => p.clone(),
                        None => dl_dir.join(&download.file_name),
                    };

                    println!("Auto-installing mod from: {}", file_path.display());

                    // Check if confirmation is required
                    if settings.confirm_before_install {
                        println!("Confirmation required for installation");
                        let _ = handle.emit("install-confirmation-needed", download_id);
                        return;
                    }

                    // Install mod
                    let temp_dir = handle.path().app_data_dir().unwrap().join("temp");
                    let installer = ModInstaller::new(handle.clone(), temp_dir);
                    let game_path = PathBuf::from(&settings.game_path);

                    let nexus_info = Some((download.nxm_url.mod_id, download.nxm_url.file_id));
                    let mod_name = download.mod_name.clone();

                    match installer.install_from_archive(&file_path, &game_path, &settings, nexus_info, mod_name).await {
                        Ok(result) => {
                            println!("Mod installed successfully: {} v{}", result.mod_name, result.version);
                        }
                        Err(e) => {
                            eprintln!("Auto-installation failed: {}", e);
                            let _ = handle.emit("mod-install-failed", e.to_string());
                        }
                    }
                });
            });

            // Register nxm:// protocol handler
            #[cfg(desktop)]
            {
                use tauri_plugin_deep_link::DeepLinkExt;

                // Register the nxm scheme
                println!("=== Registering nxm:// protocol handler ===");
                if let Err(e) = app.deep_link().register("nxm") {
                    eprintln!("❌ Failed to register nxm:// protocol: {}", e);
                } else {
                    println!("✅ nxm:// protocol registered successfully");
                }

                // Listen for deep link events
                println!("📡 Setting up deep link event listener...");
                let app_handle = app.handle().clone();

                // Handle app launch with deep link arguments
                let handle_clone = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    use std::env;
                    let args: Vec<String> = env::args().collect();
                    println!("🚀 App launched with {} arguments:", args.len());
                    for (i, arg) in args.iter().enumerate() {
                        println!("   [{}]: {}", i, arg);
                        if arg.starts_with("nxm://") {
                            println!("   ⚠️  NXM URL found in launch arguments!");
                            let _ = handle_clone.emit("debug-deep-link", arg);
                        }
                    }
                });

                app.listen("deep-link://new-url", move |event| {
                    println!("\n╔══════════════════════════════════════╗");
                    println!("║  🔗 DEEP LINK EVENT RECEIVED!       ║");
                    println!("╚══════════════════════════════════════╝");
                    println!("Raw payload: {}", event.payload());

                    // Parse payload as Vec<String>
                    let urls: Vec<String> = match serde_json::from_str(event.payload()) {
                        Ok(u) => u,
                        Err(e) => {
                            eprintln!("❌ Failed to parse deep link payload: {}", e);
                            eprintln!("   Payload was: {}", event.payload());
                            return;
                        }
                    };

                    println!("📦 Parsed {} URL(s)", urls.len());

                    for url_str in urls {
                        println!("\n🔍 Processing URL: {}", url_str);
                        let _ = app_handle.emit("debug-deep-link", &url_str);

                        // Check if it's an NXM URL
                        if !url_str.starts_with("nxm://") {
                            continue;
                        }

                        // Parse the NXM URL
                        match NxmUrl::parse(&url_str) {
                            Ok(nxm_url) => {
                                // Validate (check expiration)
                                if let Err(e) = nxm_url.validate() {
                                    eprintln!("NXM URL validation failed: {}", e);
                                    let _ = app_handle.emit("nxm-error", e.to_string());
                                    continue;
                                }

                                println!(
                                    "Parsed NXM URL: game={}, mod_id={}, file_id={}",
                                    nxm_url.game, nxm_url.mod_id, nxm_url.file_id
                                );

                                // Emit success event to frontend
                                let _ = app_handle.emit("nxm-url-received", &nxm_url);

                                // Add to download queue
                                let handle = app_handle.clone();
                                let url = nxm_url.clone();
                                tauri::async_runtime::spawn(async move {
                                    let manager = handle.state::<DownloadManager>();
                                    match manager.add_to_queue(url.clone()).await {
                                        Ok(download_id) => {
                                            println!("Download queued: {} (mod_id={}, file_id={})",
                                                download_id, url.mod_id, url.file_id);
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to queue download: {}", e);
                                            let _ = handle.emit("nxm-error", format!("Failed to queue download: {}", e));
                                        }
                                    }
                                });
                            }
                            Err(e) => {
                                eprintln!("Failed to parse NXM URL: {}", e);
                                let _ = app_handle.emit("nxm-error", e.to_string());
                            }
                        }
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            scan_mods,
            install_mod,
            load_settings,
            save_settings,
            auto_detect_paths,
            validate_game_path_cmd,
            validate_smapi_path_cmd,
            get_downloads,
            get_api_usage,
            cancel_download,
            clear_completed_downloads,
            install_mod_from_file,
            test_nxm_url,
            open_downloads_folder,
            open_downloads_folder,
            open_mod_folder,
            open_game_mods_folder,
            toggle_mod_enabled,
            delete_mod,
            delete_mod,
            check_mod_updates,
            launch_game
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn launch_game(app_handle: tauri::AppHandle) -> Result<(), String> {
    let settings = Settings::load(&app_handle).map_err(|e| e.to_string())?;
    
    if settings.smapi_path.is_empty() {
        return Err("SMAPI path not configured. Please set it in settings.".to_string());
    }

    let smapi_path = PathBuf::from(&settings.smapi_path);
    if !smapi_path.exists() {
        return Err("SMAPI executable not found at configured path".to_string());
    }

    // Determine working directory (usually parent of executable)
    let working_dir = smapi_path.parent().unwrap_or(&smapi_path);

    println!("🚀 Launching game from: {}", smapi_path.display());

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new(&smapi_path)
            .current_dir(working_dir)
            .spawn()
            .map_err(|e| format!("Failed to launch game: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&smapi_path)
            .spawn()
            .map_err(|e| format!("Failed to launch game: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new(&smapi_path)
            .current_dir(working_dir)
            .spawn()
            .map_err(|e| format!("Failed to launch game: {}", e))?;
    }

    Ok(())
}
