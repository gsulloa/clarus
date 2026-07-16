## Context

Clarus's cleanup catalog lives in a single Rust function, `catalog_defs()` in `packages/app/src-tauri/src/cleanup/mod.rs`, a faithful port of `~/disk-cleanup.sh`. Targets are either simple single-command entries (built with the `tier1(...)` helper or the `Def { ... }.into_target()` builder) or dynamic **container** targets (`command: None` + a `Vec<Item>` of per-subitem commands), built by dedicated `fn xxx_target() -> Target` functions (`ollama_target`, `nvm_target`, `conductor_target`, `android_images_target`, `ios_runtimes_target`). The frontend mirrors the serialized shapes in `packages/app/src/cleanup/types.ts`. Sizes are measured post-enumeration by `measure()`; commands run via `bash -lc` with hand-written shell escaping and `~` expanded by `expand()`.

A disk analysis of the maintainer's Mac found ~15 GB of reclaimable data in the same regenerable categories Clarus already handles, but not yet enumerated. This change adds those targets and reuses every existing mechanism — no new scan/clean plumbing, no new UI.

## Goals / Non-Goals

**Goals:**
- Add the new targets from the proposal using the existing `Def`/`tier1`/container patterns.
- Generalize updater-stub cleanup (`*.ShipIt`, `*updater*`) from one hardcoded app to all present ones, without double-counting the existing `shipit` target.
- Keep every existing target's `id`, tier, and command byte-for-byte unchanged (the spec requires script-derived commands to match verbatim).
- Extend the Rust catalog unit tests to cover the new ids and tiers.

**Non-Goals:**
- No changes to the scan/measure/clean flow, event streaming, or UI components.
- No changes to the generic heuristic folder scanner (`scan/mod.rs`).
- No deletion of Tier 3 data; `downloads` is informational only.
- Not auto-discovering arbitrary large caches — the new targets are a curated fixed set (the generic scanner already covers ad-hoc discovery).

## Decisions

### D1 — ShipIt overlap: keep `shipit`, add `shipit-updaters` that excludes it
The existing `shipit` target (`com.todesktop.230313mzl4w4u92.ShipIt`) matches the script verbatim and must stay. The new `shipit-updaters` container enumerates every `~/Library/Caches/*.ShipIt` directory as a subitem **except** `com.todesktop.230313mzl4w4u92.ShipIt`, so sizes never double-count. Each subitem's command is `rm -rf '<dir>'/*` (or removing the dir). Rationale: preserves script fidelity while covering VSCode/Claude/Slack/Notion/Discord/etc. Alternative rejected: replacing `shipit` with a single glob command would break the verbatim-match scenario and lose per-app visibility.

### D2 — `electron-updaters` container
Enumerate `~/Library/Caches/*updater*` and `~/Library/Caches/@*updater*` (e.g. `bruno-updater`, `@granolaelectron-updater`, `us.zoom.updater`) as subitems, one `rm -rf '<dir>'/*` each. Container so the user sees each app; status `Empty` when no matching dirs exist.

### D3 — Electron app HTTP caches as discrete per-app targets
Follow the existing `slack-cache` / `claude-desktop-cache` precedent: discrete targets `discord-cache`, `notion-cache`, `teams-cache`, `postman-cache`, `zoom-cache`, each removing that app's `Cache`, `Code Cache`, `GPUCache`, and `Service Worker/CacheStorage` under its Application Support (or container) directory. Each reports `NotInstalled`/`Empty` when the app dir is absent. Rationale: consistent with existing single-app cache targets; simpler than a generic Electron walker and avoids touching non-cache data. `notion-cache` and `cursor-cache` target only the cache subdirectories — the parent `notion`/`cursor` Tier 3 informational targets are untouched.

### D4 — `rustup` container keeps the active/default toolchain
Enumerate `rustup toolchain list`; keep the toolchain marked `(default)` (and the active override if any); offer every other toolchain via `rustup toolchain uninstall '<name>'`. Report `ToolMissing` when `rustup` is absent. Mirrors the existing `nvm`/`pyenv` "keep the one in use" pattern. Tier 2 (a deliberate action, re-installable via `rustup toolchain install`).

### D5 — Simple Tier 1 caches via `tier1(...)`
`docker-scout` (`rm -rf ~/.docker/scout`), `uv-cache` (`uv cache clean 2>/dev/null || rm -rf ~/.cache/uv/*`), `puppeteer-cache` (`rm -rf ~/.cache/puppeteer/*`), `node-gyp` (`rm -rf ~/Library/Caches/node-gyp/*`), `vscode-cache` (`rm -rf ~/Library/Application\ Support/Code/Cache ~/Library/Application\ Support/Code/CachedData`), `cursor-cache` (analogous for Cursor), `user-logs` (`rm -rf ~/Library/Logs/*`), `quicklook-cache` (`qlmanage -r cache`), `tableplus-cache` (`rm -rf ~/Library/Caches/com.tinyapp.TablePlus/*`). Each with a `path` for measurement and a `reason`/`caveat`.

### D6 — Tier 2 simple/gated targets
`coresimulator-caches` (`rm -rf ~/Library/Developer/CoreSimulator/Caches/*`), `xcode-devicesupport` (`rm -rf ~/Library/Developer/Xcode/iOS\ DeviceSupport/* ...watchOS... ...tvOS...`), and `trash`. `trash` empties `~/.Trash` and mounted-volume trashes (`/Volumes/*/.Trashes/$(id -u)`); Tier 2 because emptying Trash is a deliberate, non-regenerable act — but it does **not** require double-confirm (it is the OS's own hold area, not live data).

### D7 — `downloads` is Tier 3 informational
Follows the eight existing Tier 3 targets exactly: `command: None`, `status: Available`, path `~/Downloads`, risk note that Clarus never deletes Tier 3 data.

### D8 — Frontend types
The new targets reuse existing serialized shapes (`Target`/`Item`, `Tier`, `Status`); no new fields. `types.ts` needs no structural change — only re-verify `TIER_LABELS`/`STATUS_LABELS` still cover every case (they do). No new Tauri commands.

## Risks / Trade-offs

- **[Double-counting updater caches]** `shipit`, `shipit-updaters`, and `electron-updaters` could overlap → mitigated by D1 excluding the `shipit` dir from `shipit-updaters`, and by `*.ShipIt` vs `*updater*` globs being disjoint by name.
- **[`user-logs` removes app diagnostic logs]** clearing `~/Library/Logs` deletes logs some apps may reference → acceptable and regenerable (matches how the script treats `gradle-daemon` logs as Tier 1); documented in the target's `reason`.
- **[`quicklook-cache` via `qlmanage`]** diverges from a plain `rm -rf` → intentional; `qlmanage -r cache` is the supported, safe way to reset the thumbnail cache. Flagged as a non-script command per the spec's "MAY diverge" clause.
- **[Electron cache paths vary by app]** container vs Application Support location differs (e.g. Teams uses a sandbox container) → each target uses that app's actual path, verified during enumeration; missing paths yield `Empty`/`NotInstalled` rather than errors.
- **[`trash` on external volumes]** `/Volumes/*/.Trashes` may not match → command tolerates no-matches (`2>/dev/null`), mirroring existing tolerant commands.

## Migration Plan

Additive only. New targets appear on next scan; no data migration, no rollback concern. Reverting the change simply removes the new rows. Existing targets and their persisted behavior are unchanged.

## Open Questions

- Should `argus` (448 MB) and other personal/unknown app caches be included? **Deferred** — excluded from this change; the generic folder scanner already surfaces ad-hoc caches, and hardcoding project-specific apps adds little general value.
