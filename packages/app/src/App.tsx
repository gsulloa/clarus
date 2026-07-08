import { useEffect, useMemo, useRef, useState } from "react";
import {
  Activity,
  AlertTriangle,
  CheckCircle2,
  ChevronDown,
  ChevronRight,
  CircleSlash,
  HardDrive,
  Loader2,
  RefreshCw,
  ShieldCheck,
  Trash2,
} from "lucide-react";

import mark from "./assets/clarus-mark.svg";
import {
  cleanItem,
  cleanTarget,
  onCatalogEnumerated,
  onTargetMeasured,
  scanCleanupTargets,
} from "./cleanup/api";
import {
  isTargetActionable,
  STATUS_LABELS,
  TIER_LABELS,
  type CleanResult,
  type CleanupItem,
  type CleanupTarget,
  type Status,
  type Tier,
} from "./cleanup/types";
import { formatBytes } from "./format";
import { useUpdater } from "./platform/updater/useUpdater";

type ScanState = "idle" | "scanning" | "complete" | "error";

type ActionPhase = "idle" | "cleaning" | "done" | "error";
type ActionState = {
  phase: ActionPhase;
  freedGb?: number;
  message?: string;
  startedAt?: number;
};

type ConfirmRequest = {
  key: string;
  title: string;
  command: string;
  run: () => Promise<void>;
};

const TIER_ORDER: Tier[] = ["one", "two", "three"];

function targetKey(id: string) {
  return `t:${id}`;
}
function itemKey(targetId: string, itemId: string) {
  return `i:${targetId}:${itemId}`;
}

export function App() {
  const [scanState, setScanState] = useState<ScanState>("idle");
  const [error, setError] = useState<string | null>(null);
  const [targets, setTargets] = useState<CleanupTarget[]>([]);
  const [measuredIds, setMeasuredIds] = useState<Set<string>>(new Set());
  const [total, setTotal] = useState(0);

  const [freeBefore, setFreeBefore] = useState<number | null>(null);
  const [freeBeforeHuman, setFreeBeforeHuman] = useState<string>("—");
  const [freeNow, setFreeNow] = useState<number | null>(null);
  const [freeNowHuman, setFreeNowHuman] = useState<string>("—");

  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [actions, setActions] = useState<Record<string, ActionState>>({});
  const [confirmReq, setConfirmReq] = useState<ConfirmRequest | null>(null);

  const updater = useUpdater();
  const unlistenRef = useRef<null | (() => void)>(null);

  useEffect(() => () => unlistenRef.current?.(), []);

  const selected = useMemo(
    () => targets.find((t) => t.id === selectedId) ?? null,
    [selectedId, targets],
  );

  const grouped = useMemo(() => {
    const map: Record<Tier, CleanupTarget[]> = { one: [], two: [], three: [] };
    for (const t of targets) map[t.tier].push(t);
    return map;
  }, [targets]);

  const measured = measuredIds.size;

  const freedTotal =
    freeNow !== null && freeBefore !== null ? freeNow - freeBefore : 0;

  async function runScan() {
    setScanState("scanning");
    setError(null);
    setTargets([]);
    setMeasuredIds(new Set());
    setTotal(0);
    setActions({});

    unlistenRef.current?.();
    const unlistenCatalog = await onCatalogEnumerated((catalog) => {
      setTotal(catalog.length);
      setTargets(sortTargets(catalog));
    });
    const unlistenMeasured = await onTargetMeasured((target) => {
      setMeasuredIds((prev) => {
        const next = new Set(prev);
        next.add(target.id);
        return next;
      });
      setTargets((prev) => {
        const idx = prev.findIndex((t) => t.id === target.id);
        const next = idx === -1 ? [...prev, target] : prev.slice();
        if (idx !== -1) next[idx] = target;
        return sortTargets(next);
      });
    });
    const unlisten = () => {
      unlistenCatalog();
      unlistenMeasured();
    };
    unlistenRef.current = unlisten;

    try {
      const scan = await scanCleanupTargets();
      setTargets(sortTargets(scan.targets));
      setMeasuredIds(new Set(scan.targets.map((t) => t.id)));
      setTotal(scan.targets.length);
      setFreeBefore(scan.freeBeforeGb);
      setFreeBeforeHuman(scan.freeBeforeHuman);
      setFreeNow(scan.freeBeforeGb);
      setFreeNowHuman(scan.freeBeforeHuman);
      setSelectedId((prev) => prev ?? scan.targets[0]?.id ?? null);
      setScanState("complete");
    } catch (err) {
      setError(String(err));
      setScanState("error");
    } finally {
      unlisten();
      unlistenRef.current = null;
    }
  }

  function applyResult(key: string, result: CleanResult) {
    setFreeNow(result.freeGb);
    setFreeNowHuman(result.freeHuman);
    setActions((prev) => ({
      ...prev,
      [key]: result.ok
        ? { phase: "done", freedGb: result.freedGb }
        : { phase: "error", message: result.message ?? "Cleanup failed" },
    }));
  }

  async function execTarget(target: CleanupTarget) {
    const key = targetKey(target.id);
    setActions((p) => ({ ...p, [key]: { phase: "cleaning", startedAt: Date.now() } }));
    try {
      applyResult(key, await cleanTarget(target.id, true));
    } catch (err) {
      setActions((p) => ({
        ...p,
        [key]: { phase: "error", message: String(err) },
      }));
    }
  }

  async function execItem(target: CleanupTarget, item: CleanupItem) {
    const key = itemKey(target.id, item.id);
    setActions((p) => ({ ...p, [key]: { phase: "cleaning", startedAt: Date.now() } }));
    try {
      applyResult(key, await cleanItem(target.id, item.id, true));
    } catch (err) {
      setActions((p) => ({
        ...p,
        [key]: { phase: "error", message: String(err) },
      }));
    }
  }

  function handleTarget(target: CleanupTarget) {
    if (target.requiresDoubleConfirm) {
      setConfirmReq({
        key: targetKey(target.id),
        title: target.name,
        command: target.command ?? "",
        run: () => execTarget(target),
      });
    } else {
      void execTarget(target);
    }
  }

  function handleItem(target: CleanupTarget, item: CleanupItem) {
    if (item.requiresDoubleConfirm) {
      setConfirmReq({
        key: itemKey(target.id, item.id),
        title: `${target.name} — ${item.label}`,
        command: item.command,
        run: () => execItem(target, item),
      });
    } else {
      void execItem(target, item);
    }
  }

  function toggleExpand(id: string) {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }

  const busy = scanState === "scanning";

  return (
    <main className="app-shell">
      <aside className="control-rail">
        <div className="brand-lockup">
          <img src={mark} alt="" className="brand-mark" />
          <div>
            <p className="eyebrow">Clarus</p>
            <h1>Disk cleanup</h1>
          </div>
        </div>

        <section className="rail-section">
          <p className="section-label">Analyze</p>
          <button
            className="scan-action"
            type="button"
            disabled={busy}
            onClick={() => void runScan()}
          >
            {busy ? (
              <Loader2 size={17} className="spin" />
            ) : (
              <Activity size={17} />
            )}
            {busy ? "Measuring targets…" : "Analyze cleanup targets"}
          </button>
          {busy ? (
            <>
              <div
                className="scan-progress"
                role="progressbar"
                aria-valuemin={0}
                aria-valuemax={total || 1}
                aria-valuenow={measured}
              >
                <div
                  className="scan-progress-fill"
                  style={{
                    width: total ? `${(measured / total) * 100}%` : "0%",
                  }}
                />
              </div>
              <p className="muted-copy">
                {measured} / {total || "…"} targets measured…
              </p>
            </>
          ) : scanState === "complete" ? (
            <p className="muted-copy">{targets.length} targets scanned.</p>
          ) : (
            <p className="muted-copy">
              Nothing is deleted until you act on an item.
            </p>
          )}
        </section>

        <section className="rail-section">
          <p className="section-label">Disk free · data volume</p>
          <div className="disk-readout">
            <div className="disk-line">
              <span>Before</span>
              <strong>{freeBeforeHuman}</strong>
            </div>
            <div className="disk-line">
              <span>Now</span>
              <strong>{freeNowHuman}</strong>
            </div>
            <div className="disk-line freed" data-active={freedTotal > 0}>
              <span>Freed</span>
              <strong>{freedTotal > 0 ? `~${freedTotal} GB` : "—"}</strong>
            </div>
          </div>
        </section>

        <section className="rail-section rail-bottom">
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
                ? "Checking signed release manifest…"
                : updater.current === "error"
                  ? "Updater manifest is not reachable yet."
                  : "Release channel is configured."}
          </p>
        </section>
      </aside>

      <section className="intelligence-surface">
        <header className="surface-header">
          <div>
            <p className="eyebrow">Catalog review</p>
            <h2>Review known caches, then clean them one by one.</h2>
          </div>
          <div className="safety-pill">
            <ShieldCheck size={16} />
            No files touched until you act
          </div>
        </header>

        {error ? <div className="error-banner">{error}</div> : null}

        {targets.length === 0 ? (
          <div className="empty-state">
            <HardDrive size={28} />
            <p>
              {busy
                ? "Measuring known caches and regenerable data…"
                : "Run an analysis to list cleanup targets grouped by tier."}
            </p>
          </div>
        ) : (
          <div className="tier-stack">
            {TIER_ORDER.map((tier) => {
              const rows = grouped[tier];
              if (rows.length === 0) return null;
              const tierBytes = rows.reduce((sum, r) => sum + r.sizeBytes, 0);
              return (
                <div className="tier-group" key={tier} data-tier={tier}>
                  <div className="tier-header">
                    <span className="tier-dot" />
                    <span className="tier-name">{TIER_LABELS[tier]}</span>
                    <span className="tier-total">{formatBytes(tierBytes)}</span>
                  </div>
                  {rows.map((target) => (
                    <TargetRow
                      key={target.id}
                      target={target}
                      selected={target.id === selectedId}
                      expanded={expanded.has(target.id)}
                      measuring={!measuredIds.has(target.id)}
                      action={actions[targetKey(target.id)]}
                      itemAction={(itemId) =>
                        actions[itemKey(target.id, itemId)]
                      }
                      onSelect={() => setSelectedId(target.id)}
                      onToggle={() => toggleExpand(target.id)}
                      onClean={() => handleTarget(target)}
                      onCleanItem={(item) => handleItem(target, item)}
                    />
                  ))}
                </div>
              );
            })}
          </div>
        )}
      </section>

      <aside className="evidence-panel">
        <p className="section-label">Evidence</p>
        {selected ? (
          <>
            <h3>{selected.name}</h3>
            {selected.path ? (
              <p className="evidence-path">{selected.path}</p>
            ) : null}
            <div className="evidence-card">
              <span>Reason</span>
              <p>{selected.reason}</p>
            </div>
            <div className="evidence-card">
              <span>Risk</span>
              <p>{selected.riskNote}</p>
            </div>
            {selected.caveat ? (
              <div className="evidence-card warn">
                <span>Caveat</span>
                <p>{selected.caveat}</p>
              </div>
            ) : null}
            {selected.command ? (
              <div className="evidence-card">
                <span>Exact command</span>
                <code className="command-block">{selected.command}</code>
              </div>
            ) : selected.tier === "three" ? (
              <div className="evidence-card warn">
                <span>Manual only</span>
                <p>Clarus never deletes Tier 3 data automatically.</p>
              </div>
            ) : null}
          </>
        ) : (
          <div className="evidence-empty">
            <CheckCircle2 size={28} />
            <p>Select a target to inspect why Clarus flagged it.</p>
          </div>
        )}
      </aside>

      {confirmReq ? (
        <ConfirmModal
          request={confirmReq}
          onCancel={() => setConfirmReq(null)}
          onConfirm={() => {
            const req = confirmReq;
            setConfirmReq(null);
            void req.run();
          }}
        />
      ) : null}
    </main>
  );
}

function sortTargets(list: CleanupTarget[]): CleanupTarget[] {
  const rank: Record<Tier, number> = { one: 0, two: 1, three: 2 };
  return list
    .slice()
    .sort((a, b) => rank[a.tier] - rank[b.tier] || a.name.localeCompare(b.name));
}

function Elapsed({ startedAt }: { startedAt: number }) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const id = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(id);
  }, []);
  const secs = Math.max(0, Math.floor((now - startedAt) / 1000));
  const mm = Math.floor(secs / 60);
  const ss = String(secs % 60).padStart(2, "0");
  return <span className="elapsed">{`${mm}:${ss}`}</span>;
}

function StatusChip({ status }: { status: Status }) {
  return (
    <span className="status-chip" data-status={status}>
      {STATUS_LABELS[status]}
    </span>
  );
}

function ActionButton({
  action,
  disabled,
  danger,
  onClick,
}: {
  action: ActionState | undefined;
  disabled?: boolean;
  danger?: boolean;
  onClick: () => void;
}) {
  const phase = action?.phase ?? "idle";
  if (phase === "cleaning") {
    return (
      <button className="row-action" type="button" disabled>
        <Loader2 size={14} className="spin" />
        Cleaning…
        {action?.startedAt ? <Elapsed startedAt={action.startedAt} /> : null}
      </button>
    );
  }
  if (phase === "done") {
    return (
      <button className="row-action done" type="button" disabled>
        <CheckCircle2 size={14} />
        {action?.freedGb && action.freedGb > 0
          ? `Freed ~${action.freedGb} GB`
          : "Cleaned"}
      </button>
    );
  }
  if (phase === "error") {
    return (
      <button
        className="row-action error"
        type="button"
        title={action?.message}
        onClick={onClick}
      >
        <AlertTriangle size={14} />
        Retry
      </button>
    );
  }
  return (
    <button
      className={danger ? "row-action danger" : "row-action"}
      type="button"
      disabled={disabled}
      onClick={onClick}
    >
      <Trash2 size={14} />
      Clean
    </button>
  );
}

function TargetRow({
  target,
  selected,
  expanded,
  measuring,
  action,
  itemAction,
  onSelect,
  onToggle,
  onClean,
  onCleanItem,
}: {
  target: CleanupTarget;
  selected: boolean;
  expanded: boolean;
  measuring: boolean;
  action: ActionState | undefined;
  itemAction: (itemId: string) => ActionState | undefined;
  onSelect: () => void;
  onToggle: () => void;
  onClean: () => void;
  onCleanItem: (item: CleanupItem) => void;
}) {
  const isContainer = target.subitems.length > 0;
  const isTier3 = target.tier === "three";
  const actionable = isTargetActionable(target);

  return (
    <div className="target-block">
      <div
        className="target-row"
        data-active={selected}
        onClick={onSelect}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            onSelect();
          }
        }}
      >
        <div className="target-lead">
          {isContainer ? (
            <button
              className="expand-toggle"
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                onToggle();
              }}
              aria-label={expanded ? "Collapse" : "Expand"}
            >
              {expanded ? (
                <ChevronDown size={15} />
              ) : (
                <ChevronRight size={15} />
              )}
            </button>
          ) : (
            <span className="expand-spacer" />
          )}
          <div className="target-text">
            <span className="target-name">{target.name}</span>
            {target.path ? (
              <span className="target-path" title={target.path}>
                {target.path}
              </span>
            ) : isContainer ? (
              <span className="target-path">
                {target.subitems.length} item
                {target.subitems.length === 1 ? "" : "s"}
              </span>
            ) : null}
          </div>
        </div>

        {measuring ? (
          <span className="skeleton skeleton-chip" />
        ) : (
          <StatusChip status={target.status} />
        )}
        {measuring ? (
          <span className="skeleton skeleton-size" />
        ) : (
          <span className="target-size">{target.sizeHuman || "—"}</span>
        )}

        <div className="target-cta" onClick={(e) => e.stopPropagation()}>
          {isTier3 ? (
            <span className="manual-tag">
              <CircleSlash size={13} />
              Manual only
            </span>
          ) : isContainer ? (
            <span className="manual-tag subtle">Per item</span>
          ) : (
            <ActionButton
              action={action}
              disabled={!actionable}
              danger={target.requiresDoubleConfirm}
              onClick={onClean}
            />
          )}
        </div>
      </div>

      {action?.phase === "cleaning" && target.caveat ? (
        <p className="cleaning-hint">{target.caveat}</p>
      ) : null}

      {isContainer && expanded ? (
        <div className="subitem-list">
          {target.subitems.map((item) => (
            <div className="subitem-row" key={item.id}>
              <div className="subitem-text">
                <span className="subitem-label">{item.label}</span>
                {item.meta ? (
                  <span className="subitem-meta">{item.meta}</span>
                ) : null}
              </div>
              <span className="target-size">{item.sizeHuman || "—"}</span>
              <ActionButton
                action={itemAction(item.id)}
                danger={item.requiresDoubleConfirm}
                onClick={() => onCleanItem(item)}
              />
            </div>
          ))}
        </div>
      ) : null}
    </div>
  );
}

function ConfirmModal({
  request,
  onCancel,
  onConfirm,
}: {
  request: ConfirmRequest;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  const [value, setValue] = useState("");
  const confirmed = value.trim() === "SI";

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") onCancel();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onCancel]);

  return (
    <div className="modal-scrim" onClick={onCancel}>
      <div
        className="modal-card"
        role="dialog"
        aria-modal="true"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal-head">
          <AlertTriangle size={18} />
          <h3>High-risk action</h3>
        </div>
        <p className="modal-target">{request.title}</p>
        <p className="modal-copy">
          This action is destructive and cannot be undone. Type{" "}
          <strong>SI</strong> to confirm.
        </p>
        <code className="command-block">{request.command}</code>
        <input
          className="confirm-input"
          value={value}
          autoFocus
          spellCheck={false}
          placeholder="Type SI to confirm"
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && confirmed) onConfirm();
          }}
        />
        <div className="modal-actions">
          <button className="quiet-action" type="button" onClick={onCancel}>
            Cancel
          </button>
          <button
            className="row-action danger"
            type="button"
            disabled={!confirmed}
            onClick={onConfirm}
          >
            <Trash2 size={14} />
            Confirm cleanup
          </button>
        </div>
      </div>
    </div>
  );
}
