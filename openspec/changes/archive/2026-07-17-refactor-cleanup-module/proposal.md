## Why

`packages/app/src-tauri/src/cleanup/mod.rs` is a single **2,639-line** file that holds the entire disk-cleanup feature: the serializable data model, shell/disk/size helpers, a `Def` builder, ~30 per-target builder functions across three tiers, the concurrent size-measurement pass, four Tauri commands, and the test suite. Everything lives in one flat namespace, so finding a specific target builder, understanding the shared helpers, or adding a new catalog entry means scrolling through unrelated concerns. Splitting it along its already-commented section boundaries makes each concern navigable and independently testable — with zero change to observable behavior.

## What Changes

- Turn `cleanup/mod.rs` into a thin module root (module doc + `mod` declarations + `pub use` of the four Tauri commands) and split its contents into focused submodules:
  - `model.rs`: data types (`Tier`, `Status`, `Item`, `Target`, `CleanupScan`, `CleanResult`).
  - `shell.rs`: shell/env helpers (`home`, `expand`, `sq`, `run_bash`, `has_tool`).
  - `disk.rs`: disk/size helpers (`du_bytes`, `size_human`, `DiskUsage`, `disk_usage`, `gib_human`, `parse_df_field_at`, `path_exists`, `parse_human_size`).
  - `builders.rs`: the `Def` builder and generic constructors (`tier1`, `tier3_simple`, `tier3_collection`, `cache_or_missing`, `electron_cache_target`, `caches_dir_subitems`).
  - `targets/` (one file per domain): `docker.rs`, `devtools.rs` (iOS sims/runtimes, Android images), `runtimes.rs` (nvm, pnpm, pyenv, rustup, ollama), `updaters.rs` (ShipIt/electron updaters, Webex), `workspaces.rs` (Conductor + project-artifact discovery and git enumeration), `system.rs` (QuickLook, system temp).
  - `catalog.rs`: `catalog_defs()` — the assembly of every target, including the inline simple `Def` targets currently in the function.
  - `measure.rs`: the concurrent `measure()` pass.
  - `commands.rs`: the Tauri commands (`scan_cleanup_targets`, `clean_target`, `clean_item`, `disk_free`) and `clean_result`.
  - `tests.rs`: the existing `#[cfg(test)]` suite, moved verbatim with imports adjusted.
- Adjust visibility so cross-submodule sharing works: struct fields and shared helpers become `pub(in crate::cleanup)` (or `pub(crate)`), keeping the crate-external surface unchanged.
- Preserve the exact public surface consumed by `lib.rs` (`cleanup::scan_cleanup_targets`, `clean_target`, `clean_item`, `disk_free`), all cleanup-command strings, the catalog contents, the `cleanup://catalog` / `cleanup://target` events, and every serialized JSON field.
- Pure refactor: **no behavior change**, no new dependencies, no catalog additions or removals.

## Capabilities

### New Capabilities
<!-- None. Behavior is already specified by the existing disk-cleanup capability. -->

### Modified Capabilities
- `disk-cleanup`: Add a requirement pinning the module's public Tauri command surface, catalog stability, and serialization contract as invariants that any refactor (including this one) must preserve. No existing requirement's behavior changes.

## Impact

- Affected code: `packages/app/src-tauri/src/cleanup/` — `mod.rs` shrinks to a module root; new files `model.rs`, `shell.rs`, `disk.rs`, `builders.rs`, `catalog.rs`, `measure.rs`, `commands.rs`, `tests.rs`, and `targets/{mod,docker,devtools,runtimes,updaters,workspaces,system}.rs`.
- No change to `src/lib.rs` command registration, to the frontend, or to the emitted events / JSON shapes.
- No API, dependency, catalog, or data-shape changes; existing behavior is fully preserved. The existing test suite is the primary regression guard.
