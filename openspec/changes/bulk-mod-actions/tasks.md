# Tasks

- [x] Frontend: Add selection state to `App.tsx` <!-- id: 0 -->
    - [x] `selectedModIds` state (array of strings)
    - [x] `handleSelectMod` (toggle single)
    - [x] `handleSelectAll` (toggle all)
- [x] Frontend: Update `ModList.tsx` with checkboxes <!-- id: 1 -->
    - [x] Add checkbox column to header (Select All)
    - [x] Add checkbox column to rows
    - [x] Bind to `selectedModIds` and handlers
- [x] Frontend: Create Bulk Actions UI <!-- id: 2 -->
    - [x] Floating toolbar or integrated into existing toolbar
    - [x] Show only when selection > 0
    - [x] Buttons: Enable, Disable, Delete
- [x] Frontend: Implement Bulk Logic in `App.tsx` <!-- id: 3 -->
    - [x] `handleBulkEnable`: Loop through selected and call `toggle_mod_enabled`
    - [x] `handleBulkDisable`: Loop through selected and call `toggle_mod_enabled`
    - [x] `handleBulkDelete`: Show confirmation, then loop/batch delete
- [ ] Verify: Test bulk enable, disable, and delete <!-- id: 4 -->
