## ADDED Requirements

### Requirement: Additional high-impact cleanup targets

The catalog SHALL additionally include the following high-impact targets identified during disk analysis, none of which change any existing target's tier or command. Each new Tier 1/Tier 2 target SHALL report a status distinguishing Available, Empty, ToolMissing, or NotInstalled based on its detection path or backing tool, and SHALL disable its cleanup action when nothing is cleanable. New targets whose command is not present in `~/disk-cleanup.sh` MAY diverge from the script.

The catalog SHALL include, at minimum:

- **Tier 1**: `docker-scout` (`~/.docker/scout`), `uv-cache` (`~/.cache/uv`), `puppeteer-cache` (`~/.cache/puppeteer`), `node-gyp` (`~/Library/Caches/node-gyp`), `vscode-cache` (VS Code `Cache` + `CachedData`), `cursor-cache` (Cursor `Cache` + `CachedData`), `user-logs` (`~/Library/Logs`), `quicklook-cache` (QuickLook thumbnail cache), `tableplus-cache` (`~/Library/Caches/com.tinyapp.TablePlus`), `discord-cache`, `notion-cache`, `teams-cache`, `postman-cache`, and `zoom-cache` (each app's HTTP/GPU caches).
- **Tier 2**: `coresimulator-caches` (`~/Library/Developer/CoreSimulator/Caches`), `xcode-devicesupport` (iOS/watchOS/tvOS DeviceSupport), and `trash` (empty `~/.Trash` and mounted-volume trashes).
- **Tier 3**: `downloads` (`~/Downloads`), informational only.

#### Scenario: New Tier 1 targets appear with correct tier

- **WHEN** the catalog is loaded
- **THEN** `docker-scout`, `uv-cache`, `puppeteer-cache`, `node-gyp`, `vscode-cache`, `cursor-cache`, `user-logs`, `quicklook-cache`, `tableplus-cache`, `discord-cache`, `notion-cache`, `teams-cache`, `postman-cache`, and `zoom-cache` SHALL appear as Tier 1 targets

#### Scenario: New Tier 2 targets appear with correct tier

- **WHEN** the catalog is loaded
- **THEN** `coresimulator-caches`, `xcode-devicesupport`, and `trash` SHALL appear as Tier 2 targets

#### Scenario: Downloads is informational only

- **WHEN** the catalog is loaded
- **THEN** `downloads` SHALL appear as a Tier 3 target with no cleanup command
- **AND** it SHALL show its size for information only and expose no cleanup action

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
