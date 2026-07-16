## 1. Dependencies

- [x] 1.1 Add `@tauri-apps/plugin-process` (`^2.2.0`) to `packages/app/package.json` dependencies
- [x] 1.2 Run `pnpm install` and confirm the binding resolves under `packages/app/node_modules/@tauri-apps/plugin-process`

## 2. Updater hook (`useUpdater.ts`)

- [x] 2.1 Import `type Update` from `@tauri-apps/plugin-updater` and `relaunch` from `@tauri-apps/plugin-process`
- [x] 2.2 Extend `UpdaterState.current` with `downloading` and `downloaded`
- [x] 2.3 Retain the `Update` object in a `useRef`; set it when an update is found, clear it on no-update, error, and non-Tauri paths
- [x] 2.4 Add `downloadAndInstall()`: no-op without a retained update; otherwise `downloading` → `update.downloadAndInstall()` → `downloaded` → `relaunch()`, with try/catch setting `error`
- [x] 2.5 Expose `downloadAndInstall` from the returned `useMemo` (and its dep array); keep the 5s auto-check effect and `isTauriRuntime` guard

## 3. Release channel UI (`App.tsx`)

- [x] 3.1 Import `Download` from `lucide-react`
- [x] 3.2 Render an actionable "Update now" `scan-action` button while state is `available | downloading | downloaded`, wired to `updater.downloadAndInstall()`
- [x] 3.3 Disable "Update now" during `downloading | downloaded`; disable "Check for updates" during `checking | downloading | downloaded`
- [x] 3.4 Add status copy for `downloading` ("Downloading update…") and `downloaded` ("Restarting to finish update…"), preserving existing copy for the other states

## 4. Verification

- [x] 4.1 `pnpm typecheck` passes
- [x] 4.2 `pnpm lint` passes
- [x] 4.3 `pnpm test:run` passes
- [ ] 4.4 Manual: confirm download/install/relaunch in a signed, packaged build pointed at the release manifest (not verifiable in dev/browser)
