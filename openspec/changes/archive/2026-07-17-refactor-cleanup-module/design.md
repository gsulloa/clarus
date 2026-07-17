## Context

`packages/app/src-tauri/src/cleanup/mod.rs` is 2,639 lines and already internally sectioned by banner comments: TYPES, HELPERS, CATALOG DEFINITIONS, container/special target builders, SIZE MEASUREMENT, TAURI COMMANDS, TESTS. It is the only file in `cleanup/`. `src/lib.rs` consumes exactly four symbols — `cleanup::scan_cleanup_targets`, `cleanup::clean_target`, `cleanup::clean_item`, `cleanup::disk_free` — via `tauri::generate_handler!`. The frontend depends on the emitted `cleanup://catalog` / `cleanup://target` events and the camelCase JSON of `Target`/`Item`/`CleanupScan`/`CleanResult`. The `#[cfg(test)]` suite exercises `catalog_defs()` and reads private fields of `Target`/`Item`.

This is a pure structural refactor: move code into submodules, adjust visibility, change nothing observable.

## Goals / Non-Goals

**Goals:**
- Decompose the file into cohesive submodules following the existing section boundaries.
- Keep `cleanup::{scan_cleanup_targets, clean_target, clean_item, disk_free}` reachable with no change to `lib.rs`.
- Preserve every command string, catalog entry, event, and JSON field exactly.
- Keep the existing test suite passing with only import-path adjustments (no assertion changes).

**Non-Goals:**
- No behavior, catalog, or command-string changes.
- No new dependencies or new targets.
- No change to the emitted events or serialized shapes.
- No refactor of the unrelated `scan/` module.

## Decisions

**Module layout.** Split `cleanup/mod.rs` into:

| File | Contents |
|------|----------|
| `mod.rs` | Module doc comment; `mod` declarations; `pub use commands::{scan_cleanup_targets, clean_target, clean_item, disk_free};` |
| `model.rs` | `Tier`, `Status`, `Item`, `Target`, `CleanupScan`, `CleanResult` |
| `shell.rs` | `home`, `expand`, `sq`, `run_bash`, `has_tool` |
| `disk.rs` | `du_bytes`, `size_human`, `DiskUsage`, `disk_usage`, `gib_human`, `parse_df_field_at`, `path_exists`, `parse_human_size` |
| `builders.rs` | `Def` (+ `into_target`), `tier1`, `tier3_simple`, `tier3_collection`, `cache_or_missing`, `electron_cache_target`, `caches_dir_subitems` |
| `targets/mod.rs` | `mod` decls + `pub(in crate::cleanup) use` re-exporting each `*_target` fn |
| `targets/docker.rs` | `docker_raw_path`, `docker_installed`, `docker_prune_target`, `docker_raw_target` |
| `targets/devtools.rs` | `ios_unavailable_target`, `ios_runtimes_target`, `android_images_target` |
| `targets/runtimes.rs` | `nvm_target`, `pnpm_store_target`, `pyenv_target`, `rustup_target`, `ollama_target` |
| `targets/updaters.rs` | `shipit_updaters_target`, `electron_updaters_target`, `webex_upgrades_target` |
| `targets/workspaces.rs` | `is_git_dir`, `is_project_container`, `enumerate_workspaces`, `workspace_label_id`, `conductor_target`, `ARTIFACT_DIRS`, `artifact_clean_cmd`, `conductor_artifacts_target`, `DEV_ROOT_CANDIDATES`, `existing_dev_roots`, `enumerate_git_repos`, `collect_git_repos`, `project_artifacts_target` |
| `targets/system.rs` | `TEMP_STALE_DAYS`, `system_temp_target`, `quicklook_cache_target` |
| `catalog.rs` | `catalog_defs()` including the inline simple `Def` targets (xcode-archives, xcode-deriveddata, cargo, coresimulator-caches, xcode-devicesupport, trash) |
| `measure.rs` | `measure()` |
| `commands.rs` | `scan_cleanup_targets`, `clean_result`, `clean_target`, `clean_item`, `disk_free` |
| `tests.rs` | the `#[cfg(test)] mod tests` block, moved verbatim |

Rationale: the split mirrors the file's own comment sections; `targets/` groups the many per-target builders by the tool/domain they clean so a reader looking for "the Docker logic" or "the workspace logic" opens one small file. Alternative — a single `targets.rs` — was rejected because it would still be ~1,500 lines.

**Visibility.** Everything except the four Tauri commands is crate-internal. Shared items (helpers, `Def`, builder fns, `catalog_defs`, `measure`, target fns) become `pub(in crate::cleanup)` so any submodule (and `tests.rs`) can use them while nothing leaks outside the module. `Target`/`Item`/`CleanupScan`/`CleanResult` **fields** become `pub(in crate::cleanup)` so `builders.rs` (via `Def::into_target`), `measure.rs`, `commands.rs`, and `tests.rs` can construct/read them across file boundaries — they are constructed in the same module today, so this only widens from private to module-scoped. `serde` attributes stay on the fields unchanged, preserving the JSON contract. `pub(in crate::cleanup)` is preferred over `pub(crate)` to keep the surface as tight as it is today.

**`catalog.rs` imports.** `catalog_defs()` calls builders from `builders.rs` and every `*_target` from `targets::*`; it imports them with `use super::builders::*;` and `use super::targets::*;`. The inline simple `Def { … }.into_target()` targets stay in `catalog.rs` since they are one-offs tied to the assembly order.

**`parse_human_size` placement.** It is a size parser used by `ollama_target` and asserted by tests, so it lives in `disk.rs`; `runtimes.rs` and `tests.rs` import it from there.

**Test module.** `tests.rs` currently does `use super::*;`. After the move it becomes `use crate::cleanup::{catalog_defs, Target, Tier, Status, parse_human_size, ...}` (or `use super::*` if `mod tests;` is declared in `mod.rs` and `mod.rs` re-exports the needed items internally). Assertions are unchanged.

## Risks / Trade-offs

- [Behavior drift while moving ~2,600 lines] → The move is mechanical (cut/paste + adjust `use`/visibility, no logic edits). The existing test suite plus a `cargo build`/`cargo clippy` clean run guard against regressions; a manual scan+clean smoke test in the app confirms events and sizes are unchanged.
- [Visibility churn touching many fields] → Confined to widening private → `pub(in crate::cleanup)`; the external surface is unchanged, verified by `lib.rs` compiling untouched.
- [Import cycles between `catalog.rs`, `builders.rs`, and `targets/`] → Acyclic by construction: `model` and helpers (`shell`, `disk`) are leaves; `builders` and `targets` depend on them; `catalog` depends on `builders` + `targets`; `commands` depends on `catalog` + `measure` + `disk`. No submodule depends on `commands` or `catalog` except as listed.
- [Many small files for one feature] → Justified by the 2,639-line starting point; each resulting file is a single, nameable concern.

## Migration Plan

Mechanical, single-commit (or few-commit) refactor, module by module so each step compiles:
1. Create `model.rs`; move types; set fields `pub(in crate::cleanup)`; add `mod model; use model::*;` to `mod.rs`. Build.
2. Move helpers into `shell.rs` + `disk.rs`; wire `mod` + `use`. Build.
3. Move `Def` + generic builders into `builders.rs`. Build.
4. Create `targets/` and move each domain's `*_target` fns; add `targets/mod.rs` re-exports. Build after each file.
5. Move `catalog_defs` → `catalog.rs`, `measure` → `measure.rs`, commands + `clean_result` → `commands.rs`; `mod.rs` re-exports the four commands. Build.
6. Move tests → `tests.rs`; fix imports. `cargo test`.
7. `cargo clippy` clean; run the app, scan, and clean a low-risk target to confirm identical events/output.

Rollback: revert the commit(s); no data or schema migration is involved.

## Open Questions

None.
