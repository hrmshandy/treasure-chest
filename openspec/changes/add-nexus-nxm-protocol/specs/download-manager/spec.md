# Spec: Download Manager

## Overview

Manage a queue of mod downloads from Nexus Mods, track progress, handle concurrency limits, and persist state across application restarts. Provide real-time progress updates to the frontend.

## ADDED Requirements

### Requirement: Download Queue Management

The application SHALL maintain a queue of pending, active, completed, and failed downloads.

#### Scenario: Add download to empty queue

- **WHEN** a new nxm:// URL is processed
- **THEN** a download task SHALL be created with status "queued"
- **AND** the download SHALL start immediately (if concurrency limit not reached)
- **AND** a "download-queued" event SHALL be emitted to the frontend

#### Scenario: Add download to non-empty queue

- **WHEN** a new nxm:// URL is processed with 3 active downloads and concurrency limit of 1
- **THEN** a download task SHALL be created with status "queued"
- **AND** the download SHALL remain in queue (not started)
- **AND** the download SHALL start when an active download completes

#### Scenario: Remove completed download from queue

- **WHEN** the user clicks "Clear completed" button
- **THEN** the completed download SHALL be removed from the queue
- **AND** the UI SHALL update to reflect the removal
- **AND** the downloaded file SHALL remain on disk

#### Scenario: Remove failed download from queue

- **WHEN** the user clicks the "X" button on the failed download
- **THEN** the failed download SHALL be removed from the queue
- **AND** the partial download file SHALL be deleted from disk
- **AND** the UI SHALL update immediately

### Requirement: Concurrent Download Control

The application SHALL enforce a configurable limit on concurrent downloads based on user account type.

#### Scenario: Free user - enforce 1 concurrent download

- **WHEN** 3 downloads are queued with concurrent download limit set to 1
- **THEN** only 1 download SHALL be active at a time
- **AND** the other 2 SHALL remain in "queued" status
- **AND** when the first completes, the second SHALL start automatically

#### Scenario: Premium user - allow multiple concurrent downloads

- **WHEN** 5 downloads are queued with concurrent download limit set to 3
- **THEN** 3 downloads SHALL be active simultaneously
- **AND** the other 2 SHALL remain in "queued" status
- **AND** as each completes, the next queued download SHALL start

#### Scenario: User increases concurrency limit

- **WHEN** the user increases the limit to 3 in settings with 5 queued downloads and 1 active download
- **THEN** 2 additional downloads SHALL start immediately
- **AND** the total active downloads SHALL be 3

#### Scenario: User decreases concurrency limit

- **WHEN** the user decreases the limit to 1 with 3 active downloads
- **THEN** the 2 most recent downloads SHALL be paused
- **AND** only 1 download SHALL remain active
- **AND** paused downloads SHALL resume when the active one completes

### Requirement: Progress Tracking

The application SHALL track and report download progress including bytes downloaded, speed, and estimated time remaining.

#### Scenario: Track progress for active download

- **WHEN** bytes are downloaded
- **THEN** the progress percentage SHALL be calculated as (bytes_downloaded / total_bytes) * 100
- **AND** the download speed SHALL be calculated in bytes per second
- **AND** the ETA SHALL be calculated as (remaining_bytes / current_speed)
- **AND** progress updates SHALL be emitted to the frontend at most 10 times per second

#### Scenario: Display progress in UI

- **WHEN** the frontend renders the download item at 50% complete with 2 MB/s speed and 30 seconds ETA
- **THEN** a progress bar SHALL display 50% filled
- **AND** the speed SHALL be displayed as "2.0 MB/s"
- **AND** the ETA SHALL be displayed as "30s remaining"

#### Scenario: Handle unknown file size

- **WHEN** progress is tracked and the server does not provide Content-Length header
- **THEN** the progress bar SHALL display in "indeterminate" mode
- **AND** only bytes downloaded and speed SHALL be shown
- **AND** ETA SHALL not be displayed

#### Scenario: Download completes

- **WHEN** the final bytes are downloaded at 99% progress
- **THEN** the progress SHALL be set to 100%
- **AND** the status SHALL change to "completed"
- **AND** a "download-completed" event SHALL be emitted
- **AND** the download SHALL be moved to the "completed" section of the UI

### Requirement: Pause and Resume

The application SHALL allow users to pause and resume individual downloads.

#### Scenario: Pause active download

- **WHEN** the user clicks the "Pause" button at 50% progress
- **THEN** the download SHALL stop immediately
- **AND** the status SHALL change to "paused"
- **AND** the partially downloaded file SHALL be saved to disk
- **AND** the download SHALL retain its position (50% progress)

#### Scenario: Resume paused download

- **WHEN** the user clicks the "Resume" button
- **THEN** the download SHALL resume from byte position (total_bytes * 0.5)
- **AND** the status SHALL change to "downloading"
- **AND** progress SHALL continue from 50%

#### Scenario: Resume fails - server doesn't support ranges

- **WHEN** the user resumes the download and the server does not support Range requests
- **THEN** the download SHALL restart from 0%
- **AND** the existing partial file SHALL be overwritten
- **AND** a warning notification SHALL inform the user

#### Scenario: Pause all downloads

- **WHEN** the user clicks "Pause All" button
- **THEN** all 3 downloads SHALL pause immediately
- **AND** all partial files SHALL be saved
- **AND** the button SHALL change to "Resume All"

### Requirement: Cancel Downloads

The application SHALL allow users to cancel downloads and optionally delete partial files.

#### Scenario: Cancel queued download

- **WHEN** the user clicks the "Cancel" button
- **THEN** the download SHALL be removed from the queue
- **AND** no file SHALL exist on disk
- **AND** the download SHALL be removed from the UI

#### Scenario: Cancel active download

- **WHEN** the user clicks the "Cancel" button at 30% progress
- **THEN** the download SHALL stop immediately
- **AND** the status SHALL change to "cancelled"
- **AND** the partial file SHALL be deleted from disk
- **AND** the download SHALL remain in the list as "cancelled"

#### Scenario: Cancel completed download

- **WHEN** the user attempts to cancel
- **THEN** the cancel button SHALL be disabled
- **AND** a "Delete" button SHALL be available instead
- **AND** clicking "Delete" SHALL remove the downloaded file from disk

### Requirement: Retry Failed Downloads

The application SHALL allow users to retry failed downloads and automatically retry transient failures.

#### Scenario: Manual retry of failed download

- **WHEN** the user clicks the "Retry" button
- **THEN** the download SHALL be re-queued with status "queued"
- **AND** the download SHALL start again from the beginning
- **AND** the previous error message SHALL be cleared

#### Scenario: Automatic retry on transient failure

- **WHEN** a download fails with a 5xx server error
- **THEN** the application SHALL automatically retry after 5 seconds
- **AND** the retry count SHALL increment (max 3 retries)
- **AND** if all retries fail, the download SHALL be marked "failed"

#### Scenario: No automatic retry on auth failure

- **WHEN** a download fails with a 401/403 error (authentication failure)
- **THEN** the application SHALL NOT automatically retry
- **AND** the status SHALL be set to "failed"
- **AND** the error message SHALL indicate "Authentication failed - please re-download from Nexus"

#### Scenario: Retry with new key after expiration

- **WHEN** the user re-downloads from Nexus and clicks "Mod Manager Download" again
- **THEN** the old failed download SHALL be replaced with the new one
- **AND** the download SHALL start with the new key

### Requirement: Queue Persistence

The application SHALL persist the download queue to disk and restore it on application restart.

#### Scenario: Save queue on state change

- **WHEN** any download's status changes (queued, downloading, paused, etc.)
- **THEN** the entire queue state SHALL be saved to `{app_data}/downloads/queue.json`
- **AND** the save operation SHALL complete within 100ms

#### Scenario: Restore queue on app startup

- **WHEN** the application starts again
- **THEN** the download queue SHALL be loaded from `queue.json`
- **AND** paused downloads SHALL remain paused
- **AND** previously active downloads SHALL be queued for restart
- **AND** completed downloads from previous session SHALL be displayed

#### Scenario: Handle corrupted queue file

- **WHEN** the application attempts to load the queue on startup with a corrupted `queue.json` file
- **THEN** the corrupted file SHALL be backed up to `queue.json.bak`
- **AND** an empty queue SHALL be initialized
- **AND** a warning SHALL be logged (but not shown to user)

#### Scenario: Resume incomplete downloads after crash

- **WHEN** the application restarts after crashing with 2 active downloads
- **THEN** the 2 downloads SHALL be restored with "paused" status
- **AND** partial files SHALL be preserved
- **AND** the user can manually resume them

### Requirement: Download File Management

The application SHALL download files to a configurable directory and manage disk space.

#### Scenario: Download to default directory

- **WHEN** a download starts
- **THEN** the file SHALL be downloaded to `{app_data}/downloads/nexus/`
- **AND** the directory SHALL be created if it doesn't exist

#### Scenario: Download to custom directory

- **WHEN** a download starts with a custom download directory configured
- **THEN** the file SHALL be downloaded to the custom directory
- **AND** the setting SHALL be persisted

#### Scenario: Check available disk space before download

- **WHEN** the download is queued with file size 500 MB and only 300 MB of disk space available
- **THEN** the download SHALL fail immediately with status "failed"
- **AND** the error message SHALL be "Insufficient disk space"
- **AND** the user SHALL be prompted to free up space or change download directory

#### Scenario: Handle filename conflicts

- **WHEN** a new download with the same filename starts and "ModName-1.0.zip" already exists
- **THEN** the new file SHALL be renamed to "ModName-1.0 (1).zip"
- **AND** the download SHALL proceed normally

### Requirement: Download Speed Limiting

The application SHALL respect Nexus Mods' speed limits for free and premium users.

#### Scenario: Free user speed limit

- **WHEN** downloads are active with a free Nexus account
- **THEN** each download SHALL be throttled to approximately 3 MB/s max
- **AND** the actual speed MAY be lower based on network conditions
- **AND** the speed SHALL be displayed accurately in the UI

#### Scenario: Premium user no speed limit

- **WHEN** downloads are active with a premium Nexus account
- **THEN** downloads SHALL NOT be artificially throttled
- **AND** the speed SHALL be limited only by network bandwidth
- **AND** speeds > 10 MB/s SHALL be supported

## Non-Functional Requirements

### Performance
- Download progress updates SHALL be emitted at most 10 times per second per download
- Queue state persistence SHALL complete in < 100ms
- The UI SHALL remain responsive even with 50+ completed downloads in history

### Reliability
- Partial downloads SHALL survive application crashes
- Queue state SHALL be saved atomically to prevent corruption
- Failed downloads SHALL never leave corrupted files

### Usability
- Download speeds SHALL be displayed in human-readable units (KB/s, MB/s)
- ETAs SHALL be displayed in human-readable format (e.g., "2m 30s", "1h 15m")
- Progress bars SHALL be smooth and responsive

## Dependencies

- Tokio async runtime for concurrent downloads
- reqwest HTTP client with streaming support
- serde for queue state serialization
- Tauri event system for frontend communication

## Testing Criteria

### Unit Tests
- Calculate progress percentage correctly
- Calculate download speed (bytes/sec) accurately
- Calculate ETA based on speed and remaining bytes
- Enforce concurrency limits (1, 3, 5 concurrent downloads)
- Save and restore queue state from JSON

### Integration Tests
- Download multiple files concurrently
- Pause and resume downloads
- Handle network interruptions (simulate with mock server)
- Persist and restore queue across app restarts
- Handle server errors (404, 500, etc.)

### Manual Tests
- Download large mods (> 100 MB)
- Download multiple small mods simultaneously
- Test pause/resume during active download
- Test retry after network disconnection
- Verify disk space check prevents download when space low
- Test on slow network connections (verify speed display)
