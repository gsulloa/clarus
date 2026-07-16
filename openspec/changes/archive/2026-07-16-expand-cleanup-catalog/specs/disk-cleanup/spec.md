## MODIFIED Requirements

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
