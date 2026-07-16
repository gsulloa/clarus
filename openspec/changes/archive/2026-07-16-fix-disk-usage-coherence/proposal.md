## Why

The "Disk usage · data volume" readout shows Used 387Gi, Free 31Gi, Total 460Gi — but 387 + 31 = 418 ≠ 460, so the numbers visibly don't add up ("no me calza"). Two problems cause this: each of the three figures is read from a **separate** `df` invocation (violating the existing spec's single-snapshot requirement), and on APFS `df` reports the Data-volume `Used` (space attributed to that one volume) against the whole-container `Size`, so `Used + Avail` never equals `Size` regardless of timing. The user reads the three numbers as a set and expects them to reconcile.

## What Changes

- Read used, free, and total from a **single** `df /System/Volumes/Data` snapshot (one invocation, both `-g` and `-h` derived from it) so the figures are captured at the same instant — as the existing spec already requires but the implementation does not do.
- Make the three displayed figures **mutually coherent**: `used = total − free`, so `used + free = total` always holds in the UI. This reflects what a cleanup tool's user cares about — how full the disk really is and how much is actually free — rather than the APFS per-volume `Used` column that excludes space held by sibling volumes/snapshots.
- Ensure the capacity bar (used ÷ total) and the human-readable strings stay consistent with the reconciled figures.
- Apply the same coherence to the post-cleanup readout (free-now / used-now / total) and to the standalone `disk_free` path.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `disk-cleanup`: The "Scan measures targets without deleting" and "Disk usage summary is displayed" requirements change so that used/free/total are derived from a single `df` snapshot AND are mutually consistent (`used + free = total`). Adds a coherence guarantee the current spec does not state.

## Impact

- **Backend (Rust):** `packages/app/src-tauri/src/cleanup/mod.rs` — the separate `disk_used_gb`/`disk_used_human`/`disk_free_*`/`disk_total_*` helpers are replaced by a single-snapshot read; `used` is derived as `total − free`. Affects `scan_cleanup_targets`, `clean_result`, and the `disk_free` command.
- **Frontend (React):** `packages/app/src/App.tsx` disk readout and capacity bar consume the reconciled values; no shape change expected. Types in `packages/app/src/cleanup/types.ts` / `api.ts` unchanged unless the payload adds/removes a field.
- **User-visible:** the three numbers now sum; the capacity bar matches "Free" (e.g. 429Gi used, 31Gi free, 460Gi total).
