## 1. Backend — move commands off the main thread

- [x] 1.1 Convert `scan_cleanup_targets` to `async fn` and run its body (catalog build + measuring scope + emits) inside `tauri::async_runtime::spawn_blocking`, moving a cloned `AppHandle` into the closure (`packages/app/src-tauri/src/cleanup/mod.rs:962-985`).
- [x] 1.2 Convert `clean_target` and `clean_item` to `async fn` + `spawn_blocking` so long cleanups (Docker 90s wait, app quits) never run on the main thread (`cleanup/mod.rs:1008-1051`).
- [x] 1.3 Convert `scan_directory` to `async fn` + `spawn_blocking` for consistency (`packages/app/src-tauri/src/scan/mod.rs:49-136`).
- [x] 1.4 Confirm command registration in `lib.rs` still compiles with the new async signatures; no handler-list changes expected (`src-tauri/src/lib.rs:11-17`).

## 2. Backend — stream scan progress

- [x] 2.1 After `catalog_defs()` returns, emit a single `cleanup://catalog` event carrying the enumerated (unmeasured) targets and the total count, before the measuring `thread::scope` (`cleanup/mod.rs:967-978`).
- [x] 2.2 Keep the per-target `cleanup://target` emit as the measured-phase event (one per target as `du` completes); ensure it is distinct from the enumeration event so the frontend counts only measured targets.
- [x] 2.3 Verify the final `Ok(CleanupScan)` return payload is unchanged (backward compatible for any client ignoring `cleanup://catalog`).

## 3. Frontend — scan progress and skeletons

- [x] 3.1 Add a `cleanup://catalog` subscription in `packages/app/src/cleanup/api.ts` (mirroring `onTargetMeasured`) that delivers the target list + total.
- [x] 3.2 In `App.tsx runScan`, on `cleanup://catalog` render all rows immediately (skeleton sizes) and set a `total`; keep the existing `cleanup://target` upsert to fill sizes and increment `measured` (`App.tsx:92-127`).
- [x] 3.3 Add a determinate progress bar in the Analyze rail section under the button, width = `measured/total`, styled with `--accent` fill on a `--surface` track and `radius-pill` (`App.tsx:217-241`).
- [x] 3.4 Render a skeleton placeholder in the size column (and status chip) for targets that are enumerated but not yet measured (`TargetRow`, `App.tsx:470-589`).

## 4. Frontend — cleanup feedback (non-blocking)

- [x] 4.1 Add a client-side elapsed timer for rows in the `cleaning` phase and show it in the button (e.g. `Cleaning… 0:12`) in `ActionButton` (`App.tsx:414-468`).
- [x] 4.2 Surface a "may take time" hint for long cleanups (e.g. Docker) near the row/evidence using the target's existing `caveat` copy.
- [x] 4.3 Verify that while one target cleans, other actionable rows remain clickable and can clean concurrently (no global disable/overlay introduced).

## 5. Styling & accessibility

- [x] 5.1 Add `@keyframes shimmer` and a `.skeleton` class in `global.css` using `--surface`/`--surface-strong`, plus progress-bar styles, following `DESIGN.md` tokens (`global.css`).
- [x] 5.2 Under `prefers-reduced-motion: reduce`, disable shimmer (static tint) while progress state still updates; reuse the existing reduced-motion pattern (`global.css:61-65`).
- [x] 5.3 Add a guard ensuring no global `cursor: wait`/`progress` is set; keep busy affordances local (button spinner, bar, skeleton).

## 6. Verification

- [ ] 6.1 Run `pnpm --filter app tauri dev` on a machine with large caches; confirm the window stays interactive during a full scan (scroll/select while measuring).
- [ ] 6.2 Trigger a Docker cleanup (or a target that waits/sleeps) and confirm the window does not freeze, no beachball appears, the elapsed timer advances, and other rows stay actionable.
- [ ] 6.3 Confirm skeleton rows appear then fill, the progress bar advances to 100%, and the `measured` count never exceeds `total`.
- [ ] 6.4 Toggle reduced motion and confirm shimmer is disabled while progress still updates; run `pnpm --filter app lint`/typecheck and the Rust build.
