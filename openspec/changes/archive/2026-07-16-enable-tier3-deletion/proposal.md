## Why

Tier 3 targets (PostgreSQL databases, Spark emails, Claude/UTM VMs, WhatsApp data, browser profiles, Downloads) are currently informational-only — Clarus shows their size but refuses to delete them. Users who have already made the manual decision to reclaim that space have no way to act from inside Clarus and must drop to the terminal. This change lets users delete Tier 3 data from the app, behind strong, explicit confirmation, so the tool can finish the job it surfaces.

## What Changes

- **BREAKING** (behavioral): Tier 3 targets become actionable instead of read-only. Every Tier 3 target gains a real cleanup command and **requires the double-confirm (type `SI`) modal** before anything is deleted, because this data is irreplaceable and not regenerable.
- **Disaggregate collection-type Tier 3 targets into per-item rows** so users can delete one entry at a time instead of all-or-nothing:
  - `downloads` — one subitem per top-level entry in `~/Downloads`.
  - `chrome-profiles` — one subitem per Chrome profile (`Default`, `Profile N`, …).
  - `claude-vm` — one subitem per VM bundle.
  - `utm` — one subitem per `.utm` virtual machine.
- **Offer a "Clean all" group button** on disaggregated Tier 3 targets that deletes the whole group in one action (also gated by the double-confirm modal).
- Non-collection Tier 3 targets (`postgres`, `spark`, `whatsapp`, `notion`, `cursor`) get a single top-level delete action (double-confirm), no subitems.
- Update Tier 3 framing: the tier label changes from "Informational" to "Personal data", and the risk copy changes from "Clarus never deletes Tier 3 data" to a warning that deletion is permanent and irreversible.
- Introduce a general UI capability: a container target that has **both** subitems and a top-level command renders per-item buttons **and** a group "Clean all" button. Existing Tier 1/2 containers (`conductor`, `ollama`, `rustup`, …) keep no top-level command and are unaffected.

## Capabilities

### New Capabilities

_None. This change extends the existing disk-cleanup capability; it introduces no new capability._

### Modified Capabilities

- `disk-cleanup`: 
  - "Catalog of known cleanup targets" — Tier 3 is redefined from "persistent data, informational only" to "persistent personal data, deletable only behind explicit confirmation".
  - "Results grouped by tier for review" — the requirement that Tier 3 rows be read-only with a "never cleaned automatically" warning is replaced by Tier 3 rows exposing a confirmation-gated cleanup action.
  - "Per-subitem cleanup for container targets" — adds the disaggregated Tier 3 collection targets (`downloads`, `chrome-profiles`, `claude-vm`, `utm`) and a new group-level "clean all" action available on any container that also carries a top-level command.

## Impact

- **Backend:** `packages/app/src-tauri/src/cleanup/mod.rs` — replace the informational Tier 3 loop with builders that attach cleanup commands (all `requires_double_confirm: true`) and, for collection targets, enumerate subitems plus a group command. Update the `downloads_is_tier3_informational` test and add coverage that every Tier 3 target has a command + double-confirm and that collection targets expose subitems. No changes to the `Target`/`Item` structs or the Tauri command surface (`clean_target`/`clean_item` already enforce `requires_double_confirm`).
- **Frontend:** `packages/app/src/App.tsx` — remove the `isTier3` "Manual only" branch; render a group "Clean all" button when a container target has a top-level command; drop the Tier-3 "manual only" evidence card. `packages/app/src/cleanup/types.ts` — update `TIER_LABELS.three` copy. Types are otherwise unchanged (they already mirror the Rust structs).
- **Risk:** This is the first path in Clarus that deletes irreplaceable user data. Mitigated by routing every Tier 3 deletion (per-item and group) through the existing `SI` double-confirm modal, which shows the exact command before it runs. No Tier 3 action can execute without confirmation.
