## 1. Shared discovery helpers (mod.rs)

- [x] 1.1 Add a `DEV_ROOT_CANDIDATES` constant listing conventional root names (`dev`, `Developer`, `Projects`, `src`, `code`, `repos`, `work`, `git`, `workspace`)
- [x] 1.2 Add a helper that returns existing dev roots: map each candidate to `{home}/{name}` and keep those passing `path_exists()`
- [x] 1.3 Add a helper `enumerate_git_repos(root, max_depth)` that finds directories containing a `.git` entry (bounded depth, prune on first hit, skip symlinks) and returns their paths
- [x] 1.4 Add exclusion so any path under `{home}/conductor/workspaces` is filtered out of the discovered repos

## 2. Project build artifacts target (Tier 1)

- [x] 2.1 Implement `project_artifacts_target()` mirroring `conductor_artifacts_target()`: id `project-artifacts`, name "Project build artifacts", Tier 1, no double-confirm
- [x] 2.2 Return `Status::NotInstalled` when no dev roots exist; `Status::Empty` when roots exist but no repos found; else `Status::Available`
- [x] 2.3 For each discovered repo, push an `Item` whose command reuses `artifact_clean_cmd(repo)`; label by repo name (disambiguate duplicates by parent, like `workspace_label_id`)
- [x] 2.4 Set reason/risk_note/caveat consistent with `conductor-artifacts` ("Fully regenerable — reinstall/rebuild to recreate.")
- [x] 2.5 Register `project_artifacts_target()` in `catalog_defs()`

## 3. System temporary files target (Tier 1)

- [x] 3.1 Add helper to resolve the Darwin temp base via `getconf DARWIN_USER_TEMP_DIR` (mirror the QuickLook `getconf` usage); derive `T/` and sibling `X/` paths
- [x] 3.2 Add an age-gate constant (default 3 days) and build the `T/` cleanup command: `find <T> -mindepth 1 -maxdepth 1 -mtime +N -exec rm -rf {} + 2>/dev/null; true`
- [x] 3.3 Build the `X/` cleanup command targeting `com.google.Chrome.code_sign_clone` orphans, quitting Chrome first (reuse the app-quit pattern from the `chrome` target)
- [x] 3.4 Implement `system_temp_target()`: id `system-temp`, name "System temporary files", Tier 1, no double-confirm; `Status::Empty` when nothing eligible; combine the two commands
- [x] 3.5 Register `system_temp_target()` in `catalog_defs()`

## 4. Verification

- [x] 4.1 `cargo build`/`cargo clippy` clean in `packages/app/src-tauri`
- [x] 4.2 Manual scan: both targets appear as Tier 1; project target lists real repos under existing roots and none under `~/conductor/workspaces`
- [x] 4.3 Verify project cleanup removes only `ARTIFACT_DIRS` and leaves `.git`/source intact (test on a throwaway repo copy)
- [x] 4.4 Verify temp cleanup age-gate: a >N-day entry is removed, a fresh entry is preserved; Chrome is quit before `X/` removal
- [x] 4.5 Confirm the disk-free readout updates after each cleanup
- [x] 4.6 Add/adjust any TypeScript type mirrors if new ids/labels need surfacing (no path logic expected)
