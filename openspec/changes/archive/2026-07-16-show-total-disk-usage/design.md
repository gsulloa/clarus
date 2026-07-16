## Context

The cleanup backend already shells out to `df` against `/System/Volumes/Data` twice per readout: once with `df -g` (parsed for a numeric GB value) and once with `df -h` (parsed for a human string). Both go through `parse_df_field`, which grabs the **4th** whitespace field of the second output line — the `Avail` column. The same `df` line also contains the `Size` (total) and `Used` columns, so the data the user is asking for is already in output we throw away today.

Current shapes:
- Rust `CleanupScan { free_before_gb, free_before_human, targets }` and `CleanResult { ok, message, free_gb, free_human, freed_gb }` (serde camelCase).
- TS mirrors in `cleanup/types.ts`; `App.tsx` renders a "Disk free · data volume" rail with Before / Now / Freed lines driven by `freeBefore*`/`freeNow*` state.

The change is additive: surface total + used from the same `df` call, thread them through the two result structs and their TS mirrors, and expand the rail UI.

## Goals / Non-Goals

**Goals:**
- Show total capacity and used space of the data volume alongside free space.
- Recompute used/total after each cleanup, matching how free is recomputed.
- Reuse the existing `df /System/Volumes/Data` semantics — no new commands, deps, or platform assumptions.
- Keep the existing Before / Now / Freed cleanup-progress lines working.

**Non-Goals:**
- Per-volume breakdown or multi-disk support (data volume only, as today).
- Reconciling "used" against the sum of catalog target sizes (they measure different things).
- Any change to APFS purgeable-space nuance beyond what `df` already reports.

## Decisions

**1. Parse total + used from the existing `df` output instead of adding calls.**
Generalize `parse_df_field(text)` (currently hard-coded to field index 3) into `parse_df_field_at(text, index)`, and add `disk_total_*`/`disk_used_*` helpers that read field 1 (`Size`) and field 2 (`Used`) from the same `df -g` / `df -h` invocations. Rationale: the columns are already present; adding indices avoids extra process spawns and keeps free/used/total mutually consistent (same snapshot). Alternative — a `statfs`/`sysinfo` crate — rejected to avoid a dependency and to stay byte-for-byte consistent with the script's `df` semantics the rest of the module relies on.

**2. Extend both result structs with total/used (bytes-as-GB `i64` + human `String`), mirroring the existing free fields.**
Add `total_before_gb`/`total_before_human` + `used_before_gb`/`used_before_human` to `CleanupScan`, and `total_gb`/`total_human` + `used_gb`/`used_human` to `CleanResult`. Rationale: symmetric with `free_*`, so the frontend consumes them the same way and `applyResult` can update all three. Keeping both a numeric (for bar math / comparisons) and a human string (for display) matches the current free-field pattern exactly.

**3. Frontend: relabel the rail section to a usage summary with a capacity bar.**
Rename "Disk free · data volume" to a disk-usage summary. Show a horizontal capacity bar (`used / total`) with Used, Free, and Total figures, then keep the Before / Now / Freed block below for cleanup-session progress. Bar fill = `usedGb / totalGb`. Rationale: a bar answers "how full is my disk" at a glance, which is the user's actual question; the numbers give precision. Reuses existing `.disk-readout`/`.disk-line` CSS tokens plus one new bar element.

**4. State: track `usedNow`/`totalNow` (and their human strings) the same way as `freeNow`.**
On scan complete, seed used/total from the scan result; in `applyResult`, update used/total/free from each `CleanResult`. `freedTotal` math is unchanged.

## Risks / Trade-offs

- **`df` column order differs from expectation** → macOS BSD `df` reliably emits `Filesystem Size Used Avail Capacity ...`; the module already depends on `Avail` being field 3, so trusting `Size`=1/`Used`=2 is no riskier. Guard with `unwrap_or(0)` / `"?"` fallbacks exactly like the existing helpers.
- **`-g` rounds to whole GB** → used/total shown in GB granularity, same limitation the free readout already has; acceptable for an at-a-glance summary. Human string from `-h` gives a friendlier value.
- **Used may not equal Total − Free** (APFS purgeable/other volumes share the container) → we display `df`'s reported Used directly rather than deriving it, so the numbers stay internally consistent with what the OS reports; the bar uses `used/total`.

## Migration Plan

Purely additive to serde/TS payloads and UI. No persisted data, no API consumers outside this app. Ship in one change; rollback is reverting the commit.
