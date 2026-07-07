import { invoke } from "@tauri-apps/api/core";
import type { ScanSummary } from "./types";

export async function scanDirectory(path: string): Promise<ScanSummary> {
  return invoke<ScanSummary>("scan_directory", { path });
}
