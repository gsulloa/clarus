## 1. Rust helpers for workspace enumeration

- [x] 1.1 Add `is_git_dir(path)` helper — returns true if `path/.git` exists
- [x] 1.2 Add `is_project_container(path)` helper — no own `.git`, at least one child dir has `.git`
- [x] 1.3 Add `enumerate_workspaces(dir)` helper — expands project containers one level, passes flat workspaces through
- [x] 1.4 Add `workspace_label_id(ws, workspaces_dir)` helper — returns `(label, id)` using `project/workspace` format and `project__workspace` ID

## 2. Fix Conductor workspace target

- [x] 2.1 Rewrite `conductor_target()` to call `enumerate_workspaces()` instead of a single `read_dir` one level deep
- [x] 2.2 Use `workspace_label_id()` for subitem labels and IDs
- [x] 2.3 Verify git branch/dirty checks run on the correct individual workspace paths (not project container paths)
- [x] 2.4 Confirm `requires_double_confirm` and `rm -rf` command are applied per individual workspace

## 3. New Tier 1 artifact-cleanup target

- [x] 3.1 Define `ARTIFACT_DIRS` constant with the list: `node_modules`, `.next`, `dist`, `cdk.out`, `.turbo`, `target`, `__pycache__`, `.venv`, `venv`, `build`, `.cache`, `.parcel-cache`
- [x] 3.2 Implement `artifact_clean_cmd(ws_str)` — generates `find … -maxdepth 6 -prune -exec rm -rf {} +`
- [x] 3.3 Implement `conductor_artifacts_target()` — Tier 1, `requires_double_confirm: false`, empty path on subitems
- [x] 3.4 Register `conductor_artifacts_target()` in `catalog_defs()` before `conductor_target()`

## 4. Verification

- [x] 4.1 `cargo check` passes with no errors
- [ ] 4.2 Run app, trigger scan — Tier 1 shows "Conductor — regenerable artifacts" with one subitem per individual workspace (not project container)
- [ ] 4.3 Run app, trigger scan — Tier 2 shows "Conductor workspaces" with `project/workspace` labels for nested workspaces and plain names for flat ones
- [ ] 4.4 Confirm artifact cleanup action runs without confirm prompt and updates disk-free readout
- [ ] 4.5 Confirm dirty workspace still shows danger styling and requires "SI" confirm before deletion
