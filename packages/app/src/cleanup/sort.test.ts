import { describe, expect, it } from "vitest";

import { sortTargets } from "./sort";
import type { CleanupItem, CleanupTarget } from "./types";

function makeTarget(overrides: Partial<CleanupTarget> & { id: string }): CleanupTarget {
  return {
    name: overrides.id,
    tier: "one",
    path: null,
    sizeBytes: 0,
    sizeHuman: "0 B",
    status: "available",
    reason: "",
    riskNote: "",
    caveat: null,
    requiresDoubleConfirm: false,
    command: null,
    subitems: [],
    ...overrides,
  };
}

function makeItem(overrides: Partial<CleanupItem> & { id: string }): CleanupItem {
  return {
    label: overrides.id,
    path: "",
    sizeBytes: 0,
    sizeHuman: "0 B",
    meta: null,
    requiresDoubleConfirm: false,
    command: "",
    ...overrides,
  };
}

describe("sortTargets", () => {
  it("orders targets within the same tier largest-to-smallest by size", () => {
    const input = [
      makeTarget({ id: "small", tier: "one", sizeBytes: 10 }),
      makeTarget({ id: "large", tier: "one", sizeBytes: 100 }),
      makeTarget({ id: "medium", tier: "one", sizeBytes: 50 }),
    ];

    const result = sortTargets(input);

    expect(result.map((t) => t.id)).toEqual(["large", "medium", "small"]);
  });

  it("preserves tier grouping order even when a later tier has a larger target", () => {
    const input = [
      makeTarget({ id: "tier1-small", tier: "one", sizeBytes: 5 }),
      makeTarget({ id: "tier3-huge", tier: "three", sizeBytes: 1_000_000 }),
      makeTarget({ id: "tier2-medium", tier: "two", sizeBytes: 500 }),
      makeTarget({ id: "tier1-large", tier: "one", sizeBytes: 200 }),
    ];

    const result = sortTargets(input);

    expect(result.map((t) => t.id)).toEqual([
      "tier1-large",
      "tier1-small",
      "tier2-medium",
      "tier3-huge",
    ]);
  });

  it("orders a container target's subitems largest-to-smallest by size", () => {
    const input = [
      makeTarget({
        id: "container",
        tier: "one",
        subitems: [
          makeItem({ id: "b", label: "b", sizeBytes: 20 }),
          makeItem({ id: "a", label: "a", sizeBytes: 200 }),
          makeItem({ id: "c", label: "c", sizeBytes: 100 }),
        ],
      }),
    ];

    const result = sortTargets(input);

    expect(result[0].subitems.map((i) => i.id)).toEqual(["a", "c", "b"]);
  });

  it("falls back to alphabetical order for equal-size targets and subitems, including the pre-measurement 0 state", () => {
    const input = [
      makeTarget({
        id: "zebra",
        name: "Zebra",
        tier: "one",
        sizeBytes: 0,
        subitems: [
          makeItem({ id: "y", label: "Yankee", sizeBytes: 0 }),
          makeItem({ id: "x", label: "Xray", sizeBytes: 0 }),
        ],
      }),
      makeTarget({ id: "alpha", name: "Alpha", tier: "one", sizeBytes: 0 }),
      makeTarget({ id: "mike", name: "Mike", tier: "one", sizeBytes: 100 }),
      makeTarget({ id: "november", name: "November", tier: "one", sizeBytes: 100 }),
    ];

    const result = sortTargets(input);

    // Equal sizeBytes (100) tie-break by name: Mike before November.
    // Equal sizeBytes (0) tie-break by name: Alpha before Zebra.
    // Size descending groups the 100s ahead of the 0s.
    expect(result.map((t) => t.id)).toEqual(["mike", "november", "alpha", "zebra"]);
    expect(result.find((t) => t.id === "zebra")?.subitems.map((i) => i.id)).toEqual([
      "x",
      "y",
    ]);
  });

  it("does not mutate the input targets or subitems arrays", () => {
    const originalSubitems = [
      makeItem({ id: "b", label: "b", sizeBytes: 1 }),
      makeItem({ id: "a", label: "a", sizeBytes: 2 }),
    ];
    const target = makeTarget({ id: "t", tier: "one", subitems: originalSubitems });
    const input = [target];
    const inputSnapshot = JSON.parse(JSON.stringify(input));

    sortTargets(input);

    expect(input).toEqual(inputSnapshot);
    expect(input[0]).toBe(target);
    expect(input[0].subitems).toBe(originalSubitems);
    expect(originalSubitems.map((i) => i.id)).toEqual(["b", "a"]);
  });
});
