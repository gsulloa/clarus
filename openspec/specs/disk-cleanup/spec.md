# disk-cleanup Specification

## Purpose
TBD - created by archiving change disk-cleanup. Update Purpose after archive.
## Requirements
### Requirement: Catalog of known cleanup targets

The system SHALL define a fixed catalog of cleanup targets. The catalog SHALL mirror `~/disk-cleanup.sh` (the same targets, detection paths, and cleanup commands) AND SHALL additionally include high-impact targets identified during disk analysis that are not in the script. Each target SHALL be assigned a tier: Tier 1 (pure caches, no risk), Tier 2 (regenerables, low risk), or Tier 3 (persistent data, informational only). The catalog SHALL cover, at minimum:

- **Tier 1**: Yarn cache, npm cache, pip cache, Homebrew cache, ShipIt installer cache, Playwright cache, Spotify cache, Chrome cache, Bun cache, pnpm store (`pnpm-store`), Gradle caches (`gradle-caches`), Gradle wrapper dists (`gradle-wrapper`), Gradle daemon logs (`gradle-daemon`), Cisco Webex old upgrades (`webex-upgrades`), Slack cache (`slack-cache`), Claude Desktop cache (`claude-desktop-cache`), AWS Toolkit cache (`aws-toolkit-cache`), and Cursor cached VSIXs (`cursor-vsix-cache`).
- **Tier 2**: Docker prune, Docker.raw regeneration, iOS simulators, old iOS runtimes, old nvm Node versions, old pyenv Python versions (`pyenv`), Ollama models (`ollama`), Xcode Archives, Xcode DerivedData, Cargo cache, Conductor worktrees, and Android SDK system-images.
- **Tier 3**: PostgreSQL databases, Spark Desktop emails, Claude VM bundles, UTM VMs, WhatsApp data, Notion, Cursor, and Chrome profiles.

Newly added targets whose cleanup command is not present in `~/disk-cleanup.sh` MAY diverge from the script; only targets that also exist in the script SHALL match it verbatim.

#### Scenario: Every script target is represented

- **WHEN** the catalog is loaded
- **THEN** every target present in `~/disk-cleanup.sh` SHALL appear with the same tier assignment as in the script

#### Scenario: Cleanup commands match the script verbatim

- **WHEN** a Tier 1 or Tier 2 target that also exists in `~/disk-cleanup.sh` has its cleanup command inspected
- **THEN** the command string SHALL be identical to the corresponding command in `~/disk-cleanup.sh`, including tool-if-present fallbacks (e.g. `yarn cache clean` else `rm -rf ~/Library/Caches/Yarn/*`)

#### Scenario: New high-impact targets appear with correct tiers

- **WHEN** the catalog is loaded
- **THEN** `pnpm-store`, `gradle-caches`, `gradle-wrapper`, `gradle-daemon`, `webex-upgrades`, `slack-cache`, `claude-desktop-cache`, `aws-toolkit-cache`, and `cursor-vsix-cache` SHALL appear as Tier 1 targets
- **AND** `pyenv` and `ollama` SHALL appear as Tier 2 targets

#### Scenario: Webex upgrades keeps the newest version

- **WHEN** the `webex-upgrades` target's cleanup command is inspected
- **THEN** the command SHALL delete every version directory under `Webexteams_upgrades_arm/` except the highest-versioned one

#### Scenario: New target reports availability by backing tool or path

- **WHEN** a newly added target's backing tool is absent or its detection path is empty (e.g. `pnpm` not installed, `~/.gradle/caches` missing)
- **THEN** the target SHALL report a status distinguishing this case (Empty, ToolMissing, or NotInstalled) and its cleanup action SHALL be disabled

### Requirement: Scan measures targets without deleting

The system SHALL provide a scan operation that measures size and availability of every catalog target and returns the current disk usage of the data volume — **free space, total capacity, and used space** — without deleting or modifying anything. Sizes SHALL be computed in bytes for accurate totals and sorting, and free/total/used space SHALL use the same `df` semantics as the script (`/System/Volumes/Data`), read from a single `df` snapshot so the three figures are mutually consistent.

#### Scenario: Scan is non-destructive

- **WHEN** the user runs the scan
- **THEN** no target is deleted, quit, or modified
- **AND** each target reports its measured size and a status

#### Scenario: Free space captured before actions

- **WHEN** the scan completes
- **THEN** the current free disk space SHALL be recorded as the "before" baseline for later comparison

#### Scenario: Total and used space are reported

- **WHEN** the scan completes
- **THEN** the total capacity and the used space of the data volume SHALL be reported alongside free space
- **AND** these figures SHALL come from the same `df /System/Volumes/Data` snapshot used for free space

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

The system SHALL let the user clean each actionable target independently via its own action button. Clicking the button SHALL run only that target's exact command. Targets whose status is Empty, ToolMissing, or NotInstalled SHALL have their action disabled. After a cleanup runs, the row SHALL transition through cleaning → done (with freed amount) or error (with message), and the disk-usage readout SHALL be recomputed.

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

#### Scenario: Disk usage updates after action

- **WHEN** a cleanup action completes
- **THEN** the disk-usage readout SHALL update to reflect current free space, used space, and total capacity, and the amount freed relative to the baseline

### Requirement: Per-subitem cleanup for container targets

For container targets that the catalog iterates one by one — Docker prune components, old iOS runtimes, old nvm Node versions, old pyenv Python versions, Ollama models, Conductor workspaces, and Android SDK system-images — the system SHALL expose each subitem as an individually actionable row with its own size, metadata, and button. Subitem selection logic SHALL match the script where a script equivalent exists: nvm keeps the current version and the latest LTS (only offered when more than 3 are installed); iOS runtimes keep the newest per platform; Conductor workspaces enumerate individual workspaces from a two-level directory structure and show branch and git status per workspace. For pyenv, the system SHALL keep the active version (`pyenv version-name`) and offer every other installed version under `~/.pyenv/versions/*` for deletion via `pyenv uninstall <version>`. For Ollama, the system SHALL enumerate every model from `ollama list` as a deletable subitem with `ollama rm <name>` as its command, and report `NotInstalled` when the `ollama` binary is absent.

For Conductor workspaces specifically, the system SHALL distinguish between **project containers** (top-level directories with no `.git` of their own but whose children have `.git`) and **flat workspaces** (top-level directories with `.git` directly). Project containers SHALL be expanded one level so that each constituent workspace is shown as a separate subitem. Flat workspaces SHALL appear as single subitems under their own name. Each subitem label for a nested workspace SHALL include the project container name using a `project/workspace` format (e.g. `backend/paris`, `tub2/chengdu-v4`). Subitem IDs SHALL use a collision-safe separator (`project__workspace`) because two project containers can have workspaces with the same name.

#### Scenario: Container target expands to subitems

- **WHEN** a container target is displayed
- **THEN** each of its subitems SHALL appear as a row with its own action button

#### Scenario: nvm keeps current and latest LTS

- **WHEN** more than 3 nvm Node versions are installed
- **THEN** only versions other than the current and the latest LTS SHALL be offered for deletion

#### Scenario: pyenv keeps the active version

- **WHEN** the pyenv target is listed and more than one Python version is installed
- **THEN** the active version reported by `pyenv version-name` SHALL be kept
- **AND** every other installed version SHALL be offered for deletion with an `pyenv uninstall <version>` command

#### Scenario: Ollama enumerates every model

- **WHEN** the Ollama target is listed and the `ollama` binary is present
- **THEN** each model returned by `ollama list` SHALL appear as a subitem with an `ollama rm <name>` command

#### Scenario: Ollama reports NotInstalled when the binary is absent

- **WHEN** the `ollama` binary is not found on PATH
- **THEN** the Ollama target SHALL report status `NotInstalled` and expose no actionable subitems

#### Scenario: iOS runtimes keep newest per platform

- **WHEN** old iOS runtimes are listed
- **THEN** the newest runtime per platform SHALL be kept and only older available runtimes offered for deletion

#### Scenario: Conductor project containers are expanded to individual workspaces

- **WHEN** a Conductor workspace subitem is listed
- **THEN** project containers (no own `.git`, children have `.git`) SHALL NOT appear as items themselves
- **AND** each individual workspace within a project container SHALL appear as a separate subitem labeled `project/workspace`

#### Scenario: Conductor flat workspaces appear by their own name

- **WHEN** a top-level Conductor directory has `.git` directly
- **THEN** it SHALL appear as a single subitem under its own name, without a project prefix

#### Scenario: Conductor workspace shows git state

- **WHEN** a Conductor workspace subitem is listed
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

### Requirement: Background work never blocks the UI

Scan and cleanup operations SHALL execute without blocking the UI thread. While a scan or any cleanup command is running, the window SHALL keep rendering and remain interactive, and the application SHALL NOT cause the operating system to display a wait/busy ("beachball") cursor. This SHALL hold for long-running operations, including cleanups that quit applications or wait for Docker to start (up to 90 seconds).

#### Scenario: Window stays responsive during a scan

- **WHEN** the user starts an analysis and sizes are being measured
- **THEN** the window SHALL continue to paint and respond to input (scrolling, selecting a target, hovering)
- **AND** no wait/busy cursor SHALL be shown

#### Scenario: Window stays responsive during a long cleanup

- **WHEN** a cleanup command runs that takes a long time (e.g. Docker prune waiting for Docker to start)
- **THEN** the rest of the catalog SHALL remain scrollable, selectable, and actionable
- **AND** the window SHALL NOT freeze and no wait/busy cursor SHALL be shown

#### Scenario: Other targets remain actionable while one is cleaning

- **WHEN** one target is in the cleaning state
- **THEN** other actionable targets SHALL still accept clicks and start their own cleanup independently

### Requirement: Scan streams progress incrementally

The scan operation SHALL report progress as it runs rather than only on completion. It SHALL make the set of catalog targets and a total target count available before sizes are measured, and it SHALL report each target's measured size as measuring completes for that target, so the UI can advance a determinate progress indicator (measured of total) that reflects real progress.

#### Scenario: Total is known before measuring completes

- **WHEN** a scan begins and the catalog has been enumerated
- **THEN** the total number of targets SHALL be available to the UI before all sizes are measured

#### Scenario: Per-target progress is reported as it completes

- **WHEN** a target's size measurement finishes
- **THEN** that target's measured size SHALL be reported to the UI at that time, not withheld until every target is measured
- **AND** the count of measured targets SHALL not exceed the total

### Requirement: Appropriate loading feedback during background work

The UI SHALL present appropriate, non-blocking loading feedback during background work instead of a frozen screen. During analysis it SHALL show catalog rows immediately with placeholder (skeleton) sizes and a determinate progress indicator that advances as targets are measured. During a cleanup it SHALL show a live per-row busy state that conveys the operation is ongoing, including elapsed time for long-running cleanups. All loading animations SHALL respect the user's reduced-motion preference.

#### Scenario: Rows appear with skeletons before sizes arrive

- **WHEN** a scan has enumerated the catalog but not yet measured sizes
- **THEN** the target rows SHALL be visible with a placeholder/skeleton size
- **AND** each row's size SHALL be filled in as its measurement completes

#### Scenario: Determinate progress during analysis

- **WHEN** a scan is measuring targets
- **THEN** a progress indicator SHALL reflect the proportion of targets measured out of the total

#### Scenario: Long cleanup shows ongoing feedback

- **WHEN** a cleanup runs longer than a moment
- **THEN** its row SHALL show an ongoing busy state with elapsed time (and, where relevant, a hint that the step may take time, such as starting Docker)
- **AND** this feedback SHALL update live rather than appearing frozen

#### Scenario: Reduced motion is respected

- **WHEN** the user has enabled the reduced-motion preference
- **THEN** non-essential loading animations (e.g. skeleton shimmer) SHALL be disabled while progress state still updates

### Requirement: Disk usage summary is displayed

The system SHALL present a disk-usage summary for the data volume that shows used space, free space, and total capacity — not only cleanup opportunities and free space. The summary SHALL include a proportional capacity indicator (used relative to total) so the user can judge at a glance how full the disk is. The summary SHALL keep exposing the cleanup-session progress (free space before, free space now, and total freed).

#### Scenario: Summary shows used, free, and total

- **WHEN** a scan has completed
- **THEN** the readout SHALL display the used space, the free space, and the total capacity of the data volume

#### Scenario: Capacity indicator reflects fullness

- **WHEN** the disk-usage summary is shown
- **THEN** a proportional indicator SHALL represent used space relative to total capacity

#### Scenario: Cleanup progress remains visible

- **WHEN** one or more cleanups have run in the session
- **THEN** the summary SHALL still show free space before, free space now, and the total amount freed

