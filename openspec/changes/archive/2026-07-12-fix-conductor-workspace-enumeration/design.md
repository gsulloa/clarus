## Context

`~/conductor/workspaces` has a two-level structure: some top-level entries are **project containers** (directories with no `.git` of their own, whose children are individual workspaces with `.git`) and others are **flat workspaces** (`.git` directly at the top level). The old `conductor_target()` read only one level deep, so project containers appeared as single blobs — running `git` commands on them silently produced empty output and the cleanup command was `rm -rf <project-container>`, deleting all nested workspaces in one shot with no per-workspace git-state checks.

The fix lives entirely in `packages/app/src-tauri/src/cleanup/mod.rs`. No Tauri command signatures, TypeScript types, or frontend components change.

## Goals / Non-Goals

**Goals:**
- Correctly distinguish project containers from flat workspaces and enumerate individual workspaces at the leaf level
- Show `project/workspace` labels so users can identify which project each workspace belongs to
- Use collision-safe item IDs (`project__workspace`) since two projects can share a workspace name
- Add a new Tier 1 target that cleans only regenerable artifact directories without touching the workspace itself

**Non-Goals:**
- UI changes (the existing subitem rendering handles the new data transparently)
- Size measurement for the artifact-cleanup target (showing workspace total as "artifact size" would be misleading; freed space is visible in the disk meter after cleanup)
- Recursion beyond two levels (Conductor always uses exactly two levels)

## Decisions

### Option A: Two separate catalog targets vs Option B: `actions` field on `Item`

Option B would add a `soft_command` or `actions: Vec<Action>` field to the `Item` struct, exposing two buttons per subitem in the UI (Delete workspace / Clean artifacts). This requires changes in four layers: Rust struct, TypeScript mirror type, API call site, and the subitem row component.

**Chose Option A** (two catalog entries): the existing `clean_item` Tauri command routes by `(target_id, item_id)`, so a second target is handled without any command or frontend change. The tier grouping (Tier 1 above Tier 2) naturally communicates the risk difference to the user. Total blast radius: one Rust file.

### `is_project_container` heuristic

A directory is a project container if it has no `.git` entry of its own **and** at least one immediate child directory has `.git`. This correctly handles:
- Monorepos with nested worktrees (they have their own `.git`, so they are treated as flat workspaces, not expanded)
- Empty top-level directories (no children with `.git` → not a container → treated as a flat workspace, shown but no git metadata)

### `find … -prune -exec rm -rf {} +` for artifact cleanup

`-prune` prevents `find` from descending into matched directories, so it won't try to delete `node_modules/foo/node_modules` after already scheduling `node_modules` for deletion. `{}+` batches paths into as few `rm` invocations as possible. `-maxdepth 6` bounds traversal depth as a safety backstop.

### Empty `path` on artifact subitems

The scanner calls `du_bytes(&item.path)` when path is non-empty. Setting path to the workspace root would double-`du` every workspace (the Tier 2 delete target already does it) and display the total workspace size next to a "clean artifacts" button, implying the full size would be freed — which is misleading. Leaving path empty shows `—` in the size column; actual freed space appears in the disk meter immediately after the command runs.

## Risks / Trade-offs

- `build/` and `.cache/` are in the artifact list and can appear in non-regenerable contexts in unusual projects. Risk is low for typical dev workspaces. → Mitigation: omit `out/` (high false-positive risk for non-JS projects) and keep the list to well-known tool directories.
- `target/` matches both Rust (`cargo build`) and Java/Kotlin (Gradle). Both are regenerable. No mitigation needed.
- A project container whose children include non-workspace directories (plain folders without `.git`) is handled gracefully: `enumerate_workspaces` only expands children that are directories; non-git subdirs are added as flat items and show `no-git` branch in the label.
