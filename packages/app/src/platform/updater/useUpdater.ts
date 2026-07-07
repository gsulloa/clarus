import { useCallback, useEffect, useMemo, useState } from "react";
import { check } from "@tauri-apps/plugin-updater";

export type UpdaterState = {
  current: "idle" | "checking" | "available" | "current" | "error";
  version: string | null;
  error: string | null;
  checkNow: () => Promise<void>;
};

function isTauriRuntime(): boolean {
  return (
    typeof window !== "undefined" &&
    "__TAURI_INTERNALS__" in (window as unknown as Record<string, unknown>)
  );
}

export function useUpdater(): UpdaterState {
  const [current, setCurrent] = useState<UpdaterState["current"]>("idle");
  const [version, setVersion] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const checkNow = useCallback(async () => {
    if (!isTauriRuntime()) {
      setCurrent("current");
      return;
    }

    setCurrent("checking");
    setError(null);

    try {
      const update = await check();
      if (!update) {
        setCurrent("current");
        setVersion(null);
        return;
      }
      setVersion(update.version);
      setCurrent("available");
    } catch (err) {
      setError(String(err));
      setCurrent("error");
    }
  }, []);

  useEffect(() => {
    const timeout = window.setTimeout(() => void checkNow(), 5_000);
    return () => window.clearTimeout(timeout);
  }, [checkNow]);

  return useMemo(
    () => ({ current, version, error, checkNow }),
    [checkNow, current, error, version],
  );
}
