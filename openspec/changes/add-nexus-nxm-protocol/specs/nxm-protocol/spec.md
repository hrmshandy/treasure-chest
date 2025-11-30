# Spec: NXM Protocol Handler

## Overview

Register the application as the system handler for `nxm://` protocol URLs, enabling seamless integration with Nexus Mods' "Mod Manager Download" feature. Parse incoming URLs, validate authentication, and trigger the download process.

## ADDED Requirements

### Requirement: Protocol Registration

The application SHALL register itself as the handler for the `nxm://` custom URL scheme on all supported platforms.

#### Scenario: First-time installation registers handler

- **WHEN** the application is being installed for the first time
- **AND** no other application has registered the `nxm://` protocol
- **THEN** the application SHALL be registered as the default handler for `nxm://` URLs
- **AND** the registration SHALL persist across system reboots

#### Scenario: Update preserves protocol registration

- **WHEN** the application is updated to a new version
- **THEN** the protocol registration SHALL remain intact
- **AND** clicking nxm:// links SHALL still launch the updated application

#### Scenario: Conflicting handler detected

- **WHEN** the user attempts to register this application
- **THEN** the application SHALL display a warning message
- **AND** the message SHALL explain how to manually set the default handler
- **AND** the application SHALL provide instructions specific to the user's OS

#### Scenario: Uninstall removes protocol registration

- **WHEN** the application is uninstalled
- **THEN** the protocol registration SHALL be removed from the system
- **AND** clicking nxm:// links SHALL no longer launch the application

### Requirement: Deep Link Reception

The application SHALL receive and process `nxm://` URLs when triggered by the browser or system.

#### Scenario: App not running when nxm:// link clicked

- **WHEN** the user clicks an nxm:// link in their browser
- **THEN** the application SHALL launch automatically
- **AND** the application SHALL receive the full nxm:// URL
- **AND** the application SHALL process the URL after launch completes

#### Scenario: App already running when nxm:// link clicked

- **WHEN** the user clicks an nxm:// link in their browser
- **THEN** the application SHALL come to foreground/focus
- **AND** the application SHALL receive the nxm:// URL
- **AND** the application SHALL process the URL immediately

#### Scenario: Multiple nxm:// links clicked rapidly

- **WHEN** the browser sends multiple nxm:// URLs to the application
- **THEN** the application SHALL queue all URLs for processing
- **AND** each URL SHALL be processed in the order received
- **AND** no URLs SHALL be lost or ignored

### Requirement: URL Parsing

The application SHALL parse `nxm://` URLs to extract mod metadata and authentication parameters.

#### Scenario: Parse valid Stardew Valley nxm URL

- **WHEN** the parser processes the URL `nxm://stardewvalley/mods/2400/files/9567?key=abc123&expires=1735344000`
- **THEN** the game SHALL be extracted as "stardewvalley"
- **AND** the mod_id SHALL be extracted as 2400
- **AND** the file_id SHALL be extracted as 9567
- **AND** the key SHALL be extracted as "abc123"
- **AND** the expires timestamp SHALL be extracted as 1735344000

#### Scenario: Parse nxm URL without expiration

- **WHEN** the parser processes the URL `nxm://stardewvalley/mods/2400/files/9567?key=abc123`
- **THEN** the URL SHALL parse successfully
- **AND** the expires field SHALL be None/null
- **AND** the URL SHALL be considered non-expiring

#### Scenario: Parse nxm URL with user_id

- **WHEN** the parser processes the URL `nxm://stardewvalley/mods/2400/files/9567?key=abc123&user_id=12345`
- **THEN** the user_id SHALL be extracted as 12345
- **AND** the user_id SHALL be stored for tracking purposes

#### Scenario: Reject URL for wrong game

- **WHEN** the parser processes the URL `nxm://skyrim/mods/1234/files/5678?key=abc123`
- **THEN** the parser SHALL reject the URL
- **AND** an error SHALL be returned indicating "Game not supported: skyrim"
- **AND** a user-friendly message SHALL be displayed

#### Scenario: Reject malformed URL - missing key

- **WHEN** the parser processes the URL `nxm://stardewvalley/mods/2400/files/9567`
- **THEN** the parser SHALL reject the URL
- **AND** an error SHALL be returned indicating "Missing authentication key"

#### Scenario: Reject malformed URL - invalid mod ID

- **WHEN** the parser processes the URL `nxm://stardewvalley/mods/abc/files/9567?key=test`
- **THEN** the parser SHALL reject the URL
- **AND** an error SHALL be returned indicating "Invalid mod ID format"

#### Scenario: Reject malformed URL - invalid file ID

- **WHEN** the parser processes the URL `nxm://stardewvalley/mods/2400/files/xyz?key=test`
- **THEN** the parser SHALL reject the URL
- **AND** an error SHALL be returned indicating "Invalid file ID format"

### Requirement: Expiration Validation

The application SHALL validate that nxm:// URLs with expiration timestamps have not expired before processing.

#### Scenario: URL not yet expired

- **WHEN** the expiration is validated for URL with `expires=1735344000` and current Unix timestamp is 1735000000
- **THEN** the URL SHALL be considered valid
- **AND** processing SHALL continue

#### Scenario: URL expired

- **WHEN** the expiration is validated for URL with `expires=1735000000` and current Unix timestamp is 1735344000
- **THEN** the URL SHALL be considered expired
- **AND** an error message SHALL be displayed to the user
- **AND** the message SHALL include a link to re-download from Nexus

#### Scenario: URL expires during download

- **WHEN** the URL expires while a download is in progress
- **THEN** the active download SHALL be allowed to complete
- **AND** the download SHALL not be interrupted

### Requirement: Event Emission

The application SHALL emit events to notify the frontend when nxm:// URLs are received and processed.

#### Scenario: Emit event on successful URL parsing

- **WHEN** the URL is successfully parsed and validated
- **THEN** an "nxm-url-received" event SHALL be emitted to the frontend
- **AND** the event payload SHALL include mod_id, file_id, and mod name (if available)

#### Scenario: Emit event on parsing failure

- **WHEN** the URL parsing or validation fails
- **THEN** an "nxm-error" event SHALL be emitted to the frontend
- **AND** the event payload SHALL include the error message
- **AND** the frontend SHALL display the error to the user

#### Scenario: Emit event when download queued

- **WHEN** the download is added to the queue
- **THEN** a "download-queued" event SHALL be emitted
- **AND** the event SHALL include download ID and mod metadata

### Requirement: Error Handling

The application SHALL gracefully handle errors during protocol handling and provide actionable feedback.

#### Scenario: Network unavailable when fetching metadata

- **WHEN** the application attempts to fetch mod metadata and the network connection is unavailable
- **THEN** the application SHALL queue the download anyway
- **AND** the mod name SHALL be labeled "Unknown Mod"
- **AND** a warning notification SHALL inform the user

#### Scenario: Nexus servers unreachable

- **WHEN** the application attempts to process the URL and Nexus Mods servers are unreachable (5xx errors)
- **THEN** the download SHALL be queued for retry
- **AND** an error message SHALL explain the servers are unreachable
- **AND** the user SHALL be able to retry manually

#### Scenario: Invalid authentication key

- **WHEN** the application attempts to authenticate with an invalid or revoked key
- **THEN** an error SHALL be displayed: "Authentication failed"
- **AND** the user SHALL be prompted to re-download from Nexus
- **AND** a direct link to the mod page SHALL be provided (if mod_id is known)

## Non-Functional Requirements

### Performance
- URL parsing SHALL complete in < 10ms
- Protocol handler registration SHALL complete in < 500ms during installation
- The application SHALL handle up to 10 concurrent nxm:// URLs without performance degradation

### Security
- URL parameters SHALL be sanitized to prevent injection attacks
- Only `nxm://` scheme SHALL be accepted (reject http://, https://, file://, etc.)
- Game domain whitelist SHALL be enforced (only "stardewvalley")
- Authentication keys SHALL never be logged or displayed to the user

### Reliability
- Protocol registration SHALL survive OS updates
- Malformed URLs SHALL not crash the application
- Failed URL processing SHALL not block other downloads

## Dependencies

- Tauri v2.x with deep link plugin
- Rust regex crate (for URL parsing)
- Tokio async runtime (for event handling)

## Testing Criteria

### Unit Tests
- Parse valid nxm:// URLs with all parameter combinations
- Reject invalid URLs (wrong scheme, missing params, malformed IDs)
- Validate expiration logic (expired, not expired, no expiration)
- Sanitize malicious URL parameters

### Integration Tests
- Register protocol handler on fresh system
- Receive nxm:// URL when app is not running
- Receive nxm:// URL when app is running
- Handle multiple concurrent URLs
- Gracefully handle conflicting protocol handlers

### Manual Tests
- Test on Windows, macOS, and Linux
- Click "Mod Manager Download" on actual Nexus Mods website
- Verify browser permission prompt appears (first time)
- Verify app launches/focuses correctly
- Test with expired download links
