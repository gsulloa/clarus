## 1. Backend — read total & used from `df`

- [x] 1.1 Generalize `parse_df_field` in `packages/app/src-tauri/src/cleanup/mod.rs` into a field-index-aware helper (e.g. `parse_df_field_at(text, index)`), keeping the existing free-space callers working (index 3 = Avail).
- [x] 1.2 Add `disk_total_gb`/`disk_total_human` helpers reading field 1 (`Size`) from `df -g` / `df -h /System/Volumes/Data`, with the same `unwrap_or(0)` / `"?"` fallbacks as the free helpers.
- [x] 1.3 Add `disk_used_gb`/`disk_used_human` helpers reading field 2 (`Used`) from the same `df` invocations.

## 2. Backend — thread total & used through result structs

- [x] 2.1 Extend `CleanupScan` (serde camelCase) with `total_before_gb`/`total_before_human` and `used_before_gb`/`used_before_human`.
- [x] 2.2 Extend `CleanResult` (serde camelCase) with `total_gb`/`total_human` and `used_gb`/`used_human`.
- [x] 2.3 Populate the new `CleanupScan` fields in `scan_cleanup_targets`.
- [x] 2.4 Populate the new `CleanResult` fields in `clean_result` (used by `clean_target`/`clean_item`) and in the `disk_free` command.

## 3. Frontend — types

- [x] 3.1 Extend `CleanupScan` in `packages/app/src/cleanup/types.ts` with `totalBeforeGb`/`totalBeforeHuman` and `usedBeforeGb`/`usedBeforeHuman`.
- [x] 3.2 Extend `CleanResult` in `cleanup/types.ts` with `totalGb`/`totalHuman` and `usedGb`/`usedHuman`.

## 4. Frontend — disk usage summary UI

- [x] 4.1 Add state for `usedNow`/`usedNowHuman` and `totalNow`/`totalNowHuman` in `App.tsx`, seeded from the scan result on completion.
- [x] 4.2 Update `applyResult` to also set used/total (and human strings) from each `CleanResult`.
- [x] 4.3 Rework the "Disk free · data volume" rail section into a disk-usage summary: Used / Free / Total lines plus a capacity bar filled to `usedNow / totalNow`; keep the Before / Now / Freed block.
- [x] 4.4 Add CSS for the capacity bar in `packages/app/src/styles/global.css`, reusing existing readout tokens and respecting reduced-motion.

## 5. Verify

- [x] 5.1 `pnpm test` and `pnpm build` pass; TS types compile with the new fields.
- [ ] 5.2 Run `pnpm tauri:dev`, run a scan, and confirm Used / Free / Total and the capacity bar render with real values, and that all three update after a cleanup action.
