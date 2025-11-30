mod models;
mod settings;
mod nxm_protocol;
mod download_manager;
mod mod_installer;
mod api_usage_tracker;

use models::{Mod, ModManifest};
use settings::{Settings, auto_detect_game_path, detect_smapi_path, validate_game_path, validate_smapi_path};
use nxm_protocol::NxmUrl;
use download_manager::{DownloadManager, DownloadTask};
use mod_installer::{ModInstaller, InstallResult};
use api_usage_tracker::{ApiUsageTracker, ApiUsage};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
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

    let mut mods = Vec::new();

    for entry in WalkDir::new(&mods_path)
        .min_depth(1)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok()) 
    {
        if entry.file_name() == "manifest.json" {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                let content = content.trim_start_matches('\u{feff}');
                
                if let Ok(manifest) = serde_json::from_str::<ModManifest>(content) {
                    let path = entry.path().parent().unwrap().to_string_lossy().to_string();
                    let id = uuid::Uuid::new_v4().to_string();
                    
                    let folder_name = entry.path().parent().unwrap().file_name().unwrap().to_string_lossy();
                    let is_enabled = !folder_name.starts_with(".");

                    mods.push(Mod {
                        id,
                        name: manifest.name,
                        author: manifest.author,
                        version: manifest.version,
                        unique_id: manifest.unique_id,
                        description: manifest.description,
                        path,
                        is_enabled,
                    });
                }
            }
        }
    }

    Ok(mods)
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
        .install_from_archive(&PathBuf::from(file_path), &game_path, &settings)
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

                    match installer.install_from_archive(&file_path, &game_path, &settings).await {
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
            open_mod_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
