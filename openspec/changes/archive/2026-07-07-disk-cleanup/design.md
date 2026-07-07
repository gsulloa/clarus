## Context

Clarus is a Tauri desktop app (React frontend, Rust backend) currently shipping a generic `scan_directory` scaffold that walks an arbitrary folder and heuristically classifies files. The user's real, trusted workflow is `~/disk-cleanup.sh` — an interactive terminal script that checks a **fixed catalog** of known macOS caches and regenerable data, shows each one's size, and cleans them one at a time with double-confirm gates on risky operations.

The mental model of the script is *not* "walk a folder." It is "check a known list, act per item." This change re-architects the app's primary feature around that model. The cleanup commands are copied **verbatim** from the script so behavior is identical; the app adds a reviewable UI (tiers, per-row buttons, evidence panel, disk-free counter) in place of the sequential terminal prompts.

Existing reusable pieces: `formatBytes` (`src/format.ts`), the 3-zone shell layout + CSS tokens (`src/styles/global.css`, `DESIGN.md`), the `scan/api.ts` invoke pattern, and the Tauri command registration in `src-tauri/src/lib.rs`.

## Goals / Non-Goals

**Goals:**

- Do **exactly** what `~/disk-cleanup.sh` does — same targets, same detection, same commands, same tiering, same double-confirm gates.
- Scan-first, review, then act per item (and per subitem for container targets).
- Live disk-free readout (before / now / freed) using the script's `df` semantics.
- Fit the existing 3-zone shell and design tokens; prioritize function over new visual design.

**Non-Goals:**

- No move-to-Trash / undo (script does permanent removal of regenerables; we follow the script — this overrides the DESIGN.md MVP note).
- No cleanup of Tier 3 (persistent) data — informational only, matching the script.
- No cross-platform support; macOS-only paths and tools, as in the script.
- No new visual design system work beyond reusing existing tokens.
- The generic `scan_directory` command need not be deleted from Rust, but its UI is removed.

## Decisions

### Decision: Known-target catalog, not a folder walk

Model the feature as a static catalog of targets keyed by tier, each carrying detection path(s), the exact clean command, risk metadata, and (for container targets) a subitem enumerator. This mirrors the script's structure directly and makes the UI a straight render of catalog + measured sizes.

*Alternative considered:* extend the generic walker with an allowlist of paths. Rejected — it can't express tool-native cleans (`yarn cache clean`, `docker prune`, `nvm uninstall`), the quit-app steps, or the per-subitem logic without special-casing anyway.

### Decision: Run commands from Rust via `std::process::Command::new("bash").arg("-c")`

The script's commands are bash one-liners with fallbacks, pipes, and `osascript`. Running them through `bash -c` from the Rust backend reproduces them verbatim with zero translation risk and needs **no** frontend `shell` Tauri plugin. Backend Rust already has full process access.

*Alternative considered:* the `@tauri-apps/plugin-shell` from the frontend. Rejected — broader attack surface, requires allowlisting each command, and re-implements what the script already expresses as bash.

### Decision: Sizes in bytes via `du -sk`, free space via `df`

Compute sizes with `du -sk` → bytes (accurate totals and sorting; the script's `du -sh` + `bytes_to_gb` float parsing loses precision). Display with the existing `formatBytes`. Free space uses `df -g` / `df -h /System/Volumes/Data`, identical to the script's `df_free` / `df_free_human`.

### Decision: Backend command surface

- `scan_cleanup_targets() -> CleanupScan` — `{ freeBeforeGb, freeBeforeHuman, targets: [...] }`. Runs detections concurrently (threads) since `du` on Docker.raw / DerivedData is slow. Optionally emits a Tauri event per target as it finishes so the UI fills progressively.
- `clean_target(id, confirmed) -> CleanResult` — runs the target's exact command; `confirmed` required for double-confirm targets.
- `clean_item(targetId, itemId, confirmed) -> CleanResult` — for a subitem of a container target.
- `disk_free() -> { gb, human }` — recomputed after each action.

Types (serde camelCase):

```
Tier   = One | Two | Three
Status = Available | Empty | ToolMissing | NotInstalled
Target { id, name, tier, path?, sizeBytes, sizeHuman, status,
         reason, riskNote, caveat?, requiresDoubleConfirm,
         command?, subitems: [] }
Item   { id, label, path, sizeBytes, meta?, requiresDoubleConfirm, command }
```

### Decision: Progressive streaming of scan results

Emit a Tauri event per target as its `du` finishes so the UI fills in progressively rather than blocking on the slowest target. The frontend seeds rows from the catalog immediately (names, tiers, commands) and patches sizes/status as events arrive.

*Alternative considered:* one blocking `scan_cleanup_targets` call. Simpler, but a 30s+ freeze on Docker.raw/DerivedData feels broken. Streaming is worth the modest extra code.

### Decision: Frontend fits the existing 3-zone shell

- **Left rail:** "Analyze cleanup targets" scan-all button, disk-free readout (before / now / freed), scan progress. Keep the updater section.
- **Center surface:** targets grouped Tier 1 → Tier 2 → Tier 3; each Tier 1/2 row shows name · path · size · status chip · action button (disabled for Empty/ToolMissing/NotInstalled). Container rows expand to subitem rows with their own buttons. Tier 3 rows are read-only with the "manual only" warning.
- **Right evidence panel:** selected target → reason, risk note, the **exact command**, caveat.
- **Row states:** idle → cleaning (spinner) → done (freed X) / error (message).
- **Confirm dialog:** type-to-confirm modal (type `SI`, mirroring the script's `ask_double`) for Docker.raw regen and dirty Conductor workspaces.

## Risks / Trade-offs

- **Permanent deletion contradicts DESIGN.md's "move to Trash" MVP note.** → The user explicitly asked to do exactly what the script does; targets are regenerable caches/data. Mitigate with danger styling and double-confirm on the two risky targets, and by keeping Tier 3 informational-only.
- **Docker auto-start + 90s wait can stall a cleanup.** → Bound the wait at 90s exactly as the script; surface a "Docker did not start, skipped" result rather than hanging.
- **Quitting Spotify/Chrome/Docker is user-visible side effect.** → Show the caveat in the evidence panel before the action so the user knows the app will close.
- **`bash -c` with interpolated paths.** → Catalog commands are static/literal from the script; subitem paths (workspaces, images, runtime ids) are enumerated by the backend from the filesystem, not user input, and are single-quoted as the script does. No untrusted input reaches the shell.
- **`du` on large targets is slow.** → Run detections concurrently and stream results so the UI stays responsive.
- **jq / cargo-cache / nvm may be absent.** → Reproduce the script's fallbacks exactly (e.g. skip old-runtime pruning without jq; `rm -rf` fallbacks when a tool is missing).

## Migration Plan

1. Land the `cleanup` Rust module + commands alongside the existing `scan` module (no removal yet).
2. Add frontend `cleanup/` types + api.
3. Swap `App.tsx` to the cleanup surface; remove the folder-scan UI.
4. Verify with `pnpm typecheck` + `cargo check`, then `pnpm tauri:dev`: scan, then run one safe clean (pip or Playwright) end to end.
5. Rollback: revert `App.tsx` to the folder-scan surface; the `scan_directory` command remains intact throughout.

## Open Questions

- None blocking. Defaults chosen per the user's "do exactly the same as the script": permanent delete, keep Docker auto-start, replace the generic UI, stream scan results.
