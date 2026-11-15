import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useEffect, useState } from "preact/hooks";
import { logUiEvent } from "../app/uiLog";
import type { ToastType, UpdateCheckResult } from "../types/g6";

const LATEST_RELEASE_URL = "https://github.com/jackbrumley/rusty-g6/releases/latest";

export type Updates = ReturnType<typeof useUpdates>;

/// Update checks (one in the background at start-up) and the in-app update.
export function useUpdates(showToast: (message: string, type: ToastType, durationMs?: number) => void) {
  const [checkingUpdates, setCheckingUpdates] = useState(false);
  const [installingUpdate, setInstallingUpdate] = useState(false);
  const [updateResult, setUpdateResult] = useState<UpdateCheckResult | null>(null);
  const [showUpdateModal, setShowUpdateModal] = useState(false);
  const [updateError, setUpdateError] = useState<string | null>(null);
  const [lastCheckedAt, setLastCheckedAt] = useState<number | null>(null);

  const checkForUpdates = async (showUpToDateToast: boolean) => {
    if (checkingUpdates) {
      return;
    }
    setCheckingUpdates(true);
    setUpdateError(null);
    try {
      const result = await invoke<UpdateCheckResult>("check_for_updates");
      setUpdateResult(result);
      setLastCheckedAt(Date.now());
      await logUiEvent(
        `Update check completed: current=${result.currentVersion} latest=${result.latestVersion} available=${result.updateAvailable}`
      );
      if (result.updateAvailable) {
        setShowUpdateModal(true);
      } else if (showUpToDateToast) {
        showToast("You are already on the latest version.", "info");
      }
    } catch (error) {
      const message = String(error);
      setUpdateError(message);
      await logUiEvent(`Update check failed: ${message}`);
      if (showUpToDateToast) {
        showToast(`Failed to check for updates: ${message}`, "error");
      }
    } finally {
      setCheckingUpdates(false);
    }
  };

  useEffect(() => {
    void checkForUpdates(false);
  }, []);

  const openLatestReleasePage = async () => {
    const releaseUrl = updateResult?.releaseUrl || LATEST_RELEASE_URL;
    try {
      await openUrl(releaseUrl);
      await logUiEvent(`Opened release page: ${releaseUrl}`);
    } catch (error) {
      showToast(`Failed to open release page: ${error}`, "error");
    }
  };

  // Re-runs the one-line installer in the background; the app exits and the
  // installer relaunches the new version (see RELEASE_STANDARD.md).
  const installUpdate = async () => {
    if (installingUpdate) {
      return;
    }
    setInstallingUpdate(true);
    try {
      showToast("Starting update... Rusty G6 will restart shortly.", "info");
      await logUiEvent(`Installing update to v${updateResult?.latestVersion ?? "latest"}`);
      await invoke("install_update");
    } catch (error) {
      setInstallingUpdate(false);
      showToast(`Failed to start update: ${error}`, "error");
      await logUiEvent(`Failed to start update: ${error}`);
    }
  };

  return {
    checkingUpdates,
    installingUpdate,
    updateResult,
    updateAvailable: Boolean(updateResult?.updateAvailable),
    showUpdateModal,
    updateError,
    lastCheckedLabel: lastCheckedLabel(lastCheckedAt),
    checkForUpdates,
    setShowUpdateModal,
    openLatestReleasePage,
    installUpdate,
  };
}

function lastCheckedLabel(lastCheckedAt: number | null): string {
  if (!lastCheckedAt) {
    return "Not checked yet";
  }
  const elapsedMinutes = Math.floor((Date.now() - lastCheckedAt) / 60_000);
  if (elapsedMinutes < 1) {
    return "Just now";
  }
  if (elapsedMinutes < 60) {
    return `${elapsedMinutes} min ago`;
  }
  const elapsedHours = Math.floor(elapsedMinutes / 60);
  if (elapsedHours < 24) {
    return `${elapsedHours} hr ago`;
  }
  const elapsedDays = Math.floor(elapsedHours / 24);
  return `${elapsedDays} day${elapsedDays === 1 ? "" : "s"} ago`;
}
