# Design: Settings Management System

## Context
The application needs persistent configuration for game paths, SMAPI paths, and Nexus Mods integration. This is the first settings system for the project, establishing patterns for future configuration needs.

**Constraints:**
- Cross-platform desktop app (Windows, Linux, macOS)
- Tauri 2 runtime with Rust backend
- React frontend with TypeScript
- No existing state management library (React hooks only)
- Security requirement: Plain text storage is acceptable for v1

**Stakeholders:**
- End users: Need simple setup with minimal manual configuration
- Developers: Need extensible settings system for future features

## Goals / Non-Goals

**Goals:**
- Persist settings between app sessions
- Minimize user configuration effort through auto-detection
- Support manual override when auto-detection fails
- Store Nexus Mods credentials for future API integration
- Provide clear visual feedback for invalid configurations

**Non-Goals:**
- Encrypted credential storage (deferred to future enhancement)
- Cloud sync or multi-device settings
- Settings import/export functionality
- Advanced validation (e.g., verifying SMAPI version compatibility)
- Settings migration from other mod managers

## Decisions

### Decision 1: Settings Storage Location
**Choice:** Use Tauri's app data directory (`tauri::api::path::app_data_dir()`)

**Rationale:**
- Automatically handles platform-specific paths:
  - Windows: `%APPDATA%\com.tauri.sdv-mods-manager`
  - Linux: `~/.config/com.tauri.sdv-mods-manager`
  - macOS: `~/Library/Application Support/com.tauri.sdv-mods-manager`
- User-writable without admin privileges
- Persists across app updates
- Standard practice for desktop applications

**Alternatives considered:**
- Game directory: Would require write permissions in potentially protected folders
- System config directory: Harder to locate for troubleshooting

### Decision 2: Settings File Format
**Choice:** JSON with flat structure

```json
{
  "gamePath": "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Stardew Valley",
  "smapiPath": "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Stardew Valley\\StardewModdingAPI.exe",
  "nexusAuthCookie": "",
  "nexusApiKey": "",
  "theme": "System",
  "language": "English",
  "modGroups": "Folder"
}
```

**Rationale:**
- Human-readable for debugging
- Direct mapping to Rust struct with Serde
- Easy to version and migrate in the future
- Flat structure sufficient for current needs

**Alternatives considered:**
- TOML: Less familiar to JavaScript developers
- Binary format: Would complicate debugging and manual editing

### Decision 3: Auto-Detection Strategy
**Choice:** Check ordered list of Steam-specific paths per OS, use first valid path

**Detection Order (Windows):**
1. Steam default: `C:\Program Files (x86)\Steam\steamapps\common\Stardew Valley`
2. Steam alternate: `C:\Program Files\Steam\steamapps\common\Stardew Valley`

**Detection Order (Linux):**
1. Steam default: `~/.local/share/Steam/steamapps/common/Stardew Valley`
2. Steam alternate: `~/.steam/steam/steamapps/common/Stardew Valley`
3. Steam flatpak: `~/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/common/Stardew Valley`

**Detection Order (macOS):**
1. Steam: `~/Library/Application Support/Steam/steamapps/common/Stardew Valley`

**Validation Criteria:**
- Directory exists
- Contains `Stardew Valley.deps.json` or `StardewValley.exe` (Windows) or `Stardew Valley` binary (Linux/macOS)

**Rationale:**
- Simplified to Steam-only (covers majority of users)
- Reduces code complexity and maintenance burden
- Non-Steam users can still use manual path configuration
- Extensible for future distribution methods if needed

**Alternatives considered:**
- Multi-platform support (GOG, Itch.io, Microsoft Store): Adds complexity with diminishing returns
- Registry scan (Windows): Unreliable, not all installers update registry
- Process scanning: Requires user to have game running, privacy concerns

### Decision 4: SMAPI Detection
**Choice:** Derive from game path + platform-specific executable name

**Logic:**
```rust
fn detect_smapi_path(game_path: &Path) -> Option<PathBuf> {
    let smapi_name = if cfg!(windows) {
        "StardewModdingAPI.exe"
    } else {
        "StardewModdingAPI"
    };

    let smapi_path = game_path.join(smapi_name);
    if smapi_path.exists() { Some(smapi_path) } else { None }
}
```

**Rationale:**
- SMAPI installs directly into game directory
- Standard installation process doesn't change location
- Simple deterministic logic

### Decision 5: File Picker Integration
**Choice:** Use Tauri dialog plugin (`@tauri-apps/plugin-dialog`)

**Rationale:**
- Already a project dependency
- Native OS dialogs (consistent with platform UX)
- Works in Tauri's desktop environment (browser File System Access API doesn't)
- Supports both file and directory selection

**Implementation:**
```typescript
import { open } from '@tauri-apps/plugin-dialog';

// Directory picker
const selected = await open({
  directory: true,
  multiple: false,
});

// File picker
const selected = await open({
  filters: [{ name: 'Executable', extensions: ['exe'] }]
});
```

### Decision 6: Settings Initialization Flow
**Choice:** Lazy initialization with alert on failure

**Flow:**
1. App loads → attempt to read settings file
2. If file exists → load and use
3. If file missing → run auto-detection
4. If auto-detection succeeds → create settings file
5. If auto-detection fails → show alert + open modal
6. User configures manually → save settings

**Rationale:**
- First-run experience optimized for majority case (auto-detection works)
- Clear path to resolution when auto-detection fails
- Doesn't block app launch

**Alternatives considered:**
- Mandatory first-run wizard: Adds friction for users with standard installations
- Silent failure: Poor UX, user wouldn't know how to fix

### Decision 7: State Management
**Choice:** Lift settings state to `App.tsx` with prop drilling

**Rationale:**
- Settings needed by multiple components (mod list, installation, etc.)
- React Context would be overkill for single state object
- No external state library in project dependencies
- Simple to understand and maintain

**Future Migration Path:**
If settings usage grows complex, consider:
- React Context for settings
- Zustand for global state management
- Tauri state plugin for backend-owned state

## Risks / Trade-offs

### Risk 1: Auto-Detection False Positives
**Risk:** Detecting wrong installation (e.g., old uninstalled game with leftover files)

**Mitigation:**
- Validate presence of key game files, not just directory existence
- Allow manual override in settings UI
- Display detected path prominently so user can verify

### Risk 2: Plain Text Credential Storage
**Risk:** Nexus API keys stored in plain text on disk

**Mitigation (Current):**
- Document this limitation for users
- Config file is in user's app data directory (not world-readable)

**Mitigation (Future):**
- Encrypt credentials using OS keychain (requires new dependencies)
- Use OAuth flow instead of API keys (requires Nexus Mods integration work)

### Risk 3: Settings File Corruption
**Risk:** User manually edits JSON incorrectly, app fails to load settings

**Mitigation:**
- Robust error handling: log error, use defaults, notify user
- Settings validation on load (reject invalid enum values, paths)
- Document settings file format for manual editing

### Risk 4: Platform-Specific Path Edge Cases
**Risk:** Non-standard installations (custom Steam library, portable installs)

**Mitigation:**
- Manual path selection always available
- Document common non-standard paths
- Future enhancement: scan all mounted drives for game installations

## Migration Plan

**Initial Deployment:**
1. Users with hardcoded game path: Settings file created on first launch with current path
2. New users: Auto-detection runs, creates settings file
3. Existing mod configurations: Unaffected (settings are additive)

**Rollback:**
If settings system fails catastrophically:
1. App can fall back to hardcoded path from `project.md`
2. Users can delete settings file to reset to defaults
3. No data loss (mods and game files are untouched)

**Future Enhancements:**
- Settings schema versioning for migrations
- Import/export settings for backup
- Encrypted credential storage
- Auto-update detection for changed game paths

## Open Questions

1. **Should we store Steam library paths to improve detection?**
   - Pros: Better multi-library support
   - Cons: Adds complexity, may not be needed
   - Decision: Defer until we see user reports of detection failures

2. **Should settings include a "Don't ask again" option for auto-detection failures?**
   - Pros: Reduces notification spam for edge case installations
   - Cons: User might forget to configure paths later
   - Decision: Defer until we see user feedback on first-run UX

3. **Should we add settings validation on the backend or frontend?**
   - Current: Frontend validation for UX, backend re-validates for security
   - Alternative: Backend-only validation (simpler but worse UX)
   - Decision: Keep dual validation for security + UX
