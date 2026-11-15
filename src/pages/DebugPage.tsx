import { ToggleControl } from "../components/controls/ToggleControl";
import { LogsBlock } from "../components/debug/LogsBlock";
import { InfoRow } from "../components/ui/InfoRow";
import { SettingsSection } from "../components/ui/SettingsSection";
import { TabPageHeader } from "../components/ui/TabPageHeader";
import { Tooltip } from "../components/ui/Tooltip";
import type { AppPreferences } from "../hooks/useAppPreferences";
import type { DeviceConnection } from "../hooks/useDeviceConnection";
import type { Updates } from "../hooks/useUpdates";
import type { ToastType } from "../types/g6";

interface DebugPageProps {
  preferences: AppPreferences;
  updates: Updates;
  connection: DeviceConnection;
  onNavigateUiLab: () => void;
  showToast: (message: string, type: ToastType) => void;
}

export function DebugPage({ preferences, updates, connection, onNavigateUiLab, showToast }: DebugPageProps) {
  const logSeparator = (
    <div class="control-row">
      <input
        type="text"
        class="ghost-input"
        placeholder="Note for a log separator (optional)"
        value={preferences.logSeparatorMessage}
        onInput={(e) => preferences.setLogSeparatorMessage(e.currentTarget.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            preferences.insertLogSeparator();
          }
        }}
      />
      <Tooltip text="Writes a visible separator line, with your note, into the session log.">
        <button onClick={preferences.insertLogSeparator} class="btn-compact btn-secondary">
          Add separator
        </button>
      </Tooltip>
    </div>
  );

  return (
    <>
      <TabPageHeader title="Settings" subtitle="How Rusty G6 starts and updates, and tools for bug reports." />

      <SettingsSection title="General">
        <ToggleControl
          label="Start with the system"
          checked={preferences.autostartEnabled}
          onChange={preferences.toggleAutostart}
        />
        <ToggleControl
          label="Close to tray"
          checked={preferences.closeToTray}
          onChange={preferences.toggleCloseToTray}
          tooltip="Closing the window keeps Rusty G6 running in the system tray. Use Quit in the tray menu to exit."
        />
      </SettingsSection>

      <SettingsSection title="Updates">
        <InfoRow label="Installed" value={`v${preferences.appVersion || "1.0.x"}`} />
        <InfoRow
          label="Latest"
          value={updates.updateResult ? `v${updates.updateResult.latestVersion}` : "Unknown"}
        />
        <InfoRow label="Last checked" value={updates.lastCheckedLabel}>
          {updates.updateAvailable && (
            <button onClick={() => updates.setShowUpdateModal(true)} class="btn-compact">
              View update
            </button>
          )}
          <button
            onClick={() => updates.checkForUpdates(true)}
            class="btn-compact btn-secondary"
            disabled={updates.checkingUpdates}
          >
            {updates.checkingUpdates ? "Checking..." : "Check now"}
          </button>
        </InfoRow>
      </SettingsSection>

      <LogsBlock showToast={showToast} separator={logSeparator} />

      <SettingsSection title="Device">
        <InfoRow label="Connection">
          <button onClick={connection.readDeviceState} class="btn-compact btn-secondary">
            Read device state
          </button>
          <button onClick={connection.resetConnection} class="btn-compact btn-secondary">
            Reconnect
          </button>
        </InfoRow>
      </SettingsSection>

      <SettingsSection title="Developer">
        <InfoRow label="UI Lab">
          <button onClick={onNavigateUiLab} class="btn-compact btn-secondary">
            Open
          </button>
        </InfoRow>
      </SettingsSection>
    </>
  );
}
