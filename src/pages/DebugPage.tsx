import { CollapsibleCardSection } from "../components/ui/CollapsibleCardSection";
import { ToggleControl } from "../components/controls/ToggleControl";
import { TabPageHeader } from "../components/ui/TabPageHeader";
import { Tooltip } from "../components/ui/Tooltip";
import { LogsBlock } from "../components/debug/LogsBlock";
import type { AppPreferences } from "../hooks/useAppPreferences";
import type { DeviceConnection } from "../hooks/useDeviceConnection";
import type { Updates } from "../hooks/useUpdates";
import type { ToastType } from "../types/g6";
import { useState } from "preact/hooks";

interface DebugPageProps {
  preferences: AppPreferences;
  updates: Updates;
  connection: DeviceConnection;
  onNavigateUiLab: () => void;
  showToast: (message: string, type: ToastType) => void;
}

export function DebugPage({ preferences, updates, connection, onNavigateUiLab, showToast }: DebugPageProps) {
  const [activeSection, setActiveSection] = useState<string | null>(null);
  const toggleSection = (section: string) => {
    setActiveSection((current) => (current === section ? null : section));
  };

  return (
    <>
      <TabPageHeader
        title="Settings"
        subtitle="Application controls and update preferences, with advanced debug tools grouped separately below."
      />

      <div class="settings-accordion">
      <CollapsibleCardSection
        title="General Settings"
        isOpen={activeSection === "general"}
        onToggle={() => toggleSection("general")}
      >
        <div class="debug-log-tools">
          <div class="debug-log-card">
            <div class="debug-toggle-grid">
              <ToggleControl
                label="Auto-start with system"
                checked={preferences.autostartEnabled}
                onChange={preferences.toggleAutostart}
              />
              <ToggleControl
                label="Close to tray"
                checked={preferences.closeToTray}
                onChange={preferences.toggleCloseToTray}
                tooltip="Closing the window keeps Rusty G6 running in the system tray. Use Quit in the tray menu to exit."
              />
            </div>
          </div>
        </div>
      </CollapsibleCardSection>

      <CollapsibleCardSection
        title="Updates"
        isOpen={activeSection === "updates"}
        onToggle={() => toggleSection("updates")}
      >
        <div class="debug-log-tools">
          <div class="debug-log-card">
            <div class="device-details">
              <div class="read-only-item">
                <span class="readonly-label">Current Version:</span>
                <span class="readonly-value">v{preferences.appVersion || "1.0.x"}</span>
              </div>
              <div class="read-only-item">
                <span class="readonly-label">Latest Version:</span>
                <span class="readonly-value">{updates.updateResult ? `v${updates.updateResult.latestVersion}` : "Unknown"}</span>
              </div>
              <div class="read-only-item">
                <span class="readonly-label">Last Checked:</span>
                <span class="readonly-value">{updates.lastCheckedLabel}</span>
              </div>
            </div>
            <div class="debug-log-actions">
              <button onClick={() => updates.checkForUpdates(true)} class="btn-compact btn-secondary" disabled={updates.checkingUpdates}>
                {updates.checkingUpdates ? "Checking..." : "Check for Updates"}
              </button>
              {updates.updateAvailable && (
                <button onClick={() => updates.setShowUpdateModal(true)} class="btn-compact btn-secondary">
                  View Update
                </button>
              )}
            </div>
          </div>
        </div>
      </CollapsibleCardSection>

      <CollapsibleCardSection
        title="Logs"
        isOpen={activeSection === "logging"}
        onToggle={() => toggleSection("logging")}
      >
        <div class="debug-log-tools">
          <LogsBlock showToast={showToast} />
          <div class="debug-log-card">
            <div class="debug-log-row">
              <input
                type="text"
                class="log-message-input"
                placeholder="Optional: Add a note to the log separator..."
                value={preferences.logSeparatorMessage}
                onInput={(e) => preferences.setLogSeparatorMessage(e.currentTarget.value)}
                onKeyPress={(e) => {
                  if (e.key === "Enter") {
                    preferences.insertLogSeparator();
                  }
                }}
              />
              <Tooltip text="Inserts a visible separator marker in terminal logs using your optional note.">
                <button onClick={preferences.insertLogSeparator} class="btn-compact btn-secondary debug-log-button">
                  Add Log Separator
                </button>
              </Tooltip>
            </div>
          </div>
        </div>
      </CollapsibleCardSection>

      <CollapsibleCardSection
        title="Device Debug"
        isOpen={activeSection === "device-debug"}
        onToggle={() => toggleSection("device-debug")}
      >
        <div class="debug-log-tools">
          <div class="debug-log-card">
            <div class="debug-action-grid">
              <button onClick={connection.readDeviceState} class="btn-compact btn-secondary">
                Read Device State
              </button>
              <button onClick={connection.resetConnection} class="btn-compact btn-secondary">
                Reset Connection
              </button>
            </div>
          </div>
        </div>
      </CollapsibleCardSection>

      <CollapsibleCardSection
        title="UI Debug"
        isOpen={activeSection === "ui-debug"}
        onToggle={() => toggleSection("ui-debug")}
      >
        <div class="debug-log-tools">
          <div class="debug-log-card">
            <div class="debug-action-grid">
              <button onClick={onNavigateUiLab} class="btn-compact btn-secondary">
                Open UI Lab
              </button>
            </div>
          </div>
        </div>
      </CollapsibleCardSection>
      </div>
    </>
  );
}
