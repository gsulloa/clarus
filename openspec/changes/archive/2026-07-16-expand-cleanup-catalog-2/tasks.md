## 1. Simple Tier 1 cache targets

- [x] 1.1 Add `docker-scout` via `tier1(...)` in `catalog_defs()` — path `~/.docker/scout`, command `rm -rf ~/.docker/scout`
- [x] 1.2 Add `uv-cache` — path `~/.cache/uv`, command `uv cache clean 2>/dev/null || rm -rf ~/.cache/uv/*`
- [x] 1.3 Add `puppeteer-cache` — path `~/.cache/puppeteer`, command `rm -rf ~/.cache/puppeteer/*`
- [x] 1.4 Add `node-gyp` — path `~/Library/Caches/node-gyp`, command `rm -rf ~/Library/Caches/node-gyp/*`
- [x] 1.5 Add `tableplus-cache` — path `~/Library/Caches/com.tinyapp.TablePlus`, command `rm -rf ~/Library/Caches/com.tinyapp.TablePlus/*`
- [x] 1.6 Add `user-logs` — path `~/Library/Logs`, command `rm -rf ~/Library/Logs/*`, reason notes logs are regenerable
- [x] 1.7 Add `quicklook-cache` — path to the QuickLook thumbnail cache, command `qlmanage -r cache` (non-script command; note divergence)
- [x] 1.8 Add `vscode-cache` — path `~/Library/Application Support/Code/Cache`, command removing `Code/Cache` and `Code/CachedData` (backslash-escape `Application Support`)
- [x] 1.9 Add `cursor-cache` — path `~/Library/Application Support/Cursor/Cache`, command removing `Cursor/Cache` and `Cursor/CachedData`; leave the Tier 3 `cursor` target untouched

## 2. Electron app HTTP/GPU cache targets (Tier 1)

- [x] 2.1 Add `discord-cache` — remove Discord's `Cache`, `Code Cache`, `GPUCache`, `Service Worker/CacheStorage` under its Application Support dir
- [x] 2.2 Add `notion-cache` — same cache subdirs under `~/Library/Application Support/Notion`; leave the Tier 3 `notion` target untouched
- [x] 2.3 Add `teams-cache` — same cache subdirs under the Microsoft Teams container path
- [x] 2.4 Add `postman-cache` — same cache subdirs under Postman's Application Support dir
- [x] 2.5 Add `zoom-cache` — Zoom cache directory under `~/Library/Application Support/zoom.us`
- [x] 2.6 Ensure each Electron cache target reports Empty/NotInstalled when its app dir is absent, disabling the action

## 3. Pattern-based updater-stub container targets (Tier 1)

- [x] 3.1 Write `fn shipit_updaters_target() -> Target` enumerating every `~/Library/Caches/*.ShipIt` dir as a subitem, excluding `com.todesktop.230313mzl4w4u92.ShipIt`; `command: None`, per-subitem `rm -rf '<dir>'/*`
- [x] 3.2 Write `fn electron_updaters_target() -> Target` enumerating `~/Library/Caches/*updater*` and `@*updater*` dirs as subitems; `command: None`, per-subitem `rm -rf '<dir>'/*`
- [x] 3.3 Both containers report Empty when no directories match; push both in `catalog_defs()` in the Tier 1 block

## 4. Tier 2 regenerable / gated targets

- [x] 4.1 Add `coresimulator-caches` — path `~/Library/Developer/CoreSimulator/Caches`, command `rm -rf ~/Library/Developer/CoreSimulator/Caches/*`
- [x] 4.2 Add `xcode-devicesupport` — remove `iOS DeviceSupport`, `watchOS DeviceSupport`, `tvOS DeviceSupport` contents under `~/Library/Developer/Xcode`
- [x] 4.3 Add `trash` — command empties `~/.Trash/*` and `/Volumes/*/.Trashes/<uid>` (tolerate absent volumes); no double-confirm

## 5. Tier 3 informational target

- [x] 5.1 Add `downloads` to the Tier 3 loop/section — path `~/Downloads`, `command: None`, standard Tier 3 risk note (never deleted)

## 6. rustup container target (Tier 2)

- [x] 6.1 Write `fn rustup_target() -> Target` running `rustup toolchain list`, keeping the default/active toolchain, offering each other toolchain via `rustup toolchain uninstall '<name>'`
- [x] 6.2 Report `ToolMissing` when the `rustup` binary is absent; push in `catalog_defs()` Tier 2 block

## 7. Tests & type sync

- [x] 7.1 Extend the catalog composition unit tests in `cleanup/mod.rs` to assert every new id exists with its expected tier and that container targets (`shipit-updaters`, `electron-updaters`, `rustup`) have no top-level command
- [x] 7.2 Add a test asserting `shipit-updaters` never includes `com.todesktop.230313mzl4w4u92.ShipIt` (no overlap with `shipit`)
- [x] 7.3 Verify `packages/app/src/cleanup/types.ts` still covers all tiers/statuses (no structural change expected); update `TIER_LABELS`/`STATUS_LABELS` only if needed
- [x] 7.4 Run `cargo test` in `src-tauri` and confirm the catalog tests pass

## 8. Verification

- [x] 8.1 Build the app and run a scan; confirm the new targets appear under the correct tiers with measured sizes on the maintainer's machine
- [x] 8.2 Spot-check that container targets (`shipit-updaters`, `electron-updaters`, `rustup`) list expected subitems and that `downloads` shows no action button
