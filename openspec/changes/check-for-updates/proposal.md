# Check For Updates

## Context
The application already has UI elements for update checking (update button, status badges) but they're not functional. Mods installed from Nexus have their `nexus_mod_id` and `nexus_file_id` stored in `.nexus_meta` files, but the application doesn't check for newer versions.

## Problem
1. `handleUpdateMod` only logs to console
2. Mod `status` field is hardcoded to 'working'
3. No mechanism to fetch latest version from Nexus API
4. Users can't see when updates are available
5. Update indicator (↑) never shows

## Solution
1. **Backend Command**: `check_mod_updates` to query Nexus API for latest versions
2. **Version Comparison**: Compare installed version with latest using semver
3. **Status Update**: Set mod status to 'update-available' when newer version exists
4. **UI Integration**: Click update button to download and install latest version
5. **Batch Check**: "Check All Updates" button to scan all mods at once

## Impact
- **Backend**: New `check_mod_updates` command, Nexus API integration for mod info
- **Frontend**: Working update button, batch update checker
- **User Experience**: Clear visibility of available updates, one-click updates
