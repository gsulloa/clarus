## MODIFIED Requirements

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

## ADDED Requirements

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
