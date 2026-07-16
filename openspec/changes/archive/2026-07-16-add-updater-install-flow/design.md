## Context

Clarus is a Tauri v2 + React desktop app. The updater backend is already fully wired: `tauri-plugin-updater` and `tauri-plugin-process` are registered in `src-tauri/src/lib.rs`, the `updater:default` and `process:default` capabilities are granted, and `tauri.conf.json` points at the signed `latest.json` manifest with the public key. The `@tauri-apps/plugin-updater` JS package is installed.

The gap was entirely in the frontend. `useUpdater` called `check()` but only copied `update.version` into state and dropped the `Update` object — the one thing that exposes `downloadAndInstall()`. Its state machine had no `downloading`/`downloaded` states, and `App.tsx` rendered only a static "Version X is available." string with no install control. The `@tauri-apps/plugin-process` JS binding (needed for `relaunch()`) was also missing even though its Rust plugin was registered.

## Goals / Non-Goals

**Goals:**
- Let a user install an available update from inside the app: download → install → relaunch.
- Keep the existing auto-check-on-launch and manual "Check for updates" behavior intact.
- Give clear UI feedback for each phase of the flow.

**Non-Goals:**
- Changing the release pipeline, signing, notarization, or the `latest.json` manifest (covered by `release-credentials`).
- Background/silent auto-install or download-progress percentage UI.
- Any behavior outside the packaged Tauri runtime (dev/browser stays a no-op).

## Decisions

- **Store the `Update` object in a `useRef`, not state.** The `Update` instance is a live handle, not serializable display data, and re-rendering on it buys nothing. Only `version`/`current`/`error` drive the UI, so they stay in state. Alternative (state) was rejected as needless re-renders and awkward typing.
- **Extend the existing string-union state machine with `downloading` and `downloaded`** rather than introducing a separate progress flag. Keeps one source of truth for what the UI shows and what controls are disabled.
- **Relaunch via `@tauri-apps/plugin-process`.** The Rust plugin and capability already exist; only the JS binding was missing, so we add `@tauri-apps/plugin-process` (`^2.2.0`) to the app package. Alternative (a custom Rust command) was rejected as reinventing the official plugin.
- **Show the "Update now" button across `available | downloading | downloaded`, disabled during install**, instead of only on `available`. This gives continuous feedback through the flow and avoids a TypeScript narrowing conflict between an `available`-only render guard and a disabled check against `downloading`/`downloaded`.
- **Guard everything behind `isTauriRuntime()`.** Outside Tauri the flow resolves to `current`, so dev and browser test runs never attempt a real install.

## Risks / Trade-offs

- [Real install path can only be exercised in a signed, packaged build] → Verify in a packaged binary pointed at the release manifest; dev/tests only cover state transitions and rendering logic.
- [`relaunch()` restarts the app mid-session] → Acceptable and expected for applying an update; the `downloaded` copy ("Restarting to finish update…") sets the expectation.
- [No download-progress percentage] → Accept for now; `downloading` copy is enough for the small update size. Progress events can be layered onto `downloadAndInstall` later without changing the state machine.

## Open Questions

- None blocking. Whether to add a percentage progress indicator can be decided after observing real-world update sizes.
