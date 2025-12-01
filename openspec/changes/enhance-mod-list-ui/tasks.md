# Tasks: Enhanced Mod List UI

- [ ] **State Management** <!-- id: 0 -->
    - [ ] Add `filterStatus` state (All, Enabled, Disabled, Updates) to `App.tsx` <!-- id: 1 -->
    - [ ] Add `sortConfig` state (key, direction) to `App.tsx` <!-- id: 2 -->
    - [ ] Add `pagination` state (currentPage, itemsPerPage) to `App.tsx` <!-- id: 3 -->
    - [ ] Implement `useMemo` hook to process mods (filter -> sort -> paginate) <!-- id: 4 -->

- [ ] **Filtering** <!-- id: 5 -->
    - [ ] Update `Toolbar.tsx` to accept `filterStatus` and `onFilterChange` props <!-- id: 6 -->
    - [ ] Connect Toolbar filter buttons to state <!-- id: 7 -->

- [ ] **Sorting** <!-- id: 8 -->
    - [ ] Update `ModList.tsx` to accept `sortConfig` and `onSort` props <!-- id: 9 -->
    - [ ] Make table headers clickable with sort indicators (↑/↓) <!-- id: 10 -->

- [ ] **Pagination** <!-- id: 11 -->
    - [ ] Create `Pagination` component (or add to `ModList` footer) <!-- id: 12 -->
    - [ ] Add "Rows per page" selector <!-- id: 13 -->
    - [ ] Connect pagination controls to state <!-- id: 14 -->

- [ ] **Verification** <!-- id: 15 -->
    - [ ] Verify filtering works with search <!-- id: 16 -->
    - [ ] Verify sorting works on all columns <!-- id: 17 -->
    - [ ] Verify pagination updates correctly when filtering/sorting changes <!-- id: 18 -->
