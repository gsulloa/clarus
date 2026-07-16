## 1. Backend: single-snapshot disk read

- [x] 1.1 In `packages/app/src-tauri/src/cleanup/mod.rs`, add one function that runs `df -g /System/Volumes/Data` and `df -h /System/Volumes/Data` once each and parses Size (col 1) and Avail (col 3) from each, returning a struct `{ total_gb, free_gb, used_gb, total_human, free_human, used_human }`.
- [x] 1.2 Derive `used_gb = total_gb − free_gb` and format `used_human` from that value (same GiB unit as the readout) — do NOT read `df` column 2 for Used.
- [x] 1.3 Add a code comment explaining that "Used" is `total − free` on purpose (APFS per-volume Used does not reconcile with container Size), so a future reader does not revert it.
- [x] 1.4 Remove or replace the now-redundant `disk_used_gb` / `disk_used_human` / `disk_free_gb` / `disk_free_human` / `disk_total_gb` / `disk_total_human` helpers; keep `parse_df_field_at` if still used.

## 2. Backend: wire the snapshot into payloads

- [x] 2.1 Update `scan_cleanup_targets` to populate `CleanupScan` (`freeBeforeGb/Human`, `totalBeforeGb/Human`, `usedBeforeGb/Human`) from the single snapshot.
- [x] 2.2 Update `clean_result` so `CleanResult` (`freeGb/Human`, `totalGb/Human`, `usedGb/Human`, `freedGb`) uses the single snapshot and the derived `used`.
- [x] 2.3 Update the standalone `disk_free` command to use the same snapshot function.

## 3. Frontend: consume reconciled values

- [x] 3.1 In `packages/app/src/App.tsx`, confirm `usedNow*/freeNow*/totalNow*` state is set from the reconciled scan/clean payload (lines ~148–173); no shape change expected.
- [x] 3.2 Verify the capacity bar (`usedNow / totalNow`, lines ~106–109) now reads consistently with Free (unfilled = free ÷ total); adjust only if it drifts.
- [x] 3.3 Confirm `packages/app/src/cleanup/types.ts` and `api.ts` match the payload; update only if a field changed.

## 4. Verify

- [x] 4.1 Run a scan and confirm the readout shows `used + free = total` (e.g. ~429Gi + 31Gi = 460Gi) matching live `df -h /System/Volumes/Data` for Size and Avail.
- [x] 4.2 Run a cleanup and confirm the post-clean readout still reconciles and "total freed" is correct against the baseline.
- [x] 4.3 Confirm the capacity bar fill visually matches "Free" (disk shows near-full when only ~31Gi free).
- [x] 4.4 `cargo build`/`cargo clippy` for the Tauri crate and the app typecheck/build pass.
