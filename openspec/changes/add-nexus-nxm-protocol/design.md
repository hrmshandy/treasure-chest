# Design Document: Nexus Mods NXM Protocol Integration

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Nexus Mods Website                       │
│  User clicks "Mod Manager Download"                         │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ nxm://stardewvalley/mods/123/files/456?key=abc...
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                   Operating System                          │
│  Looks up nxm:// handler → SDV Mods Manager                 │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ Deep Link Event
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Tauri Deep Link Plugin                         │
│  Receives URL, emits event to Rust backend                  │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              NXM Protocol Handler (Rust)                    │
│  • Parse URL (game, mod_id, file_id, key, expires)          │
│  • Validate key expiration                                  │
│  • Fetch mod metadata (optional, for display)               │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Download Manager (Rust)                        │
│  • Add to download queue                                    │
│  • Track progress (bytes downloaded, speed, ETA)            │
│  • Save to downloads folder                                 │
│  • Emit progress events to frontend                         │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Mod Installer (Rust)                           │
│  • Extract archive (ZIP/7z/RAR)                             │
│  • Detect mod structure (find manifest.json)                │
│  • Copy to game mods folder                                 │
│  • Add to mod list                                          │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              React Frontend                                 │
│  • Show download progress notifications                     │
│  • Display Download Manager UI                              │
│  • Update mod list with new mods                            │
└─────────────────────────────────────────────────────────────┘
```

## Key Architectural Decisions

### 1. NXM Protocol Registration Strategy

**Decision:** Use Tauri's official deep link plugin for cross-platform protocol registration.

**Rationale:**
- Handles platform-specific registry/plist/desktop file management
- Automatic cleanup on uninstall
- Battle-tested by Tauri community
- Single API for all platforms

**Implementation:**
```rust
use tauri_plugin_deep_link;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                app.deep_link().register("nxm")?;

                app.listen_global("deep-link://nxm", |event| {
                    if let Some(url) = event.payload() {
                        // Handle NXM URL
                    }
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Platform-Specific Behavior:**
- **Windows:** Registry key at `HKEY_CURRENT_USER\Software\Classes\nxm`
- **macOS:** Info.plist `CFBundleURLSchemes` entry
- **Linux:** Desktop file with `x-scheme-handler/nxm` MIME type

### 2. NXM URL Parsing

**Decision:** Use regex-based parsing with fallback to URL crate.

**NXM URL Format:**
```
nxm://stardewvalley/mods/2400/files/9567?key=eyJ0eXAiOi...&expires=1735344000&user_id=12345
```

**Parsed Components:**
- `game_domain`: "stardewvalley" (must match "stardewvalley" for our app)
- `mod_id`: 2400
- `file_id`: 9567
- `key`: Authentication token (base64-encoded JWT-like string)
- `expires`: Unix timestamp (validate not expired)
- `user_id`: Nexus user ID (optional, for tracking)

**Validation Rules:**
1. Scheme must be exactly "nxm"
2. Game domain must be "stardewvalley" (reject other games)
3. mod_id and file_id must be positive integers
4. key must be present and non-empty
5. expires must be in the future (if present)

**Implementation:**
```rust
pub struct NxmUrl {
    pub game: String,
    pub mod_id: u32,
    pub file_id: u32,
    pub key: String,
    pub expires: Option<u64>,
    pub user_id: Option<u32>,
}

impl NxmUrl {
    pub fn parse(url: &str) -> Result<Self, NxmError> {
        // Parse nxm://game/mods/{mod_id}/files/{file_id}?params
        // Validate expiration
        // Return parsed struct
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            expires < now
        } else {
            false // No expiration = never expires
        }
    }
}
```

### 3. Download Manager Architecture

**Decision:** Async queue with Tokio channels and semaphore-based concurrency control.

**Why Async?**
- Non-blocking downloads don't freeze UI
- Efficient resource usage
- Natural fit with Tokio ecosystem

**Queue Design:**
```rust
pub struct DownloadManager {
    queue: Arc<Mutex<VecDeque<DownloadTask>>>,
    active: Arc<Mutex<HashMap<String, DownloadHandle>>>,
    semaphore: Arc<Semaphore>, // Limit concurrent downloads
    progress_tx: broadcast::Sender<DownloadProgress>,
}

pub struct DownloadTask {
    pub id: String,              // UUID for tracking
    pub nxm_url: NxmUrl,
    pub mod_name: Option<String>, // Fetched from Nexus or unknown
    pub file_name: String,
    pub download_url: String,     // Generated from nxm_url.key
    pub status: DownloadStatus,
}

pub enum DownloadStatus {
    Queued,
    Downloading { progress: f64, speed: u64, eta: u64 },
    Paused { progress: f64 },
    Completed { file_path: PathBuf },
    Failed { error: String },
}
```

**Concurrency Control:**
- Default: 1 concurrent download (free account limit)
- Premium users: Increase via settings (3-5 concurrent)
- Use `tokio::sync::Semaphore` to enforce limit

**Progress Tracking:**
```rust
pub struct DownloadProgress {
    pub download_id: String,
    pub bytes_downloaded: u64,
    pub bytes_total: u64,
    pub speed_bps: u64,        // Bytes per second
    pub eta_seconds: u64,      // Estimated time remaining
}
```

**State Persistence:**
- Save queue state to JSON file on each change
- On app startup, restore queue and resume incomplete downloads
- Location: `{app_data}/downloads/queue.json`

### 4. Download URL Generation

**Decision:** Use the NXM key parameter directly to download files.

**How It Works:**
The `key` parameter in the nxm:// URL IS the authorization token. Nexus generates this when the user clicks "Mod Manager Download" while logged in.

**Download Request:**
```rust
async fn download_file(nxm_url: &NxmUrl) -> Result<PathBuf, DownloadError> {
    let download_url = format!(
        "https://www.nexusmods.com/Core/Libs/Common/Widgets/DownloadPopUp?id={}&game_id=1303",
        nxm_url.file_id
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&download_url)
        .query(&[
            ("fid", nxm_url.file_id.to_string()),
            ("key", nxm_url.key.clone()),
        ])
        .send()
        .await?;

    // Stream to file with progress tracking
}
```

**Alternative Approach (If Direct Download Fails):**
Query Nexus API to get actual download link using the key:
```rust
// GET https://api.nexusmods.com/v1/games/stardewvalley/mods/{mod_id}/files/{file_id}/download_link.json
// Header: apikey: {user's API key from settings}
// This returns the actual CDN download URL
```

**Fallback Strategy:**
1. Try direct download with key parameter
2. If 401/403, check if user has API key in settings
3. If yes, use API to get download link
4. If no API key, show error with link to Nexus to re-download

### 5. Archive Extraction

**Decision:** Use `zip` crate for ZIP files, with future support for 7z/RAR.

**Why ZIP Only Initially?**
- 99% of Stardew Valley mods are distributed as ZIP
- `zip` crate is pure Rust, no external dependencies
- 7z and RAR require external binaries or complex libraries

**Extraction Strategy:**
```rust
use zip::ZipArchive;
use std::fs::File;

pub async fn extract_mod(archive_path: &Path, dest_dir: &Path) -> Result<PathBuf, ExtractError> {
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Extract all files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = dest_dir.join(file.name());

        if file.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(dest_dir.to_path_buf())
}
```

**Mod Structure Detection:**
Look for `manifest.json` in extracted files:
- If found at root: mod is properly structured
- If found in subfolder: mod root is that subfolder
- If not found: Assume entire archive is mod content

### 6. Auto-Installation Flow

**Decision:** Auto-install by default with optional confirmation toggle in settings.

**Installation Steps:**
1. Download completes → Extract to temp folder
2. Detect mod structure (find manifest.json)
3. Parse manifest for metadata:
   ```json
   {
     "Name": "Mod Name",
     "Version": "1.0.0",
     "UniqueID": "author.modname",
     "EntryDll": "ModName.dll"
   }
   ```
4. Copy mod folder to `{game_path}/Mods/{UniqueID}/`
5. Add entry to mod list JSON
6. Emit event to frontend to refresh mod list
7. Show success notification
8. Clean up temp files

**Error Handling:**
- If manifest.json missing: Show warning, offer manual installation
- If mod already exists: Offer to replace or keep both (rename)
- If game path not set: Show error, prompt user to configure in settings

### 7. Frontend State Management

**Decision:** Use React hooks with Tauri event listeners (no external state library).

**Download Manager State:**
```typescript
interface DownloadState {
  downloads: Download[];
  activeCount: number;
  queuedCount: number;
}

interface Download {
  id: string;
  modName: string;
  fileName: string;
  status: 'queued' | 'downloading' | 'paused' | 'completed' | 'failed';
  progress: number;      // 0-100
  speed: number;         // bytes/sec
  eta: number;           // seconds
  error?: string;
}

// Hook
function useDownloads() {
  const [state, setState] = useState<DownloadState>({
    downloads: [],
    activeCount: 0,
    queuedCount: 0,
  });

  useEffect(() => {
    const unlisten = listen('download-progress', (event) => {
      // Update download state
    });

    return () => { unlisten(); };
  }, []);

  return state;
}
```

**Event Flow:**
```
Backend (Rust)           →  Frontend (React)
─────────────────────────────────────────────
download-queued          →  Add to downloads list
download-progress        →  Update progress bar
download-completed       →  Show success notification
download-failed          →  Show error notification
mod-installed            →  Refresh mod list
```

### 8. Settings Integration

**Decision:** Extend existing Settings struct with download preferences.

**New Settings Fields:**
```rust
#[derive(Serialize, Deserialize)]
pub struct Settings {
    // ... existing fields ...

    // Download settings
    pub download_directory: PathBuf,        // Default: {app_data}/downloads
    pub concurrent_downloads: u8,           // Default: 1, max: 5
    pub auto_install: bool,                 // Default: true
    pub confirm_before_install: bool,       // Default: false
    pub delete_after_install: bool,         // Default: false
    pub nexus_api_key: Option<String>,      // For Premium users (optional)
}
```

**Settings UI:**
Add "Downloads" section in SettingsModal:
```tsx
<div className="setting-group">
  <h3>Downloads</h3>

  <div className="setting-item">
    <label>Download Directory</label>
    <input type="text" value={settings.downloadDirectory} readOnly />
    <button onClick={selectDownloadDir}>Browse</button>
  </div>

  <div className="setting-item">
    <label>Concurrent Downloads</label>
    <input type="number" min="1" max="5" value={settings.concurrentDownloads} />
    <span className="hint">Free users: 1, Premium: up to 5</span>
  </div>

  <div className="setting-item">
    <label>
      <input type="checkbox" checked={settings.autoInstall} />
      Automatically install downloaded mods
    </label>
  </div>

  <div className="setting-item">
    <label>Nexus API Key (Optional - for Premium users)</label>
    <input type="password" value={settings.nexusApiKey || ''} />
    <a href="https://www.nexusmods.com/users/myaccount?tab=api" target="_blank">
      Get API Key
    </a>
  </div>
</div>
```

## Data Flow Diagrams

### Download Flow
```
User clicks "Mod Manager Download" on Nexus
  ↓
Browser triggers nxm://stardewvalley/mods/123/files/456?key=...
  ↓
OS launches SDV Mods Manager (or sends to running instance)
  ↓
Tauri deep link plugin emits event
  ↓
NxmProtocolHandler::parse_and_queue()
  ├─→ Parse URL
  ├─→ Validate expiration
  ├─→ Create DownloadTask
  └─→ DownloadManager::add_to_queue()
       ↓
DownloadManager acquires semaphore permit
  ↓
DownloadManager::start_download()
  ├─→ Request file with key parameter
  ├─→ Stream to disk with progress tracking
  ├─→ Emit progress events to frontend
  └─→ On completion → ModInstaller::install()
       ↓
ModInstaller::install()
  ├─→ Extract archive
  ├─→ Detect manifest.json
  ├─→ Copy to game mods folder
  ├─→ Update mod list JSON
  ├─→ Emit mod-installed event
  └─→ Clean up temp files
       ↓
Frontend receives events
  ├─→ Update download progress UI
  ├─→ Show notification
  └─→ Refresh mod list
```

### Error Handling Flow
```
Download fails (network error, key expired, etc.)
  ↓
DownloadManager::handle_error()
  ├─→ Update task status to Failed
  ├─→ Emit download-failed event with error message
  └─→ Keep task in queue for retry
       ↓
Frontend shows error notification
  ├─→ "Download failed: [error message]"
  └─→ Show "Retry" button
       ↓
User clicks "Retry"
  ↓
Frontend calls retry_download(task_id)
  ↓
DownloadManager re-queues task
  ↓
... (repeat download flow)
```

## Security Considerations

### 1. NXM URL Validation
- **Risk:** Malicious nxm:// URLs could trigger unintended downloads
- **Mitigation:**
  - Strict URL format validation
  - Game domain whitelist (only "stardewvalley")
  - Key expiration checks
  - Mod ID/File ID must be positive integers

### 2. Download Path Validation
- **Risk:** Path traversal in extracted files (e.g., `../../../etc/passwd`)
- **Mitigation:**
  - Sanitize all extracted file paths
  - Ensure all paths are within download directory
  - Reject symlinks during extraction

### 3. Malicious Archives
- **Risk:** ZIP bombs, corrupted files, malware
- **Mitigation:**
  - Set max extraction size limit (e.g., 500MB)
  - Scan manifest.json for suspicious patterns
  - Future: Optional virus scanning integration

### 4. API Key Storage
- **Risk:** Nexus API key leaked if stored in plaintext
- **Mitigation:**
  - Store in OS keychain/credential manager
  - Use `keyring` crate for secure storage
  - Never log or display API key

## Performance Considerations

### 1. Download Speed
- Free users: 1.5-3 MB/s (Nexus-imposed limit)
- Premium users: Uncapped (limited by bandwidth)
- Use chunked downloads for large files

### 2. Memory Usage
- Stream downloads to disk (don't load entire file in memory)
- Use buffered readers/writers (8KB buffer)
- Extract archives in streaming mode when possible

### 3. UI Responsiveness
- All download operations run on Tokio async runtime
- Progress events throttled to max 10 updates/sec
- Download manager UI virtualizes large lists

## Testing Strategy

### Unit Tests
- NXM URL parsing (valid/invalid formats)
- Expiration validation
- Download progress calculation
- Archive extraction

### Integration Tests
- Full download flow (mock Nexus server)
- Queue management (add/pause/resume/cancel)
- Auto-installation flow

### Manual Testing
- Test on all platforms (Windows, macOS, Linux)
- Test with real Nexus mods
- Test concurrent downloads
- Test network interruption recovery

## Future Enhancements

### Phase 2 (Post-MVP)
1. **Premium API Integration**
   - Use Nexus API for faster downloads
   - Batch downloads via Collections
   - Mod update checking

2. **Advanced Features**
   - Download scheduling (download overnight)
   - Bandwidth throttling
   - Download history/statistics
   - Mod dependency auto-download

3. **Multi-Game Support**
   - Support nxm:// for other games
   - Multi-game mod library

## References

- [Tauri Deep Link Plugin](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/deep-link)
- [NXM Protocol Implementation](https://github.com/TanninOne/nxmproxy)
- [Nexus Mods API](https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0)
- [Tokio Async Runtime](https://tokio.rs/)
- [ZIP Crate Documentation](https://docs.rs/zip/latest/zip/)
