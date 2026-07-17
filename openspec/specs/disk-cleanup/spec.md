# disk-cleanup Specification

## Purpose
TBD - created by archiving change disk-cleanup. Update Purpose after archive.
## Requirements
### Requirement: Catalog of known cleanup targets

The system SHALL define a fixed catalog of cleanup targets. The catalog SHALL mirror `~/disk-cleanup.sh` (the same targets, detection paths, and cleanup commands) AND SHALL additionally include high-impact targets identified during disk analysis that are not in the script. Each target SHALL be assigned a tier: Tier 1 (pure caches, no risk), Tier 2 (regenerables, low risk), or Tier 3 (persistent personal data, irreplaceable — deletable only behind explicit confirmation). The catalog SHALL cover, at minimum:

- **Tier 1**: Yarn cache, npm cache, pip cache, Homebrew cache, ShipIt installer cache, Playwright cache, Spotify cache, Chrome cache, Bun cache, pnpm store (`pnpm-store`), Gradle caches (`gradle-caches`), Gradle wrapper dists (`gradle-wrapper`), Gradle daemon logs (`gradle-daemon`), Cisco Webex old upgrades (`webex-upgrades`), Slack cache (`slack-cache`), Claude Desktop cache (`claude-desktop-cache`), AWS Toolkit cache (`aws-toolkit-cache`), and Cursor cached VSIXs (`cursor-vsix-cache`).
- **Tier 2**: Docker prune, Docker.raw regeneration, iOS simulators, old iOS runtimes, old nvm Node versions, old pyenv Python versions (`pyenv`), Ollama models (`ollama`), Xcode Archives, Xcode DerivedData, Cargo cache, Conductor worktrees, and Android SDK system-images.
- **Tier 3**: PostgreSQL databases, Spark Desktop emails, Claude VM bundles, UTM VMs, WhatsApp data, Notion, Cursor, Chrome profiles, and Downloads. Every Tier 3 target SHALL carry a cleanup command and SHALL require the explicit double confirmation (`requires_double_confirm`).

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

#### Scenario: Every Tier 3 target is deletable behind confirmation

- **WHEN** the catalog is loaded
- **THEN** each Tier 3 target SHALL carry a non-null cleanup command (a top-level command, per-subitem commands, or both)
- **AND** every Tier 3 cleanup action (top-level or per-subitem) SHALL require the explicit double confirmation before it runs

### Requirement: Scan measures targets without deleting

The system SHALL provide a scan operation that measures size and availability of every catalog target and returns the current disk usage of the data volume — **free space, total capacity, and used space** — without deleting or modifying anything. Sizes SHALL be computed in bytes for accurate totals and sorting. Free, total, and used space SHALL use the same `df` semantics as the script (`/System/Volumes/Data`) and SHALL be read from a **single `df` snapshot** (one invocation) so the three figures are mutually consistent. The **used** figure reported to the UI SHALL be derived as `total − free` (not the APFS per-volume `Used` column), so that `used + free = total` holds exactly for the reported figures.

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
- **AND** these figures SHALL come from a single `df /System/Volumes/Data` snapshot used for free space
- **AND** the reported used space SHALL equal total capacity minus free space

#### Scenario: Reported figures are mutually consistent

- **WHEN** the scan reports used, free, and total for the data volume
- **THEN** used plus free SHALL equal total for both the numeric (GB) fields and the human-readable strings within rounding
- **AND** the numeric and human-readable variants SHALL be derived from the same `df` snapshot rather than from independent invocations taken at different instants

#### Scenario: Target status reflects availability

- **WHEN** a target's path is empty, its backing tool is missing, or the app is not installed
- **THEN** the target SHALL report a status distinguishing these cases (e.g. Available, Empty, ToolMissing, NotInstalled)

### Requirement: Results grouped by tier for review

The system SHALL present scanned results grouped by Tier 1, Tier 2, and Tier 3. Each Tier 1 and Tier 2 row SHALL show its name, path, measured size, status, and the exact command that would run. Each Tier 3 row SHALL show its name, path, measured size, and status, and SHALL expose a cleanup action; because Tier 3 data is irreplaceable, that action SHALL run only through the explicit double-confirmation flow, and the row (or its evidence detail) SHALL display a warning that the deletion is permanent and cannot be undone.

#### Scenario: Tiers rendered in order

- **WHEN** the review surface is shown after a scan
- **THEN** targets appear grouped Tier 1 → Tier 2 → Tier 3

#### Scenario: Tier 3 exposes a confirmation-gated cleanup action

- **WHEN** a Tier 3 target is displayed
- **THEN** it SHALL show its measured size and a cleanup control (a per-item control, a group "clean all" control, or both)
- **AND** activating any Tier 3 cleanup control SHALL open the double-confirmation prompt before anything is deleted
- **AND** the target SHALL surface a warning that its data is permanent and irreplaceable

#### Scenario: Exact command is visible

- **WHEN** the user inspects any target with a cleanup command
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

For container targets that the catalog iterates one by one — Docker prune components, old iOS runtimes, old nvm Node versions, old pyenv Python versions, Ollama models, Conductor workspaces, Android SDK system-images, and the disaggregated Tier 3 collection targets — the system SHALL expose each subitem as an individually actionable row with its own size, metadata, and button. Subitem selection logic SHALL match the script where a script equivalent exists: nvm keeps the current version and the latest LTS (only offered when more than 3 are installed); iOS runtimes keep the newest per platform; Conductor workspaces enumerate individual workspaces from a two-level directory structure and show branch and git status per workspace. For pyenv, the system SHALL keep the active version (`pyenv version-name`) and offer every other installed version under `~/.pyenv/versions/*` for deletion via `pyenv uninstall <version>`. For Ollama, the system SHALL enumerate every model from `ollama list` as a deletable subitem with `ollama rm <name>` as its command, and report `NotInstalled` when the `ollama` binary is absent.

The Tier 3 collection targets `downloads` (each top-level entry in `~/Downloads`), `chrome-profiles` (each Chrome profile such as `Default`, `Profile N`, `Guest Profile`), `claude-vm` (each VM bundle), and `utm` (each `.utm` virtual machine) SHALL enumerate their members as individually actionable subitems, each with a `rm -rf`-equivalent command that removes only that member and each requiring the double confirmation. These targets SHALL also carry a top-level group command (see "Group clean-all action for disaggregated targets") that removes every member at once. When a collection target's backing directory is absent, it SHALL report `NotInstalled`; when the directory exists but has no members, it SHALL report `Empty`.

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

#### Scenario: Tier 3 collection targets enumerate their members

- **WHEN** a `downloads`, `chrome-profiles`, `claude-vm`, or `utm` target is listed and its backing directory has members
- **THEN** each member SHALL appear as a subitem that deletes only that member
- **AND** each subitem's action SHALL require the double confirmation

#### Scenario: Tier 3 collection target reports Empty or NotInstalled

- **WHEN** a Tier 3 collection target's backing directory is absent
- **THEN** it SHALL report status `NotInstalled`
- **WHEN** the directory exists but has no members
- **THEN** it SHALL report status `Empty`

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

The system SHALL present a disk-usage summary for the data volume that shows used space, free space, and total capacity — not only cleanup opportunities and free space. The three figures SHALL be mutually coherent such that used plus free equals total. The summary SHALL include a proportional capacity indicator (used relative to total, equivalently `1 − free/total`) so the user can judge at a glance how full the disk is. The summary SHALL keep exposing the cleanup-session progress (free space before, free space now, and total freed).

#### Scenario: Summary shows used, free, and total

- **WHEN** a scan has completed
- **THEN** the readout SHALL display the used space, the free space, and the total capacity of the data volume

#### Scenario: Displayed figures add up

- **WHEN** the used, free, and total figures are displayed together
- **THEN** the displayed used and free SHALL sum to the displayed total (within unit rounding), so the three numbers reconcile for the user

#### Scenario: Capacity indicator reflects fullness

- **WHEN** the disk-usage summary is shown
- **THEN** a proportional indicator SHALL represent used space relative to total capacity
- **AND** the indicator's filled proportion SHALL be consistent with the displayed free space (i.e. the unfilled portion corresponds to free ÷ total)

#### Scenario: Cleanup progress remains visible

- **WHEN** one or more cleanups have run in the session
- **THEN** the summary SHALL still show free space before, free space now, and the total amount freed

### Requirement: Additional high-impact cleanup targets

The catalog SHALL additionally include the following high-impact targets identified during disk analysis, none of which change any existing target's tier or command. Each new Tier 1/Tier 2 target SHALL report a status distinguishing Available, Empty, ToolMissing, or NotInstalled based on its detection path or backing tool, and SHALL disable its cleanup action when nothing is cleanable. New targets whose command is not present in `~/disk-cleanup.sh` MAY diverge from the script.

The catalog SHALL include, at minimum:

- **Tier 1**: `docker-scout` (`~/.docker/scout`), `uv-cache` (`~/.cache/uv`), `puppeteer-cache` (`~/.cache/puppeteer`), `node-gyp` (`~/Library/Caches/node-gyp`), `vscode-cache` (VS Code `Cache` + `CachedData`), `cursor-cache` (Cursor `Cache` + `CachedData`), `user-logs` (`~/Library/Logs`), `quicklook-cache` (QuickLook thumbnail cache), `tableplus-cache` (`~/Library/Caches/com.tinyapp.TablePlus`), `discord-cache`, `notion-cache`, `teams-cache`, `postman-cache`, and `zoom-cache` (each app's HTTP/GPU caches).
- **Tier 2**: `coresimulator-caches` (`~/Library/Developer/CoreSimulator/Caches`), `xcode-devicesupport` (iOS/watchOS/tvOS DeviceSupport), and `trash` (empty `~/.Trash` and mounted-volume trashes).
- **Tier 3**: `downloads` (`~/Downloads`), disaggregated into one subitem per top-level entry with a group "clean all" action, gated by the double confirmation.

#### Scenario: New Tier 1 targets appear with correct tier

- **WHEN** the catalog is loaded
- **THEN** `docker-scout`, `uv-cache`, `puppeteer-cache`, `node-gyp`, `vscode-cache`, `cursor-cache`, `user-logs`, `quicklook-cache`, `tableplus-cache`, `discord-cache`, `notion-cache`, `teams-cache`, `postman-cache`, and `zoom-cache` SHALL appear as Tier 1 targets

#### Scenario: New Tier 2 targets appear with correct tier

- **WHEN** the catalog is loaded
- **THEN** `coresimulator-caches`, `xcode-devicesupport`, and `trash` SHALL appear as Tier 2 targets

#### Scenario: Downloads is deletable and disaggregated

- **WHEN** the catalog is loaded
- **THEN** `downloads` SHALL appear as a Tier 3 target with per-entry subitems and a top-level group command
- **AND** every `downloads` cleanup action (per-entry and group) SHALL require the double confirmation

#### Scenario: New cache target reports availability

- **WHEN** a new target's detection path is empty or its backing tool is absent (e.g. `~/.docker/scout` missing, Discord not installed)
- **THEN** the target SHALL report a status of Empty, ToolMissing, or NotInstalled and its cleanup action SHALL be disabled

#### Scenario: Docker Scout is independent of Docker prune

- **WHEN** the `docker-scout` target's cleanup command is inspected
- **THEN** it SHALL remove `~/.docker/scout` only, without affecting the Docker VM, images, or volumes handled by `docker-prune` and `docker-raw`

#### Scenario: Electron app cache targets remove only caches

- **WHEN** the `discord-cache`, `notion-cache`, `teams-cache`, `postman-cache`, or `zoom-cache` target's command is inspected
- **THEN** it SHALL remove only that app's `Cache`, `Code Cache`, `GPUCache`, and `Service Worker/CacheStorage` directories
- **AND** it SHALL NOT touch that app's persistent user data (the `notion` and `cursor` Tier 3 targets remain untouched)

#### Scenario: Trash empties the OS hold area without double confirmation

- **WHEN** the `trash` target's command is inspected
- **THEN** it SHALL remove the contents of `~/.Trash` and mounted-volume trashes (`/Volumes/*/.Trashes/<uid>`), tolerating absent volumes
- **AND** it SHALL NOT require a double confirmation

### Requirement: Pattern-based updater-stub cleanup

The catalog SHALL enumerate application auto-updater caches as container targets whose subitems are discovered by pattern rather than hardcoded per app, so that updater stubs for all installed apps are covered. A `shipit-updaters` container target SHALL enumerate every `~/Library/Caches/*.ShipIt` directory as an individually actionable subitem, **excluding** `com.todesktop.230313mzl4w4u92.ShipIt` (which the existing `shipit` target already covers) so that sizes are never double-counted. An `electron-updaters` container target SHALL enumerate `electron-updater` download caches matching `~/Library/Caches/*updater*` (including `@*updater*`) as subitems. Both SHALL be Tier 1, have no top-level cleanup command, and report Empty when no matching directories exist. Each subitem SHALL delete only its own directory's contents.

#### Scenario: ShipIt updater container enumerates all but the covered app

- **WHEN** the `shipit-updaters` target is listed
- **THEN** every `~/Library/Caches/*.ShipIt` directory SHALL appear as a subitem
- **AND** `com.todesktop.230313mzl4w4u92.ShipIt` SHALL NOT appear (it is covered by the `shipit` target)
- **AND** each subitem's command SHALL remove only that directory's contents

#### Scenario: ShipIt updater container has no top-level command

- **WHEN** the `shipit-updaters` target is inspected
- **THEN** it SHALL be Tier 1 and expose no top-level cleanup command, acting only per subitem

#### Scenario: Electron updater container enumerates updater caches

- **WHEN** the `electron-updaters` target is listed
- **THEN** each `~/Library/Caches/*updater*` directory (including `@*updater*`) SHALL appear as a subitem with a command removing only that directory's contents

#### Scenario: Pattern containers report Empty when nothing matches

- **WHEN** no directories match a pattern container's glob
- **THEN** the target SHALL report status Empty and expose no actionable subitems

### Requirement: rustup old-toolchain cleanup keeps the active toolchain

The catalog SHALL include a `rustup` Tier 2 container target that enumerates installed rustup toolchains (`rustup toolchain list`) and offers every non-active toolchain for deletion via `rustup toolchain uninstall '<name>'`, keeping the default/active toolchain. It SHALL report status `ToolMissing` when the `rustup` binary is absent. This mirrors the "keep the version in use" behavior of the `nvm` and `pyenv` targets.

#### Scenario: rustup keeps the default toolchain

- **WHEN** the `rustup` target is listed and more than one toolchain is installed
- **THEN** the default/active toolchain SHALL be kept
- **AND** every other installed toolchain SHALL be offered for deletion with a `rustup toolchain uninstall '<name>'` command

#### Scenario: rustup reports ToolMissing when the binary is absent

- **WHEN** the `rustup` binary is not found on PATH
- **THEN** the `rustup` target SHALL report status `ToolMissing` and expose no actionable subitems

### Requirement: Tier 3 deletion requires explicit confirmation

Because Tier 3 targets hold irreplaceable personal data that no tool regenerates, the system SHALL NOT delete any Tier 3 data without an explicit second confirmation. Every Tier 3 cleanup path — a target's top-level command, a group "clean all" command, and every per-subitem command — SHALL be marked `requires_double_confirm` so that both the frontend confirmation modal and the backend command guard block execution until the user confirms deliberately (mirroring the `SI` double-confirm used for `Docker.raw`). The confirmation prompt SHALL display the exact command before it runs, and cancelling SHALL leave the data untouched. The backend `clean_target`/`clean_item` commands SHALL reject a Tier 3 request that arrives without `confirmed = true`.

#### Scenario: Tier 3 top-level deletion is gated

- **WHEN** the user activates the cleanup action for a non-collection Tier 3 target (e.g. `postgres`, `spark`, `whatsapp`, `notion`, `cursor`)
- **THEN** the double-confirmation prompt SHALL appear showing the exact command
- **AND** the data SHALL be deleted only after the user confirms

#### Scenario: Tier 3 per-subitem deletion is gated

- **WHEN** the user activates a per-subitem cleanup action on a Tier 3 collection target
- **THEN** the double-confirmation prompt SHALL appear before that member is deleted

#### Scenario: Backend rejects unconfirmed Tier 3 requests

- **WHEN** `clean_target` or `clean_item` is invoked for a Tier 3 command with `confirmed = false`
- **THEN** the command SHALL return an error and delete nothing

#### Scenario: Cancelling a Tier 3 confirmation deletes nothing

- **WHEN** the user cancels the double-confirmation prompt for any Tier 3 action
- **THEN** no Tier 3 data SHALL be modified

### Requirement: Group clean-all action for disaggregated targets

The system SHALL let the user delete an entire disaggregated group in one action via a group-level "clean all" control, in addition to per-subitem deletion. A container target that carries a top-level command AND has one or more subitems SHALL render both a group "clean all" control at the target level and individual controls per subitem. The group command SHALL remove every member the subitems represent. For Tier 3 collection targets, the group command SHALL require the double confirmation. Existing container targets that carry no top-level command (e.g. `conductor`, `ollama`, `rustup`, `shipit-updaters`, `electron-updaters`, Android images) SHALL remain per-subitem only and SHALL NOT show a group control.

#### Scenario: Disaggregated Tier 3 target offers a group clean-all control

- **WHEN** a Tier 3 collection target (`downloads`, `chrome-profiles`, `claude-vm`, `utm`) with at least one member is displayed
- **THEN** a group "clean all" control SHALL be shown at the target level alongside the per-subitem controls
- **AND** activating it SHALL open the double-confirmation prompt, then remove every member on confirmation

#### Scenario: Per-subitem-only containers show no group control

- **WHEN** a container target with subitems but no top-level command is displayed (e.g. `conductor`, `ollama`, `rustup`)
- **THEN** no group "clean all" control SHALL be shown and cleanup remains per subitem

### Requirement: Targets and subitems ordered by size within their group

The system SHALL order cleanup candidates by measured size **descending** (largest first) within their group, where a "group" is a single tier for targets and a single container target for its subitems. Ordering SHALL never cross group boundaries: the fixed Tier 1 → Tier 2 → Tier 3 group order SHALL be preserved, and a larger target in a later tier SHALL NOT appear above a smaller target in an earlier tier. When two candidates in the same group have equal `sizeBytes` — including the pre-measurement state where sizes are still `0` — the system SHALL fall back to a deterministic alphabetical order (by target `name`, and by subitem `label`) so that skeleton rows shown before measurement remain stable.

#### Scenario: Targets ordered largest-first within a tier

- **WHEN** the review surface shows measured targets within a tier
- **THEN** those targets SHALL appear ordered by measured size from largest to smallest
- **AND** the Tier 1 → Tier 2 → Tier 3 grouping order SHALL be unchanged

#### Scenario: Size order never crosses tiers

- **WHEN** a Tier 3 target is larger than every Tier 1 target
- **THEN** the Tier 3 target SHALL still render within the Tier 3 group, below all Tier 1 and Tier 2 targets

#### Scenario: Subitems ordered largest-first within a container target

- **WHEN** a container target is expanded to show its subitems
- **THEN** the subitems SHALL appear ordered by measured size from largest to smallest

#### Scenario: Deterministic order before and during measurement

- **WHEN** the catalog is shown before sizes are measured (all sizes `0`) or several candidates in a group share the same size
- **THEN** those candidates SHALL be ordered alphabetically (targets by `name`, subitems by `label`)
- **AND** rows SHALL settle into size-descending order as each measurement streams in

### Requirement: Stable cleanup command and serialization surface

The disk-cleanup backend SHALL expose exactly four Tauri commands — `scan_cleanup_targets`, `clean_target`, `clean_item`, and `disk_free` — reachable from the crate root as `cleanup::<command>`, regardless of how the module is internally organized. Refactoring the module's file layout SHALL NOT change these command names, their argument or return types, the camelCase JSON field names of `Target`, `Item`, `CleanupScan`, and `CleanResult`, the `cleanup://catalog` and `cleanup://target` event names and payloads, the catalog contents (target ids, tiers, statuses, and cleanup command strings), or any observable runtime behavior. Internal helpers, builders, and target constructors MAY be moved between files and MAY have their visibility narrowed to the module tree.

#### Scenario: Command surface unchanged after refactor

- **WHEN** `lib.rs` registers `cleanup::scan_cleanup_targets`, `cleanup::clean_target`, `cleanup::clean_item`, and `cleanup::disk_free` in `tauri::generate_handler!`
- **THEN** all four commands SHALL resolve without changes to `lib.rs`
- **AND** their argument and return types SHALL be unchanged

#### Scenario: Serialized shapes unchanged after refactor

- **WHEN** `scan_cleanup_targets` emits `cleanup://catalog` and `cleanup://target` events and returns a `CleanupScan`
- **THEN** the event names and the camelCase JSON field names of every payload SHALL be identical to before the refactor

#### Scenario: Catalog contents unchanged after refactor

- **WHEN** `catalog_defs()` is built after the refactor
- **THEN** it SHALL contain the same set of target ids with the same tier, status-detection logic, and cleanup command strings as before the refactor
- **AND** the existing cleanup test suite SHALL pass without modification to its assertions

