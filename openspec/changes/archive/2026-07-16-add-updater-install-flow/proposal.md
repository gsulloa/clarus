## Why

Clarus already checks for signed updates via the Tauri updater and shows "Version X is available.", but the in-app update flow ended there: the `useUpdater` hook discarded the `Update` object returned by `check()` (keeping only the version string) and the UI never rendered an actionable install button. Users could learn an update existed but had no way to install it from inside the app. This change adds the missing download-install-relaunch step so an available update can actually be applied.

## What Changes

- Persist the `Update` object returned by `check()` (in a ref) instead of discarding it, so it can be installed later.
- Extend the updater state machine with `downloading` and `downloaded` states in addition to the existing `idle | checking | available | current | error`.
- Expose a new `downloadAndInstall()` action from `useUpdater` that downloads the update, installs it, and relaunches the app.
- Add a primary **"Update now"** button in the Release channel section that appears while an update is available/downloading/downloaded, and disable it during the install.
- Disable the existing "Check for updates" button while checking/downloading/downloaded.
- Add status copy for the new states ("Downloading update…", "Restarting to finish update…").
- Add the `@tauri-apps/plugin-process` JS dependency (needed for `relaunch()`); the corresponding Rust plugin and `process:default` capability were already wired.

## Capabilities

### New Capabilities
- `app-self-update`: In-app self-update flow — checking for a signed update, surfacing that one is available, and downloading, installing, and relaunching to apply it.

### Modified Capabilities
<!-- No existing capability's requirements change; release-credentials covers only credential provisioning for the updater backend, not the in-app UI flow. -->

## Impact

- `packages/app/src/platform/updater/useUpdater.ts` — new states, ref-persisted `Update`, `downloadAndInstall()` action.
- `packages/app/src/App.tsx` — "Update now" button, disabled states, status copy.
- `packages/app/package.json` + `pnpm-lock.yaml` — adds `@tauri-apps/plugin-process`.
- Runtime: install flow only executes inside the packaged, signed Tauri binary (guarded by `isTauriRuntime()`); no effect in `pnpm dev`/browser.
- No breaking changes; no backend/release-pipeline changes.
