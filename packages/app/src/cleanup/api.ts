import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { CleanResult, CleanupScan, CleanupTarget } from "./types";

/** Kick off a full scan. Sizes stream in via `onTargetMeasured` as they finish. */
export async function scanCleanupTargets(): Promise<CleanupScan> {
  return invoke<CleanupScan>("scan_cleanup_targets");
}

export async function cleanTarget(
  id: string,
  confirmed = false,
): Promise<CleanResult> {
  return invoke<CleanResult>("clean_target", { id, confirmed });
}

export async function cleanItem(
  targetId: string,
  itemId: string,
  confirmed = false,
): Promise<CleanResult> {
  return invoke<CleanResult>("clean_item", { targetId, itemId, confirmed });
}

export async function diskFree(): Promise<CleanResult> {
  return invoke<CleanResult>("disk_free");
}

/** Subscribe to per-target size events emitted during a scan. */
export async function onTargetMeasured(
  handler: (target: CleanupTarget) => void,
): Promise<UnlistenFn> {
  return listen<CleanupTarget>("cleanup://target", (event) => {
    handler(event.payload);
  });
}

/** Subscribe to the one-shot catalog enumeration emitted at scan start. */
export async function onCatalogEnumerated(
  handler: (targets: CleanupTarget[]) => void,
): Promise<UnlistenFn> {
  return listen<CleanupTarget[]>("cleanup://catalog", (event) => {
    handler(event.payload);
  });
}
