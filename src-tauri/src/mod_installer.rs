use crate::models::ModManifest;
use crate::settings::Settings;
use serde::Serialize;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager};
use walkdir::WalkDir;
use zip::ZipArchive;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstallResult {
    pub mod_name: String,
    pub version: String,
    pub unique_id: String,
    pub install_path: PathBuf,
}

#[derive(Debug)]
pub enum InstallError {
    ExtractionFailed(String),
    ManifestNotFound,
    InvalidManifest(String),
    InstallationFailed(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallError::ExtractionFailed(e) => write!(f, "Failed to extract archive: {}", e),
            InstallError::ManifestNotFound => write!(f, "No manifest.json found in mod archive"),
            InstallError::InvalidManifest(e) => write!(f, "Invalid manifest.json: {}", e),
            InstallError::InstallationFailed(e) => write!(f, "Installation failed: {}", e),
            InstallError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl From<std::io::Error> for InstallError {
    fn from(err: std::io::Error) -> Self {
        InstallError::IoError(err)
    }
}

impl From<walkdir::Error> for InstallError {
    fn from(err: walkdir::Error) -> Self {
        InstallError::InstallationFailed(format!("Walkdir error: {}", err))
    }
}

pub struct ModInstaller {
    app_handle: AppHandle,
    temp_dir: PathBuf,
}

impl ModInstaller {
    pub fn new(app_handle: AppHandle, temp_dir: PathBuf) -> Self {
        Self {
            app_handle,
            temp_dir,
        }
    }

    /// Install a mod from an archive file
    pub async fn install_from_archive(
        &self,
        archive_path: &Path,
        game_path: &Path,
        settings: &Settings,
    ) -> Result<InstallResult, InstallError> {
        println!("Installing mod from: {}", archive_path.display());

        // Create temp directory if it doesn't exist
        fs::create_dir_all(&self.temp_dir)?;

        // Extract archive to temp directory
        let extract_dir = self.extract_archive(archive_path).await?;

        // Detect archive structure and get mod folders to install
        let mod_folders = self.detect_mod_folders(&extract_dir)?;

        if mod_folders.is_empty() {
            return Err(InstallError::ManifestNotFound);
        }

        println!("Found {} mod folder(s) in archive", mod_folders.len());

        // Install each mod folder
        let mut installed_mods = Vec::new();

        for (mod_folder_path, folder_name) in mod_folders {
            println!("Installing mod folder: {}", folder_name);

            // Try to parse manifest if it exists
            let manifest_path = mod_folder_path.join("manifest.json");
            let (manifest, install_folder_name) = if manifest_path.exists() {
                match self.parse_manifest(&manifest_path) {
                    Ok(m) => {
                        println!(
                            "   {} {} v{} ({})",
                            if m.content_pack_for.is_some() { "Content Pack:" } else { "SMAPI Mod:" },
                            m.name,
                            m.version,
                            m.unique_id
                        );
                        // Use the Name field which includes prefixes like [CP], [FTM]
                        let folder = m.name.clone();
                        (m, folder)
                    }
                    Err(e) => {
                        println!("   ⚠ Manifest parsing failed: {}", e);
                        println!("   ✓ Using original folder name: {}", folder_name);
                        // Use original folder name which includes [CP]/[FTM] prefix
                        let manifest = ModManifest {
                            name: folder_name.clone(),
                            author: "Unknown".to_string(),
                            version: "1.0.0".to_string(),
                            unique_id: folder_name.clone(),
                            description: None,
                            dependencies: None,
                            content_pack_for: None,
                        };
                        (manifest, folder_name.clone())
                    }
                }
            } else {
                println!("   ⚠ No manifest.json found, using original folder name");
                // Use original folder name
                let manifest = ModManifest {
                    name: folder_name.clone(),
                    author: "Unknown".to_string(),
                    version: "1.0.0".to_string(),
                    unique_id: folder_name.clone(),
                    description: None,
                    dependencies: None,
                    content_pack_for: None,
                };
                (manifest, folder_name.clone())
            };

            // Use determined folder name
            let install_path = game_path.join("Mods").join(&install_folder_name);

            // Handle existing mod
            if install_path.exists() {
                println!("   Mod folder already exists, backing up and replacing");

                if let Err(e) = self.backup_mod(&install_path, &install_folder_name) {
                    eprintln!("   Failed to backup mod: {}", e);
                }

                fs::remove_dir_all(&install_path)?;
            }

            // Install mod
            if settings.auto_install {
                match self.install_mod_files_with_rollback(&mod_folder_path, &install_path) {
                    Ok(_) => {
                        println!("   ✓ Installed to: {}", install_path.display());
                        installed_mods.push((manifest, install_path.clone()));
                    }
                    Err(e) => {
                        eprintln!("   ✗ Failed to install: {}", e);
                        return Err(e);
                    }
                }
            } else {
                return Err(InstallError::InstallationFailed(
                    "Auto-install is disabled".to_string(),
                ));
            }
        }

        // Check if at least one mod was installed
        if installed_mods.is_empty() {
            return Err(InstallError::InstallationFailed(
                "No mod folders found in archive".to_string(),
            ));
        }

        println!("Successfully installed {} mod folder(s)", installed_mods.len());

        // Cleanup temp directory
        if let Err(e) = fs::remove_dir_all(&extract_dir) {
            eprintln!("Failed to cleanup temp directory: {}", e);
        }

        // Delete archive if requested
        if settings.delete_after_install {
            if let Err(e) = fs::remove_file(archive_path) {
                eprintln!("Failed to delete archive: {}", e);
            } else {
                println!("Deleted archive: {}", archive_path.display());
            }
        }

        // Emit events for all installed mods
        for (manifest, install_path) in &installed_mods {
            let result = InstallResult {
                mod_name: manifest.name.clone(),
                version: manifest.version.clone(),
                unique_id: manifest.unique_id.clone(),
                install_path: install_path.clone(),
            };
            let _ = self.app_handle.emit("mod-installed", &result);
        }

        // Return the first mod's result
        let (first_mod, first_path) = &installed_mods[0];
        Ok(InstallResult {
            mod_name: first_mod.name.clone(),
            version: first_mod.version.clone(),
            unique_id: first_mod.unique_id.clone(),
            install_path: first_path.clone(),
        })
    }

    /// Extract a ZIP archive to the temp directory
    async fn extract_archive(&self, archive_path: &Path) -> Result<PathBuf, InstallError> {
        // Generate unique extract directory
        let extract_dir = self.temp_dir.join(
            archive_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        );

        // Remove old extraction if exists
        if extract_dir.exists() {
            fs::remove_dir_all(&extract_dir)?;
        }

        fs::create_dir_all(&extract_dir)?;

        // Open ZIP file
        let file = File::open(archive_path)?;
        let mut archive = ZipArchive::new(file)
            .map_err(|e| InstallError::ExtractionFailed(format!("Invalid ZIP: {}", e)))?;

        // Extract all files
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| InstallError::ExtractionFailed(e.to_string()))?;

            let outpath = match file.enclosed_name() {
                Some(path) => extract_dir.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                // Directory
                fs::create_dir_all(&outpath)?;
            } else {
                // File
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent)?;
                }

                let mut outfile = File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }

            // Set permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
                }
            }
        }

        println!("Extracted archive to: {}", extract_dir.display());
        Ok(extract_dir)
    }

    /// Detect mod folders in the extracted archive
    /// Returns: Vec<(mod_folder_path, folder_name)>
    fn detect_mod_folders(&self, extract_dir: &Path) -> Result<Vec<(PathBuf, String)>, InstallError> {
        let mut mod_folders = Vec::new();

        println!("🔍 Detecting mod folders in: {}", extract_dir.display());

        // Read the extracted directory
        let entries: Vec<_> = fs::read_dir(extract_dir)?
            .filter_map(|e| e.ok())
            .collect();

        println!("   Found {} entries at root level", entries.len());
        for (i, entry) in entries.iter().enumerate() {
            println!("   Entry {}: {} ({})",
                i + 1,
                entry.file_name().to_string_lossy(),
                if entry.path().is_dir() { "dir" } else { "file" }
            );
        }

        // Case 1: Single file/folder at root - check if it contains mods
        if entries.len() == 1 && entries[0].path().is_dir() {
            let root_folder = &entries[0].path();
            let root_name = entries[0].file_name().to_string_lossy().to_string();

            println!("\n📁 Archive has single root folder: {}", root_name);
            println!("   Checking contents of root folder...");

            // Check if this root folder contains multiple mod folders
            let subentries: Vec<_> = fs::read_dir(root_folder)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .collect();

            println!("   Found {} subfolders", subentries.len());
            for (i, entry) in subentries.iter().enumerate() {
                let has_manifest = entry.path().join("manifest.json").exists();
                println!("   Subfolder {}: {} (manifest: {})",
                    i + 1,
                    entry.file_name().to_string_lossy(),
                    if has_manifest { "✓" } else { "✗" }
                );
            }

            // Check if subfolders look like mods (have manifest.json or are named like mods)
            let subfolders_with_manifests: Vec<_> = subentries
                .iter()
                .filter(|e| e.path().join("manifest.json").exists())
                .collect();

            println!("   {} subfolders have manifest.json", subfolders_with_manifests.len());

            if subfolders_with_manifests.len() > 1 {
                // Multiple mod folders detected - install each one
                println!("\n✅ Decision: Install {} mod folders from inside root", subfolders_with_manifests.len());
                for entry in subfolders_with_manifests {
                    let folder_name = entry.file_name().to_string_lossy().to_string();
                    println!("   → Will install: {}", folder_name);
                    mod_folders.push((entry.path(), folder_name));
                }
            } else if !subentries.is_empty() && subentries.iter().all(|e| e.path().join("manifest.json").exists()) {
                // All subfolders have manifests - install each one
                println!("\n✅ Decision: All {} subfolders have manifests, installing each", subentries.len());
                for entry in subentries {
                    let folder_name = entry.file_name().to_string_lossy().to_string();
                    println!("   → Will install: {}", folder_name);
                    mod_folders.push((entry.path(), folder_name));
                }
            } else if root_folder.join("manifest.json").exists() {
                // Root folder itself is a mod
                println!("\n✅ Decision: Root folder is a single mod");
                println!("   → Will install: {}", root_name);
                mod_folders.push((root_folder.clone(), root_name));
            } else {
                // No manifests found, but install all subfolders anyway (content packs, etc.)
                println!("\n✅ Decision: Installing all {} subfolders (no manifest check)", subentries.len());
                for entry in subentries {
                    let folder_name = entry.file_name().to_string_lossy().to_string();
                    println!("   → Will install: {}", folder_name);
                    mod_folders.push((entry.path(), folder_name));
                }
            }
        } else {
            // Case 2: Multiple files/folders at root - each folder is potentially a mod
            println!("\n✅ Decision: Multiple items at root, installing all {} folders", entries.len());

            for entry in entries {
                if entry.path().is_dir() {
                    let folder_name = entry.file_name().to_string_lossy().to_string();
                    println!("   → Will install: {}", folder_name);
                    mod_folders.push((entry.path(), folder_name));
                }
            }
        }

        println!("\n📊 Total mod folders detected: {}\n", mod_folders.len());
        Ok(mod_folders)
    }

    /// Find all manifest.json files in the extracted directory (legacy)
    #[allow(dead_code)]
    fn find_all_manifests(&self, extract_dir: &Path) -> Result<Vec<(PathBuf, PathBuf)>, InstallError> {
        let mut manifests = Vec::new();

        for entry in WalkDir::new(extract_dir).max_depth(3) {
            let entry = entry?;
            if entry.file_name() == "manifest.json" {
                let manifest_path = entry.path().to_path_buf();
                let mod_root = entry
                    .path()
                    .parent()
                    .ok_or(InstallError::ManifestNotFound)?
                    .to_path_buf();

                manifests.push((manifest_path, mod_root));
            }
        }

        Ok(manifests)
    }

    /// Find manifest.json in the extracted directory (legacy - finds first one)
    #[allow(dead_code)]
    fn find_manifest(&self, extract_dir: &Path) -> Result<(PathBuf, PathBuf), InstallError> {
        for entry in WalkDir::new(extract_dir).max_depth(3) {
            let entry = entry?;
            if entry.file_name() == "manifest.json" {
                let manifest_path = entry.path().to_path_buf();
                let mod_root = entry
                    .path()
                    .parent()
                    .ok_or(InstallError::ManifestNotFound)?
                    .to_path_buf();

                return Ok((manifest_path, mod_root));
            }
        }

        Err(InstallError::ManifestNotFound)
    }

    /// Extract just the UniqueID from a manifest.json (fallback for malformed manifests)
    fn extract_unique_id(&self, manifest_path: &Path) -> Result<String, InstallError> {
        let file = File::open(manifest_path)?;
        let mut reader = BufReader::new(file);

        // Read and handle BOM
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        let mut content = content.trim_start_matches('\u{feff}').to_string();

        // Fix common JSON issues: remove trailing commas before closing braces/brackets
        // This handles manifests with trailing commas which are common in Stardew mods
        content = content
            .replace(",\n}", "\n}")
            .replace(",\r\n}", "\r\n}")
            .replace(", }", " }")
            .replace(",]", "]")
            .replace(", ]", " ]");

        // Parse as generic JSON
        let value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| InstallError::InvalidManifest(format!("Invalid JSON: {}", e)))?;

        // Extract UniqueID from root level (NOT from ContentPackFor)
        // Content packs have two UniqueID fields:
        // - Root level: The mod's own UniqueID (e.g., "FlashShifter.StardewValleyExpandedCP")
        // - ContentPackFor.UniqueID: The framework it depends on (e.g., "Pathoschild.ContentPatcher")
        // We want the root level one!
        value
            .get("UniqueID")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| InstallError::InvalidManifest("UniqueID field not found at root level".to_string()))
    }

    /// Parse manifest.json
    fn parse_manifest(&self, manifest_path: &Path) -> Result<ModManifest, InstallError> {
        let file = File::open(manifest_path)?;
        let mut reader = BufReader::new(file);

        // Read and handle BOM
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        let mut content = content.trim_start_matches('\u{feff}').to_string();

        // Fix common JSON issues: remove trailing commas before closing braces/brackets
        content = content
            .replace(",\n}", "\n}")
            .replace(",\r\n}", "\r\n}")
            .replace(", }", " }")
            .replace(",]", "]")
            .replace(", ]", " ]");

        // Try to parse as generic JSON first to check structure
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(value) => {
                // Check if it's an object
                if !value.is_object() {
                    return Err(InstallError::InvalidManifest(
                        "Manifest is not a JSON object".to_string()
                    ));
                }

                // Try to parse as ModManifest
                serde_json::from_str::<ModManifest>(&content)
                    .map_err(|e| {
                        // Show which required fields might be missing
                        let obj = value.as_object().unwrap();
                        let has_name = obj.contains_key("Name");
                        let has_version = obj.contains_key("Version");
                        let has_unique_id = obj.contains_key("UniqueID");

                        let missing_fields = vec![
                            if !has_name { Some("Name") } else { None },
                            if !has_version { Some("Version") } else { None },
                            if !has_unique_id { Some("UniqueID") } else { None },
                        ]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>();

                        if !missing_fields.is_empty() {
                            InstallError::InvalidManifest(format!(
                                "Missing required fields: {}. Error: {}",
                                missing_fields.join(", "),
                                e
                            ))
                        } else {
                            InstallError::InvalidManifest(e.to_string())
                        }
                    })
            }
            Err(e) => Err(InstallError::InvalidManifest(format!(
                "Invalid JSON syntax: {}",
                e
            ))),
        }
    }

    /// Install mod files with rollback support
    fn install_mod_files_with_rollback(&self, source: &Path, destination: &Path) -> Result<(), InstallError> {
        println!(
            "Installing mod files from {} to {}",
            source.display(),
            destination.display()
        );

        // Create destination directory
        if let Err(e) = fs::create_dir_all(destination) {
            return Err(InstallError::IoError(e));
        }

        // Copy all files recursively
        if let Err(e) = self.copy_dir_recursive(source, destination) {
            eprintln!("Installation failed, rolling back...");
            // Rollback: Delete the destination directory
            let _ = fs::remove_dir_all(destination);
            return Err(e);
        }

        Ok(())
    }

    /// Verify that files were copied correctly
    fn verify_installation(&self, source: &Path, destination: &Path) -> Result<(), String> {
        let source_count = WalkDir::new(source).into_iter().count();
        let dest_count = WalkDir::new(destination).into_iter().count();

        if source_count != dest_count {
            return Err(format!(
                "File count mismatch: Source={}, Destination={}",
                source_count, dest_count
            ));
        }
        Ok(())
    }

    /// Get version of an installed mod
    fn get_mod_version(&self, mod_path: &Path) -> Option<String> {
        let manifest_path = mod_path.join("manifest.json");
        if let Ok(manifest) = self.parse_manifest(&manifest_path) {
            Some(manifest.version)
        } else {
            None
        }
    }

    /// Generate a unique path for "Keep Both" strategy
    #[allow(dead_code)]
    fn get_unique_path(&self, base_dir: &Path, unique_id: &str) -> PathBuf {
        let mut path = base_dir.join(unique_id);
        let mut counter = 1;
        while path.exists() {
            path = base_dir.join(format!("{} ({})", unique_id, counter));
            counter += 1;
        }
        path
    }

    /// Recursively copy directory contents
    fn copy_dir_recursive(&self, source: &Path, destination: &Path) -> Result<(), InstallError> {
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let source_path = entry.path();
            let dest_path = destination.join(entry.file_name());

            if source_path.is_dir() {
                fs::create_dir_all(&dest_path)?;
                self.copy_dir_recursive(&source_path, &dest_path)?;
            } else {
                fs::copy(&source_path, &dest_path)?;
            }
        }

        Ok(())
    }

    /// Backup a mod to the backups directory
    fn backup_mod(&self, mod_path: &Path, unique_id: &str) -> Result<PathBuf, std::io::Error> {
        let app_data_dir = self.app_handle.path().app_data_dir().unwrap();
        let backups_dir = app_data_dir.join("backups").join(unique_id);
        
        // Create timestamped backup folder
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let backup_path = backups_dir.join(timestamp.to_string());

        fs::create_dir_all(&backup_path)?;

        // Copy mod to backup
        // We can't use copy_dir_recursive here because it returns InstallError
        // So we'll implement a simple recursive copy for backup
        self.copy_dir_all(mod_path, &backup_path)?;

        println!("Backed up mod to: {}", backup_path.display());
        Ok(backup_path)
    }

    fn copy_dir_all(&self, src: &Path, dst: &Path) -> std::io::Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            if ty.is_dir() {
                self.copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), dst.join(entry.file_name()))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parsing() {
        let manifest_json = r#"{
            "Name": "Test Mod",
            "Author": "Test Author",
            "Version": "1.0.0",
            "Description": "A test mod",
            "UniqueID": "TestAuthor.TestMod",
            "EntryDll": "TestMod.dll"
        }"#;

        let manifest: Result<ModManifest, _> = serde_json::from_str(manifest_json);
        assert!(manifest.is_ok());

        let manifest = manifest.unwrap();
        assert_eq!(manifest.name, "Test Mod");
        assert_eq!(manifest.unique_id, "TestAuthor.TestMod");
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("1.1.0").unwrap();
        let v3 = Version::parse("1.0.0").unwrap();

        assert!(v2 > v1);
        assert!(v1 == v3);
        assert!(v1 < v2);
    }
}
