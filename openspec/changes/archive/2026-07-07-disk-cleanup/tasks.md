## 1. Backend — cleanup module scaffolding

- [x] 1.1 Create `packages/app/src-tauri/src/cleanup/mod.rs` and declare `pub mod cleanup;` in `lib.rs`
- [x] 1.2 Define serde (camelCase) types: `Tier`, `Status`, `Target`, `Item`, `CleanupScan`, `CleanResult`
- [x] 1.3 Add helpers: `du_bytes(path)` via `du -sk`, `size_human` via `formatBytes`-equivalent, `disk_free()` via `df -g`/`df -h /System/Volumes/Data`, and a `run_bash(cmd)` wrapper around `std::process::Command::new("bash").arg("-c")`

## 2. Backend — catalog faithful to the script

- [x] 2.1 Define Tier 1 targets with exact commands: Yarn, npm, pip, Homebrew, ShipIt, Playwright, Spotify (quit first), Chrome (quit first), Bun — including tool-if-present fallbacks
- [x] 2.2 Define Tier 2 simple targets with exact commands: Xcode Archives, Xcode DerivedData, Cargo cache (cargo-cache autoclean else manual)
- [x] 2.3 Implement Docker target: prune sequence (builder/image/container/volume/system), auto-start Docker + wait ≤90s, and Docker.raw regeneration as a `requiresDoubleConfirm` step (quit Docker, `rm -f Docker.raw`, reopen)
- [x] 2.4 Implement container-target subitem enumerators matching the script: iOS runtimes (jq group-by-platform, keep newest; fallback to `simctl delete unavailable` without jq), nvm versions (only if >3, keep current + latest LTS), Conductor worktrees (branch + git status, `requiresDoubleConfirm` if dirty), Android system-images
- [x] 2.5 Define Tier 3 informational targets (Postgres, Spark, Claude VMs, UTM, WhatsApp, Notion, Cursor, Chrome profiles) — size only, no command

## 3. Backend — commands

- [x] 3.1 Implement `scan_cleanup_targets()` running detections concurrently (threads) and emitting a Tauri event per target as it finishes
- [x] 3.2 Implement `clean_target(id, confirmed)` — runs the target's exact command; reject double-confirm targets unless `confirmed`
- [x] 3.3 Implement `clean_item(targetId, itemId, confirmed)` for container subitems
- [x] 3.4 Implement `disk_free()` command
- [x] 3.5 Register all four commands in `lib.rs` `invoke_handler`

## 4. Frontend — types & api

- [x] 4.1 Create `packages/app/src/cleanup/types.ts` mirroring the Rust types
- [x] 4.2 Create `packages/app/src/cleanup/api.ts` with `scanCleanupTargets`, `cleanTarget`, `cleanItem`, `diskFree`, and the scan-progress event subscription

## 5. Frontend — UI

- [x] 5.1 Left rail: "Analyze cleanup targets" scan-all button, disk-free readout (before / now / freed), scan progress; keep updater section
- [x] 5.2 Center surface: targets grouped Tier 1 → Tier 2 → Tier 3; Tier 1/2 rows with name · path · size · status chip · action button (disabled for Empty/ToolMissing/NotInstalled)
- [x] 5.3 Container rows expand to per-subitem rows each with its own action button
- [x] 5.4 Tier 3 rows read-only with "manual only" warning
- [x] 5.5 Right evidence panel: selected target reason, risk note, exact command, caveat
- [x] 5.6 Row action states: idle → cleaning (spinner) → done (freed X) / error (message); update disk-free readout after each action
- [x] 5.7 Type-to-confirm modal (type `SI`) for Docker.raw regen and dirty Conductor workspaces
- [x] 5.8 Remove the folder-picker + generic candidate table from `App.tsx`

## 6. Verify

- [x] 6.1 Run `pnpm typecheck` and `cargo check` (in `src-tauri`) — both clean
- [ ] 6.2 Run `pnpm tauri:dev`, perform a scan, confirm tiers/sizes/commands render and disk-free baseline shows
- [ ] 6.3 Run one safe cleanup end to end (pip or Playwright), confirm row transitions and disk-free updates
- [ ] 6.4 Confirm double-confirm modal blocks Docker.raw regen and dirty Conductor workspace deletion until `SI` is typed
