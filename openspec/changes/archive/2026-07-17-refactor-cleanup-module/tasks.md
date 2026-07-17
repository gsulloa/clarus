## 1. Extract data model

- [x] 1.1 Create `cleanup/model.rs`; move `Tier`, `Status`, `Item`, `Target`, `CleanupScan`, `CleanResult` with their `derive`/`serde` attributes unchanged.
- [x] 1.2 Change their fields from private to `pub(in crate::cleanup)` so sibling submodules can construct/read them; keep all serde renames.
- [x] 1.3 In `mod.rs` add `mod model;` and confirm the crate builds.

## 2. Extract shell and disk helpers

- [x] 2.1 Create `cleanup/shell.rs` with `home`, `expand`, `sq`, `run_bash`, `has_tool` (marked `pub(in crate::cleanup)`).
- [x] 2.2 Create `cleanup/disk.rs` with `du_bytes`, `size_human`, `DiskUsage`, `disk_usage`, `gib_human`, `parse_df_field_at`, `path_exists`, `parse_human_size` (marked `pub(in crate::cleanup)`; keep the `DiskUsage` doc comments explaining derived `used`).
- [x] 2.3 Wire `mod shell; mod disk;` and fix imports; build.

## 3. Extract generic builders

- [x] 3.1 Create `cleanup/builders.rs`; move `Def` (+ `into_target`), `tier1`, `tier3_simple`, `tier3_collection`, `cache_or_missing`, `electron_cache_target`, `caches_dir_subitems`.
- [x] 3.2 Import `model` types and `shell`/`disk` helpers; mark items `pub(in crate::cleanup)`; build.

## 4. Extract per-domain target builders

- [x] 4.1 Create `cleanup/targets/mod.rs` (declares the domain submodules and re-exports their `*_target` fns via `pub(in crate::cleanup) use`).
- [x] 4.2 `targets/docker.rs`: `docker_raw_path`, `docker_installed`, `docker_prune_target`, `docker_raw_target`.
- [x] 4.3 `targets/devtools.rs`: `ios_unavailable_target`, `ios_runtimes_target`, `android_images_target`.
- [x] 4.4 `targets/runtimes.rs`: `nvm_target`, `pnpm_store_target`, `pyenv_target`, `rustup_target`, `ollama_target` (imports `parse_human_size` from `disk`).
- [x] 4.5 `targets/updaters.rs`: `shipit_updaters_target`, `electron_updaters_target`, `webex_upgrades_target`.
- [x] 4.6 `targets/workspaces.rs`: git/enum helpers (`is_git_dir`, `is_project_container`, `enumerate_workspaces`, `workspace_label_id`, `enumerate_git_repos`, `collect_git_repos`, `existing_dev_roots`), consts (`ARTIFACT_DIRS`, `DEV_ROOT_CANDIDATES`), `artifact_clean_cmd`, `conductor_target`, `conductor_artifacts_target`, `project_artifacts_target`.
- [x] 4.7 `targets/system.rs`: `TEMP_STALE_DAYS`, `system_temp_target`, `quicklook_cache_target`.
- [x] 4.8 Add `mod targets;` and build after each file compiles.

## 5. Extract catalog, measurement, and commands

- [x] 5.1 Create `cleanup/catalog.rs`; move `catalog_defs()` including the inline simple `Def` targets (xcode-archives, xcode-deriveddata, cargo, coresimulator-caches, xcode-devicesupport, trash); import from `builders` and `targets`.
- [x] 5.2 Create `cleanup/measure.rs`; move `measure()`.
- [x] 5.3 Create `cleanup/commands.rs`; move `scan_cleanup_targets`, `clean_result`, `clean_target`, `clean_item`, `disk_free`.
- [x] 5.4 In `mod.rs` add the remaining `mod` decls and re-export the four commands (see deviation note below); keep the top-of-file module doc comment.

## 6. Move tests

- [x] 6.1 Create `cleanup/tests.rs` with the `#[cfg(test)] mod tests` block moved verbatim; adjust imports (`use super::*` / explicit `use crate::cleanup::...`) so assertions compile unchanged.
- [x] 6.2 Declare `#[cfg(test)] mod tests;` in `mod.rs`.

## 7. Verify

- [x] 7.1 `cargo build` and `cargo clippy` are clean (no new warnings) for the `app` crate.
- [x] 7.2 `cargo test` passes with no assertion changes.
- [x] 7.3 Confirm `src/lib.rs` still resolves `cleanup::{scan_cleanup_targets, clean_target, clean_item, disk_free}` with no edits.
- [ ] 7.4 Run the app: trigger a cleanup scan and confirm the `cleanup://catalog` / `cleanup://target` events, target list, sizes, and JSON field names are identical to pre-refactor; clean one low-risk Tier 1 target to confirm `clean_target` behavior is unchanged.

**Deviation from plan (5.4):** `mod.rs` re-exports the four commands with `pub use commands::*;` instead of the originally planned `pub use commands::{scan_cleanup_targets, clean_target, clean_item, disk_free};`. `#[tauri::command]` generates hidden `__cmd__<fn>` / `__tauri_command_name_<fn>` helper items alongside each function in the same module; `tauri::generate_handler!` in `lib.rs` resolves those at `cleanup::__cmd__<fn>`, so they must be re-exported too, or the four named-only re-exports leave those hidden items unreachable and `lib.rs` fails to compile (verified: `error[E0433]: cannot find __cmd__scan_cleanup_targets in cleanup`, etc.). The glob re-export brings in exactly the same four public commands plus their required macro-generated companions; no other public surface changes.
