# Change: Improve Mod Management Operations

## Why

**User Pain Points:**

1. **Multi-folder mods appear as separate entries**: Mods like Stardew Valley Expanded contain 3 separate mod folders ([CP], [FTM], and Code) that show up as 3 unrelated entries in the mod list. Users can't tell they're related and may accidentally disable only part of the mod.

2. **Delete only removes from UI**: Clicking delete on a mod only removes it from the app's state, but the actual folder remains on disk. Users expect delete to actually remove the mod files.

3. **Manual changes not detected**: If users manually delete mod folders from the file system, the app still shows them as enabled. This creates confusion about which mods are actually installed.

4. **No enable/disable functionality**: Users cannot temporarily disable mods without uninstalling them. SMAPI uses a `.disabled` suffix convention, but the app doesn't support toggling this.

**Impact:** These issues make the mod manager feel incomplete and unreliable. Users lose trust when the UI doesn't match reality, and basic operations like delete don't work as expected.

## What Changes

### New Capabilities

1. **Mod Groups** (`specs/mod-groups/`)
   - Track which mods came from the same download/archive
   - Display grouped mods visually in the UI
   - Apply operations (delete, toggle) to entire groups
   - Show relationship between related mod folders

2. **Mod Toggle** (`specs/mod-toggle/`)
   - Enable/disable mods by renaming folders with `.disabled` suffix
   - Update SMAPI to recognize changes
   - Show accurate enabled/disabled state in UI
   - Apply toggle to mod groups atomically

3. **File System Sync** (`specs/file-sync/`)
   - Detect when mod folders are manually deleted
   - Detect when mods are manually added to Mods folder
   - Detect when mods are manually renamed (enabled/disabled)
   - Update app state to match file system reality

4. **Mod Deletion** (`specs/mod-deletion/`)
   - Actually delete mod folders from disk when user clicks delete
   - Show confirmation dialog before deletion
   - Handle deletion failures gracefully
   - Support group deletion (delete all related mods)

### Modified Components

- **Mod Model (`src-tauri/src/models.rs`)**: Add group tracking
  - `group_id: Option<String>` - UUID linking related mods
  - `install_source: Option<String>` - "nexus", "manual", etc.
  - `download_id: Option<String>` - Link back to download task

- **Mod Scanner (`src-tauri/src/mod_scanner.rs`)**: Enhance detection
  - Detect `.disabled` suffix and set `is_enabled` accordingly
  - Return accurate state matching file system
  - Detect when previously-scanned mods no longer exist

- **Mod List UI (`src/components/features/mods/ModList.tsx`)**: Show groups
  - Visual grouping indicator (collapsible sections)
  - Group operations (toggle/delete affects all members)
  - Show which mods are related

- **Toggle Handler (`src-tauri/src/mod_toggle.rs`)**: New module
  - Rename folder to add/remove `.disabled` suffix
  - Handle group toggles atomically
  - Emit events to frontend

- **Delete Handler (`src-tauri/src/commands.rs`)**: Fix existing logic
  - Actually delete folders from disk (currently only updates UI)
  - Show confirmation dialog
  - Handle group deletions

### Breaking Changes

**None.** All changes are additive or fix broken behavior. Existing mods without group_id will continue to work as individual entries.

## Impact

### Affected Specs
- **New:** `mod-groups`, `mod-toggle`, `file-sync`, `mod-deletion`
- **Modified:** `mod-installation` (add group tracking on install)

### Affected Code

**Backend (Rust):**
```
src-tauri/src/
├── models.rs              # Add group_id, install_source, download_id fields
├── mod_scanner.rs         # Detect .disabled suffix, detect missing folders
├── mod_toggle.rs          # NEW: Enable/disable logic
├── commands.rs            # Fix delete_mod to actually delete folders
└── mod_installer.rs       # Track group_id when installing multi-folder mods
```

**Frontend (React/TypeScript):**
```
src/
├── types/mod.ts           # Add groupId, installSource, downloadId fields
├── components/features/mods/
│   ├── ModList.tsx        # Show grouped mods
│   ├── ModGroup.tsx       # NEW: Collapsible group component
│   └── ModItem.tsx        # Add toggle button, fix delete
└── App.tsx                # Implement handleToggleMod, fix handleDeleteMod
```

## User Experience Flow

### Enable/Disable Flow
1. User clicks toggle switch next to mod
2. If mod is part of a group: "This will disable all 3 related mods. Continue?"
3. User confirms
4. App renames all folders to add `.disabled` suffix
5. UI updates to show disabled state
6. SMAPI automatically detects change on next game launch

### Delete Flow
1. User clicks delete button
2. If mod is part of a group: "Delete all 3 related mods from Stardew Valley Expanded?"
3. User confirms
4. App deletes all folders from disk
5. App removes from state
6. UI updates to remove mods from list

### Sync Flow
1. User manually deletes a mod folder from file system
2. User switches back to app window
3. App detects folder is missing (on window focus or periodic check)
4. App removes mod from list automatically
5. No error shown (user clearly intended to delete it)

### Multi-Folder Install Flow
1. User downloads Stardew Valley Expanded from Nexus
2. App extracts and finds 3 mod folders
3. App generates group_id for this installation
4. All 3 mods get same group_id and download_id
5. UI shows them as a group: "📦 Stardew Valley Expanded (3 mods)"
6. Expanding shows: [CP] Stardew Valley Expanded, [FTM] Stardew Valley Expanded, Stardew Valley Expanded

## Success Criteria

1. ✅ Multi-folder mods from same download are visually grouped in UI
2. ✅ Toggling a group disables/enables all member mods atomically
3. ✅ Deleting a mod actually removes the folder from disk
4. ✅ Deleting a group removes all member folders
5. ✅ Manually deleted mods disappear from app UI automatically
6. ✅ Manually added mods appear in app UI automatically
7. ✅ Manually renaming folder to `.disabled` updates toggle state in app
8. ✅ All operations work reliably across Windows, macOS, Linux

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Folder deletion cannot be undone | High | High | Show confirmation dialog, consider recycle bin integration |
| File system permissions prevent deletion | Medium | Medium | Show clear error, offer to open folder in file manager |
| Sync causes performance issues | Low | Medium | Use file system watchers instead of polling, debounce events |
| Group operations fail partially | Medium | High | Implement atomic operations with rollback on failure |
| User manually moves mod between groups | Low | Low | Document that groupId is immutable after install |

## Alternatives Considered

### Alternative 1: Wrapper Folder
**Approach:** Install multi-folder mods into a wrapper folder like `TC Installed Mods/SVE/[CP]`, `TC Installed Mods/SVE/[FTM]`

❌ **Rejected:** SMAPI only scans direct children of Mods folder, not nested folders. This would break the mods.

### Alternative 2: Metadata File
**Approach:** Store group relationships in external `groups.json` file instead of in mod metadata

❌ **Rejected:** More complex, requires separate file management, harder to keep in sync.

### Alternative 3: Tags Instead of Groups
**Approach:** Let users manually tag mods to organize them

❌ **Rejected:** Doesn't solve the automatic detection problem. Users would have to manually tag 3 folders as related every time they install SVE.

### Why Group Tracking Wins
- ✅ Automatic: no user action needed
- ✅ Reliable: tracks actual installation relationship
- ✅ Simple: just one field in existing mod metadata
- ✅ Backwards compatible: existing mods work fine without it

## Migration Plan

### Phase 1: Core Infrastructure (Week 1)
- Add group tracking fields to Mod model
- Update mod_installer to generate group_id for multi-folder installs
- Implement mod_toggle module with folder rename logic

### Phase 2: UI Integration (Week 1)
- Create ModGroup component for collapsible groups
- Update ModList to display grouped mods
- Add toggle buttons and wire up backend commands

### Phase 3: File System Sync (Week 2)
- Implement file system watcher
- Detect manual additions/deletions/renames
- Update state automatically

### Phase 4: Delete Fix (Week 2)
- Fix delete_mod command to actually delete folders
- Add confirmation dialogs
- Handle group deletions

### Phase 5: Polish (Week 2)
- Error handling for permission issues
- Loading states for async operations
- Testing across platforms

## Open Questions

1. **File system watcher vs polling:** Should we use notify crate (file system events) or periodic scanning?
   - **Proposal:** Use notify crate with 1-second debounce. More efficient than polling.

2. **Recycle bin integration:** Should delete send to recycle bin instead of permanent deletion?
   - **Proposal:** Yes on Windows/macOS (use trash crate), fallback to permanent delete on Linux or if trash fails.

3. **Group display:** How should groups be shown in the UI?
   - **Proposal:** Collapsible sections with parent row showing group name and count, child rows showing individual mods.

4. **Partial group operations:** What if user wants to disable only one mod in a group?
   - **Proposal:** Show "Ungroup" action that removes the mod from the group, making it independent.

## References

- [SMAPI Mod Folder Structure](https://stardewvalleywiki.com/Modding:Modder_Guide/Get_Started#Folder_structure)
- [SMAPI Disabled Mods Convention](https://github.com/Pathoschild/SMAPI/blob/develop/docs/technical/mod-package.md#disabled-mods)
- [Rust notify crate](https://github.com/notify-rs/notify) - File system watcher
- [Rust trash crate](https://github.com/Byron/trash-rs) - Cross-platform recycle bin
