# app-self-update Specification

## Purpose
Defines the in-app self-update flow for Clarus: checking for a signed update via the Tauri updater, surfacing that one is available, and downloading, installing, and relaunching to apply it — all guarded so it only runs inside the packaged Tauri runtime.

## Requirements
### Requirement: Updater retains the available update for installation

The updater SHALL retain the `Update` object returned by the Tauri updater's `check()` so it can be installed later, rather than discarding it after reading the version. When no update is available, the check fails, or the app is not running inside the Tauri runtime, the retained update MUST be cleared.

#### Scenario: Update found is retained

- **WHEN** `checkNow()` runs and `check()` returns an update
- **THEN** the update's version is recorded and state becomes `available`
- **AND** the `Update` object is retained for a later install

#### Scenario: No update clears retained update

- **WHEN** `checkNow()` runs and `check()` returns no update
- **THEN** state becomes `current`, the version is cleared, and any retained `Update` is cleared

#### Scenario: Non-Tauri runtime

- **WHEN** `checkNow()` runs outside the Tauri runtime
- **THEN** state becomes `current` and no update is retained

### Requirement: Updater exposes install action with progress states

The updater SHALL expose a `downloadAndInstall()` action and SHALL model the states `idle`, `checking`, `available`, `downloading`, `downloaded`, `current`, and `error`. Invoking the action downloads and installs the retained update and then relaunches the app.

#### Scenario: Install progresses through states

- **WHEN** `downloadAndInstall()` is invoked with a retained update
- **THEN** state becomes `downloading` while the update downloads and installs
- **AND** state becomes `downloaded` once installation completes
- **AND** the app is relaunched to finish applying the update

#### Scenario: Install without a retained update is a no-op

- **WHEN** `downloadAndInstall()` is invoked and no update is retained
- **THEN** no download or relaunch occurs and no state change is made

#### Scenario: Install failure surfaces an error

- **WHEN** downloading or installing the update throws
- **THEN** the error is recorded and state becomes `error`

### Requirement: Release channel UI offers an actionable install

The Release channel section SHALL present an actionable "Update now" control whenever an update is available or being installed, and SHALL communicate progress through status copy. Controls MUST be disabled while an action is in flight.

#### Scenario: Update-now button appears when available

- **WHEN** the updater state is `available`
- **THEN** an "Update now" button is shown and status copy states the available version

#### Scenario: Controls disabled during install

- **WHEN** the updater state is `downloading` or `downloaded`
- **THEN** the "Update now" and "Check for updates" buttons are disabled
- **AND** status copy shows "Downloading update…" while downloading and "Restarting to finish update…" once downloaded

#### Scenario: Check button disabled while checking

- **WHEN** the updater state is `checking`
- **THEN** the "Check for updates" button is disabled and status copy indicates the check is in progress
