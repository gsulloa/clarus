import type { CleanupTarget, Tier } from "./types";

/**
 * Orders targets by tier (Tier 1 -> Tier 2 -> Tier 3), then by measured size
 * descending within each tier, falling back to alphabetical `name` for ties
 * (including the pre-measurement state where every size is `0`).
 *
 * Each target is returned as a shallow copy with its `subitems` similarly
 * sorted by size descending (ties broken by `label`), so the input array and
 * its target/subitem objects — delivered by Tauri events — are never mutated.
 */
export function sortTargets(list: CleanupTarget[]): CleanupTarget[] {
  const rank: Record<Tier, number> = { one: 0, two: 1, three: 2 };
  return list
    .map((t) => ({
      ...t,
      subitems: [...t.subitems].sort(
        (a, b) => b.sizeBytes - a.sizeBytes || a.label.localeCompare(b.label),
      ),
    }))
    .sort(
      (a, b) =>
        rank[a.tier] - rank[b.tier] ||
        b.sizeBytes - a.sizeBytes ||
        a.name.localeCompare(b.name),
    );
}
