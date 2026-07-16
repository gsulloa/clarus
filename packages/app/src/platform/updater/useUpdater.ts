import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export type UpdaterState = {
  current:
    | "idle"
    | "checking"
    | "available"
    | "downloading"
    | "downloaded"
    | "current"
    | "error";
  version: string | null;
  error: string | null;
  checkNow: () => Promise<void>;
  downloadAndInstall: () => Promise<void>;
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
  const updateRef = useRef<Update | null>(null);

  const checkNow = useCallback(async () => {
    if (!isTauriRuntime()) {
      updateRef.current = null;
      setCurrent("current");
      return;
    }

    setCurrent("checking");
    setError(null);

    try {
      const update = await check();
      if (!update) {
        updateRef.current = null;
        setCurrent("current");
        setVersion(null);
        return;
      }
      updateRef.current = update;
      setVersion(update.version);
      setCurrent("available");
    } catch (err) {
      updateRef.current = null;
      setError(String(err));
      setCurrent("error");
    }
  }, []);

  const downloadAndInstall = useCallback(async () => {
    const update = updateRef.current;
    if (!update) {
      return;
    }

    setCurrent("downloading");
    setError(null);

    try {
      await update.downloadAndInstall();
      setCurrent("downloaded");
      await relaunch();
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
    () => ({ current, version, error, checkNow, downloadAndInstall }),
    [checkNow, current, downloadAndInstall, error, version],
  );
}
