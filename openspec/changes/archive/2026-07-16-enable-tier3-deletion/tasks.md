## 1. Backend — Tier 3 catalog builders (`src-tauri/src/cleanup/mod.rs`)

- [x] 1.1 Add a `tier3_simple(...)` helper that builds a single-command Tier 3 target: `requires_double_confirm: true`, no subitems, command `rm -rf '<expanded-path>'/* '<expanded-path>'/.[!.]* 2>/dev/null; true`, reason "Persistent personal data" and risk "Permanent and irreplaceable — deleting this cannot be undone."
- [x] 1.2 Add a `tier3_collection(id, name, display_path, enum_dir, member_filter, group_command)` helper that enumerates immediate members of `enum_dir` (optionally filtered), builds one subitem per member (`rm -rf '<member>'`, `requires_double_confirm: true`), sets the target's top-level group `command` (double-confirm), and sets status Available/Empty/NotInstalled per member count and directory presence (mirroring existing container builders).
- [x] 1.3 Replace the informational Tier 3 loop in `catalog_defs()`: build `postgres`, `spark`, `whatsapp`, `notion`, `cursor` via `tier3_simple`; build `downloads`, `chrome-profiles`, `claude-vm`, `utm` via `tier3_collection`.
- [x] 1.4 Wire `downloads` → enumerate `~/Downloads` entries; group command clears `~/Downloads` contents.
- [x] 1.5 Wire `claude-vm` → enumerate `~/Library/Application Support/Claude/vm_bundles` entries; group command clears that dir's contents.
- [x] 1.6 Wire `utm` → enumerate `*.utm` under `~/Library/Containers/com.utmapp.UTM/Data/Documents`; group command removes those `.utm` bundles. Keep display path as the Data container.
- [x] 1.7 Wire `chrome-profiles` → enumerate profile dirs under `~/Library/Application Support/Google/Chrome` matching `Default`, `Guest Profile`, `System Profile`, or `Profile *`; group command removes only those profile dirs; keep display path `.../Google`.

## 2. Backend — tests

- [x] 2.1 Update `downloads_is_tier3_informational` → assert `downloads` is Tier 3, has subitems (when `~/Downloads` present) and a top-level group command, and every action requires double confirm; rename to reflect new behavior.
- [x] 2.2 Add a test asserting every Tier 3 target (`postgres`, `spark`, `claude-vm`, `utm`, `whatsapp`, `notion`, `cursor`, `chrome-profiles`, `downloads`) has a non-null command OR subitems, and that every Tier 3 command/subitem sets `requires_double_confirm`.
- [x] 2.3 Add a test asserting the collection Tier 3 targets (`downloads`, `chrome-profiles`, `claude-vm`, `utm`) carry a top-level group command in addition to per-subitem commands, while existing per-subitem-only containers (`conductor`, `ollama`, `rustup`) keep `command: None`.
- [x] 2.4 Run `cargo test` for the cleanup module and confirm the suite is green.

## 3. Frontend — types & copy (`src/cleanup/types.ts`)

- [x] 3.1 Change `TIER_LABELS.three` from "Tier 3 · Informational" to "Tier 3 · Personal data".

## 4. Frontend — TargetRow group/per-item rendering (`src/App.tsx`)

- [x] 4.1 Remove the `isTier3` "Manual only" branch in `TargetRow`'s CTA slot.
- [x] 4.2 In the CTA slot, render the target-level `ActionButton` as a group "Clean all" control when `isContainer && target.command` (danger styling; routes through `onClean` → `handleTarget` → confirm modal). Keep the "Per item" tag when `isContainer && !target.command`. Non-container path unchanged.
- [x] 4.3 Confirm the group "Clean all" label/affordance is distinguishable from the single "Clean" (e.g. label text "Clean all" for containers) without breaking `ActionButton`'s cleaning/done/error states.
- [x] 4.4 Remove the `selected.tier === "three"` "Manual only" evidence card; let the command block render for Tier 3 like any other target.

## 5. Verify end-to-end

- [x] 5.1 `pnpm typecheck` and `pnpm lint` pass.
- [x] 5.2 Build/run the app; run a scan and confirm: simple Tier 3 targets show a single "Clean" that opens the `SI` modal; collection Tier 3 targets show a group "Clean all" plus per-item buttons that each open the `SI` modal; per-subitem-only containers (`conductor`, `ollama`) are unchanged.
- [x] 5.3 Confirm cancelling the modal deletes nothing and that an unconfirmed backend request is rejected.
