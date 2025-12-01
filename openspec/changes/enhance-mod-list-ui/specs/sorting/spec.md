# Spec: Sorting

## ADDED Requirements

### Requirement: Column Sorting

#### Scenario: User sorts by column
- **Given** the mod list is displayed
- **When** the user clicks the "Name" column header
- **Then** the list should be sorted by mod name in ascending order
- **And** an "up" arrow indicator should appear next to the header

#### Scenario: User reverses sort order
- **Given** the list is already sorted by "Name" ascending
- **When** the user clicks the "Name" column header again
- **Then** the list should be sorted by mod name in descending order
- **And** a "down" arrow indicator should appear next to the header

#### Scenario: Default sort
- **Given** the app is loaded
- **Then** the list should be sorted by "Name" ascending by default (or "Status" if preferred)
