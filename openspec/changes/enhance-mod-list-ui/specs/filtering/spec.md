# Spec: Filtering

## ADDED Requirements

### Requirement: Status Filtering

#### Scenario: User filters by Enabled
- **Given** the mod list contains both enabled and disabled mods
- **When** the user clicks the "Enabled" filter button
- **Then** only enabled mods should be displayed
- **And** the "Enabled" button should appear active

#### Scenario: User filters by Disabled
- **Given** the mod list contains both enabled and disabled mods
- **When** the user clicks the "Disabled" filter button
- **Then** only disabled mods should be displayed

#### Scenario: User filters by Updates
- **Given** some mods have updates available
- **When** the user clicks the "Updates" (or "Config") filter button
- **Then** only mods with status `update-available` should be displayed

#### Scenario: Filter combines with Search
- **Given** the "Enabled" filter is active
- **When** the user types "Expanded" in the search bar
- **Then** only enabled mods containing "Expanded" in their name should be displayed
