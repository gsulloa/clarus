export type ScanCandidateKind =
  "cache" | "temporary" | "log" | "large-file" | "empty-directory";

export type ScanCandidate = {
  id: string;
  path: string;
  kind: ScanCandidateKind;
  sizeBytes: number;
  reason: string;
  confidence: "low" | "medium" | "high";
};

export type ScanSummary = {
  root: string;
  scannedFiles: number;
  scannedDirectories: number;
  totalBytes: number;
  candidateBytes: number;
  candidates: ScanCandidate[];
};
