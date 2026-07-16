## Why

Clarus today only tells the user how much free space they have on the data volume (and how much a cleanup freed). It never shows the whole picture — total capacity and how much is currently used — so the user can't judge how full the disk really is or put the cleanup opportunities in context. Seeing "I have 40 GB of opportunities" means little without knowing "out of 500 GB, 460 GB used, 40 GB free."

## What Changes

- Extend the disk readout to report **total capacity** and **used space** on the data volume, not just free space.
- Backend: `scan_cleanup_targets`, `disk_free`, `clean_target`, and `clean_item` results carry total and used bytes (from the same `df /System/Volumes/Data` call already used for free), alongside the existing free value.
- Frontend: the "Disk free · data volume" rail section becomes a full disk-usage summary showing Used / Free / Total (and keeps Before/Now/Freed for cleanup progress), ideally with a capacity bar so the user sees at a glance how full the disk is.
- Used and Total are recomputed after each cleanup action, consistent with how Free is recomputed today.

## Capabilities

### New Capabilities
<!-- None; this extends the existing disk-cleanup capability. -->

### Modified Capabilities
- `disk-cleanup`: The scan/readout requirement changes from reporting only free space to reporting total capacity and used space as well; the live disk readout updates all three (used, free, total) after each action.

## Impact

- **Backend (Rust)** `packages/app/src-tauri/src/cleanup/mod.rs`: add helpers to read total and used from `df` (fields already available in the same output), extend `CleanupScan` and `CleanResult` serde structs with total/used bytes (+ human strings), and populate them in `scan_cleanup_targets`, `disk_free`, and `clean_result`.
- **Frontend (React)** `packages/app/src/cleanup/types.ts` + `App.tsx`: extend `CleanupScan`/`CleanResult` TS types and the rail's disk-readout UI to display used/free/total and a capacity bar; keep Before/Now/Freed.
- **Spec**: update `openspec/specs/disk-cleanup/spec.md` scan/readout requirements.
- **Platform**: macOS-only, unchanged `df /System/Volumes/Data` semantics — no new dependencies or commands.
