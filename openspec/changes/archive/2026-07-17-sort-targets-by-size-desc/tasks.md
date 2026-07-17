## 1. Implement size-descending ordering in `sortTargets`

- [x] 1.1 In `packages/app/src/App.tsx`, update the `sortTargets()` comparator (currently `rank[a.tier] - rank[b.tier] || a.name.localeCompare(b.name)`) to `rank[a.tier] - rank[b.tier] || b.sizeBytes - a.sizeBytes || a.name.localeCompare(b.name)` so targets sort by size descending within each tier with `name` as the deterministic tie-break.
- [x] 1.2 In the same function, return each target as a shallow copy with its `subitems` sorted by `b.sizeBytes - a.sizeBytes || a.label.localeCompare(b.label)` (e.g. `{ ...t, subitems: [...t.subitems].sort(...) }`), so subitems render largest-first without mutating the objects delivered by Tauri events.
- [x] 1.3 Export `sortTargets` (or extract it into a small module such as `src/cleanup/sort.ts`) so it can be unit-tested; update the import in `App.tsx` accordingly.

## 2. Tests

- [x] 2.1 Add a `sortTargets` unit test (Vitest, alongside `src/format.test.ts`): targets in the same tier are ordered largest-to-smallest by `sizeBytes`.
- [x] 2.2 Assert tier grouping order is preserved — a large Tier 3 target stays below all Tier 1/Tier 2 targets (no cross-tier reordering).
- [x] 2.3 Assert a container target's `subitems` come back ordered largest-to-smallest by `sizeBytes`.
- [x] 2.4 Assert the tie-break: equal-size targets order by `name` and equal-size subitems order by `label`; all-`0` sizes (pre-measurement) fall back to alphabetical order.

## 3. Verify

- [x] 3.1 Run `pnpm --filter app typecheck`, `pnpm --filter app lint`, and `pnpm --filter app test:run` — all pass. (Package filter name is `clarus`, not `app`; ran `pnpm --filter clarus ...`. All pass: typecheck clean, lint clean, 6/6 tests.)
- [ ] 3.2 Run the app (`pnpm --filter app tauri:dev`), trigger a scan, and confirm targets render largest-first within each tier and subitems render largest-first when a container target is expanded; confirm skeleton rows stay stable (alphabetical) until measurements stream in.
