## Why

A disk-analysis session surfaced ~20 GB of recoverable space across cleanup targets that Clarus does not yet enumerate (pnpm store, Gradle caches, Webex/Slack/Claude Desktop caches, AWS Toolkit, Cursor VSIX cache, Ollama models, pyenv versions). Adding these to the catalog closes gaps between what a manual `disk-cleanup.sh` run finds and what the app can show and act on.

## What Changes

- Add **9 Tier 1** (auto-regenerable, single-click) targets: `pnpm-store`, `gradle-caches`, `gradle-wrapper`, `gradle-daemon`, `webex-upgrades`, `slack-cache`, `claude-desktop-cache`, `aws-toolkit-cache`, `cursor-vsix-cache`.
- Add **2 Tier 2** container targets with per-subitem cleanup: `ollama` (enumerate `ollama list`, delete per model) and `pyenv` (enumerate installed versions, keep the active version, offer the rest) — mirroring the existing `nvm` pattern.
- `webex-upgrades` is a keep-newest target: it deletes every version directory under `Webexteams_upgrades_arm/` except the highest-versioned one.
- Targets whose backing tool may be absent (`pnpm`, `ollama`, `pyenv`, `webex`, `slack`, `claude-desktop`, `cursor`, `aws`) report `ToolMissing`/`NotInstalled`/`Empty` when not present, and their action is disabled — no new destructive behavior on machines that lack them.
- Tier 3 `notion` from the issue is **already present** in the catalog; no change needed there (noted for completeness).

## Capabilities

### New Capabilities

_None. This change extends existing catalog behavior; it introduces no new capability._

### Modified Capabilities

- `disk-cleanup`: the "Catalog of known cleanup targets" requirement expands to include the new Tier 1 and Tier 2 targets, and the "Per-subitem cleanup for container targets" requirement adds `ollama` and `pyenv` as container targets (pyenv keeps the active version, matching the nvm rule).

## Impact

- **Code:** `packages/app/src-tauri/src/cleanup/mod.rs` — new `tier1(...)` entries in `catalog_defs()` plus new container builders `ollama_target()`, `pyenv_target()`, and a keep-newest `webex_upgrades_target()`. No changes to the `Target`/`Item` structs, the scan/measure pipeline, or the Tauri command surface.
- **Frontend:** none — new targets render through the existing tier-grouped catalog UI and `clean_target` / `clean_item` invoke paths; the TS mirror in `types.ts` is unchanged.
- **Risk:** all additions are Tier 1 caches or Tier 2 regenerables with the existing "disabled when nothing to clean" guard; no Tier 3 deletion, no new double-confirm gates.
