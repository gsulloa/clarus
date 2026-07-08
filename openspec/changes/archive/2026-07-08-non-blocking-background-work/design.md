## Context

Clarus is a Tauri v2 app (Rust core + React/Vite WebView, plain CSS in `global.css`). The disk-cleanup flow lives in `packages/app/src-tauri/src/cleanup/mod.rs` and `packages/app/src/App.tsx`.

Current state (verified in code):

- All four heavy Tauri commands are synchronous `pub fn`: `scan_cleanup_targets`, `clean_target`, `clean_item` (`cleanup/mod.rs`), and `scan_directory` (`scan/mod.rs`). Tauri v2 runs **synchronous commands on the main thread**, which on macOS also drives the WKWebView event loop. Any blocking work there stalls rendering and input → frozen window + beachball cursor.
- The blocking work is substantial: `scan_cleanup_targets` builds the catalog with dozens of serial `bash -lc` / `git` / `du` subprocess calls (`catalog_defs`, `cleanup/mod.rs:275-514`), then a `thread::scope` (`:970-978`) whose join blocks the calling (main) thread until every `du -sk` finishes. `clean_target`/`clean_item` rebuild the whole catalog again and then run cleanup commands that can block for a long time (Docker prune waits up to 90s, `cleanup/mod.rs:534-544`).
- The frontend already has the right *state* shape: `ScanState`/`busy` (`App.tsx:204`), a live `measured` counter fed by the `cleanup://target` event (`App.tsx:100-108`), and per-row `ActionButton` phases (`App.tsx:414-468`). None of it animates today because the core's main thread is blocked, so events queue and the WebView cannot repaint.

The only loader primitive is `.spin` + `@keyframes spin` (`global.css:51-66`); there is no progress bar or skeleton. The design language is defined in `DESIGN.md` (dark instrument panel, single cyan accent `#2EE8D6`, thin borders, `radius-pill` for status, minimal-functional motion, must respect `prefers-reduced-motion`).

Constraints: keep the existing IPC contract (command names + payload shapes) so nothing else breaks; keep the destructive semantics faithful to the script; stay within the `DESIGN.md` visual system.

## Goals / Non-Goals

**Goals:**

- The window never freezes and the OS never shows the wait/beachball cursor during analysis or cleanup — the WebView keeps painting and stays interactive.
- Analysis shows honest, appropriate progress: rows appear immediately as skeletons, a determinate indicator advances as targets are measured (measured / total), and the existing counter animates smoothly.
- Cleanup stays interactive: the rest of the catalog remains scrollable/selectable/cleanable while one (or several) targets clean; long cleanups (Docker) show elapsed time and a step hint instead of a dead spinner.
- All new motion respects `prefers-reduced-motion` and reuses the `DESIGN.md` tokens.

**Non-Goals:**

- Changing which targets exist, their tiers, commands, or destructive semantics (that catalog stays a faithful port of `disk-cleanup.sh`).
- Caching the catalog across calls or refactoring `catalog_defs()` for speed. Moving work off the main thread already removes the freeze; catalog caching is a separate optimization with staleness trade-offs and is out of scope.
- Cancellation / abort of an in-flight scan or cleanup (worth a follow-up, not required to fix the freeze).
- Reworking `scan_directory` UX (it is currently unused by the UI); it only gets the same off-main-thread treatment for consistency.

## Decisions

### 1. Run heavy commands off the main thread with `async fn` + `spawn_blocking`

Convert `scan_cleanup_targets`, `clean_target`, `clean_item`, and `scan_directory` to `async fn` and move their blocking bodies into `tauri::async_runtime::spawn_blocking`. The synchronous helpers (`run_bash`, `du_bytes`, `catalog_defs`, `measure`, `df`) stay unchanged; only the command wrappers move execution off-thread.

- **Why not just `async fn`?** An `async fn` command runs on Tauri's async (tokio) runtime, off the main thread — that alone unfreezes the UI. But the bodies are pure blocking syscalls/subprocesses (including a 90s Docker wait); running them directly on a tokio worker ties up a runtime worker for the whole duration and can starve other async work. `spawn_blocking` dispatches them to the dedicated blocking-thread pool, which is exactly what it exists for.
- **Why not `#[tauri::command(async)]` on the existing sync fns?** That macro form does move a sync command off the main thread, and is the smallest diff, but it runs the blocking body on a runtime thread with the same starvation caveat and reads less explicitly. `async fn` + `spawn_blocking` is the idiomatic, self-documenting choice and composes with the event emitting below. (Acceptable fallback if `spawn_blocking` ergonomics with `AppHandle` get awkward.)
- `AppHandle` is `Clone + Send`, so it can be moved into the blocking closure for event emission. The `thread::scope` concurrent measuring stays as-is, now running inside the blocking task rather than on the main thread.

### 2. Two-phase scan events for immediate skeletons + a determinate bar

Today `scan_cleanup_targets` emits one `cleanup://target` per target *only after* measuring, and returns the full result at the end. To render rows instantly and drive a determinate progress bar, split emission into two phases:

1. After `catalog_defs()` returns, emit a single `cleanup://catalog` event carrying the enumerated (unmeasured) targets and the total count. The frontend renders every row immediately with a **skeleton** size and sets `total`.
2. Keep emitting `cleanup://target` per target as each `du` finishes (measured phase). The frontend upserts by `id` (existing logic at `App.tsx:100-108` already updates in place) and increments `measured`.

- **Counter correctness:** `measured` must only count phase-2 events, so enumeration uses a *distinct* event name (`cleanup://catalog`), not a second `cleanup://target`. Progress = `measured / total`.
- **Why up-front total?** A determinate bar needs the denominator before measuring starts; the catalog length is known the moment `catalog_defs()` returns.
- Backward-compatible: the final `Ok(CleanupScan)` return is unchanged; a client that ignores `cleanup://catalog` still works.

### 3. Frontend progress UI, faithful to `DESIGN.md`

- **Determinate scan progress:** add a thin progress bar in the Analyze rail section under the button, cyan (`--accent`) fill on `--surface` track, `radius-pill`, width = `measured/total`. Keep the `{measured} targets measured…` copy; it now animates because the main thread is free.
- **Skeleton rows:** render catalog rows on `cleanup://catalog` with a shimmering placeholder in the size column (and status chip) until the matching `cleanup://target` fills it. Add a `.skeleton` class + `@keyframes shimmer` using `--surface`/`--surface-strong`, disabled under `prefers-reduced-motion` (bar still fills; shimmer becomes a static tint).
- **Interactive during cleanup:** no change to interactivity is required once the backend is async — the per-row `actions` map already isolates state, and multiple rows can clean concurrently. Verify no global disabling/overlay is introduced.
- **Long-cleanup feedback:** for a target that is `cleaning`, show an elapsed timer in the button (e.g. `Cleaning… 0:12`) driven by a client-side interval started when the phase enters `cleaning`. For Docker, surface the existing `caveat` copy ("Starts Docker if it is not running (waits up to 90s)") as a hint near the row so the wait is expected. This stays honest without parsing bash stdout.
- **Cursor guard:** the beachball is OS-level from the blocked main thread and disappears once commands are async. Add an explicit guard so nothing sets `cursor: wait`/`progress` globally; busy affordances are local (button spinner, bar, skeleton), never a whole-window cursor change.

## Risks / Trade-offs

- **Double-counting scan progress** if enumeration reused `cleanup://target`. → Mitigated by a distinct `cleanup://catalog` event; `measured` only increments on `cleanup://target`.
- **Blocking-pool pressure** if the user triggers many cleanups at once via `spawn_blocking`. → Bounded by the number of catalog rows (tens), well within the pool; acceptable. A future concurrency cap is possible but unnecessary now.
- **`clean_target`/`clean_item` still rebuild the full catalog** (re-running subprocesses) on every action. → No longer a freeze (it runs off-thread), just latency before the command executes. Caching is deliberately out of scope to avoid staleness bugs; noted as a follow-up.
- **Skeleton flash on fast machines** where measuring is near-instant. → Keep transitions short (per `DESIGN.md` durations) so skeletons that resolve quickly read as a subtle settle, not a flash.
- **No cancellation:** a long Docker cleanup still runs to completion; the user can use the rest of the app but cannot abort. → Accepted for this change; explicit non-goal.

## Migration Plan

Pure code change; no data or config migration.

1. Backend: convert the four commands to `async fn` + `spawn_blocking`; add the `cleanup://catalog` enumeration event before the measuring scope.
2. Frontend: consume `cleanup://catalog` (render skeleton rows + set total), add the progress bar, skeleton shimmer, and per-row elapsed timer; add the cursor guard.
3. Verify with `pnpm --filter app tauri dev` on a machine with large caches (Docker/DerivedData) that the window stays responsive during both a scan and a Docker cleanup, and that reduced-motion disables shimmer.

Rollback: revert the change; the previous synchronous commands and single-phase events return. No persisted state is affected.

## Open Questions

- Should the Analyze progress bar also reflect the catalog-build phase (before measuring), which can itself be slow due to serial subprocesses? Current plan shows the bar only during measuring; a small indeterminate pre-phase indicator could cover the enumeration gap if it proves noticeable.
- Is a client-side elapsed timer enough for Docker, or do we want real step events (`starting docker`, `pruning images`, …) emitted from the backend? Starting with the timer; revisit if users find the 90s wait opaque.
