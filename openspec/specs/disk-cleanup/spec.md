# disk-cleanup Specification

## Purpose
TBD - created by archiving change disk-cleanup. Update Purpose after archive.
## Requirements
### Requirement: Catalog of known cleanup targets

The system SHALL define a fixed catalog of cleanup targets that mirrors `~/disk-cleanup.sh` exactly: the same targets, the same detection paths, and the same cleanup commands. Each target SHALL be assigned a tier: Tier 1 (pure caches, no risk), Tier 2 (regenerables, low risk), or Tier 3 (persistent data, informational only). The catalog SHALL cover, at minimum: Yarn cache, npm cache, pip cache, Homebrew cache, ShipIt installer cache, Playwright cache, Spotify cache, Chrome cache, and Bun cache (Tier 1); Docker prune, Docker.raw regeneration, iOS simulators, old iOS runtimes, old nvm Node versions, Xcode Archives, Xcode DerivedData, Cargo cache, Conductor worktrees, and Android SDK system-images (Tier 2); and PostgreSQL databases, Spark Desktop emails, Claude VM bundles, UTM VMs, WhatsApp data, Notion, Cursor, and Chrome profiles (Tier 3).

#### Scenario: Every script target is represented

- **WHEN** the catalog is loaded
- **THEN** every target present in `~/disk-cleanup.sh` SHALL appear with the same tier assignment as in the script

#### Scenario: Cleanup commands match the script verbatim

- **WHEN** a Tier 1 or Tier 2 target's cleanup command is inspected
- **THEN** the command string SHALL be identical to the corresponding command in `~/disk-cleanup.sh`, including tool-if-present fallbacks (e.g. `yarn cache clean` else `rm -rf ~/Library/Caches/Yarn/*`)

### Requirement: Scan measures targets without deleting

The system SHALL provide a scan operation that measures size and availability of every catalog target and returns the current free disk space, without deleting or modifying anything. Sizes SHALL be computed in bytes for accurate totals and sorting, and free space SHALL use the same `df` semantics as the script (`/System/Volumes/Data`).

#### Scenario: Scan is non-destructive

- **WHEN** the user runs the scan
- **THEN** no target is deleted, quit, or modified
- **AND** each target reports its measured size and a status

#### Scenario: Free space captured before actions

- **WHEN** the scan completes
- **THEN** the current free disk space SHALL be recorded as the "before" baseline for later comparison

#### Scenario: Target status reflects availability

- **WHEN** a target's path is empty, its backing tool is missing, or the app is not installed
- **THEN** the target SHALL report a status distinguishing these cases (e.g. Available, Empty, ToolMissing, NotInstalled)

### Requirement: Results grouped by tier for review

The system SHALL present scanned results grouped by Tier 1, Tier 2, and Tier 3. Each Tier 1 and Tier 2 row SHALL show its name, path, measured size, status, and the exact command that would run. Tier 3 rows SHALL be read-only and display a warning that they require manual decision and are never cleaned automatically.

#### Scenario: Tiers rendered in order

- **WHEN** the review surface is shown after a scan
- **THEN** targets appear grouped Tier 1 → Tier 2 → Tier 3

#### Scenario: Tier 3 has no cleanup action

- **WHEN** a Tier 3 target is displayed
- **THEN** it SHALL show its size for information only and expose no cleanup button

#### Scenario: Exact command is visible

- **WHEN** the user inspects a Tier 1 or Tier 2 target
- **THEN** the exact shell command that would run SHALL be shown before any action is taken

### Requirement: Per-item cleanup action

The system SHALL let the user clean each actionable target independently via its own action button. Clicking the button SHALL run only that target's exact command. Targets whose status is Empty, ToolMissing, or NotInstalled SHALL have their action disabled. After a cleanup runs, the row SHALL transition through cleaning → done (with freed amount) or error (with message), and the free-disk readout SHALL be recomputed.

#### Scenario: Single target cleanup

- **WHEN** the user clicks the action button for an available target
- **THEN** only that target's command runs
- **AND** the row shows a cleaning state, then the freed amount on success

#### Scenario: Disabled when nothing to clean

- **WHEN** a target's status is Empty, ToolMissing, or NotInstalled
- **THEN** its action button SHALL be disabled

#### Scenario: Error is reported without cascading

- **WHEN** a target's cleanup command fails
- **THEN** the row SHALL show an error message
- **AND** no other target SHALL be affected

#### Scenario: Free space updates after action

- **WHEN** a cleanup action completes
- **THEN** the disk-free readout SHALL update to reflect current free space and the amount freed relative to the baseline

### Requirement: Per-subitem cleanup for container targets

For container targets that the script iterates one by one — Docker prune components, old iOS runtimes, old nvm Node versions, Conductor worktrees, and Android SDK system-images — the system SHALL expose each subitem as an individually actionable row with its own size, metadata, and button. Subitem selection logic SHALL match the script: nvm keeps the current version and the latest LTS (only offered when more than 3 are installed); iOS runtimes keep the newest per platform; Conductor worktrees show branch and git status per workspace.

#### Scenario: Container target expands to subitems

- **WHEN** a container target is displayed
- **THEN** each of its subitems SHALL appear as a row with its own action button

#### Scenario: nvm keeps current and latest LTS

- **WHEN** more than 3 nvm Node versions are installed
- **THEN** only versions other than the current and the latest LTS SHALL be offered for deletion

#### Scenario: iOS runtimes keep newest per platform

- **WHEN** old iOS runtimes are listed
- **THEN** the newest runtime per platform SHALL be kept and only older available runtimes offered for deletion

#### Scenario: Conductor workspace shows git state

- **WHEN** a Conductor workspace is listed
- **THEN** its branch and whether it has uncommitted changes SHALL be shown

### Requirement: Double confirmation for high-risk targets

The system SHALL require an explicit second confirmation (mirroring the script's `ask_double`) before executing the two high-risk operations the script gates: regenerating `Docker.raw` (destroys remaining Docker images/volumes) and deleting a Conductor workspace that has uncommitted changes. The confirmation SHALL require deliberate user input distinct from a normal click, and cancelling SHALL leave the target untouched.

#### Scenario: Docker.raw regeneration requires double confirm

- **WHEN** the user requests Docker.raw regeneration
- **THEN** a double-confirm prompt SHALL appear
- **AND** the regeneration runs only after explicit confirmation

#### Scenario: Dirty Conductor workspace requires double confirm

- **WHEN** the user requests deletion of a Conductor workspace with uncommitted changes
- **THEN** a double-confirm prompt SHALL appear before deletion

#### Scenario: Cancelling leaves target untouched

- **WHEN** the user cancels a double-confirm prompt
- **THEN** the target SHALL not be modified

### Requirement: Faithful destructive semantics

The system SHALL perform the same destructive operations as the script — permanent removal of regenerable caches and data via the script's exact commands (e.g. `rm -rf`, tool-native clean, `docker ... prune`, `xcrun simctl ... delete`, `nvm uninstall`) — rather than moving items to Trash. Targets that require quitting an application (Spotify, Chrome, Docker) SHALL perform the same quit step the script does before removal. Docker cleanup SHALL start Docker if not running and wait up to 90 seconds, matching the script.

#### Scenario: Regenerable caches are permanently removed

- **WHEN** a Tier 1 cache cleanup runs
- **THEN** the same permanent-removal command from the script SHALL execute (no move-to-Trash)

#### Scenario: App is quit before cache removal

- **WHEN** cleaning a cache that requires the app closed (Spotify, Chrome)
- **THEN** the app SHALL be quit first, matching the script's `osascript` quit step

#### Scenario: Docker started before prune

- **WHEN** Docker cleanup runs and Docker is not running
- **THEN** the system SHALL attempt to start Docker and wait up to 90 seconds before pruning, skipping if it does not start

