import { invoke } from "@tauri-apps/api/core";
import {
  disable as disableAutostart,
  enable as enableAutostart,
  isEnabled as isAutostartEnabled,
} from "@tauri-apps/plugin-autostart";
import { useEffect, useState } from "preact/hooks";
import { logUiEvent } from "../app/uiLog";
import type { AppSettings, ToastType } from "../types/g6";

export type AppPreferences = ReturnType<typeof useAppPreferences>;

/// App-level preferences in Settings: auto-start, close to tray, the app
/// version and the log separator tool.
export function useAppPreferences(showToast: (message: string, type: ToastType, durationMs?: number) => void) {
  const [appVersion, setAppVersion] = useState("");
  const [autostartEnabled, setAutostartEnabled] = useState(false);
  const [closeToTray, setCloseToTray] = useState(true);
  const [logSeparatorMessage, setLogSeparatorMessage] = useState("");

  useEffect(() => {
    invoke<string>("get_app_version")
      .then((version) => {
        setAppVersion(version);
        void logUiEvent(`Session metadata: app_version=${version}`);
      })
      .catch((error) => console.error("Failed to get app version:", error));
    isAutostartEnabled()
      .then(setAutostartEnabled)
      .catch((error) => console.error("Failed to read the auto-start state:", error));
    invoke<AppSettings>("get_app_settings")
      .then((appSettings) => setCloseToTray(appSettings.close_to_tray))
      .catch((error) => console.error("Failed to read app settings:", error));
  }, []);

  const toggleAutostart = async (enabled: boolean) => {
    try {
      await (enabled ? enableAutostart() : disableAutostart());
      setAutostartEnabled(enabled);
      showToast(`Auto-start ${enabled ? "enabled" : "disabled"}`, "success");
    } catch (error) {
      console.error("Failed to toggle autostart:", error);
      showToast(`Failed to toggle auto-start: ${error}`, "error");
    }
  };

  const toggleCloseToTray = async (enabled: boolean) => {
    try {
      const updated = await invoke<AppSettings>("set_close_to_tray", { enabled });
      setCloseToTray(updated.close_to_tray);
      showToast(`Close to tray ${enabled ? "enabled" : "disabled"}`, "success");
    } catch (error) {
      console.error("Failed to change close to tray:", error);
      showToast(`Failed to change close to tray: ${error}`, "error");
    }
  };

  const insertLogSeparator = async () => {
    try {
      await invoke("clear_terminal", { message: logSeparatorMessage || null });
      showToast("Log separator added", "success", 2000);
      setLogSeparatorMessage("");
    } catch (error) {
      console.error("Failed to add log separator:", error);
      showToast(`Failed to add log separator: ${error}`, "error", 3000);
    }
  };

  return {
    appVersion,
    autostartEnabled,
    closeToTray,
    logSeparatorMessage,
    toggleAutostart,
    toggleCloseToTray,
    setLogSeparatorMessage,
    insertLogSeparator,
  };
}
