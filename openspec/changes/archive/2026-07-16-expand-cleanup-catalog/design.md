## Context

The cleanup catalog is defined entirely in `packages/app/src-tauri/src/cleanup/mod.rs` via `catalog_defs() -> Vec<Target>`. Each entry is a `Target` (id, name, tier, optional `path` for `du` sizing, `command: Option<String>`, and `subitems: Vec<Item>`). Two builder shapes already exist:

- **Simple target** — `tier1(id, name, path, reason, caveat, command)` for pure caches (single path, single command).
- **Container target** — a `Def { command: None, subitems: [...] }` where each `Item` carries its own `command`; used by `nvm_target()`, `ios_runtimes_target()`, `conductor_target()`, `android_images_target()`.

Sizing (`measure`), command execution (`run_bash` via `bash -lc`), tool detection (`has_tool` / `command -v`), and the Tauri command surface (`scan_cleanup_targets`, `clean_target`, `clean_item`) are generic and require no changes. The frontend renders whatever the catalog emits, grouped by tier, so no UI work is needed. There are no Rust unit tests for the catalog today.

This change adds 9 Tier 1 simple targets and 2 Tier 2 container targets by following those existing patterns.

## Goals / Non-Goals

**Goals:**
- Add the Tier 1 and Tier 2 targets from issue #4 using the established `tier1(...)` and container-builder patterns.
- Ensure each new target degrades gracefully (Empty / ToolMissing / NotInstalled, action disabled) on machines lacking the tool or path.
- Keep additions faithful to the issue's cleanup commands; introduce no new destructive semantics or confirmation gates.

**Non-Goals:**
- No changes to the `Target`/`Item` structs, the scan/measure pipeline, event protocol, or Tauri commands.
- No frontend changes.
- No addition of `notion` (already present as a Tier 3 target).
- No general multi-path sizing mechanism (see Risks for the single-path trade-off).

## Decisions

### 1. Tier 1 additions use `tier1(...)` where a single path exists; a custom `Def` where the tool may be missing

`pnpm-store`, `gradle-caches`, `gradle-wrapper`, `gradle-daemon`, `slack-cache`, `claude-desktop-cache`, `aws-toolkit-cache`, `cursor-vsix-cache` map cleanly onto a single detection path and command. For pure `rm -rf` targets whose directory may simply be absent, `tier1(...)` is sufficient — `measure` already downgrades `Available → Empty` when a simple command target measures 0 bytes, which disables the action.

For `pnpm-store` the command is tool-native (`pnpm store prune`), so when `pnpm` is absent the target SHALL be built as a `Def` with `command: None` and `status: ToolMissing` (mirroring the Homebrew-not-installed branch), rather than emitting a command that would fail. Resolve the store path with `pnpm store path` when `pnpm` is present, falling back to `~/Library/pnpm` for sizing.

- Commands (verbatim from the issue):
  - `pnpm-store`: `pnpm store prune`
  - `gradle-caches`: `gradle --stop 2>/dev/null; rm -rf ~/.gradle/caches`
  - `gradle-wrapper`: `rm -rf ~/.gradle/wrapper/dists`
  - `gradle-daemon`: `rm -rf ~/.gradle/daemon`
  - `slack-cache`: `rm -rf ~/Library/Application\ Support/Slack/Cache ~/Library/Application\ Support/Slack/Service\ Worker`
  - `claude-desktop-cache`: `rm -rf ~/Library/Application\ Support/Claude/Cache ~/Library/Application\ Support/Claude/Code\ Cache`
  - `aws-toolkit-cache`: `rm -rf ~/Library/Caches/aws`
  - `cursor-vsix-cache`: `rm -rf ~/Library/Application\ Support/Cursor/CachedExtensionVSIXs`

**Alternative considered:** making Slack/Claude multi-path targets first-class (summing du over both dirs). Rejected — it would require a struct/`measure` change disproportionate to the benefit; a single primary path (see Risks) is good enough.

### 2. `webex-upgrades` is a keep-newest simple target, not a full-dir wipe

The `Webexteams_upgrades_arm/` directory holds one folder per version; only the newest must be kept. Implement as a simple Tier 1 target with `path` = the upgrades dir (for sizing) and the issue's keep-newest one-liner as the command:

```sh
ls -d ~/Library/Application\ Support/Cisco\ Spark/Webexteams_upgrades_arm/*/ 2>/dev/null \
  | sort -V | head -n -1 | xargs rm -rf
```

**Alternative considered:** modeling it as a container with a per-version subitem (like Android images / iOS runtimes) and keeping the newest. Rejected for now — the issue classifies it as a zero-risk, single-click Tier 1 target, and the one-liner is self-contained. The trade-off is that the reported size includes the kept newest version (see Risks).

### 3. `pyenv` mirrors `nvm_target()` exactly

Build `pyenv_target()` as a Tier 2 container: resolve `~/.pyenv/versions`, return `NotInstalled` if the dir/binary is absent, determine the active version via `pyenv version-name`, and emit one `Item` per other installed version with `command: pyenv uninstall -f '<version>' 2>&1 || rm -rf '<path>'` and `path` = the version dir (so per-subitem sizing works). Keep the same `Status::Empty` when no removable versions remain.

### 4. `ollama` enumerates `ollama list`

Build `ollama_target()` as a Tier 2 container: if `ollama` is not on PATH, `status: NotInstalled`, no subitems. Otherwise parse `ollama list` (tab/space-delimited: NAME, ID, SIZE, MODIFIED) and emit one `Item` per model with `command: ollama rm '<name>'`. Because models are not backed by a single simple filesystem path we can `du`, set each subitem's `path` empty and its `size_bytes` from the parsed `ollama list` SIZE column (converting the human size, e.g. `4.1 GB`, to bytes). This keeps `measure` from double-counting and gives an accurate per-model and total size without touching the store internals.

**Alternative considered:** `du` the Ollama blob store. Rejected — models share deduplicated blobs, so per-model `du` is not well-defined; the `ollama list` SIZE column is the authoritative per-model figure.

### 5. Insertion points in `catalog_defs()`

New Tier 1 targets are pushed within the Tier 1 block (after `bun`); `pyenv` and `ollama` are pushed in the Tier 2 block near `nvm`. Ordering is cosmetic (UI groups by tier), so group related targets (all Gradle entries adjacent) for readability.

## Risks / Trade-offs

- **Single-path sizing undercounts multi-dir targets** (`slack-cache`, `claude-desktop-cache`) → point `path` at the dominant cache dir (Slack `Cache`, Claude `Cache`); the command still clears the secondary dir (`Service Worker` / `Code Cache`). Reported size is slightly conservative, which is the safe direction (never overstates recoverable space).
- **`webex-upgrades` size includes the kept newest version** → the row's size over-reports what will actually be freed. Acceptable for a Tier 1 informational size estimate; the command itself is correct. Note it in the target's `caveat`.
- **`pnpm store prune` only removes orphaned packages** (not the whole store) → freed space will be less than the measured store size. Set a `caveat` explaining prune semantics so the size is not read as fully recoverable.
- **Parsing `ollama list` output format** could break if Ollama changes its columns → guard the parse (skip the header row, tolerate missing SIZE by defaulting to 0) so a format change degrades to a zero-size but still-actionable subitem rather than a panic.
- **No automated tests exist for the catalog** → verification is manual (run a scan, confirm new rows appear with correct tier/status). Adding a Rust test harness is out of scope; if desired, a small unit test asserting `catalog_defs()` contains the new IDs with expected tiers would be the minimal first test.

## Open Questions

- Should the three Gradle entries be one combined target or three separate ones? This design keeps them **separate** (matching the issue's three IDs and giving per-item control); revisit only if the UI feels cluttered.
