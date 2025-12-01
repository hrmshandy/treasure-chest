# Mod Management

## ADDED Requirements

## ADDED Requirements

### Requirement: Enable/Disable Mods

The system MUST allow users to enable and disable installed mods.

#### Scenario: Disable a mod
- **Given** an enabled mod "Content Patcher" at `Mods/Content Patcher`
- **When** the user disables the mod
- **Then** the folder is renamed to `Mods/Content Patcher.disabled`
- **And** the mod status becomes "disabled"

#### Scenario: Enable a mod
- **Given** a disabled mod "Content Patcher" at `Mods/Content Patcher.disabled`
- **When** the user enables the mod
- **Then** the folder is renamed to `Mods/Content Patcher`
- **And** the mod status becomes "working"

#### Scenario: Scan disabled mods
- **Given** a folder `Mods/Content Patcher.disabled` containing a valid manifest
- **When** the application scans for mods
- **Then** it detects "Content Patcher"
- **And** its `isEnabled` property is `false`

## MODIFIED Requirements

### Requirement: Mod Scanning

The system MUST recognize mods with `.disabled` suffix as valid but disabled mods.

#### Scenario: Detect disabled suffix
- **Given** a mod folder ending in `.disabled`
- **When** scanning mods
- **Then** the mod is included in the list
- **And** it is marked as disabled
