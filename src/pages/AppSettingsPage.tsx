import { ToggleControl } from "../components/controls/ToggleControl";
import { LogsBlock } from "../components/debug/LogsBlock";
import { InfoRow } from "../components/ui/InfoRow";
import { SettingsPage } from "../components/ui/SettingsPage";
import { SettingsSection } from "../components/ui/SettingsSection";
import type { AppPreferences } from "../hooks/useAppPreferences";
import type { DeviceConnection } from "../hooks/useDeviceConnection";
import type { Updates } from "../hooks/useUpdates";
import type { ToastType } from "../types/g6";

interface AppSettingsPageProps {
  preferences: AppPreferences;
  updates: Updates;
  connection: DeviceConnection;
  showToast: (message: string, type: ToastType) => void;
}

export function AppSettingsPage({ preferences, updates, connection, showToast }: AppSettingsPageProps) {
  return (
    <SettingsPage>

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
        <InfoRow
          label={`v${preferences.appVersion || "1.0.x"}`}
          value={`Checked ${updates.lastCheckedLabel.toLowerCase()}`}
          muted
        >
          {updates.updateAvailable && updates.updateResult && (
            <button onClick={() => updates.setShowUpdateModal(true)} class="btn-compact">
              Get v{updates.updateResult.latestVersion}
            </button>
          )}
          <button
            onClick={() => updates.checkForUpdates(true)}
            class="btn-compact btn-secondary"
            disabled={updates.checkingUpdates}
          >
            {updates.checkingUpdates ? "Checking..." : "Check"}
          </button>
        </InfoRow>
      </SettingsSection>

      <LogsBlock showToast={showToast} />

      <SettingsSection title="Device">
        <InfoRow label="Connection">
          <button onClick={connection.resetConnection} class="btn-compact btn-secondary">
            Reconnect
          </button>
        </InfoRow>
      </SettingsSection>
    </SettingsPage>
  );
}
