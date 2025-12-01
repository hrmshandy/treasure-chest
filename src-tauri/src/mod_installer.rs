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
        nexus_info: Option<(u32, u32)>,
        mod_name: Option<String>,
    ) -> Result<InstallResult, InstallError> {
        println!("Installing mod from: {}", archive_path.display());

        // Create temp directory if it doesn't exist
        fs::create_dir_all(&self.temp_dir)?;

        // Extract archive to temp directory
        let extract_dir = self.extract_archive(archive_path).await?;

        // Determine installation strategy
        let (source_path, target_name) = self.determine_install_strategy(&extract_dir, archive_path, mod_name.clone())?;

        // Check for Frameworks
        let is_framework = if let Some(name) = &mod_name {
            settings.core_frameworks.contains(name)
        } else {
            // Fallback to checking target name if mod_name not provided
            settings.core_frameworks.contains(&target_name)
        };

        let install_base = if is_framework {
            game_path.join("Mods").join("_Frameworks")
        } else {
            game_path.join("Mods")
        };

        let install_path = install_base.join(&target_name);
        println!("   Target install path: {}", install_path.display());

        // Handle existing mod
        if install_path.exists() {
            println!("   Mod folder already exists, backing up and replacing");

            if let Err(e) = self.backup_mod(&install_path, &target_name) {
                eprintln!("   Failed to backup mod: {}", e);
            }

            self.force_remove_dir_all(&install_path)?;
        }

        // Install mod
        if settings.auto_install {
            match self.install_mod_files_with_rollback(&source_path, &install_path) {
                Ok(_) => {
                    println!("   ✓ Installed to: {}", install_path.display());
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

        // Cleanup temp directory
        if let Err(e) = self.force_remove_dir_all(&extract_dir) {
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

        // Try to find manifest in the installed location to get version/ID
        let manifest_path = install_path.join("manifest.json");
        let (version, unique_id) = if manifest_path.exists() {
            match self.parse_manifest(&manifest_path) {
                Ok(m) => (m.version, m.unique_id),
                Err(_) => ("Unknown".to_string(), target_name.clone()),
            }
        } else {
            ("Unknown".to_string(), target_name.clone())
        };

        // Write Nexus metadata if available
        if let Some((mod_id, file_id)) = nexus_info {
            if let Err(e) = self.write_nexus_meta(&install_path, mod_id, file_id) {
                eprintln!("Failed to write Nexus metadata: {}", e);
            }
        }

        let result = InstallResult {
            mod_name: mod_name.unwrap_or(target_name),
            version,
            unique_id,
            install_path: install_path.clone(),
        };

        let _ = self.app_handle.emit("mod-installed", &result);

        Ok(result)
    }

    /// Determine installation strategy based on extracted contents
    /// Returns (source_path_to_copy_from, target_folder_name)
    fn determine_install_strategy(
        &self,
        extract_dir: &Path,
        archive_path: &Path,
        mod_name: Option<String>,
    ) -> Result<(PathBuf, String), InstallError> {
        let entries: Vec<_> = fs::read_dir(extract_dir)?
            .filter_map(|e| e.ok())
            .collect();

        // Case A: Single folder
        if entries.len() == 1 && entries[0].path().is_dir() {
            let folder_name = entries[0].file_name().to_string_lossy().to_string();
            println!("   Strategy: Single folder detected ({})", folder_name);
            Ok((entries[0].path(), folder_name))
        } else {
            // Case B: Multi-folder / Loose files
            // Use mod_name if available, otherwise archive filename
            let target_name = mod_name.unwrap_or_else(|| {
                archive_path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            });
            println!("   Strategy: Multi-item/Loose files detected. Using container: {}", target_name);
            Ok((extract_dir.to_path_buf(), target_name))
        }
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
            self.force_remove_dir_all(&extract_dir)?;
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
                    // Ensure we always have write permissions for the user
                    // This prevents "Permission denied" errors when extracting files into read-only directories
                    // or when trying to overwrite read-only files (though we clean up first)
                    let safe_mode = if file.name().ends_with('/') {
                        // For directories, ensure rwx for user (0o700)
                        mode | 0o700
                    } else {
                        // For files, ensure rw for user (0o600)
                        mode | 0o600
                    };

                    fs::set_permissions(&outpath, fs::Permissions::from_mode(safe_mode))?;
                }
            }
        }

        println!("Extracted archive to: {}", extract_dir.display());
        Ok(extract_dir)
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


    /// Strip JSON comments (/* */ and //) from a string
    fn strip_json_comments(input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        let mut in_string = false;
        let mut escape_next = false;

        while let Some(ch) = chars.next() {
            if escape_next {
                result.push(ch);
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => {
                    result.push(ch);
                    escape_next = true;
                }
                '"' => {
                    in_string = !in_string;
                    result.push(ch);
                }
                '/' if !in_string => {
                    if let Some(&next) = chars.peek() {
                        if next == '/' {
                            // Single-line comment
                            chars.next(); // consume second /
                            while let Some(c) = chars.next() {
                                if c == '\n' || c == '\r' {
                                    result.push(c);
                                    break;
                                }
                            }
                        } else if next == '*' {
                            // Multi-line comment
                            chars.next(); // consume *
                            let mut prev = ' ';
                            while let Some(c) = chars.next() {
                                if prev == '*' && c == '/' {
                                    break;
                                }
                                prev = c;
                            }
                            result.push(' '); // Replace comment with space
                        } else {
                            result.push(ch);
                        }
                    } else {
                        result.push(ch);
                    }
                }
                _ => result.push(ch),
            }
        }

        result
    }

    /// Parse manifest.json
    fn parse_manifest(&self, manifest_path: &Path) -> Result<ModManifest, InstallError> {
        let file = File::open(manifest_path)?;
        let mut reader = BufReader::new(file);

        // Read and handle BOM
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        let mut content = content.trim_start_matches('\u{feff}').to_string();

        // Strip JSON comments (/* */ and //)
        content = Self::strip_json_comments(&content);

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
            let _ = self.force_remove_dir_all(destination);
            return Err(e);
        }

        Ok(())
    }

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

    /// Write Nexus metadata to a hidden file in the mod directory
    fn write_nexus_meta(&self, mod_path: &Path, mod_id: u32, file_id: u32) -> std::io::Result<()> {
        let meta_path = mod_path.join(".nexus_meta");
        let meta_content = serde_json::json!({
            "mod_id": mod_id,
            "file_id": file_id
        });
        
        let file = File::create(meta_path)?;
        serde_json::to_writer_pretty(file, &meta_content)?;
        Ok(())
    }

    /// Force remove a directory by ensuring write permissions first
    fn force_remove_dir_all(&self, path: &Path) -> std::io::Result<()> {
        if !path.exists() {
            return Ok(());
        }

        // Try normal remove first
        if fs::remove_dir_all(path).is_ok() {
            return Ok(());
        }

        println!("   ⚠ Normal remove failed, attempting to force permissions on: {}", path.display());

        // Make everything writable
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
}

/// Scan a directory for mods
pub fn scan_mods(game_path: &Path) -> Vec<crate::models::Mod> {
    let mods_dir = game_path.join("Mods");
    let mut mods = Vec::new();

    if !mods_dir.exists() {
        return mods;
    }

    // Helper function to scan a directory recursively
    fn scan_dir(dir: &Path, mods: &mut Vec<crate::models::Mod>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    // Check if this folder is a mod (has manifest.json)
                    let manifest_path = path.join("manifest.json");
                    if manifest_path.exists() {
                        if let Ok(manifest_content) = fs::read_to_string(&manifest_path) {
                            // Strip BOM and JSON comments
                            let content = manifest_content.trim_start_matches('\u{feff}');
                            let content = ModInstaller::strip_json_comments(content);
                            
                            if let Ok(manifest) = serde_json::from_str::<ModManifest>(&content) {
                                // Check if enabled based on folder name
                                // Convention: folder name ending in ".disabled" means disabled
                                let folder_name = path.file_name().unwrap().to_string_lossy();
                                let is_enabled = !folder_name.ends_with(".disabled");

                                // Generate a new ID for the mod, as it's not stored in the manifest
                                let id = uuid::Uuid::new_v4().to_string();

                                mods.push(crate::models::Mod {
                                    id,
                                    name: manifest.name,
                                    author: manifest.author,
                                    version: manifest.version,
                                    unique_id: manifest.unique_id,
                                    description: manifest.description,
                                    dependencies: manifest.dependencies,
                                    content_pack_for: manifest.content_pack_for,
                                    path: path.to_string_lossy().to_string(),
                                    is_enabled,
                                    nexus_mod_id: {
                                        // Read from .nexus_meta if available
                                        let meta_path = path.join(".nexus_meta");
                                        if meta_path.exists() {
                                            fs::read_to_string(&meta_path)
                                                .ok()
                                                .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
                                                .and_then(|json| json.get("mod_id").and_then(|v| v.as_u64()).map(|v| v as u32))
                                        } else {
                                            None
                                        }
                                    },
                                    nexus_file_id: {
                                        // Read from .nexus_meta if available
                                        let meta_path = path.join(".nexus_meta");
                                        if meta_path.exists() {
                                            fs::read_to_string(&meta_path)
                                                .ok()
                                                .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
                                                .and_then(|json| json.get("file_id").and_then(|v| v.as_u64()).map(|v| v as u32))
                                        } else {
                                            None
                                        }
                                    },
                                });
                            }
                        }
                    } else {
                        // Recurse into subdirectories (e.g. for _Frameworks or organized folders)
                        // But don't recurse if it's a disabled mod folder (which might contain the manifest inside)
                        // Actually, we should recurse to find nested mods, but standard structure is Mod/manifest.json
                        scan_dir(&path, mods);
                    }
                }
            }
        }
    }

    scan_dir(&mods_dir, &mut mods);
    
    // Also scan _Frameworks if it exists (it's already covered by recursion above, but just to be sure/explicit if logic changes)
    // The recursion above handles it.

    mods
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;

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

    #[test]
    fn test_nexus_metadata() {
        // Setup temp directory
        let temp_dir = std::env::temp_dir().join("sdv_mgr_test_nexus");
        let mods_dir = temp_dir.join("Mods");
        let mod_dir = mods_dir.join("TestMod");
        
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&mod_dir).unwrap();

        // Create manifest
        let manifest_json = r#"{
            "Name": "Test Mod",
            "Author": "Test Author",
            "Version": "1.0.0",
            "UniqueID": "TestAuthor.TestMod"
        }"#;
        fs::write(mod_dir.join("manifest.json"), manifest_json).unwrap();

        // Write metadata manually (simulating write_nexus_meta)
        let meta_path = mod_dir.join(".nexus_meta");
        let meta_content = serde_json::json!({
            "mod_id": 12345,
            "file_id": 67890
        });
        fs::write(meta_path, serde_json::to_string(&meta_content).unwrap()).unwrap();

        // Scan mods
        let mods = scan_mods(&temp_dir);

        // Verify
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].name, "Test Mod");
        assert_eq!(mods[0].nexus_mod_id, Some(12345));
        assert_eq!(mods[0].nexus_file_id, Some(67890));

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
