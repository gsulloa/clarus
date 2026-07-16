## Context

Clarus's cleanup catalog groups targets into three tiers. Tier 1 (caches) and Tier 2 (regenerables) are actionable: each has a `command` and a "Clean" button, and container targets (e.g. `conductor`, `ollama`, `rustup`) expose per-subitem buttons. Tier 3 is deliberately inert — built in a single loop in `catalog_defs()` with `command: None`, and rendered read-only with a "Manual only" tag and a "Clarus never deletes Tier 3 data automatically" evidence card.

The user wants Tier 3 to become deletable, with per-item deletion where it makes sense and a "delete whole group" button. The safety machinery already exists: `requires_double_confirm` on targets/items drives the frontend `SI` modal (`ConfirmModal`) and is enforced in the backend `clean_target`/`clean_item` guards. This change reuses that machinery rather than inventing new confirmation UI.

Relevant code:
- `packages/app/src-tauri/src/cleanup/mod.rs` — `catalog_defs()` (Tier 3 loop at the "TIER 3" section), container builders like `conductor_target()`/`ollama_target()`, `measure()`, `clean_target`/`clean_item`.
- `packages/app/src/App.tsx` — `TargetRow` (the `isTier3` branch and the container "Per item" branch), the evidence panel, `handleTarget`/`handleItem`, `ConfirmModal`.
- `packages/app/src/cleanup/types.ts` — `TIER_LABELS`, `isTargetActionable`.

## Goals / Non-Goals

**Goals:**
- Make every Tier 3 target deletable from the app, always behind the existing `SI` double-confirm modal.
- Disaggregate the natural "collection" Tier 3 targets (`downloads`, `chrome-profiles`, `claude-vm`, `utm`) into per-member subitems so users can delete one at a time.
- Add a group "Clean all" button for disaggregated targets, reusing the target-level action path.
- Keep existing Tier 1/2 behavior — including per-subitem-only containers — unchanged.

**Non-Goals:**
- No new confirmation mechanism; reuse `requires_double_confirm` + `ConfirmModal`.
- No move-to-Trash / soft delete / undo. Deletions stay `rm -rf`-style, consistent with the rest of the catalog.
- No fine-grained enumeration of `postgres` databases, `spark` mailboxes, or `whatsapp` internals — those stay single top-level deletes.
- No change to the `Target`/`Item` struct shapes or the Tauri command signatures.

## Decisions

### Decision 1: Two Tier 3 shapes — "simple" and "collection"

Split the current uniform Tier 3 loop into two builders:

- **Simple Tier 3** (`postgres`, `spark`, `whatsapp`, `notion`, `cursor`): a single top-level `command` that deletes the target's contents, `requires_double_confirm: true`, no subitems. Command form: `rm -rf '<expanded-path>'/* '<expanded-path>'/.[!.]* 2>/dev/null; true` so it clears contents (including dotfiles) while leaving the parent directory, and tolerates an empty/absent directory.
- **Collection Tier 3** (`downloads`, `chrome-profiles`, `claude-vm`, `utm`): enumerate members as subitems (each `rm -rf '<member>'`, `requires_double_confirm: true`) **and** set a top-level group `command` that clears the whole backing directory's contents (also double-confirm). The group command is what the "Clean all" button runs.

**Enumeration directory per collection target** (the display `path` may differ from the enumeration dir):
- `downloads` → enumerate immediate entries of `~/Downloads`.
- `claude-vm` → enumerate immediate entries of `~/Library/Application Support/Claude/vm_bundles`.
- `chrome-profiles` → enumerate profile dirs under `~/Library/Application Support/Google/Chrome` whose names are `Default`, `Guest Profile`, `System Profile`, or match `Profile *`. The target keeps its existing display path (`.../Google`) but the group command and subitems operate on the `Chrome/<profile>` dirs so only profiles are removed, not the whole `Google` support tree. _Rationale:_ the informational path was `.../Google`, but meaningful units are individual Chrome profiles.
- `utm` → enumerate `*.utm` entries under `~/Library/Containers/com.utmapp.UTM/Data/Documents`. Group command removes those `.utm` bundles.

_Alternatives considered:_ a single generic "clear contents" delete for every Tier 3 target (rejected — loses the per-item control the user asked for); deep enumeration of database/mailbox internals (rejected — high risk, low value, brittle).

### Decision 2: Reuse `requires_double_confirm`, no new plumbing

Every Tier 3 command (simple top-level, collection group, and each subitem) sets `requires_double_confirm: true`. The frontend already routes such actions through `ConfirmModal` via `handleTarget`/`handleItem`, and the backend `clean_target`/`clean_item` already return an error when `requires_double_confirm && !confirmed`. No changes to the confirm flow are needed — only the data now carries the flag.

_Note on the modal copy:_ `ConfirmModal` currently reads "This action is destructive and cannot be undone. Type SI to confirm" and shows the command — already appropriate for irreplaceable data, so no copy change is required there.

### Decision 3: Group "Clean all" = target-level command on a container

The UI currently assumes: a target with subitems is "per item" (shows a "Per item" tag, no target button); a target without subitems shows a "Clean" button. Generalize this: **if a target has subitems AND a non-null `command`, render the target-level `ActionButton` as the group "Clean all" control** (danger styling, since Tier 3 group commands are double-confirm), in addition to the per-subitem buttons in the expanded list.

Concretely in `TargetRow`'s CTA slot:
- `isContainer && target.command` → group `ActionButton` (label conveys "Clean all"; routes through `handleTarget` → confirm modal).
- `isContainer && !target.command` → keep the existing "Per item" tag (unchanged for `conductor`, `ollama`, `rustup`, updater containers, Android images).
- non-container → existing single `ActionButton`.

Remove the `isTier3` "Manual only" branch entirely; Tier 3 now flows through the same logic (simple Tier 3 = non-container with command → single button; collection Tier 3 = container with command → group button + per-item).

_Alternatives considered:_ a separate bespoke "Clean all" button distinct from `ActionButton` (rejected — `ActionButton` already handles idle/cleaning/done/error and confirm routing; reusing it keeps state handling uniform).

### Decision 4: Status semantics for collection targets

Reuse the existing container status rule: `Available` when there is ≥1 subitem, `Empty` when the backing directory exists but has no members, `NotInstalled` when the backing directory is absent. `measure()` already sums subitem sizes for the target total; the group command's freed space is measured by the existing before/after `df` diff. For simple Tier 3 targets, `measure()` `du`s the display path as today.

### Decision 5: Copy / framing changes

- `TIER_LABELS.three`: "Tier 3 · Informational" → "Tier 3 · Personal data".
- Tier 3 `reason`/`risk_note`: from "informational only" / "Clarus never deletes Tier 3 data" to reason "Persistent personal data" and risk "Permanent and irreplaceable — deleting this cannot be undone." (per target where helpful).
- Remove the `selected.tier === "three"` "Manual only" evidence card in `App.tsx`; the command block now renders for Tier 3 like any other target.

## Risks / Trade-offs

- **[Deleting irreplaceable data is genuinely dangerous]** → Every Tier 3 action (top-level, group, per-item) is `requires_double_confirm`, so it is impossible to trigger without the `SI` modal, which shows the exact command. Backend guards reject unconfirmed requests as a second line of defense.
- **[`chrome-profiles` group command could over-delete the whole `Google` support tree]** → The group command targets `Chrome/<profile>` dirs specifically, not `.../Google`, so Chrome's app-level data outside profiles is preserved. Subitems each remove one profile only.
- **[Deleting a Chrome profile while Chrome is running can corrupt state]** → Out of scope for this change (the informational entry never quit apps either); the double-confirm command is shown verbatim so the user sees exactly what runs. A follow-up could add a quit step like the cache targets.
- **[Enumeration adds `read_dir` work during catalog build]** → Small and bounded (top-level entries of a handful of dirs), consistent with existing collection builders (`android_images_target`, `caches_dir_subitems`).
- **[Test breakage]** → `downloads_is_tier3_informational` asserts `command.is_none()` and will fail. It is updated to assert the new deletable/disaggregated behavior; new tests assert every Tier 3 target has a command + double-confirm and that collection targets expose subitems.

## Migration Plan

Pure in-app behavior change; no data migration, no persisted state, no API/versioned contract. Ships in the normal app release. Rollback is reverting the change — Tier 3 returns to informational-only with no residual state.

## Open Questions

- Should Tier 3 app targets that are typically running (`cursor`, `notion`, Chrome profiles) quit the app before deleting, as the cache targets do? Deferred: keep parity with the previous informational entries (no quit) for this change; revisit if users report corruption. The exact command is always shown before running.
