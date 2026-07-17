## Why

Clarus currently orders cleanup targets alphabetically by name within each tier, and subitems alphabetically (or by version) within each container target. For a disk-recovery tool, the size of a candidate is the primary decision signal — the user wants the biggest space wins first, not to hunt alphabetically. Ordering everything largest-to-smallest inside its group makes the highest-impact candidates immediately visible.

## What Changes

- Within each tier group, targets are ordered by measured size **descending** (largest first) instead of alphabetically by name.
- Within each container/granular target ("elimina granular"), subitems are ordered by measured size **descending** instead of alphabetically/by version.
- The tier grouping and the fixed Tier 1 → Tier 2 → Tier 3 order are unchanged; sorting by size happens **inside** each group, never across tiers.
- Ties (equal size, including the pre-measurement state where sizes are still `0`) fall back to the current alphabetical order (`name` for targets, `label` for subitems) so ordering stays deterministic and skeleton rows stay stable before measurements stream in.

## Capabilities

### New Capabilities

_None. This change extends the existing `disk-cleanup` capability; it introduces no new capability._

### Modified Capabilities

- `disk-cleanup`:
  - "Results grouped by tier for review" — adds a requirement that, within each tier, targets are ordered by measured size descending (ties broken by name), while the Tier 1 → Tier 2 → Tier 3 grouping order is preserved.
  - "Per-subitem cleanup for container targets" — adds a requirement that a container target's subitems are ordered by measured size descending (ties broken by label).

## Impact

- **Frontend:** `packages/app/src/App.tsx` — `sortTargets()` (currently sorts by tier rank then `name.localeCompare`) gains a size-descending comparator inside each tier and also returns each target with its `subitems` sorted by size descending. This is the single place ordering is applied (it already re-runs on every `cleanup://catalog` and `cleanup://target` streaming event and on final scan), so no backend change is required.
- **Behavioral note:** Because target sizes stream in asynchronously (`cleanup://target` events), rows will re-order as measurements arrive during a scan; before measurement all sizes are `0` and rows keep their alphabetical fallback order. This is expected and matches the existing per-event re-sort behavior.
- **Backend:** No changes. `sizeBytes` is already computed for every target and subitem (`measure()` sums subitem sizes), so the data to sort by is already present.
- **Tests:** Frontend unit coverage for `sortTargets` (size-descending within tier, tier order preserved, subitems sorted, alphabetical tie-break, `0`-size stability).
