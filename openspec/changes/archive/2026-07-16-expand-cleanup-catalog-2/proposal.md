## Why

A real disk analysis of the maintainer's Mac surfaced ~15 GB of reclaimable space that the current 39-target catalog does not cover — most notably a 5.9 GB Docker Scout cache, ~2.7 GB of app-updater stubs (`*.ShipIt`) the catalog only handles for a single app, and several GB of package-manager and Electron caches. These are the same safe, regenerable categories Clarus already targets; they are simply not yet enumerated.

## What Changes

Add high-impact cleanup targets discovered during disk analysis, extending the existing catalog without changing any existing target's command or tier.

- **New Tier 1 (pure caches):**
  - `docker-scout` — Docker Scout CVE database cache (`~/.docker/scout`), ~5.9 GB.
  - `shipit-updaters` — container target enumerating **every** `~/Library/Caches/*.ShipIt` updater stub as a subitem (VSCode, Claude, Slack, Notion, Discord, Bruno, Postman, …), superseding the single-app `shipit` target's narrow scope while preserving it verbatim for script fidelity.
  - `electron-updaters` — container target for `electron-updater` download caches (`~/Library/Caches/*updater*`, `@*updater*`).
  - `uv-cache` — Python `uv` cache (`~/.cache/uv`).
  - `puppeteer-cache` — Puppeteer downloaded browsers (`~/.cache/puppeteer`).
  - `node-gyp` — node-gyp headers cache (`~/Library/Caches/node-gyp`).
  - `discord-cache`, `notion-cache`, `teams-cache`, `postman-cache`, `zoom-cache` — Electron app HTTP/GPU caches (`Cache`, `Code Cache`, `GPUCache`, `Service Worker/CacheStorage`).
  - `vscode-cache`, `cursor-cache` — VS Code / Cursor `Cache` + `CachedData`.
  - `user-logs` — user log directory (`~/Library/Logs`).
  - `quicklook-cache` — QuickLook thumbnail cache (`qlmanage -r cache`).
  - `tableplus-cache` — TablePlus cache.
- **New Tier 2 (regenerables):**
  - `rustup` — container target offering old rustup toolchains for deletion, keeping the active/default toolchain.
  - `coresimulator-caches` — CoreSimulator caches (`~/Library/Developer/CoreSimulator/Caches`).
  - `xcode-devicesupport` — Xcode iOS/watchOS/tvOS DeviceSupport.
  - `trash` — empty the user Trash (`~/.Trash`) and mounted-volume trashes.
- **New Tier 3 (informational only):**
  - `downloads` — `~/Downloads`, shown for awareness, never auto-cleaned.
- Extend the Rust catalog tests to assert the new targets and their tiers, and mirror any new subitem-container behavior in the TypeScript types.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `disk-cleanup`: the "Catalog of known cleanup targets" requirement grows to include the new targets above; new pattern/container behaviors (`shipit-updaters`, `electron-updaters`, `rustup`) add per-subitem enumeration semantics, and a new informational Tier 3 target (`downloads`) is introduced. No existing target's tier or command changes.

## Impact

- **Code:** `packages/app/src-tauri/src/cleanup/mod.rs` (`catalog_defs()` and new builder functions for the container targets), its catalog unit tests, and `packages/app/src/cleanup/types.ts` if any serialized shape changes.
- **Behavior:** users see additional actionable rows; ~15 GB of new reclaimable space on a typical developer machine. No change to scan/clean flow, UI structure, or existing targets.
- **Risk:** all new Tier 1/2 targets are regenerable caches or user-gated Trash; the one Tier 3 addition is informational and never deleted.
