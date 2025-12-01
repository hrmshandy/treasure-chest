# Spec: Pagination

## ADDED Requirements

### Requirement: Pagination Controls

#### Scenario: User navigates pages
- **Given** the mod list has more items than the current "rows per page" setting
- **When** the user clicks the "Next" button
- **Then** the list should show the next set of items
- **And** the "Previous" button should be enabled

#### Scenario: User changes rows per page
- **Given** the user is viewing the mod list
- **When** the user selects "50" from the "Rows per page" dropdown
- **Then** the list should display up to 50 items
- **And** the total number of pages should update accordingly
- **And** the current page should reset to 1 to avoid out-of-bounds errors

#### Scenario: Pagination resets on filter change
- **Given** the user is on page 3 of the mod list
- **When** the user changes the filter or search query
- **Then** the current page should reset to 1
