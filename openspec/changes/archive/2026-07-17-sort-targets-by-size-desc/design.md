## Context

Clarus streams a cleanup catalog to the UI in two phases (see `scan_cleanup_targets` in `packages/app/src-tauri/src/cleanup/mod.rs:2323`): first an unmeasured catalog (`cleanup://catalog`, all `sizeBytes = 0`) for skeleton rows, then one `cleanup://target` event per target as `measure()` fills in sizes. The frontend (`packages/app/src/App.tsx`) holds a flat `targets` array and derives:

- `grouped` (App.tsx:94) — a `Record<Tier, CleanupTarget[]>` built by iterating `targets` in order and pushing into per-tier buckets. **Within-tier display order therefore equals the order of `targets`.**
- The rendered tier blocks (Tier 1 → Tier 2 → Tier 3) and, for expanded container targets, subitem rows rendered in `target.subitems` array order (App.tsx:707).

All ordering flows through one function, `sortTargets()` (App.tsx:506), called on every catalog event (line 121), every measured event (line 133), and after the final scan (line 144). It currently sorts by tier rank then `name.localeCompare`. Subitem order is whatever the backend produced (alphabetical / version sort); the frontend never reorders subitems.

`sizeBytes` is authoritative and already computed for both targets and subitems by `measure()` (mod.rs:2294), which sums subitem sizes into the target total. So the data needed to sort by size already exists end-to-end.

## Goals / Non-Goals

**Goals:**
- Order targets largest-to-smallest **within** each tier group.
- Order subitems largest-to-smallest **within** each container/granular target.
- Keep the fixed Tier 1 → Tier 2 → Tier 3 group order.
- Keep ordering deterministic and stable during the streaming/skeleton phase (all sizes `0`).
- Do it in one place, without a backend change.

**Non-Goals:**
- No cross-tier sorting (a huge Tier 3 target must not jump above Tier 1).
- No user-configurable sort direction or sort key. "Always largest-to-smallest" is fixed.
- No change to backend enumeration order, measurement, or the Tauri command surface.
- No change to which targets/subitems exist or their actions.

## Decisions

### Decision 1: Sort entirely in the frontend `sortTargets()`

`sortTargets()` is already the single choke point for ordering and already re-runs on every streaming event, so extending it covers targets and subitems in one place with zero backend churn.

- **Targets:** comparator becomes `rank[a.tier] - rank[b.tier] || b.sizeBytes - a.sizeBytes || a.name.localeCompare(b.name)`. Tier rank stays first so grouping order is preserved; size descending is the new within-tier key; `name` is the deterministic tie-break.
- **Subitems:** `sortTargets` returns each target as a shallow copy with `subitems` re-sorted by `b.sizeBytes - a.sizeBytes || a.label.localeCompare(b.label)`. Returning copies (`{ ...t, subitems: [...t.subitems].sort(...) }`) avoids mutating the objects delivered by Tauri events.

**Alternative considered — sort subitems in the Rust `measure()`:** subitem sizes only exist after `measure()`, so that is the one backend spot where a size sort is possible, and targets arrive at the UI pre-sorted. Rejected because it splits ordering across two languages/locations (targets in TS, subitems in Rust), duplicates the tie-break rule, and needs its own Rust test. Keeping all ordering in `sortTargets` is simpler and keeps one source of truth.

### Decision 2: Tie-break by the current alphabetical key, not by insertion order

During the catalog/skeleton phase every `sizeBytes` is `0`, so the size comparator is a no-op and the alphabetical tie-break (`name` for targets, `label` for subitems) fully determines order. This reproduces today's alphabetical skeleton exactly, then rows settle into size order as measurements stream in. Using array/insertion order as the tie-break instead would make skeleton order depend on backend emission timing — rejected for non-determinism.

### Decision 3: Keep tier rank in the comparator even though `grouped` re-buckets by tier

`grouped` already separates tiers for rendering, so tier rank in `sortTargets` is technically redundant for the grouped view. It is kept because other consumers read the flat `targets` order directly — notably the default selection `scan.targets[0]` (App.tsx:155) and any future flat rendering — and keeping tier-first order there avoids surprising behavior. It is cheap and preserves a single well-defined global order.

## Risks / Trade-offs

- **Rows visibly re-order during a scan** as `cleanup://target` events arrive and `0`-sizes become real → This already happens on every measured event (the list is re-sorted each time); today the alphabetical key just makes movement invisible. It is expected and acceptable for a tool whose value is "biggest wins first." Mitigation: none needed; the skeleton stays alphabetical and stable until a target's own measurement lands.
- **Shallow-copying targets to sort subitems changes object identity each sort** → could in theory disturb React reconciliation, but rows are keyed by stable `id`/`item.id` (App.tsx:708), so keys, not identity, drive reconciliation. No functional impact.
- **Equal-size items reorder if names change** → tie-break is deterministic on `name`/`label`, which are stable for a given catalog, so no flicker.

## Migration Plan

Pure display-ordering change, no data migration. Ship in `sortTargets` behind no flag. Rollback is reverting the single function to the tier-then-name comparator.
