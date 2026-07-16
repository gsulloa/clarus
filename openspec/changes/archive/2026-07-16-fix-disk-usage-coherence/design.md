## Context

The disk readout in `packages/app/src/App.tsx` (lines 303–318) shows Used / Free / Total for the data volume. The values are produced in `packages/app/src-tauri/src/cleanup/mod.rs` by **three independent `df` invocations**, one per figure:

- `disk_free_gb` / `disk_free_human` → `df -g|-h /System/Volumes/Data`, column 3 (Avail)
- `disk_total_gb` / `disk_total_human` → same command, column 1 (Size)
- `disk_used_gb` / `disk_used_human` → same command, column 2 (Used)

Two independent defects make the numbers fail to reconcile (Used 387 + Free 31 ≠ Total 460):

1. **Multiple snapshots.** Each helper runs its own `df`; the six calls happen at slightly different instants and `df -g` rounds to whole GiB while `df -h` self-selects units. The existing `disk-cleanup` spec already says these must come from a single snapshot — the implementation does not honor it.
2. **APFS `Used` semantics.** On APFS, `/System/Volumes/Data` is one volume in a shared container. `df`'s `Used` (col 2) reflects only that volume's attributed usage, while `Size` (col 1) is the whole container and `Avail` (col 3) is container-wide free space net of APFS reserved space. So `Used + Avail ≠ Size` by design — the ~42Gi gap is space held by sibling volumes (System, Preboot, VM, Recovery), snapshots, and reserved overhead. No timing fix makes column 2 reconcile with columns 1 and 3.

For a cleanup tool the user's mental model is "how full is my disk and how much is really free." The honest, reconcilable pair is **Total** (container size) and **Free** (Avail); **Used** is best expressed as everything that is not free.

## Goals / Non-Goals

**Goals:**
- The three displayed figures reconcile: `used + free = total`.
- All figures come from one `df` snapshot so numeric (GB) and human strings agree.
- The capacity bar's fill matches the displayed Free (unfilled = free ÷ total).
- Apply consistently to scan (`scan_cleanup_targets`), post-clean (`clean_result`), and the standalone `disk_free` command.

**Non-Goals:**
- Reporting a per-volume breakdown (System/Data/snapshots/purgeable). Out of scope; would need `diskutil apfs`/`statfs` and a richer UI.
- Changing the target-sizing (`du`) logic.
- Changing the API/type shape unless a field must be added (default: keep the existing `*Gb` / `*Human` fields).

## Decisions

### Decision: Derive `used` as `total − free` rather than reading `df` column 2

`df` column 2 (per-volume Used) is the source of the discrepancy. Instead compute `used_gb = total_gb − free_gb` and format `used_human` from that same byte/GB value. This guarantees reconciliation and yields the intuitive "429Gi used of 460Gi, 31Gi free."

- **Alternative — keep column 2, add an explanatory note in the UI:** rejected. Leaves the numbers not summing; a footnote does not fix "no me calza" and adds UI clutter.
- **Alternative — derive `free = total − used`:** rejected. `Avail` (real allocatable space, 31Gi) is the number the user acts on and matches what Finder/`df` report as available; it must be preserved verbatim. `Used` is the derived/soft figure.

### Decision: Single `df` snapshot parsed for all fields

Run `df -g /System/Volumes/Data` once and `df -h` once (or capture both from one code path), parse Size/Avail from each line, and derive Used. Replace the six single-purpose helpers with one function returning a small struct (e.g. `{ total_gb, free_gb, used_gb, total_human, free_human, used_human }`). This satisfies the existing single-snapshot spec requirement and removes redundant process spawns.

- Keep `-g` for numeric GB (stable integer math for the bar) and `-h` for display strings, but derive both `used` variants arithmetically from that snapshot's total and free so all three human strings and all three GB values are mutually consistent.
- Human `used` string: format from `total_gb − free_gb` using the same GiB unit the readout already uses, rather than trusting `df -h` column 2.

### Decision: Capacity bar uses free-consistent fullness

The bar in `App.tsx` currently uses `usedNow / totalNow`. With `used = total − free` this is now identical to `1 − free/total`, so no separate change is required beyond consuming the reconciled `usedNow`. Verify the bar reads ~93% (429/460), consistent with 31Gi free.

## Risks / Trade-offs

- **[The displayed "Used" no longer equals `df`'s raw per-volume Used]** → This is intentional and is the fix; "Used" now means "occupied capacity (total − free)", which matches user expectation for a cleanup dashboard. Document the meaning in a code comment so a future reader does not "restore" column 2.
- **[Rounding drift between `-g` integer math and `-h` unit choice]** → Derive the human "Used" from the same total/free used for the numeric fields; do not mix a `-h` column-2 read with `-g`-derived total/free. Within-unit rounding differences (≤1 GiB) are acceptable and covered by the "within rounding" wording in the spec.
- **[Other call sites expecting raw per-volume Used]** → Grep for `disk_used`, `usedBefore`, `used_gb`, `usedNow`; the only consumers are the scan/clean payloads and the App readout/bar, all of which want the reconciled figure.

## Migration Plan

Pure display/derivation change; no persisted data or API contract change (field names preserved). Deploy with the app. Rollback is reverting the diff. No user migration needed.

## Open Questions

- None blocking. If a future iteration wants a true per-category breakdown (System vs Data vs purgeable), that is a separate proposal requiring `statfs`/`diskutil apfs` data.
