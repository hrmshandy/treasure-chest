# Enable/Disable Mods

## Context
Currently, the application scans mods but does not provide a way to enable or disable them from the UI. The existing convention (documented in `project.md`) suggests prepending `.` to disable mods, but the user has explicitly requested using a `.disabled` suffix.

## Problem
Users need a way to temporarily disable mods without deleting them.

## Solution
Implement `enable_mod` and `disable_mod` commands in the backend that rename mod folders.
- **Disable**: Append `.disabled` to the folder name.
- **Enable**: Remove `.disabled` from the folder name.
- **Scan**: Update `scan_mods` to recognize folders ending in `.disabled` as disabled mods.

## Impact
- **Backend**: New commands in `lib.rs` or `mod_installer.rs`. Update `scan_mods`.
- **Frontend**: Connect `Toggle` switch in `ModList` to these commands.
- **Conventions**: Updates the project convention for disabling mods.
