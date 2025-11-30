# Change: Add Nexus Mods NXM Protocol Support

## Why

**User Pain Point:**
Currently, users must manually download mods from Nexus Mods through their browser, navigate to the downloads folder, and manually import them into the mod manager. This is tedious and error-prone.

**Solution:**
Integrate with Nexus Mods' official `nxm://` protocol handler, which is the legitimate, ToS-compliant method used by all major mod managers (Vortex, MO2, etc.).

**Why NXM Protocol?**
- ✅ Works with **free Nexus accounts** (no Premium required)
- ✅ Official, legitimate, ToS-compliant method
- ✅ Simple implementation (no web scraping or automation)
- ✅ Reliable (official protocol, won't break with site updates)
- ✅ Cross-platform support (Windows, macOS, Linux)
- ✅ One-click download experience

## What Changes

### New Capabilities

1. **NXM Protocol Handler** (`specs/nxm-protocol/`)
   - Register app as `nxm://` protocol handler on all platforms
   - Parse incoming `nxm://` URLs to extract mod metadata and auth key
   - Handle app launching/focusing when protocol is triggered

2. **Download Manager** (`specs/download-manager/`)
   - Queue system for managing multiple downloads
   - Progress tracking with speed/ETA calculation
   - Pause/resume/cancel functionality
   - Download persistence across app restarts

3. **Mod Installation** (`specs/mod-installation/`)
   - Auto-extract downloaded mod archives
   - Detect mod metadata from manifest.json
   - Add to mod list automatically
   - Handle installation to correct game mods folder

### Modified Components

- **Settings (`src-tauri/src/settings.rs`)**: Add download preferences
  - Download directory (default: `{app_data}/downloads`)
  - Concurrent downloads limit (default: 1 for free accounts)
  - Auto-install after download toggle

- **Mod List (`src/types/mod.ts`)**: Add download source tracking
  - Nexus mod ID and file ID
  - Download timestamp
  - Source URL

- **App Initialization (`src-tauri/src/lib.rs`)**: Register protocol handler
  - Deep link plugin integration
  - NXM URL event handling

## Impact

### New Dependencies

**Rust (Cargo.toml):**
- `tauri-plugin-deep-link` - Official Tauri plugin for protocol handling
- `reqwest` - HTTP client for downloading files (already used)
- `tokio` - Async runtime (already used)
- `zip` or `tar` - Archive extraction

**Frontend (package.json):**
- No new dependencies (use existing React state management)

### Files to Create

```
src-tauri/src/
├── nxm_protocol.rs      # NXM URL parsing and validation
├── download_manager.rs  # Download queue and progress tracking
└── mod_installer.rs     # Auto-extract and install logic

src/components/features/
└── downloads/
    ├── DownloadManager.tsx    # Download queue UI
    ├── DownloadItem.tsx       # Individual download progress
    └── DownloadNotification.tsx # Toast notifications

src/types/
└── download.ts          # TypeScript types for downloads
```

### Files to Modify

```
src-tauri/src/lib.rs              # Add protocol handler registration
src-tauri/src/settings.rs         # Add download settings
src-tauri/Cargo.toml              # Add new dependencies
src/App.tsx                       # Add download manager UI
src/types/mod.ts                  # Add source tracking fields
```

### Breaking Changes

**None.** This is a purely additive feature. Existing functionality remains unchanged.

## User Experience Flow

### First-Time Setup
1. User installs SDV Mods Manager
2. App registers as `nxm://` handler during installation
3. User logs into Nexus Mods on their browser (one-time)

### Download Flow
1. User browses Nexus Mods website
2. Navigates to a Stardew Valley mod page
3. Clicks "Files" tab
4. Clicks "Mod Manager Download" button
5. Browser shows "Open SDV Mods Manager?" prompt (one-time per browser)
6. App launches/focuses
7. Download starts automatically
8. Toast notification: "Downloading [Mod Name]..."
9. Progress shown in Download Manager panel
10. On completion: Auto-extracts and adds to mod list
11. Notification: "[Mod Name] installed successfully!"

### Download Manager UI
- Accessible via button in toolbar
- Shows active, queued, completed, and failed downloads
- Each item shows: mod name, progress bar, speed, ETA
- Actions: pause, resume, cancel, retry, open folder

## Success Criteria

1. User can click "Mod Manager Download" on Nexus and mod downloads automatically
2. Downloads work for users with free Nexus accounts
3. Multiple downloads can be queued (sequential for free, respecting rate limits)
4. Downloaded mods are automatically extracted and added to mod list
5. Download progress is visible and accurate
6. Failed downloads can be retried
7. App handles concurrent nxm:// URLs gracefully (queue them)
8. Works cross-platform (Windows, macOS, Linux)

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Nexus changes nxm:// format | Low | High | Monitor Nexus forums, implement flexible parsing |
| Download key expiration | Medium | Low | Show clear error message, link to re-download |
| Disk space exhaustion | Medium | Medium | Check available space before download, show warning |
| Corrupted downloads | Low | Medium | Verify file hash if provided, allow retry |
| Protocol handler conflicts | Low | Medium | Clear error if another app owns nxm://, offer instructions |

## Alternatives Considered

### Alternative 1: Direct URL Input
**Approach:** User pastes Nexus mod URL, app scrapes download link
- ❌ Violates Nexus ToS (web scraping)
- ❌ Requires browser automation (complex)
- ❌ Fragile (breaks with site changes)

### Alternative 2: Nexus API Only
**Approach:** Use official Nexus API for downloads
- ❌ Requires Premium account ($5/month)
- ❌ Excludes free users
- ✅ Faster downloads (uncapped speed)
- **Decision:** Offer as optional enhancement for Premium users in future

### Alternative 3: Manual Download + File Watcher
**Approach:** Watch downloads folder, auto-import new mods
- ❌ User must still manually click download
- ❌ No progress tracking
- ❌ Can't distinguish mod downloads from other files

### Why NXM Protocol Wins
- Works for free users
- Official and legitimate
- Simple and reliable
- Industry standard

## Migration Plan

### Phase 1: Foundation (Week 1)
- Implement NXM protocol registration
- Basic URL parsing and validation
- Simple single-file download

### Phase 2: Download Manager (Week 2)
- Queue system with progress tracking
- UI components for download list
- Pause/resume/cancel functionality

### Phase 3: Auto-Installation (Week 3)
- Archive extraction
- Metadata detection
- Auto-add to mod list

### Phase 4: Polish (Week 4)
- Error handling and retry logic
- Download persistence
- Toast notifications
- Settings integration

## Open Questions

1. **Download location:** Should we download to app data or allow user to choose?
   - **Proposal:** Default to `{app_data}/downloads/nexus`, with setting to change

2. **Auto-install vs. manual confirmation:** Should mods auto-install or require confirmation?
   - **Proposal:** Auto-install by default, add "Require confirmation before installing" toggle in settings

3. **Concurrent downloads for free users:** Nexus limits free users to 1 concurrent download. Should we enforce this?
   - **Proposal:** Yes, default to 1 concurrent. Premium users can increase to 3-5.

4. **Failed download retention:** How long should we keep failed downloads in the list?
   - **Proposal:** Keep indefinitely until user manually clears, allow "Clear completed" and "Clear failed" actions

## References

- [Nexus Mods NXM Protocol](https://github.com/TanninOne/nxmproxy)
- [Tauri Deep Link Plugin](https://github.com/tauri-apps/tauri/issues/323)
- [Mod Organizer 2 NXM Handler](https://github.com/TanninOne/modorganizer-nxmhandler)
- [Nexus Mods API Documentation](https://github.com/Nexus-Mods/node-nexus-api)
