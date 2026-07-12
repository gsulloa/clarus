## Why

The Conductor cleanup target reads `~/conductor/workspaces` only one level deep, so project containers like `backend/` or `tub2/` appear as single deletable blobs instead of being expanded to their individual workspaces (`backend/paris`, `backend/kingston-v2`, etc.). This means a user who wants to delete one workspace must delete an entire project at once, and the per-workspace git-state checks are never applied to nested workspaces. Additionally, there is no way to reclaim disk space from regenerable build artifacts (node_modules, .next, cdk.out, dist, etc.) inside a workspace without deleting the whole workspace.

## What Changes

- **Fix Conductor workspace enumeration**: detect project containers (directories with no `.git` of their own but with children that do) and expand them one level; each individual workspace becomes its own subitem labeled `project/workspace` (e.g. `backend/paris`, `tub2/chengdu-v4`). Flat workspaces (`.git` at the top level) continue to appear by their own name.
- **Deduplicate item IDs**: workspace IDs use `project__workspace` double-underscore convention to avoid collisions when two projects have a workspace with the same name (e.g. `backend/kingston-v2` vs `clarus/kingston-v2`).
- **Add new Tier 1 target — "Conductor — regenerable artifacts"**: enumerates the same workspace list but offers per-workspace cleanup of only regenerable artifact directories (`node_modules`, `.next`, `dist`, `target`, `cdk.out`, `.turbo`, `__pycache__`, `.venv`, `venv`, `build`, `.cache`, `.parcel-cache`) using a `find … -prune -exec rm -rf` command. No double-confirm required.

## Capabilities

### New Capabilities

- `conductor-artifact-cleanup`: Tier 1 sub-catalog target that cleans only regenerable build outputs inside Conductor workspaces without touching git history or source files.

### Modified Capabilities

- `disk-cleanup`: The per-subitem requirement for Conductor workspaces now requires two-level enumeration (project containers expanded to constituent workspaces), and the catalog gains the new Tier 1 artifact-cleanup target.

## Impact

- `packages/app/src-tauri/src/cleanup/mod.rs`: three new helpers (`is_git_dir`, `is_project_container`, `enumerate_workspaces`), rewritten `conductor_target()`, new `conductor_artifacts_target()`, new `ARTIFACT_DIRS` constant, new `artifact_clean_cmd()` helper, one additional `targets.push()` in `catalog_defs()`.
- No frontend changes required; the existing subitem rendering and `clean_item` Tauri command handle the new target transparently.
- No new Tauri commands or TypeScript types needed.
