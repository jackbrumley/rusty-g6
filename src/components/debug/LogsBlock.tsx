import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "preact/hooks";
import { SelectControl } from "../controls/SelectControl";
import { InfoRow } from "../ui/InfoRow";
import { SettingsSection } from "../ui/SettingsSection";
import type { SystemInfo, ToastType } from "../../types/g6";

// The Logs and System info sections in Settings (App Standard section 4). Bug reports
// ask for "Copy log" plus "System info".

const LOG_LEVELS = ["error", "warn", "info", "debug", "trace"] as const;

interface LogsBlockProps {
  showToast: (message: string, type: ToastType) => void;
}

const systemInfoText = (info: SystemInfo) =>
  [
    `App: ${info.appName} ${info.appVersion} (${info.buildProfile})`,
    `OS: ${info.os} (${info.distribution})`,
    `Architecture: ${info.architecture}`,
    `Desktop: ${info.desktop}`,
    `Session: ${info.sessionType}`,
    `Executable: ${info.executable}`,
  ].join("\n");

export function LogsBlock({ showToast }: LogsBlockProps) {
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
  const [logLevel, setLogLevel] = useState("info");

  useEffect(() => {
    invoke<SystemInfo>("get_system_info")
      .then(setSystemInfo)
      .catch((error) => showToast(`Failed to read system info: ${error}`, "error"));
    invoke<string>("get_log_level")
      .then(setLogLevel)
      .catch((error) => showToast(`Failed to read the log level: ${error}`, "error"));
  }, []);

  const run = async (action: string, command: string, success?: string) => {
    try {
      await invoke(command);
      if (success) {
        showToast(success, "success");
      }
    } catch (error) {
      showToast(`Failed to ${action}: ${error}`, "error");
    }
  };

  const copySystemInfo = async () => {
    if (!systemInfo) {
      return;
    }
    try {
      await invoke("plugin:clipboard-manager|write_text", { text: systemInfoText(systemInfo) });
      showToast("System info copied to clipboard.", "success");
    } catch (error) {
      showToast(`Failed to copy system info: ${error}`, "error");
    }
  };

  const changeLogLevel = async (level: string) => {
    try {
      setLogLevel(await invoke<string>("set_log_level", { level }));
    } catch (error) {
      showToast(`Failed to set the log level: ${error}`, "error");
    }
  };

  return (
    <>
      <SettingsSection title="Logs">
        <SelectControl
          label="Log level"
          value={logLevel}
          options={LOG_LEVELS.map((level) => ({ id: level, name: level }))}
          onChange={changeLogLevel}
        />
        <InfoRow label="Session log">
          <button
            onClick={() => run("copy the log", "copy_session_log", "Session log copied to clipboard.")}
            class="btn-compact"
          >
            Copy
          </button>
          <button onClick={() => run("open the log", "open_session_log")} class="btn-compact">
            Open
          </button>
          <button onClick={() => run("open the log folder", "open_log_folder")} class="btn-compact">
            Folder
          </button>
        </InfoRow>
      </SettingsSection>

      <SettingsSection title="System info">
        <pre class="system-info-text">{systemInfo ? systemInfoText(systemInfo) : "Loading..."}</pre>
        <InfoRow label="For bug reports">
          <button onClick={copySystemInfo} class="btn-compact" disabled={!systemInfo}>
            Copy system info
          </button>
        </InfoRow>
      </SettingsSection>
    </>
  );
}
