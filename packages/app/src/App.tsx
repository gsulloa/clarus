import { useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Activity,
  ArrowUpRight,
  CheckCircle2,
  FolderOpen,
  RefreshCw,
  ShieldCheck,
  Sparkles,
  Trash2,
} from "lucide-react";

import mark from "./assets/clarus-mark.svg";
import { formatBytes } from "./format";
import { scanDirectory } from "./scan/api";
import type { ScanCandidate, ScanSummary } from "./scan/types";
import { useUpdater } from "./platform/updater/useUpdater";

const kindLabels: Record<ScanCandidate["kind"], string> = {
  cache: "Cache",
  temporary: "Temporary",
  log: "Log",
  "large-file": "Large file",
  "empty-directory": "Empty directory",
};

function emptySummary(): ScanSummary {
  return {
    root: "",
    scannedFiles: 0,
    scannedDirectories: 0,
    totalBytes: 0,
    candidateBytes: 0,
    candidates: [],
  };
}

export function App() {
  const [rootPath, setRootPath] = useState("");
  const [summary, setSummary] = useState<ScanSummary>(emptySummary);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [status, setStatus] = useState<
    "idle" | "scanning" | "complete" | "error"
  >("idle");
  const [error, setError] = useState<string | null>(null);
  const updater = useUpdater();

  const selected = useMemo(
    () =>
      summary.candidates.find((candidate) => candidate.id === selectedId) ??
      summary.candidates[0],
    [selectedId, summary.candidates],
  );

  const categoryTotals = useMemo(() => {
    const totals = new Map<ScanCandidate["kind"], number>();
    for (const candidate of summary.candidates) {
      totals.set(
        candidate.kind,
        (totals.get(candidate.kind) ?? 0) + candidate.sizeBytes,
      );
    }
    return [...totals.entries()].sort((a, b) => b[1] - a[1]);
  }, [summary.candidates]);

  async function chooseFolder() {
    const selectedPath = await open({ directory: true, multiple: false });
    if (typeof selectedPath !== "string") return;
    setRootPath(selectedPath);
    setStatus("idle");
    setError(null);
  }

  async function runScan() {
    if (!rootPath) return;
    setStatus("scanning");
    setError(null);
    try {
      const result = await scanDirectory(rootPath);
      setSummary(result);
      setSelectedId(result.candidates[0]?.id ?? null);
      setStatus("complete");
    } catch (err) {
      setError(String(err));
      setStatus("error");
    }
  }

  return (
    <main className="app-shell">
      <aside className="control-rail">
        <div className="brand-lockup">
          <img src={mark} alt="" className="brand-mark" />
          <div>
            <p className="eyebrow">Clarus</p>
            <h1>Disk intelligence</h1>
          </div>
        </div>

        <section className="rail-section">
          <p className="section-label">Scope</p>
          <button
            className="primary-action"
            type="button"
            onClick={chooseFolder}
          >
            <FolderOpen size={17} />
            Select folder
          </button>
          <div className="path-readout">{rootPath || "No folder selected"}</div>
          <button
            className="scan-action"
            type="button"
            disabled={!rootPath || status === "scanning"}
            onClick={runScan}
          >
            <Activity size={17} />
            {status === "scanning"
              ? "Analyzing structure..."
              : "Start deep scan"}
          </button>
        </section>

        <section className="rail-section">
          <p className="section-label">Release channel</p>
          <button
            className="quiet-action"
            type="button"
            onClick={() => void updater.checkNow()}
          >
            <RefreshCw size={16} />
            Check for updates
          </button>
          <p className="muted-copy">
            {updater.current === "available" && updater.version
              ? `Version ${updater.version} is available.`
              : updater.current === "checking"
                ? "Checking signed release manifest..."
                : updater.current === "error"
                  ? "Updater manifest is not reachable yet."
                  : "Release channel is configured."}
          </p>
        </section>
      </aside>

      <section className="intelligence-surface">
        <header className="surface-header">
          <div>
            <p className="eyebrow">Dry-run analysis</p>
            <h2>Review candidates before taking action.</h2>
          </div>
          <div className="safety-pill">
            <ShieldCheck size={16} />
            No files are modified
          </div>
        </header>

        {error ? <div className="error-banner">{error}</div> : null}

        <div className="metric-grid">
          <div className="metric">
            <span>Scanned</span>
            <strong>{formatBytes(summary.totalBytes)}</strong>
          </div>
          <div className="metric">
            <span>Candidates</span>
            <strong>{formatBytes(summary.candidateBytes)}</strong>
          </div>
          <div className="metric">
            <span>Files</span>
            <strong>{summary.scannedFiles.toLocaleString()}</strong>
          </div>
          <div className="metric">
            <span>Folders</span>
            <strong>{summary.scannedDirectories.toLocaleString()}</strong>
          </div>
        </div>

        <div className="category-strip">
          {categoryTotals.length === 0 ? (
            <div className="category-empty">
              Run a scan to classify optimization candidates.
            </div>
          ) : (
            categoryTotals.map(([kind, bytes]) => (
              <div className="category-chip" key={kind}>
                <span>{kindLabels[kind]}</span>
                <strong>{formatBytes(bytes)}</strong>
              </div>
            ))
          )}
        </div>

        <div className="candidate-table">
          <div className="table-head">
            <span>Candidate</span>
            <span>Type</span>
            <span>Size</span>
            <span>Confidence</span>
          </div>
          {summary.candidates.length === 0 ? (
            <div className="empty-state">
              <Sparkles size={28} />
              <p>
                Clarus will list candidates here with the reason for each
                recommendation.
              </p>
            </div>
          ) : (
            summary.candidates.map((candidate) => (
              <button
                className="candidate-row"
                data-active={candidate.id === selected?.id}
                key={candidate.id}
                type="button"
                onClick={() => setSelectedId(candidate.id)}
              >
                <span className="candidate-path">{candidate.path}</span>
                <span>{kindLabels[candidate.kind]}</span>
                <span>{formatBytes(candidate.sizeBytes)}</span>
                <span>{candidate.confidence}</span>
              </button>
            ))
          )}
        </div>
      </section>

      <aside className="evidence-panel">
        <p className="section-label">Evidence</p>
        {selected ? (
          <>
            <h3>{kindLabels[selected.kind]}</h3>
            <p className="evidence-path">{selected.path}</p>
            <div className="evidence-card">
              <span>Reason</span>
              <p>{selected.reason}</p>
            </div>
            <div className="evidence-card">
              <span>Estimated recovery</span>
              <p>{formatBytes(selected.sizeBytes)}</p>
            </div>
            <button className="quiet-action" type="button">
              <ArrowUpRight size={16} />
              Exclude folder
            </button>
            <button className="danger-action" type="button" disabled>
              <Trash2 size={16} />
              Move to Trash
            </button>
            <p className="muted-copy">
              Trash actions are intentionally gated until review and undo flows
              are implemented.
            </p>
          </>
        ) : (
          <div className="evidence-empty">
            <CheckCircle2 size={28} />
            <p>Select a candidate to inspect why Clarus marked it.</p>
          </div>
        )}
      </aside>
    </main>
  );
}
