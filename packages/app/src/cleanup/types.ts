// Mirrors packages/app/src-tauri/src/cleanup/mod.rs (serde camelCase).

export type Tier = "one" | "two" | "three";

export type Status = "available" | "empty" | "toolMissing" | "notInstalled";

export type CleanupItem = {
  id: string;
  label: string;
  path: string;
  sizeBytes: number;
  sizeHuman: string;
  meta: string | null;
  requiresDoubleConfirm: boolean;
  command: string;
};

export type CleanupTarget = {
  id: string;
  name: string;
  tier: Tier;
  path: string | null;
  sizeBytes: number;
  sizeHuman: string;
  status: Status;
  reason: string;
  riskNote: string;
  caveat: string | null;
  requiresDoubleConfirm: boolean;
  command: string | null;
  subitems: CleanupItem[];
};

export type CleanupScan = {
  freeBeforeGb: number;
  freeBeforeHuman: string;
  totalBeforeGb: number;
  totalBeforeHuman: string;
  usedBeforeGb: number;
  usedBeforeHuman: string;
  targets: CleanupTarget[];
};

export type CleanResult = {
  ok: boolean;
  message: string | null;
  freeGb: number;
  freeHuman: string;
  freedGb: number;
  totalGb: number;
  totalHuman: string;
  usedGb: number;
  usedHuman: string;
};

export const TIER_LABELS: Record<Tier, string> = {
  one: "Tier 1 · Caches",
  two: "Tier 2 · Regenerables",
  three: "Tier 3 · Personal data",
};

export const STATUS_LABELS: Record<Status, string> = {
  available: "Ready",
  empty: "Empty",
  toolMissing: "Tool missing",
  notInstalled: "Not installed",
};

/** A target can be cleaned directly only if it has a command and is ready. */
export function isTargetActionable(target: CleanupTarget): boolean {
  return target.command !== null && target.status === "available";
}
