# Proposal: Enhanced Mod List UI

## Goal
Improve the usability of the mod list by adding pagination, sorting, and filtering capabilities. This will help users manage large collections of mods more efficiently.

## Problem
Currently, the mod list displays all mods in a single long list.
- **Performance**: Rendering a large number of mods can be slow.
- **Usability**: Finding specific mods is difficult without sorting or filtering (other than search).
- **Organization**: Users cannot easily see just their enabled or disabled mods.

## Solution
Implement a comprehensive set of list management features:
1.  **Pagination**: Break the list into pages with configurable rows per page (e.g., 10, 20, 50, 100).
2.  **Sorting**: Allow clicking on column headers (Name, Author, Status, etc.) to sort the list ascending/descending.
3.  **Filtering**: Activate the existing filter buttons in the toolbar to filter by status (All, Enabled, Disabled, Updates).

## Impact
- **Frontend**:
    - `ModList.tsx`: Add sorting headers and pagination controls.
    - `Toolbar.tsx`: Activate filter buttons.
    - `App.tsx`: Manage new state for `page`, `pageSize`, `sortColumn`, `sortDirection`, and `filterStatus`.
- **Backend**: No changes required (client-side processing for now).

## Risks
- **State Complexity**: Managing combinations of search, filter, sort, and pagination requires careful logic to ensure the view is always consistent (e.g., resetting to page 1 when filter changes).
