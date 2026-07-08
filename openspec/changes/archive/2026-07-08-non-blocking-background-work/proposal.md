## Why

While Clarus analyzes targets or runs a cleanup, the whole window freezes and the OS shows the spinning-wait cursor (beachball). The React loading states already exist (`Measuring targets…`, `Cleaning…`), but they never animate because the backend blocks the UI thread — so the app looks broken during exactly the moments a user is most anxious about a disk operation. This contradicts the product promise of a calm, trustworthy instrument.

The root cause is that every Tauri command (`scan_cleanup_targets`, `clean_target`, `clean_item`, `scan_directory`) is declared as a **synchronous** `pub fn`. Tauri runs synchronous commands on the main thread, which on macOS also owns the WKWebView event loop. Heavy blocking work — `du` over large trees, `bash -lc` cleanup commands, Docker startup waits up to 90s — therefore stalls rendering, input, and the cursor until the command returns.

## What Changes

- Move all heavy Tauri commands off the main thread so the webview keeps painting and responding during scans and cleanups (backend fix — this is what removes the freeze and the beachball cursor).
- Make `scan_cleanup_targets` stream results progressively instead of returning only after every target is measured, so the "N targets measured" counter and rows fill in live.
- Add real progress feedback in the UI during analysis: per-target measuring/skeleton states, a determinate progress indicator (measured / total), and a non-blocking busy affordance — replacing the single frozen "Measuring targets…" line.
- Add per-row cleanup progress that stays interactive: the rest of the catalog remains scrollable, selectable, and cleanable while one target is being cleaned; long-running cleanups (Docker) show elapsed/step feedback rather than a dead spinner.
- Ensure the app never sets `cursor: wait`/`progress` globally and honors `prefers-reduced-motion` for all new loaders.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `disk-cleanup`: Add requirements that scan and cleanup operations execute without blocking the UI thread, that scan results stream progressively, and that the UI presents appropriate non-blocking progress feedback (skeletons, determinate scan progress, live per-row cleanup state) during background work.

## Impact

- **Backend (Rust, Tauri):** `packages/app/src-tauri/src/cleanup/mod.rs` (`scan_cleanup_targets`, `clean_target`, `clean_item`) and `packages/app/src-tauri/src/scan/mod.rs` (`scan_directory`) — change command signatures/execution to run off the main thread (async command + `spawn_blocking`, or `#[tauri::command(async)]`), and emit scan progress incrementally.
- **Frontend (React):** `packages/app/src/App.tsx` scan and cleanup state handling; `packages/app/src/cleanup/api.ts` event wiring; `packages/app/src/styles/global.css` new skeleton/progress loader styles (reusing the existing `spin` keyframe and dark design tokens from `DESIGN.md`).
- **No API contract removal:** command names and payloads stay the same; behavior becomes non-blocking and progress-emitting. No breaking changes for callers.
- **Dependencies:** no new runtime dependencies expected (uses Tauri's built-in async runtime and existing event channel).
