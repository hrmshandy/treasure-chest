# Improve Delete Functionality

## Context
Currently, the `handleDeleteMod` function only removes mods from the application state, not from the actual filesystem. Additionally, when users manually delete mod folders, the application doesn't detect this change and continues to show the deleted mods in the list.

## Problem
1. Clicking "Delete" in the UI only removes from React state
2. Mod folders remain on disk after "deletion"
3. Manual folder deletions are not reflected in the UI until app restart/rescan

## Solution
1. **Backend Command**: Create `delete_mod` command that removes the mod folder from disk
2. **Delete Logic**: Use the existing `force_remove_dir_all` function from `mod_installer.rs` for robust deletion
3. **Frontend Integration**: Update `handleDeleteMod` to call backend command
4. **Auto-Refresh**: After deletion, trigger a rescan to update the listing

## Impact
- **Backend**: New `delete_mod` command in `lib.rs`
- **Frontend**: Update `handleDeleteMod` in `App.tsx` to call backend and rescan
- **User Experience**: Mods are truly deleted from disk, and manual deletions are reflected on next scan
