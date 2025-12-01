# Bulk Mod Actions

## Context
Currently, users can only perform actions (enable, disable, delete) on one mod at a time. This is inefficient when managing large mod lists.

## Problem
1.  Managing many mods is tedious.
2.  No way to select multiple mods.
3.  No way to perform batch operations.

## Solution
1.  **Selection UI**: Add checkboxes to the mod list (header for "select all", rows for individual selection).
2.  **Bulk Actions Toolbar**: Show a toolbar when items are selected with options:
    *   **Enable Selected**: Enables all selected mods.
    *   **Disable Selected**: Disables all selected mods.
    *   **Delete Selected**: Deletes all selected mods (with confirmation).
3.  **State Management**: Track selected mod IDs in the main application state.

## Impact
-   **Frontend**: `ModList` update for checkboxes, new `BulkActions` component (or update to `Toolbar`), `App.tsx` state logic.
-   **Backend**: No changes required (can reuse existing single-mod commands in a loop or parallel).
-   **User Experience**: Significantly faster mod management.
