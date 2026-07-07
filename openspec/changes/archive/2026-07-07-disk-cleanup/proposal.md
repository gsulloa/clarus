## Why

The Clarus app today ships a generic folder scanner (`scan_directory`) that walks an arbitrary directory and heuristically classifies files. That is not the product the user needs. The real workflow the user already trusts lives in `~/disk-cleanup.sh`: a curated, interactive pass over a **fixed list of known caches and regenerable data** on macOS, reviewing each target's size and letting the user clean them one by one. This change turns that proven script into the app's first real feature so the user gets the exact same behavior with a reviewable UI instead of a terminal prompt loop.

## What Changes

- **BREAKING**: Replace the generic `scan_directory` folder-walk feature (as the primary UI surface) with a **catalog-driven cleanup** over the same fixed targets the script checks. The `scan_directory` Rust command may remain in the codebase but its UI is removed.
- Add a **known-target catalog** modeled 1:1 on `~/disk-cleanup.sh`: Tier 1 pure caches, Tier 2 regenerables, Tier 3 informational-only. Each target keeps the script's **exact** detection path and cleanup command so the app does exactly the same thing.
- **Scan-first flow**: measure size + availability of every catalog target before anything is deleted; results grouped by tier with path, size, tool status, and the exact command that would run.
- **Per-item action**: each actionable row (and each subitem of container targets — Docker prune list, iOS runtimes, nvm versions, Conductor workspaces, Android images) has its own button that runs just that cleanup and reports freed space / errors.
- **Double-confirm gate** for the two high-risk cases the script gates: Docker.raw regeneration and deleting a Conductor workspace with uncommitted changes.
- **Live disk-free readout**: free space before / now / freed, recomputed after each action (same `df` semantics as the script).

## Capabilities

### New Capabilities
- `disk-cleanup`: Catalog-driven scan and per-target cleanup of known macOS caches and regenerable data, faithful to `~/disk-cleanup.sh` — including tiered grouping, per-item and per-subitem actions, double-confirm on risky targets, and live disk-free tracking.

### Modified Capabilities
<!-- No existing capability specs exist yet; the generic scan scaffold has no spec. -->

## Impact

- **Backend (Rust)**: new `cleanup` module in `packages/app/src-tauri/src/` (types, catalog, concurrent detection, `scan_cleanup_targets`, `clean_target`, `clean_item`, `disk_free`); commands registered in `lib.rs`. Cleanup commands run via `std::process::Command` bash, matching the script verbatim.
- **Frontend (React)**: new `cleanup/types.ts` + `cleanup/api.ts`; `App.tsx` reworked from the folder-scan surface to the tiered catalog surface (left rail scan-all + disk readout, center tiered list with per-row actions, right evidence panel with exact command, type-to-confirm modal). Reuses `formatBytes` and existing CSS tokens/layout.
- **Removed UI**: folder-picker + generic candidate table in `App.tsx`.
- **Dependencies**: no new frontend Tauri plugins required (shell runs from backend Rust). Optional Tauri event channel for progressive scan results.
- **Platform**: macOS-only behavior (paths, `df -g /System/Volumes/Data`, `osascript`, `xcrun`, `brew`), consistent with the script.
