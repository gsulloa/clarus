## Why

A real disk audit (Mac 96% full) surfaced two large reclaimable areas the current catalog does not reach: ~45 GB of stale system temporary files under the per-user `DARWIN_USER_TEMP_DIR` (`/private/var/folders/<hash>/T` and `.../X`), and ~15–20 GB of regenerable build artifacts (`node_modules`, `.next`, `target`, …) living inside git repositories outside Conductor. Today `conductor-artifacts` only cleans artifacts under the hardcoded `~/conductor/workspaces` path — a fixed Conductor product convention — so repos under a personal dev root (e.g. `~/dev`) are invisible. That personal root is not universal, so the new target must discover project roots generically rather than hardcode one user's layout.

## What Changes

- Add a **Tier 1 target "Project build artifacts"** that discovers git repositories under conventional developer-root directories that actually exist on the machine (e.g. `~/dev`, `~/Developer`, `~/Projects`, `~/src`, `~/code`, `~/repos`, `~/work`, `~/git`, `~/workspace`) — never a hardcoded personal path — and offers per-repository cleanup of only the regenerable artifact directory names already used by `conductor-artifacts` (`ARTIFACT_DIRS`). Conductor workspaces are excluded to avoid duplicating the existing target.
- Add a **Tier 1 target "System temporary files"** that reclaims stale entries under the per-user Darwin temp dir, resolved portably via `getconf DARWIN_USER_TEMP_DIR` (same mechanism already used for the QuickLook cache). It clears orphaned Chrome `code_sign_clone` copies under `.../X/` (quitting Chrome first, consistent with other Chrome targets) and age-gated stale entries under `.../T/` (only items untouched for N days) to avoid deleting temp files currently in use.
- Reuse existing catalog scaffolding (`Def`/`Item`/`Tier`, `path_exists`, `home()`, `enumerate_workspaces` patterns) — no new tiers or confirmation flows.

## Capabilities

### New Capabilities
- `project-artifact-cleanup`: Tier 1 target that discovers git repos under conventional dev-root directories (existence-checked, not hardcoded) and cleans only regenerable artifact directories per repo, excluding Conductor workspaces.
- `system-temp-cleanup`: Tier 1 target that reclaims orphaned code-sign clones and age-gated stale entries under the per-user Darwin temp directory, resolved via `getconf`.

### Modified Capabilities
<!-- None: conductor-artifact-cleanup and disk-cleanup behaviors are unchanged; new targets are additive. -->

## Impact

- Code: `packages/app/src-tauri/src/cleanup/mod.rs` — two new target builder functions registered in `catalog_defs()`; likely a shared helper to enumerate git repos under candidate roots and a reusable age-gate for temp cleanup. TypeScript side (`packages/app/src/cleanup/*`) needs no path changes (it mirrors shapes and invokes commands).
- Behavior: two additional Tier 1 rows appear in the app; both no-confirm, both fully regenerable/temporary.
- Risk: bounded — artifact cleanup only removes names in `ARTIFACT_DIRS`; temp cleanup is age-gated and quits Chrome before removing clones.
