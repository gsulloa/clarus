## 1. Tier 1 — simple cache targets

- [x] 1.1 In `catalog_defs()` (Tier 1 block, after `bun`), add `gradle-caches` via `tier1(...)` with path `~/.gradle/caches` and command `gradle --stop 2>/dev/null; rm -rf ~/.gradle/caches`
- [x] 1.2 Add `gradle-wrapper` via `tier1(...)` with path `~/.gradle/wrapper/dists` and command `rm -rf ~/.gradle/wrapper/dists`
- [x] 1.3 Add `gradle-daemon` via `tier1(...)` with path `~/.gradle/daemon` and command `rm -rf ~/.gradle/daemon`
- [x] 1.4 Add `slack-cache` via `tier1(...)` with path `~/Library/Application Support/Slack/Cache` and command removing both `Slack/Cache` and `Slack/Service Worker`
- [x] 1.5 Add `claude-desktop-cache` via `tier1(...)` with path `~/Library/Application Support/Claude/Cache` and command removing both `Claude/Cache` and `Claude/Code Cache`
- [x] 1.6 Add `aws-toolkit-cache` via `tier1(...)` with path `~/Library/Caches/aws` and command `rm -rf ~/Library/Caches/aws`
- [x] 1.7 Add `cursor-vsix-cache` via `tier1(...)` with path `~/Library/Application Support/Cursor/CachedExtensionVSIXs` and command `rm -rf .../CachedExtensionVSIXs`

## 2. Tier 1 — targets needing tool/keep-newest handling

- [x] 2.1 Add `pnpm-store`: when `pnpm` is present resolve the store path via `pnpm store path` (fallback `~/Library/pnpm`) and set command `pnpm store prune` with a `caveat` explaining prune only removes orphaned packages; when `pnpm` is absent build a `Def` with `command: None` and `status: ToolMissing`
- [x] 2.2 Add `webex_upgrades_target()` (Tier 1) with path `~/Library/Application Support/Cisco Spark/Webexteams_upgrades_arm` and the keep-newest command (`ls -d .../*/ | sort -V | head -n -1 | xargs rm -rf`); set a `caveat` noting the newest version is kept and included in the reported size

## 3. Tier 2 — container targets

- [x] 3.1 Add `pyenv_target()` mirroring `nvm_target()`: resolve `~/.pyenv/versions`, `NotInstalled` if absent, keep the active version (`pyenv version-name`), emit one `Item` per other version with `path` = version dir and command `pyenv uninstall -f '<v>' 2>&1 || rm -rf '<path>'`; `Status::Empty` when nothing is removable
- [x] 3.2 Add `ollama_target()` (Tier 2 container): `NotInstalled` when `ollama` is off PATH; otherwise parse `ollama list` (skip header), emit one `Item` per model with empty `path`, `size_bytes` derived from the SIZE column, and command `ollama rm '<name>'`; tolerate missing/garbled SIZE by defaulting to 0
- [x] 3.3 Register `pyenv_target()` and `ollama_target()` in the Tier 2 block of `catalog_defs()` near `nvm`

## 4. Verification

- [x] 4.1 `cargo check` / `pnpm typecheck` pass with the new targets
- [x] 4.2 Run a scan (`pnpm tauri:dev`) and confirm the 9 Tier 1 and 2 Tier 2 targets render under the correct tier headings
- [x] 4.3 Confirm graceful degradation: on a machine lacking a given tool/path the row shows Empty/ToolMissing/NotInstalled and its action is disabled
- [x] 4.4 Confirm `pyenv` keeps the active version and `ollama` lists one row per `ollama list` model, each with a working per-item clean action
- [x] 4.5 Update the disk-cleanup spec (`openspec/specs/disk-cleanup/spec.md`) at archive time via `openspec archive`, and confirm `notion` remains the only Tier 3 entry (no duplicate added)
