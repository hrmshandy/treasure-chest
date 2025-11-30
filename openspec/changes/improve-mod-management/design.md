# Design: Mod Management Improvements

## Context

The current mod management system has several gaps that create a poor user experience:
1. Multi-folder mods (common for large content packs) are shown as separate, unrelated entries
2. Delete operations don't actually remove files from disk
3. Manual file system changes (additions, deletions, renames) are not reflected in the app
4. No way to temporarily disable mods without uninstalling

These issues stem from the initial implementation focusing on basic mod listing without considering the full lifecycle of mod management operations.

**Stakeholders:**
- End users: Need reliable, predictable mod management
- Mod authors: Expect SMAPI conventions (.disabled suffix) to be supported
- Developers: Need maintainable, testable code for file operations

**Constraints:**
- Must respect SMAPI folder conventions (no nested folders in Mods/)
- Must work across Windows, macOS, Linux with different file systems
- Cannot break existing mods installed before this change
- File operations must be reversible where possible (recycle bin)

## Goals / Non-Goals

### Goals
- ✅ Automatically group mods installed from the same archive
- ✅ Show accurate mod state reflecting file system reality
- ✅ Support SMAPI's `.disabled` suffix convention
- ✅ Actually delete files when user requests deletion
- ✅ Detect manual file system changes within 1-2 seconds
- ✅ Handle errors gracefully with actionable messages

### Non-Goals
- ❌ Manual tagging or organization systems (keep it automatic)
- ❌ Backup/restore functionality (separate feature)
- ❌ Mod conflict detection (separate feature)
- ❌ Version control or rollback (separate feature)
- ❌ Cloud sync of mod configurations

## Decisions

### Decision 1: Group Tracking via UUID

**What:** Add `group_id: Option<String>` field to Mod model. All mods from the same archive get the same UUID.

**Why:**
- Simple: Just one field, generated once at install time
- Reliable: Immutable after creation, always tracks true relationship
- Backwards compatible: Existing mods work fine with `None`
- No external dependencies: No separate groups.json file to manage

**Alternatives considered:**
- External groups.json file → More complex, harder to keep in sync
- Hash-based grouping → Can't determine relationship for manually-installed mods
- User-defined tags → Requires manual work, doesn't solve auto-detection

**Implementation:**
```rust
// In models.rs
pub struct Mod {
    pub id: String,
    pub name: String,
    // ... existing fields ...
    pub group_id: Option<String>,      // NEW: UUID linking related mods
    pub install_source: Option<String>, // NEW: "nexus", "manual", "import"
    pub download_id: Option<String>,    // NEW: Link back to download task
}
```

### Decision 2: File System Watcher (notify crate)

**What:** Use the `notify` crate to watch Mods folder for changes in real-time.

**Why:**
- Efficient: OS-level file events, no polling overhead
- Fast: Sub-second detection of changes
- Cross-platform: Works on Windows, macOS, Linux
- Well-maintained: Popular crate with 5.6M downloads

**Alternatives considered:**
- Polling every 5 seconds → Wastes CPU, delays detection
- Only scan on window focus → Misses changes while app is active
- Manual refresh button → Poor UX, users will forget

**Implementation:**
```rust
use notify::{Watcher, RecursiveMode, watcher};
use std::time::Duration;

// Watch Mods folder
let (tx, rx) = channel();
let mut watcher = watcher(tx, Duration::from_secs(1))?;
watcher.watch(mods_path, RecursiveMode::NonRecursive)?;

// Handle events with debouncing
for event in rx {
    match event {
        DebouncedEvent::Create(path) => handle_mod_added(path),
        DebouncedEvent::Remove(path) => handle_mod_removed(path),
        DebouncedEvent::Rename(old, new) => handle_mod_renamed(old, new),
        _ => {}
    }
}
```

**Debouncing:** 1-second debounce prevents processing rapid-fire events (e.g., when extracting many files).

### Decision 3: Toggle via Folder Rename

**What:** Enable/disable mods by appending/removing `.disabled` suffix from folder name.

**Why:**
- SMAPI standard: This is the official SMAPI convention
- Simple: Just a folder rename, no manifest editing
- Instant: SMAPI detects on next launch automatically
- Reversible: Easy to undo

**Alternatives considered:**
- Edit manifest.json → More complex, risk corrupting file
- Move to separate disabled folder → Breaks relative paths in mod files
- Hidden files/attributes → Not portable across platforms

**Implementation:**
```rust
pub async fn toggle_mod(mod_path: &Path, enable: bool) -> Result<(), Error> {
    let folder_name = mod_path.file_name().unwrap().to_str().unwrap();

    if enable {
        // Remove .disabled suffix
        if folder_name.ends_with(".disabled") {
            let new_name = folder_name.trim_end_matches(".disabled");
            let new_path = mod_path.with_file_name(new_name);
            fs::rename(mod_path, new_path)?;
        }
    } else {
        // Add .disabled suffix
        if !folder_name.ends_with(".disabled") {
            let new_name = format!("{}.disabled", folder_name);
            let new_path = mod_path.with_file_name(new_name);
            fs::rename(mod_path, new_path)?;
        }
    }
    Ok(())
}
```

### Decision 4: Delete to Recycle Bin

**What:** Use `trash` crate to send deleted mods to recycle bin instead of permanent deletion.

**Why:**
- Reversible: Users can recover accidentally deleted mods
- Safe: Standard OS behavior users expect
- Cross-platform: Works on Windows, macOS, Linux

**Alternatives considered:**
- Permanent deletion → Cannot undo mistakes
- Manual backup before delete → Complex, disk space issues
- Confirmation dialog only → Still risky for accidents

**Implementation:**
```rust
use trash;

pub async fn delete_mod(mod_path: &Path) -> Result<(), Error> {
    // Try to move to trash first
    match trash::delete(mod_path) {
        Ok(_) => Ok(()),
        Err(e) => {
            // Fallback to permanent delete if trash fails
            eprintln!("Failed to move to trash: {}. Deleting permanently.", e);
            fs::remove_dir_all(mod_path)?;
            Ok(())
        }
    }
}
```

**Fallback:** If trash fails (e.g., on some Linux systems), fall back to permanent deletion with warning.

### Decision 5: Atomic Group Operations

**What:** When toggling/deleting a group, apply changes atomically - all succeed or all roll back.

**Why:**
- Consistency: Prevents partial states (some mods disabled, others enabled)
- Reliability: Users can trust operations complete fully
- Error handling: Clear failure messages, no cleanup needed

**Implementation:**
```rust
pub async fn toggle_mod_group(group_id: &str, mods: Vec<Mod>, enable: bool) -> Result<(), Error> {
    // Phase 1: Validate all paths exist and are writable
    for mod in &mods {
        let path = Path::new(&mod.path);
        if !path.exists() {
            return Err(Error::ModNotFound(mod.path.clone()));
        }
        // Test write permission
        if !is_writable(path) {
            return Err(Error::PermissionDenied(mod.path.clone()));
        }
    }

    // Phase 2: Perform renames
    let mut completed = Vec::new();
    for mod in &mods {
        let path = Path::new(&mod.path);
        match toggle_mod(path, enable).await {
            Ok(_) => completed.push(path),
            Err(e) => {
                // Rollback: Reverse all completed renames
                for completed_path in completed {
                    let _ = toggle_mod(completed_path, !enable).await;
                }
                return Err(e);
            }
        }
    }

    Ok(())
}
```

### Decision 6: UI Grouping with Collapsible Sections

**What:** Display grouped mods as collapsible parent/child rows in the mod list.

**Why:**
- Familiar: Standard UI pattern users understand
- Compact: Reduces clutter for multi-folder mods
- Clear: Shows relationship at a glance
- Flexible: Users can expand/collapse as needed

**Alternatives considered:**
- Side-by-side columns → Wastes horizontal space
- Separate "Groups" tab → Splits related info
- Badges only → Doesn't reduce clutter

**UI Mockup:**
```
┌─────────────────────────────────────────────────────┐
│ 📦 Stardew Valley Expanded (3 mods)      [Toggle] [Delete]
│ ├─ [CP] Stardew Valley Expanded          ✓ Enabled
│ ├─ [FTM] Stardew Valley Expanded         ✓ Enabled
│ └─ Stardew Valley Expanded               ✓ Enabled
└─────────────────────────────────────────────────────┘
```

**Behavior:**
- Clicking toggle on parent row affects all children
- Clicking toggle on child row only affects that mod + shows "Ungroup" option
- Clicking delete on parent row shows "Delete all 3 mods?"
- Clicking delete on child row shows "Delete just this mod?" + "Ungroup"

## Risks / Trade-offs

### Risk 1: File Watcher Resource Usage
**Risk:** notify crate watching Mods folder could use significant CPU/memory on folders with many mods.

**Mitigation:**
- Use NonRecursive mode (only watch Mods folder, not subfolders)
- Debounce events to 1 second
- Lazy initialization: only start watcher after first mod scan completes
- Monitoring: Log watcher performance in debug mode

**Trade-off:** Small constant CPU overhead for real-time sync.

### Risk 2: Partial Group Operation Failures
**Risk:** When toggling/deleting a group, some operations might fail (permissions, locked files).

**Mitigation:**
- Atomic operations: Validate all paths before starting, rollback on any failure
- Clear error messages: "Cannot disable Stardew Valley Expanded: [CP] folder is locked by SMAPI"
- Offer partial completion: "Disabled 2 of 3 mods. Continue with remaining mod?"

**Trade-off:** More complex code for rollback logic.

### Risk 3: Trash Integration Failures
**Risk:** trash crate might not work on all Linux distros or in sandboxed environments.

**Mitigation:**
- Fallback to permanent deletion with warning
- Log trash failures for debugging
- Add setting: "Always delete permanently (skip recycle bin)"

**Trade-off:** Less safety on systems where trash doesn't work.

### Risk 4: Accidental Ungrouping
**Risk:** Users might accidentally ungroup mods, losing the relationship.

**Mitigation:**
- Require explicit "Ungroup" action (don't make it easy to trigger accidentally)
- Show warning: "This mod is part of Stardew Valley Expanded. Ungroup it?"
- Document that ungrouping is permanent

**Trade-off:** Extra clicks for intentional ungrouping.

## Migration Plan

### Phase 1: Data Model (1-2 days)
1. Add `group_id`, `install_source`, `download_id` fields to Mod struct
2. Add serde annotations for serialization
3. Update existing mods to have `None` for new fields
4. Test backwards compatibility

### Phase 2: Group Tracking on Install (1 day)
1. Modify `install_from_archive()` to generate UUID when installing multiple mods
2. Assign same `group_id` to all mods from archive
3. Set `install_source` to "nexus" or "manual"
4. Link `download_id` if from download manager

### Phase 3: Toggle Implementation (2 days)
1. Create `mod_toggle.rs` module
2. Implement folder rename logic
3. Add `toggle_mod` and `toggle_mod_group` commands
4. Handle errors (permissions, locked files)
5. Add unit tests

### Phase 4: Delete Fix (1 day)
1. Add `trash` crate dependency
2. Modify `delete_mod` command to actually delete folders
3. Implement recycle bin integration
4. Add confirmation dialogs in frontend
5. Handle group deletions

### Phase 5: File System Watcher (2-3 days)
1. Add `notify` crate dependency
2. Implement watcher initialization
3. Handle Create, Remove, Rename events
4. Debounce event processing
5. Update app state on changes
6. Test on all platforms

### Phase 6: UI Components (2-3 days)
1. Create ModGroup component
2. Update ModList to render groups
3. Add toggle buttons to UI
4. Wire up delete confirmation dialogs
5. Handle loading/error states
6. Style grouped rows

### Phase 7: Testing & Polish (2 days)
1. Test on Windows, macOS, Linux
2. Test with various mod structures
3. Test edge cases (permissions, locked files, network drives)
4. Fix bugs
5. Update documentation

**Total Estimated Time:** 10-14 days

### Backwards Compatibility

**Existing Mods:** All mods installed before this change will have `group_id: None` and appear as individual entries (no change in behavior).

**Future Installs:** New multi-folder mod installs will automatically get grouped.

**Manual Migration:** Users can manually install mods multiple times to get grouping (not recommended, but possible).

## Open Questions

### Q1: Should we allow manual grouping?
**Options:**
- A) Auto-grouping only (current design)
- B) Add "Create Group" action to manually group selected mods
- C) Add "Add to Group" action to add mod to existing group

**Current Decision:** A (auto-grouping only). Keep it simple for v1. Can add manual grouping in v2 if users request it.

**Rationale:** 99% of use cases are handled by auto-grouping multi-folder installs. Manual grouping adds complexity without clear benefit.

---

### Q2: How to handle manually-installed multi-folder mods?
**Scenario:** User manually copies 3 mod folders (e.g., via file manager, not through app).

**Options:**
- A) Detect patterns in folder names (e.g., "[CP]", "[FTM]" prefixes) and auto-group
- B) Don't auto-group, treat as separate mods
- C) Offer "Detect Groups" action in UI

**Current Decision:** B (don't auto-group). File system watcher will detect them as 3 separate additions.

**Rationale:** Too risky to auto-group based on naming patterns. User can use "Create Group" if we add manual grouping later.

---

### Q3: Should group toggle be atomic or partial?
**Scenario:** Toggling a group fails for one mod (e.g., file locked).

**Options:**
- A) Atomic: Rollback all changes, show error
- B) Partial: Disable successful mods, show warning about failures
- C) Ask user: "2 of 3 mods disabled successfully. Retry the failed mod?"

**Current Decision:** A (atomic). Consistency is more important than partial success.

**Rationale:** Prevents confusing partial states. Users can retry the operation after resolving the issue.

---

### Q4: File watcher on window focus or always running?
**Options:**
- A) Start watcher on app startup, run continuously
- B) Start watcher when window gains focus, stop when loses focus
- C) Manual refresh button only

**Current Decision:** A (always running).

**Rationale:** Real-time sync is worth the small CPU overhead. Users might make changes while app is in background.

**Optimization:** Use notify's debouncing to reduce event processing overhead.
