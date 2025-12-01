# Tasks

- [x] Backend: Implement `delete_mod` command <!-- id: 0 -->
    - [x] Accept `mod_path` parameter
    - [x] Use `force_remove_dir_all` for robust deletion
    - [x] Return success/error
- [x] Frontend: Update `handleDeleteMod` in `App.tsx` <!-- id: 1 -->
    - [x] Call `delete_mod` backend command
    - [x] Show confirmation dialog before deletion
    - [x] Trigger `loadMods()` after successful deletion
    - [x] Show error toast on failure
- [x] Verify: Test deletion functionality <!-- id: 2 -->
