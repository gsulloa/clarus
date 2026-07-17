## Context

The cleanup catalog (`packages/app/src-tauri/src/cleanup/mod.rs`, `catalog_defs()`) builds targets from `Def`/`Item`/`Tier` structs. Directory-scoped targets follow a consistent shape: resolve a base path from `home()` or a `getconf` command, `path_exists()`-guard it (returning `Status::NotInstalled` when absent), enumerate sub-items, and emit `rm -rf`-style shell commands per item. Existing reference points:

- `conductor_artifacts_target()` (mod.rs:1898) — cleans `ARTIFACT_DIRS` (mod.rs:1863) inside `~/conductor/workspaces` via `artifact_clean_cmd()`, using a bounded `find -maxdepth 6 ... -prune -exec rm -rf`.
- `quicklook` target — resolves its path through `getconf DARWIN_USER_CACHE_DIR`, proving `getconf`-based path resolution is already an accepted pattern.
- Chrome cache / Spotify targets — quit the app before deleting.

Two gaps motivated this change (see proposal): `~/conductor/workspaces` is hardcoded because it is a fixed Conductor product convention, but a personal dev root like `~/dev` is not universal and must be discovered.

## Goals / Non-Goals

**Goals:**
- Add `project-artifact-cleanup`: discover git repos under conventional dev roots that exist on the machine, clean only `ARTIFACT_DIRS` per repo, exclude Conductor workspaces.
- Add `system-temp-cleanup`: reclaim orphaned Chrome `code_sign_clone` dirs and age-gated stale entries under the per-user Darwin temp dir, resolved via `getconf`.
- Reuse existing scaffolding, tiers, and command patterns; no new confirmation flow.

**Non-Goals:**
- No user-configurable root list / settings UI (candidate for a follow-up).
- No change to `conductor-artifacts` or `conductor` behavior.
- No cleaning of arbitrary non-git directories, and no descent below the bounded depth.
- No sudo-only areas (system `/Library`, Time Machine snapshots, `sleepimage`).

## Decisions

### Decision 1: Discover dev roots by convention + existence check, never hardcode a personal path
Scan a fixed candidate list of conventional roots relative to `home()` — `dev`, `Developer`, `Projects`, `src`, `code`, `repos`, `work`, `git`, `workspace` — and keep only those that `path_exists()`. This directly answers "how did you find `~/dev`?": we do not assume it; we probe well-known names and use whichever exist. On a machine with none, the target reports `Status::NotInstalled`.

*Alternatives considered:* (a) Hardcode `~/dev` — rejected, not portable. (b) Full-home recursive scan for `.git` — rejected, too slow and would surface unrelated repos (e.g. dotfiles, vendored deps). (c) User-configured roots — deferred; conventional discovery covers the common case with zero setup.

### Decision 2: A "project" = a directory containing `.git`, enumerated to bounded depth
Under each existing root, find directories that contain a `.git` entry, to a shallow bounded depth (e.g. `-maxdepth 3`), and stop descending once a repo is found (do not treat submodules/vendored repos as separate top-level projects). Each repo becomes one sub-item whose command is the existing `artifact_clean_cmd(repo)` so cleanup logic is shared verbatim with `conductor-artifacts`.

*Alternatives considered:* treating every `ARTIFACT_DIRS` match anywhere under a root as flat items — rejected; per-repo grouping matches the existing Conductor UX and keeps the list legible.

### Decision 3: Exclude Conductor workspaces to avoid overlap
Skip any candidate path under `~/conductor/workspaces` (already covered by `conductor-artifacts`). If `~/conductor` were itself picked up by a root convention, it is filtered so the two targets never double-list the same repo.

### Decision 4: Resolve the temp dir via `getconf DARWIN_USER_TEMP_DIR`
Mirror the QuickLook target's `getconf` approach for portability across machines/users (the `/private/var/folders/<hash>` segment is per-user and must not be hardcoded). From that base: the sibling `X/` dir holds `code_sign_clone` copies and `T/` holds temp files.

### Decision 5: Age-gate `T/` cleanup; quit Chrome before removing clones
`T/` legitimately contains temp files for running processes, so deletion MUST be age-gated: only remove top-level entries not modified within a threshold (default 3 days) via `find <T> -mindepth 1 -maxdepth 1 -mtime +3 ... -exec rm -rf`. The `X/com.google.Chrome.code_sign_clone` orphans are removed only after quitting Chrome (consistent with the `chrome` cache target). Both are Tier 1 (temporary, regenerable), no double-confirm.

*Alternatives considered:* deleting all of `T/` unconditionally — rejected, risks killing in-use temp files of running apps; a fixed junk-pattern allowlist (`cdk-nextjs-archive-*`, `DockerDesktopUpdates`) — considered as an additional safety layer but age-gating generalizes better across unknown future junk.

## Risks / Trade-offs

- **[Deleting an in-use temp file under `T/`]** → age-gate (`-mtime +N`) so only stale entries are touched; operate only at depth 1 so a live subtree with any recently-touched file is spared by choosing a conservative default (3 days) and never following symlinks.
- **[Removing artifacts from a repo the user did not expect]** → only `ARTIFACT_DIRS` names are removed (never source or `.git`); items are per-repo and listed before the user clicks; Tier 1 no-confirm matches existing `conductor-artifacts` risk posture.
- **[Slow discovery on large home dirs]** → bounded `-maxdepth`, prune on first `.git` hit, and only scan roots that exist; matches the cost profile of existing enumerated targets.
- **[Chrome running during clone cleanup]** → quit Chrome first, as other Chrome targets already do.
- **[A conventional root name collides with something non-project]** → require a `.git` child before treating a dir as a project, so empty/non-repo roots yield no items.

## Migration Plan

Additive only — two new Tier 1 rows. No data migration, no schema change, no removal. Rollback = revert the two builder functions and their `catalog_defs()` registrations; existing targets are untouched.

## Open Questions

- Threshold for `T/` age-gate: default 3 days proposed — confirm during implementation/QA.
- Should the dev-root candidate list be user-extensible via settings now, or in a follow-up? (Proposal defers it.)
